//! L'instance unique (bloc N de l'organigramme) : une seule copie du logiciel par session
//! Windows, par un verrou nommé dans l'espace de la session (`Local\<nom>-une-seule`, C-18).
//!
//! Un second lancement ne s'ouvre pas (C-15) : il prévient la copie déjà lancée, puis
//! s'arrête. Selon le [`Reveil`] qu'on lui donne, il lui demande de s'ouvrir (sa fenêtre
//! principale, sinon ses Paramètres, C-17), lui transmet ses arguments (bornés à
//! [`ARGUMENTS_MAX`] et [`ARGUMENTS_OCTETS_MAX`], marqués, vérifiés à l'arrivée), ou ne lui dit
//! rien.
//!
//! Ce que ce bloc ne fait pas : interpréter les arguments (le logiciel, par
//! `Horloge::recevoir_arguments`) ; créer la fenêtre cachée qui les reçoit (l'icône, bloc M) ;
//! ouvrir la page demandée (le canal `Liens::montrer_page` de la boucle des fenêtres, bloc J).
//!
//! La partie pure (noms, codage, décisions) est dans `unique/codage.rs` ; ici, seulement les
//! appels à Windows, chacun commenté. Repris à son chemin d'aujourd'hui :
//! `interface_commune::horloge::{instance_unique, instance_unique_avec, Unique, …}`.

use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, WPARAM,
};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, PostMessageW, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_APP, WM_COPYDATA,
};
use windows::core::{HSTRING, PCWSTR};

use ic_coeur::contrat::Reveil;

mod codage;
#[cfg(test)]
mod essais;

pub use codage::{ARGUMENTS_MAX, ARGUMENTS_OCTETS_MAX};
use codage::{
    Decision, ErreurUnique, Issue, copie_a_lire, decoder_arguments, ligne_du_journal, que_dire, traduire,
};

/// Le message qu'un second lancement poste à la fenêtre cachée de la copie déjà lancée : « ouvre
/// le logiciel ». Il vient d'un autre programme : il ne porte pas le jeton des messages
/// internes de la fenêtre cachée.
pub(crate) const WM_OUVRIR_PARAMETRES: u32 = WM_APP + 3;

/// Le temps que le second lancement attend la copie en cours (ms ; au-delà, la demande est
/// perdue : un second lancement ne reste jamais accroché).
const ATTENTE_TRANSMISSION: u32 = 3000;

/// Le nom de la classe de la fenêtre cachée, en chaîne de Windows (créée par l'icône, bloc M ;
/// cherchée par un second lancement).
pub(crate) fn classe(nom: &str) -> HSTRING {
    HSTRING::from(codage::nom_de_la_fenetre(nom))
}

/// Le verrou « une seule instance », gardé jusqu'à la fin du programme.
pub struct Unique(HANDLE);

impl Drop for Unique {
    fn drop(&mut self) {
        // SAFETY: poignée créée par `prendre_le_verrou`, rangée dans un seul `Unique`, fermée
        // une seule fois.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

/// Une seule instance par session et par logiciel (`nom`, sans espace ni barre oblique). Si
/// le logiciel tourne déjà : lui demande d'ouvrir ses Paramètres et rend `Err(true)` ;
/// `Err(false)` si sa fenêtre est introuvable (il démarre ou se ferme).
pub fn instance_unique(nom: &str) -> Result<Unique, bool> {
    instance_unique_avec(nom, &Reveil::Parametres)
}

/// Comme [`instance_unique`], en disant ce que le second lancement transmet ([`Reveil`]).
/// `Err(true)` : l'instance en cours est prévenue (ou rien n'était à lui dire) ;
/// `Err(false)` : sa fenêtre est introuvable ou n'a pas répondu en 3 s (elle démarre, se
/// ferme ou ne répond plus) : la demande est perdue, ce second lancement s'arrête quand même.
pub fn instance_unique_avec(nom: &str, reveil: &Reveil) -> Result<Unique, bool> {
    let issue = lancement(nom, reveil);
    if let Some(ligne) = ligne_du_journal(&issue) {
        crate::sonde::noter(ligne);
    }
    traduire(issue)
}

/// Essaie le verrou ; s'il est déjà pris, fait ce que le [`Reveil`] demande.
fn lancement(nom: &str, reveil: &Reveil) -> Issue<Unique> {
    if let Some(verrou) = prendre_le_verrou(nom) {
        return Issue::Seul(verrou);
    }
    match que_dire(reveil) {
        Decision::SArreter => Issue::RienADemander,
        Decision::Refuser(e) => Issue::Perdue(e),
        Decision::Ouvrir => prevenir(nom, demander_l_ouverture),
        Decision::Transmettre(octets) => prevenir(nom, |f| transmettre(f, &octets)),
    }
}

/// Prend le verrou `Local\<nom>-une-seule`. `None` s'il est déjà pris par une autre copie ;
/// aussi si Windows le refuse (un verrou du même nom qu'on ne peut pas ouvrir) : le lancement
/// se conduit alors en second, comme avant.
fn prendre_le_verrou(nom: &str) -> Option<Unique> {
    let verrou = HSTRING::from(codage::nom_du_verrou(nom));
    // SAFETY: nom terminé par zéro, vivant pendant l'appel ; la poignée rendue va aussitôt
    // dans un `Unique`, qui la fermera.
    let h = unsafe { CreateMutexW(None, false, &verrou) };
    // SAFETY: lecture de la dernière erreur du fil, juste après l'appel.
    let deja = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let h = h.ok().map(Unique)?;
    // Déjà là : la poignée ouverte sur le verrou de l'autre copie est fermée tout de suite
    // (`drop`), sinon elle le garderait vivant après la fin de l'autre.
    (!deja).then_some(h)
}

/// Cherche la fenêtre cachée de la copie déjà lancée et lui fait `envoyer`.
fn prevenir(nom: &str, envoyer: impl FnOnce(HWND) -> Result<(), ErreurUnique>) -> Issue<Unique> {
    let c = classe(nom);
    // SAFETY: nom de classe vivant pendant l'appel ; la fenêtre trouvée ne sert qu'à lui
    // envoyer un message.
    let Ok(fenetre) = (unsafe { FindWindowW(&c, PCWSTR::null()) }) else {
        return Issue::Perdue(ErreurUnique::FenetreIntrouvable);
    };
    match envoyer(fenetre) {
        Ok(()) => Issue::Prevenue,
        Err(e) => Issue::Perdue(e),
    }
}

/// Demande à la copie en cours de s'ouvrir (sans attendre sa réponse).
fn demander_l_ouverture(fenetre: HWND) -> Result<(), ErreurUnique> {
    // SAFETY: fenêtre trouvée juste avant ; PostMessageW n'attend rien et échoue sans dommage
    // si elle a disparu entre-temps.
    let poste = unsafe { PostMessageW(Some(fenetre), WM_OUVRIR_PARAMETRES, WPARAM(0), LPARAM(0)) };
    poste.map_err(|_| ErreurUnique::MessageNonParti)
}

/// Envoie les arguments codés à la fenêtre cachée de la copie en cours (`WM_COPYDATA`, qui
/// copie les octets dans l'autre processus). `Ok` si elle les a pris.
fn transmettre(fenetre: HWND, octets: &[u8]) -> Result<(), ErreurUnique> {
    let copie = COPYDATASTRUCT {
        dwData: codage::MARQUE_ARGUMENTS,
        // Borné à ARGUMENTS_OCTETS_MAX par le codage : tient dans un u32.
        cbData: octets.len() as u32,
        lpData: octets.as_ptr().cast_mut().cast(),
    };
    let mut reponse = 0usize;
    // SAFETY: `copie` et `octets` vivent pendant tout l'appel (envoi synchrone : Windows
    // copie les octets avant de rendre la main, et ne les modifie pas) ; la fenêtre est celle
    // d'une autre copie, et l'appel s'arrête au bout de 3 s ou si elle ne répond plus.
    let r = unsafe {
        SendMessageTimeoutW(
            fenetre,
            WM_COPYDATA,
            WPARAM(0),
            LPARAM(&raw const copie as isize),
            SMTO_ABORTIFHUNG,
            ATTENTE_TRANSMISSION,
            Some(&raw mut reponse),
        )
    };
    if r.0 == 0 {
        Err(ErreurUnique::PasDeReponse)
    } else if reponse == 0 {
        Err(ErreurUnique::ArgumentsNonPris)
    } else {
        Ok(())
    }
}

/// Lit les arguments d'un `WM_COPYDATA` reçu par la fenêtre cachée (`lparam`) : seulement ceux
/// qui portent la marque et qu'on sait décoder. La fenêtre répond 1 si `Some`, 0 sinon.
pub(crate) fn lire_arguments(lparam: isize) -> Option<Vec<String>> {
    // SAFETY: pour un WM_COPYDATA, `lparam` est l'adresse d'une COPYDATASTRUCT que Windows
    // garde valable pendant le traitement du message (ou 0, refusé par `as_ref`).
    let copie = unsafe { (lparam as *const COPYDATASTRUCT).as_ref() }?;
    let n = copie.cbData as usize;
    if !copie_a_lire(copie.dwData, n, copie.lpData.is_null()) {
        return None;
    }
    // SAFETY: Windows garantit `cbData` octets lisibles à `lpData` pendant le message ;
    // l'adresse n'est pas nulle et `n` est borné (vérifié juste au-dessus).
    let octets = unsafe { std::slice::from_raw_parts(copie.lpData.cast::<u8>(), n) };
    decoder_arguments(octets)
}
