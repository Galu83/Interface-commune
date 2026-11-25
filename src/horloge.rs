//! L'icône près de l'horloge et le fil principal (règles : interface Windows 11, point 10) :
//! une fenêtre cachée reçoit les gestes sur l'icône, les notifications cliquées, les
//! changements de thème et le redémarrage de l'Explorateur ; elle envoie au fil des fenêtres
//! le volet, le petit menu ou les Paramètres. Une seule instance par session : relancer le
//! logiciel l'ouvre (sa fenêtre principale s'il en a une, sinon ses Paramètres).
//!
//! Utilisation, sur le fil principal du logiciel :
//!
//! ```no_run
//! # fn exemple(logiciel: std::sync::Arc<dyn interface_commune::Logiciel>) -> Result<(), String> {
//! use interface_commune::horloge::{Horloge, instance_unique};
//! let _verrou = match instance_unique("MonLogiciel") {
//!     Ok(v) => v,
//!     Err(_) => return Ok(()), // déjà lancé : il s'ouvre
//! };
//! let horloge = Horloge::new("MonLogiciel");
//! let poste = horloge.poste(); // à garder : notifications, info-bulle, quitter
//! horloge.lancer(logiciel, None) // rend la main quand le logiciel se ferme
//! # }
//! ```
//!
//! Chaque `unsafe` est commenté.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, POINT, WPARAM,
};
use windows::Win32::System::DataExchange::COPYDATASTRUCT;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForSystem, GetSystemMetricsForDpi,
    SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_ERROR, NIIF_INFO, NIIF_RESPECT_QUIET_TIME,
    NIIF_WARNING, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION, NOTIFYICON_VERSION_4, NOTIFYICONDATAW,
    NOTIFYICONIDENTIFIER, Shell_NotifyIconGetRect, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DEVICE_NOTIFY_WINDOW_HANDLE, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, DispatchMessageW, FindWindowW, GWLP_USERDATA, GetCursorPos, GetMessageW,
    GetWindowLongPtrW, HICON, KillTimer, MF_SEPARATOR, MF_STRING, MSG, PBT_APMRESUMEAUTOMATIC,
    PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, PBT_POWERSETTINGCHANGE, PostMessageW, PostQuitMessage,
    RegisterClassW, RegisterWindowMessageW, SM_CXSMICON, SMTO_ABORTIFHUNG, SendMessageTimeoutW,
    SetForegroundWindow, SetMenuDefaultItem, SetTimer, SetWindowLongPtrW, TPM_BOTTOMALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WM_APP, WM_CLOSE, WM_COPYDATA,
    WM_DESTROY, WM_DISPLAYCHANGE, WM_ENDSESSION, WM_NULL, WM_POWERBROADCAST, WM_QUERYENDSESSION,
    WM_SETTINGCHANGE, WM_TIMER, WNDCLASSW, WS_OVERLAPPED,
};
use windows::core::{HSTRING, PCWSTR};

use crate::Logiciel;
use crate::clics::{Clics, Suite, evenement, suite};
use crate::fenetres::{self, Aiguillage, Demande, Liens, Ouvreur, Reception};
use crate::menu::{self, Choix};
use crate::place::Rect;
use crate::systeme::{choix_windows, hwnd, reveiller, temps_double_clic};
use crate::theme::Surface;

/// Clic sur l'icône.
const WM_ICONE: u32 = WM_APP + 1;
/// Une deuxième instance demande les Paramètres.
const WM_OUVRIR_PARAMETRES: u32 = WM_APP + 3;
/// Une commande du logiciel attend dans la file.
const WM_COMMANDE: u32 = WM_APP + 4;
/// Le menu moderne ne peut pas s'ouvrir : le menu de secours à `lparam` (x, y).
const WM_MENU_DE_SECOURS: u32 = WM_APP + 6;
const ID_ICONE: u32 = 1;
/// La minuterie qui redessine l'icône quand un changement arrive trop tôt.
const MINUTERIE_ICONE: usize = 0x1C0E;
/// Au plus un changement d'icône par seconde (contre le clignotement).
pub const DELAI_ICONE: Duration = Duration::from_secs(1);

/// Le temps à attendre avant de changer l'icône ; `None` : tout de suite.
fn attente_icone(derniere: Option<Instant>, maintenant: Instant) -> Option<Duration> {
    let passe = maintenant.checked_duration_since(derniere?)?;
    DELAI_ICONE.checked_sub(passe).filter(|d| !d.is_zero())
}
/// Un clic sur une notification (`NIN_BALLOONUSERCLICK`).
const NIN_BALLOONUSERCLICK: u32 = 0x405;

/// Un nombre tiré au hasard au lancement, connu de ce seul processus. Les messages internes le
/// portent dans `wparam` : un autre programme qui les posterait à la fenêtre cachée serait
/// ignoré.
fn jeton() -> usize {
    use std::hash::{BuildHasher, Hasher};
    static JETON: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *JETON.get_or_init(|| {
        // RandomState est semé par le système (nombres aléatoires de Windows).
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u64(u64::from(std::process::id()));
        (h.finish() as usize) | 1
    })
}

/// Ce que la fenêtre cachée accepte : les messages internes doivent porter le jeton.
fn message_permis(msg: u32, wp: usize) -> bool {
    match msg {
        WM_COMMANDE | WM_MENU_DE_SECOURS => wp == jeton(),
        _ => true,
    }
}

/// Le nom de la classe de la fenêtre cachée (une deuxième instance la cherche par ce nom).
fn classe(nom: &str) -> HSTRING {
    HSTRING::from(format!("{nom}-principal"))
}

/// Avant toute fenêtre : chaque écran à sa propre échelle (sinon Windows étire tout).
pub fn reglage_dpi() {
    // SAFETY: simple appel sans pointeur ; un échec (déjà réglé) est sans conséquence.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// Le verrou « une seule instance », gardé jusqu'à la fin du programme.
pub struct Unique(HANDLE);

impl Drop for Unique {
    fn drop(&mut self) {
        // SAFETY: poignée créée par `instance_unique`, fermée une seule fois.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

// Le contrat (bloc A), repris du cœur à ce chemin.
pub use ic_coeur::contrat::{Niveau, Reveil};
// Le guichet (bloc K), repris des bords à ce chemin.
pub use crate::bords::poste::{AuClic, Poste};
use crate::bords::poste::{Commande, ReveilParMessage};

/// Le nombre d'arguments qu'un second lancement peut transmettre.
pub const ARGUMENTS_MAX: usize = 16;
/// La taille de ce qu'un second lancement peut transmettre (octets, tous arguments ensemble).
pub const ARGUMENTS_OCTETS_MAX: usize = 4096;
/// Le temps que le second lancement attend l'instance en cours (au-delà, la demande est
/// perdue : un second lancement ne reste jamais accroché).
const ATTENTE_TRANSMISSION: u32 = 3000;
/// Le `dwData` d'un `WM_COPYDATA` d'arguments : tout autre est ignoré.
const MARQUE_ARGUMENTS: usize = 0x4943_4152;

/// Les arguments en octets : chacun en UTF-8, suivi d'un zéro. `None` s'il y en a trop, s'ils
/// sont trop gros, ou si l'un contient un zéro.
fn coder_arguments(args: &[String]) -> Option<Vec<u8>> {
    if args.len() > ARGUMENTS_MAX {
        return None;
    }
    let mut octets = Vec::new();
    for a in args {
        if a.contains('\0') {
            return None;
        }
        octets.extend_from_slice(a.as_bytes());
        octets.push(0);
    }
    (octets.len() <= ARGUMENTS_OCTETS_MAX).then_some(octets)
}

/// L'inverse de [`coder_arguments`] ; `None` pour des octets qui ne viennent pas de lui
/// (trop longs, trop d'arguments, UTF-8 invalide, zéro final absent).
fn decoder_arguments(octets: &[u8]) -> Option<Vec<String>> {
    if octets.len() > ARGUMENTS_OCTETS_MAX {
        return None;
    }
    if octets.is_empty() {
        return Some(Vec::new());
    }
    let (corps, fin) = octets.split_at(octets.len() - 1);
    if fin != [0] {
        return None;
    }
    let args: Vec<String> = corps
        .split(|o| *o == 0)
        .map(|a| std::str::from_utf8(a).map(str::to_string))
        .collect::<Result<_, _>>()
        .ok()?;
    (args.len() <= ARGUMENTS_MAX).then_some(args)
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
    let verrou = HSTRING::from(format!("Local\\{nom}-une-seule"));
    // SAFETY: nom terminé par zéro, vivant pendant l'appel ; la poignée va dans `Unique`.
    let h = unsafe { CreateMutexW(None, false, &verrou) };
    // SAFETY: lecture de la dernière erreur du fil, juste après l'appel.
    let deja = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    match h {
        Ok(h) if !deja => Ok(Unique(h)),
        autre => {
            if let Ok(h) = autre {
                drop(Unique(h));
            }
            if *reveil == Reveil::Rien {
                return Err(true);
            }
            let c = classe(nom);
            // SAFETY: nom de classe vivant pendant l'appel ; on ne fait que poster un message
            // à la fenêtre trouvée.
            let Ok(f) = (unsafe { FindWindowW(&c, PCWSTR::null()) }) else {
                return Err(false);
            };
            match reveil {
                Reveil::Parametres => {
                    // SAFETY: fenêtre trouvée juste avant ; PostMessageW n'attend rien.
                    let poste = unsafe { PostMessageW(Some(f), WM_OUVRIR_PARAMETRES, WPARAM(0), LPARAM(0)) };
                    Err(poste.is_ok())
                }
                Reveil::Arguments(args) => Err(transmettre_arguments(f, args)),
                Reveil::Rien => Err(true),
            }
        }
    }
}

/// Envoie les arguments à la fenêtre cachée de l'instance en cours (`WM_COPYDATA`, qui copie
/// les octets dans l'autre processus). Vrai si elle les a pris.
fn transmettre_arguments(fenetre: HWND, args: &[String]) -> bool {
    let Some(octets) = coder_arguments(args) else {
        return false;
    };
    let copie = COPYDATASTRUCT {
        dwData: MARQUE_ARGUMENTS,
        cbData: octets.len() as u32,
        lpData: octets.as_ptr().cast_mut().cast(),
    };
    let mut reponse = 0usize;
    // SAFETY: `copie` et `octets` vivent pendant tout l'appel (envoi synchrone : Windows
    // copie les octets avant de rendre la main) ; la fenêtre est celle d'une autre instance,
    // et l'appel s'arrête au bout de 3 s ou si elle ne répond plus.
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
    r.0 != 0 && reponse != 0
}

/// Lit les arguments d'un `WM_COPYDATA` reçu (`lparam`) : seulement ceux qui portent la marque
/// et qu'on sait décoder.
fn lire_arguments(lparam: isize) -> Option<Vec<String>> {
    // SAFETY: pour un WM_COPYDATA, `lparam` est l'adresse d'une COPYDATASTRUCT que Windows
    // garde valable pendant le traitement du message (ou 0, refusé ici).
    let copie = unsafe { (lparam as *const COPYDATASTRUCT).as_ref() }?;
    if copie.dwData != MARQUE_ARGUMENTS || copie.lpData.is_null() {
        return None;
    }
    let n = copie.cbData as usize;
    if n > ARGUMENTS_OCTETS_MAX {
        return None;
    }
    // SAFETY: Windows garantit `cbData` octets lisibles à `lpData` pendant le message ;
    // `n` est borné ci-dessus.
    let octets = unsafe { std::slice::from_raw_parts(copie.lpData.cast::<u8>(), n) };
    decoder_arguments(octets)
}

/// Un message de Windows reçu par la fenêtre cachée du logiciel (celle de l'icône).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageWindows {
    /// La fenêtre cachée (son adresse).
    pub fenetre: isize,
    pub message: u32,
    pub wparam: usize,
    pub lparam: isize,
}

/// Ce que le logiciel fait des messages de Windows de la fenêtre cachée (`WM_HOTKEY`, ses
/// messages à lui, `WM_POWERBROADCAST`…), sur le fil de l'icône. `Some(résultat)` : traité ;
/// `None` : Windows fait le traitement par défaut. Il voit aussi, sans pouvoir les arrêter,
/// `WM_QUERYENDSESSION`, `WM_ENDSESSION`, `WM_SETTINGCHANGE`, `WM_DISPLAYCHANGE` et
/// `WM_POWERBROADCAST` (veille, reprise, écran allumé ou éteint). Il ne voit jamais les
/// messages propres à la bibliothèque.
///
/// **Fin de session** : le récepteur est appelé de façon synchrone, avant que la bibliothèque
/// ferme ses fenêtres. C'est là qu'enregistrer, sans attendre : Windows ne laisse que quelques
/// secondes. `crate::zone::FinDeSession::de(message, wparam, lparam)` décode `WM_ENDSESSION`
/// (`Some` seulement si la session se ferme vraiment).
pub type Recepteur = Box<dyn FnMut(&MessageWindows) -> Option<isize>>;

/// Ce que le logiciel fait des arguments d'un second lancement ([`Reveil::Arguments`]), sur le
/// fil de l'icône : à rendre vite (poster un message, noter un ordre), le second lancement
/// attend jusqu'à 3 s.
pub type RecepteurArguments = Box<dyn FnMut(Vec<String>)>;

/// Qui reçoit un message de la fenêtre cachée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Destinataire {
    /// La bibliothèque seule (ses messages, la fermeture).
    Bibliotheque,
    /// Le logiciel le voit, puis la bibliothèque le traite.
    LesDeux,
    /// Le logiciel (sinon : traitement par défaut de Windows).
    Logiciel,
}

/// `GUID_CONSOLE_DISPLAY_STATE` : l'écran de la console allumé (1), éteint (0) ou assombri (2).
const ECRAN_DE_LA_CONSOLE: windows::core::GUID =
    windows::core::GUID::from_u128(0x6fe69556_704a_47a0_8f24_c28d936fda47);

/// Ce qu'un `WM_POWERBROADCAST` dit du réveil de Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alimentation {
    Veille,
    Reprise,
    Ecran { allume: bool },
}

/// Lit un `WM_POWERBROADCAST` : `pbt` (son `wParam`), et l'état de l'écran quand c'est un
/// changement de `GUID_CONSOLE_DISPLAY_STATE` (`None` sinon). Pur.
fn alimentation(pbt: u32, ecran: Option<u32>) -> Option<Alimentation> {
    match pbt {
        PBT_APMSUSPEND => Some(Alimentation::Veille),
        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => Some(Alimentation::Reprise),
        PBT_POWERSETTINGCHANGE => ecran.map(|e| Alimentation::Ecran { allume: e != 0 }),
        _ => None,
    }
}

/// Transmet un `WM_POWERBROADCAST` à [`crate::reprise`].
///
/// # Safety
/// `lp` est le `lParam` du message reçu à l'instant (pour PBT_POWERSETTINGCHANGE, un
/// POWERBROADCAST_SETTING que Windows garde valable pendant le message).
unsafe fn signaler_l_alimentation(wp: usize, lp: isize) {
    let pbt = wp as u32;
    let ecran = if pbt == PBT_POWERSETTINGCHANGE && lp != 0 {
        // SAFETY: voir la fonction ; on ne lit le DWORD de `Data` qu'après avoir vérifié le
        // GUID et la longueur annoncée.
        unsafe {
            let r = lp as *const windows::Win32::System::Power::POWERBROADCAST_SETTING;
            let ps = std::ptr::addr_of!((*r).PowerSetting).read_unaligned();
            let long = std::ptr::addr_of!((*r).DataLength).read_unaligned();
            (ps == ECRAN_DE_LA_CONSOLE && long >= 4)
                .then(|| std::ptr::addr_of!((*r).Data).cast::<u32>().read_unaligned())
        }
    } else {
        None
    };
    match alimentation(pbt, ecran) {
        Some(Alimentation::Veille) => crate::reprise::veille(),
        Some(Alimentation::Reprise) => crate::reprise::reveil(),
        Some(Alimentation::Ecran { allume }) => crate::reprise::ecran(allume),
        None => {}
    }
}

fn destinataire(msg: u32, wp: usize, taskbar_created: u32) -> Destinataire {
    match msg {
        WM_TIMER if wp == MINUTERIE_ICONE => Destinataire::Bibliotheque,
        WM_ICONE | WM_COMMANDE | WM_OUVRIR_PARAMETRES | WM_MENU_DE_SECOURS | WM_COPYDATA | WM_CLOSE
        | WM_DESTROY => Destinataire::Bibliotheque,
        m if m == taskbar_created && m != 0 => Destinataire::Bibliotheque,
        WM_QUERYENDSESSION | WM_ENDSESSION | WM_SETTINGCHANGE | WM_DISPLAYCHANGE | WM_POWERBROADCAST => {
            Destinataire::LesDeux
        }
        _ => Destinataire::Logiciel,
    }
}

/// L'icône près de l'horloge, avant son lancement.
pub struct Horloge {
    nom: String,
    poste: Poste,
    reception: Reception,
    messages: Option<Recepteur>,
    arguments: Option<RecepteurArguments>,
    sans_fantome: bool,
}

impl Horloge {
    /// `nom` : le nom court du logiciel, le même que pour [`instance_unique`].
    pub fn new(nom: &str) -> Horloge {
        let (demandes, reception) = fenetres::canal();
        let liens = Arc::new(Liens::default());
        // Le guichet réveille la fenêtre cachée par son message des commandes, avec son jeton.
        let reveil = Arc::new(ReveilParMessage {
            liens: liens.clone(),
            message: WM_COMMANDE,
            jeton: jeton(),
        });
        Horloge {
            nom: nom.into(),
            poste: Poste::nouveau(liens, demandes, reveil),
            reception,
            messages: None,
            arguments: None,
            sans_fantome: false,
        }
    }

    /// Windows ne remplace jamais une fenêtre du logiciel par son « fantôme » (cadre gris,
    /// « Ne répond pas ») quand elle est occupée un moment : voir
    /// [`crate::systeme::sans_fantome`]. À choisir pour un logiciel dont les fenêtres peuvent
    /// être retenues par un travail long, qu'on ne veut pas voir arrêter de force.
    pub fn sans_fantome(mut self) -> Horloge {
        self.sans_fantome = true;
        self
    }

    /// Reçoit les arguments qu'un second lancement transmet ([`Reveil::Arguments`] de
    /// [`instance_unique_avec`]), par exemple `--choisir=2`. Sans récepteur, ils sont ignorés.
    pub fn recevoir_arguments(mut self, f: impl FnMut(Vec<String>) + 'static) -> Horloge {
        self.arguments = Some(Box::new(f));
        self
    }

    /// Les messages de Windows du logiciel passent par la boucle de l'icône (une seule boucle
    /// pour tout le programme) : voir [`Recepteur`].
    pub fn recevoir_messages(mut self, f: impl FnMut(&MessageWindows) -> Option<isize> + 'static) -> Horloge {
        self.messages = Some(Box::new(f));
        self
    }

    pub fn poste(&self) -> Poste {
        self.poste.clone()
    }

    /// Lance le fil des fenêtres (le volet préparé au repos), pose l'icône et traite les
    /// messages jusqu'à « Quitter ». `parametres` : les Paramètres à ouvrir tout de suite
    /// (`--parametres <page>` : `Some(page)`).
    pub fn lancer(
        self,
        logiciel: Arc<dyn Logiciel>,
        parametres: Option<Option<String>>,
    ) -> Result<(), String> {
        reglage_dpi();
        let Horloge {
            nom,
            poste,
            reception,
            messages,
            arguments,
            sans_fantome,
        } = self;
        if sans_fantome {
            crate::systeme::sans_fantome();
        }
        if let Some(page) = parametres {
            let _ = poste.demandes().send(Demande::Parametres(page));
        }
        // La sonde des fils écrit dans le journal du logiciel (clics, boucles imbriquées, blocages).
        let pour_la_sonde = logiciel.clone();
        crate::sonde::brancher_le_journal(move |ligne| pour_la_sonde.noter(ligne));
        let fil = lancer_le_fil_des_fenetres(
            logiciel.clone(),
            poste.liens().clone(),
            reception,
            secours_de_l_horloge(poste.liens().clone()),
        )?;
        let pour_noter = logiciel.clone();
        let r = boucle(&nom, logiciel, poste.clone(), messages, arguments);
        poste.liens().tout_fermer();
        let _ = poste.demandes().send(Demande::Quitter);
        if !attendre_la_fin(fil, ATTENTE_FERMETURE) {
            pour_noter.noter("Une fenêtre ne répond plus : le logiciel se ferme sans l'attendre.".into());
        }
        r
    }
}

/// Le temps qu'une fenêtre qui ne répond plus peut retenir la fermeture du logiciel.
pub const ATTENTE_FERMETURE: Duration = Duration::from_secs(5);

/// Attend la fin d'un fil au plus `delai` ; rend faux (et le laisse partir seul) s'il n'a pas
/// fini : le programme se ferme quand même, rien ne le retient.
pub(crate) fn attendre_la_fin(fil: std::thread::JoinHandle<()>, delai: Duration) -> bool {
    let fin = Instant::now() + delai;
    while !fil.is_finished() {
        if Instant::now() >= fin {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = fil.join();
    true
}

// ---------------------------------------------------------------------------------------
// Le fil des fenêtres

/// Le menu de secours de Windows, demandé par le fil des fenêtres (voir
/// [`lancer_le_fil_des_fenetres`]).
pub type Secours = Box<dyn FnMut(&Demande) + Send>;

/// Les vraies fenêtres (egui).
struct Fenetres {
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    polices: egui::FontDefinitions,
    secours: Secours,
}

impl Ouvreur for Fenetres {
    fn surface(
        &mut self,
        demandes: &Reception,
        premiere: Option<Demande>,
    ) -> Result<Option<Demande>, (String, Option<Demande>)> {
        crate::volet::veiller(
            self.logiciel.clone(),
            self.liens.clone(),
            self.polices.clone(),
            demandes,
            premiere,
        )
    }

    fn parametres(&mut self, page: Option<String>) -> Result<(), String> {
        crate::parametres::fenetre::ouvrir_seuls(
            self.logiciel.clone(),
            Some(self.liens.clone()),
            page,
            self.polices.clone(),
        )
    }

    fn secours(&mut self, d: &Demande) {
        (self.secours)(d);
    }

    fn noter(&mut self, ligne: String) {
        self.logiciel.noter(ligne);
    }
}

/// Le menu de secours de l'icône unique : demandé à la fenêtre cachée du fil principal, qui
/// l'ouvre au point du clic.
fn secours_de_l_horloge(liens: Arc<Liens>) -> Secours {
    Box::new(move |d: &Demande| {
        let point = match d {
            Demande::Volet { icone, .. } => [icone[0], icone[1]],
            Demande::Menu { ancre, .. } => *ancre,
            _ => return,
        };
        let lp = (i64::from(point[0]) & 0xFFFF) | ((i64::from(point[1]) & 0xFFFF) << 16);
        let h = liens.fenetre.load(Ordering::Acquire);
        if h != 0 {
            // SAFETY: comme `ReveilParMessage::reveiller` (le guichet).
            let _ = unsafe {
                PostMessageW(
                    Some(hwnd(h)),
                    WM_MENU_DE_SECOURS,
                    WPARAM(jeton()),
                    LPARAM(lp as isize),
                )
            };
        }
    })
}

/// Lance **le fil des fenêtres de la bibliothèque** : la fenêtre du volet et du petit menu
/// préparée au repos (`volet::veiller`), les Paramètres et les autres fenêtres dans sa
/// boucle, une fenêtre perdue recréée, la garde du fil ; jusqu'à `Demande::Quitter` (ou
/// [`Liens::tout_fermer`]). Les demandes arrivent par `reception` (l'autre bout de
/// [`fenetres::canal`]). C'est le fil que [`Horloge::lancer`] lance ; un logiciel qui garde
/// sa propre boucle de messages (plusieurs icônes, `zone`) le lance aussi, par
/// `zone::FenetresCommunes`.
///
/// `secours` : le menu gris de Windows à la place du volet ou du petit menu de cette demande
/// (les fenêtres modernes sont impossibles, ou perdues en rafale). Il est appelé **sur le fil
/// des fenêtres** : il doit seulement transmettre la demande au fil de l'icône (un message
/// posté à sa fenêtre), jamais ouvrir le menu lui-même.
pub fn lancer_le_fil_des_fenetres(
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    reception: Reception,
    secours: Secours,
) -> Result<std::thread::JoinHandle<()>, String> {
    std::thread::Builder::new()
        .name("fenêtres".into())
        .spawn(move || fil_des_fenetres(logiciel, liens, reception, secours))
        .map_err(|e| e.to_string())
}

fn fil_des_fenetres(logiciel: Arc<dyn Logiciel>, liens: Arc<Liens>, reception: Reception, secours: Secours) {
    // Le lieu d'une panique est gardé pour le journal (la boucle des fenêtres l'attrape et
    // recrée la fenêtre) ; si le fil s'arrête quand même, l'icône passe au menu de secours.
    crate::sonde::retenir_les_paniques();
    let pour_la_garde = logiciel.clone();
    let _garde = fenetres::GardeDuFil::new(liens.clone(), move |t| pour_la_garde.noter(t));
    // Surveillé : une image demandée le réveille s'il dort (le volet caché ne tourne qu'à la demande).
    let pour_reveiller = liens.clone();
    let _surveille = crate::sonde::inscrire(
        "fenêtres",
        Some(Box::new(move || {
            if let Ok(c) = pour_reveiller.contexte.try_lock()
                && let Some(c) = c.as_ref()
            {
                c.request_repaint();
            }
        })),
    );
    match crate::polices::polices() {
        Ok(polices) => {
            let mut f = Fenetres {
                logiciel,
                liens: liens.clone(),
                polices,
                secours,
            };
            fenetres::boucle_des_fenetres(&liens, &mut f, &reception);
        }
        Err(e) => {
            logiciel.noter(format!("Interface : {e} ; menu de secours de Windows."));
            liens.egui_impossible.store(true, Ordering::Release);
            let mut f = Fenetres {
                logiciel,
                liens: liens.clone(),
                polices: egui::FontDefinitions::empty(),
                secours,
            };
            // Sans polices : seul le menu de secours (fil principal).
            while let Some(d) = reception.recevoir() {
                if d == Demande::Quitter {
                    break;
                }
                f.secours(&d);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// L'icône

/// Fabrique l'icône du logiciel à la taille des petites icônes.
fn fabriquer_icone(logiciel: &dyn Logiciel, barre_sombre: bool) -> Option<HICON> {
    // SAFETY: simples lectures de réglages de Windows.
    let taille = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem()) }.clamp(16, 64);
    let px = logiciel
        .icone_etat(taille as u32, barre_sombre)
        .unwrap_or_else(|| (logiciel.identite().icone)(taille as u32, barre_sombre));
    crate::zone::icone_windows(taille, &px)
}

use crate::zone::copier;

fn donnees_icone(h: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: h,
        uID: ID_ICONE,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------------------
// Le fil principal

struct Principal {
    logiciel: Arc<dyn Logiciel>,
    poste: Poste,
    fenetre: HWND,
    taskbar_created: u32,
    icone: Option<HICON>,
    barre_sombre: bool,
    infobulle: String,
    clics: Clics,
    /// Ce que fait un clic sur la dernière notification.
    au_clic: AuClic,
    /// Les messages de Windows du logiciel.
    messages: Option<Recepteur>,
    /// Les arguments d'un second lancement.
    arguments: Option<RecepteurArguments>,
    /// Le dernier changement d'icône, et un changement qui attend la fin du délai.
    icone_changee: Option<Instant>,
    icone_attendue: bool,
}

/// L'état du fil principal, rangé dans la fenêtre cachée (`GWLP_USERDATA`) : aucune
/// variable globale. Vide pendant la création de la fenêtre.
type Etat = RefCell<Option<Principal>>;

impl Principal {
    fn liens(&self) -> &Liens {
        self.poste.liens()
    }

    fn ajouter_icone(&mut self) {
        let mut d = donnees_icone(self.fenetre);
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
        d.uCallbackMessage = WM_ICONE;
        d.hIcon = self.icone.unwrap_or_default();
        copier(&mut d.szTip, &self.infobulle);
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        crate::sonde::etape("Shell_NotifyIcon (ajout de l'icône)");
        // Icône déjà là (Explorateur redémarré, reste d'une instance précédente) : modifiée à
        // la place, comme dans `zone`.
        crate::zone::ajouter_avec_repli(
            // SAFETY (les trois appels) : `d` est local, complet et de la taille annoncée.
            || unsafe { Shell_NotifyIconW(NIM_ADD, &d) }.as_bool(),
            || unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) }.as_bool(),
            || {
                let _ = unsafe { Shell_NotifyIconW(NIM_SETVERSION, &d) };
            },
        );
    }

    fn modifier_icone(&self) {
        let mut d = donnees_icone(self.fenetre);
        d.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        d.hIcon = self.icone.unwrap_or_default();
        copier(&mut d.szTip, &self.infobulle);
        crate::sonde::etape("Shell_NotifyIcon (changement de l'icône)");
        // SAFETY: comme ci-dessus.
        let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) };
    }

    fn redessiner_icone(&mut self, toujours: bool) {
        let barre = choix_windows().barre_sombre;
        if !toujours && self.icone.is_some() && barre == self.barre_sombre {
            return;
        }
        self.barre_sombre = barre;
        let ancienne = self.icone.take();
        self.icone = fabriquer_icone(&*self.logiciel, barre);
        self.modifier_icone();
        if let Some(a) = ancienne {
            // SAFETY: l'ancienne icône n'est plus utilisée (remplacée juste avant).
            let _ = unsafe { DestroyIcon(a) };
        }
    }

    fn commandes(&mut self) {
        for c in self.poste.commandes() {
            match c {
                Commande::Notifier {
                    texte,
                    niveau,
                    au_clic,
                } => {
                    self.au_clic = au_clic;
                    let mut d = donnees_icone(self.fenetre);
                    d.uFlags = NIF_INFO;
                    copier(&mut d.szInfoTitle, &self.logiciel.identite().nom);
                    copier(&mut d.szInfo, &texte);
                    d.dwInfoFlags = match niveau {
                        Niveau::Information => NIIF_INFO,
                        Niveau::Avertissement => NIIF_WARNING,
                        Niveau::Erreur => NIIF_ERROR,
                    } | NIIF_RESPECT_QUIET_TIME;
                    crate::sonde::etape("Shell_NotifyIcon (notification)");
                    // SAFETY: comme `ajouter_icone`.
                    let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) };
                }
                Commande::Infobulle(t) => {
                    self.infobulle = t;
                    self.modifier_icone();
                }
                Commande::Icone => self.icone_demandee(),
                Commande::Executer(f) => f(self.fenetre.0 as isize),
                Commande::Quitter => {
                    self.fermer();
                    return;
                }
            }
        }
    }

    /// Le rectangle de l'icône (pixels) ; à défaut, un petit carré autour de la souris.
    fn rect_icone(&self) -> Rect {
        let id = NOTIFYICONIDENTIFIER {
            cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
            hWnd: self.fenetre,
            uID: ID_ICONE,
            ..Default::default()
        };
        // SAFETY: `id` local et complet ; repli sur la position de la souris (POINT local).
        unsafe { Shell_NotifyIconGetRect(&id) }.ok().map_or_else(
            || {
                let mut p = POINT::default();
                let _ = unsafe { GetCursorPos(&mut p) };
                [p.x - 8, p.y - 8, p.x + 8, p.y + 8]
            },
            |r| [r.left, r.top, r.right, r.bottom],
        )
    }

    fn geste(&mut self, e: crate::clics::Evenement, ancre: [i32; 2]) {
        crate::sonde::etape("geste sur l'icône");
        crate::sonde::clic(match e {
            crate::clics::Evenement::Selection => "gauche",
            crate::clics::Evenement::Clavier => "clavier",
            crate::clics::Evenement::MenuContextuel => "droit",
            crate::clics::Evenement::DoubleClic => "double",
            crate::clics::Evenement::Relache => "relâché",
        });
        let clic = Instant::now();
        let g = self.clics.recevoir(e, clic, temps_double_clic());
        match suite(g, self.logiciel.fenetre_principale()) {
            Suite::Rien => {}
            Suite::Volet => self.ouvrir(Surface::Volet, None, clic),
            Suite::Menu => self.ouvrir(Surface::Menu, Some(ancre), clic),
            // Double-clic : le volet ouvert par le premier clic lit la demande des
            // Paramètres, se cache, puis laisse la place.
            Suite::Parametres => self.ouvrir_parametres(None),
            Suite::FenetrePrincipale => self.ouvrir_fenetre_principale(),
        }
    }

    /// La fenêtre principale du logiciel : le volet ou le petit menu ouvert par le premier
    /// clic d'un double-clic se ferme, pour lui laisser la place.
    fn ouvrir_fenetre_principale(&self) {
        if matches!(self.liens().ouvert(), fenetres::Ouvert::Surface(_)) {
            self.liens().fermer_la_fenetre();
        }
        self.logiciel.ouvrir_fenetre_principale();
    }

    /// Ouvre le logiciel : sa fenêtre principale s'il en a une, sinon ses Paramètres.
    fn ouvrir_le_logiciel(&self) {
        if self.logiciel.fenetre_principale().is_some() {
            self.ouvrir_fenetre_principale();
        } else {
            self.ouvrir_parametres(None);
        }
    }

    /// Ouvre le volet ou le petit menu ; un nouveau clic quand il est ouvert le referme.
    fn ouvrir(&mut self, surface: Surface, ancre: Option<[i32; 2]>, clic: Instant) {
        let icone = self.rect_icone();
        let deja_impossible = self.liens().egui_impossible.load(Ordering::Acquire);
        // Les Paramètres ouverts ne bloquent pas le clic : le volet ou le petit menu s'ouvre
        // devant eux, qui restent ouverts derrière.
        match fenetres::envoyer_le_clic(self.liens(), self.poste.demandes(), surface, icone, ancre, clic) {
            Aiguillage::Rien => {}
            // La fenêtre prête se montre tout de suite, encore invisible.
            Aiguillage::Envoye => reveiller(self.liens().surface_prete.load(Ordering::Acquire)),
            Aiguillage::Secours => {
                if !deja_impossible {
                    // Le fil des fenêtres n'existe plus : le menu de secours, pour que le
                    // clic ouvre quand même quelque chose.
                    self.logiciel.noter(
                        "Clic sur l'icône : le fil des fenêtres ne répond plus ; menu de secours de Windows."
                            .into(),
                    );
                }
                let [x, y] = ancre.unwrap_or([icone[0], icone[1]]);
                self.menu_de_secours(x, y);
            }
        }
    }

    fn ouvrir_parametres(&self, page: Option<String>) {
        self.liens().montrer_page(self.poste.demandes(), page);
    }

    /// L'icône doit changer : tout de suite, ou à la fin du délai contre le clignotement.
    fn icone_demandee(&mut self) {
        if self.icone_attendue {
            return;
        }
        match attente_icone(self.icone_changee, Instant::now()) {
            None => self.changer_icone(),
            Some(d) => {
                self.icone_attendue = true;
                // SAFETY: minuterie de la fenêtre de ce fil ; WM_TIMER la rappelle.
                unsafe {
                    SetTimer(
                        Some(self.fenetre),
                        MINUTERIE_ICONE,
                        d.as_millis().max(1) as u32,
                        None,
                    )
                };
            }
        }
    }

    fn changer_icone(&mut self) {
        self.icone_changee = Some(Instant::now());
        self.redessiner_icone(true);
    }

    /// Passe un message au logiciel ; `Some(résultat)` s'il l'a traité.
    fn transmettre(&mut self, msg: u32, wp: usize, lp: isize) -> Option<isize> {
        let m = MessageWindows {
            fenetre: self.fenetre.0 as isize,
            message: msg,
            wparam: wp,
            lparam: lp,
        };
        self.messages.as_mut().and_then(|f| f(&m))
    }

    /// Un clic sur la dernière notification.
    fn clic_bulle(&self) {
        match self.au_clic.clone() {
            AuClic::Parametres(page) => self.ouvrir_parametres(page),
            AuClic::Action(cle) => self.logiciel.action(&cle),
        }
    }

    /// Le menu gris de Windows : seulement en secours, si le menu moderne ne peut pas
    /// s'ouvrir (carte graphique sans OpenGL, polices absentes). Les mêmes lignes, dans le
    /// même ordre.
    fn menu_de_secours(&mut self, x: i32, y: i32) {
        let l = &self.logiciel;
        let nom = l.fenetre_principale().map(|_| l.identite().nom.clone());
        let lignes = menu::lignes_avec(l.textes(), nom.as_deref(), &l.actions_menu(), &l.lignes_en_plus());
        // SAFETY: menu créé, rempli de chaînes vivantes pendant chaque appel, affiché puis
        // détruit ici ; la fenêtre est celle de ce fil.
        let choix = unsafe {
            let Ok(m) = CreatePopupMenu() else { return };
            for (i, l) in lignes.iter().enumerate() {
                match l {
                    None => {
                        let _ = AppendMenuW(m, MF_SEPARATOR, 0, PCWSTR::null());
                    }
                    Some(l) => {
                        let _ = AppendMenuW(m, MF_STRING, i + 1, &HSTRING::from(l.texte.as_str()));
                        if l.quoi == Choix::Ouvrir {
                            // « Ouvrir <logiciel> » en gras : la ligne par défaut du menu.
                            let _ = SetMenuDefaultItem(m, (i + 1) as u32, 0);
                        }
                    }
                }
            }
            let _ = SetForegroundWindow(self.fenetre);
            let _imbriquee = crate::sonde::imbriquee("TrackPopupMenu (menu de secours)");
            let c = TrackPopupMenu(
                m,
                TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                x,
                y,
                None,
                self.fenetre,
                None,
            )
            .0 as usize;
            let _ = DestroyMenu(m);
            c
        };
        let Some(Some(l)) = choix.checked_sub(1).and_then(|i| lignes.get(i)) else {
            return;
        };
        match &l.quoi {
            Choix::Action(cle) => self.logiciel.action(cle),
            Choix::Ouvrir => self.ouvrir_fenetre_principale(),
            Choix::Parametres => self.ouvrir_parametres(None),
            Choix::RechercherMiseAJour => {
                self.logiciel.rechercher_mise_a_jour();
                self.ouvrir_parametres(Some(crate::parametres::modele::MISE_A_JOUR.into()));
            }
            Choix::Quitter => self.logiciel.quitter(),
        }
    }

    fn fermer(&mut self) {
        let d = donnees_icone(self.fenetre);
        // SAFETY: retire l'icône de cette fenêtre, puis détruit l'icône et la fenêtre.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &d);
            if let Some(i) = self.icone.take() {
                let _ = DestroyIcon(i);
            }
            let _ = DestroyWindow(self.fenetre);
            // WM_DESTROY arrive pendant que l'état est emprunté : finir ici.
            PostQuitMessage(0);
        }
    }
}

extern "system" fn procedure(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // L'état est rangé dans la fenêtre ; pendant sa création, il n'y est pas encore (ou
    // encore vide) : traitement par défaut.
    // SAFETY: la valeur rangée est 0 ou l'adresse de l'état de `boucle`, qui vit jusqu'à ce
    // que `boucle` la remette à 0 ; les messages arrivent sur ce même fil.
    let etat = unsafe { GetWindowLongPtrW(h, GWLP_USERDATA) } as *const Etat;
    let mut resultat = 0isize;
    // SAFETY: voir ci-dessus.
    let traite = unsafe { etat.as_ref() }.is_some_and(|p| {
        let Ok(mut p) = p.try_borrow_mut() else {
            return false;
        };
        let Some(p) = p.as_mut() else { return false };
        if !message_permis(msg, wp.0) {
            // Message interne sans le jeton : venu d'un autre programme, ignoré.
            return true;
        }
        match destinataire(msg, wp.0, p.taskbar_created) {
            Destinataire::Bibliotheque => {}
            Destinataire::LesDeux => {
                let _ = p.transmettre(msg, wp.0, lp.0);
            }
            Destinataire::Logiciel => {
                return match p.transmettre(msg, wp.0, lp.0) {
                    Some(r) => {
                        resultat = r;
                        true
                    }
                    None => false,
                };
            }
        }
        match msg {
            WM_ICONE => {
                if (lp.0 as u32) & 0xFFFF == NIN_BALLOONUSERCLICK {
                    p.clic_bulle();
                } else if let Some(e) = evenement(lp.0) {
                    // Avec NOTIFYICON_VERSION_4, `wparam` porte le point du geste (x, y).
                    let ancre = [
                        i32::from(wp.0 as u16 as i16),
                        i32::from((wp.0 >> 16) as u16 as i16),
                    ];
                    p.geste(e, ancre);
                }
                true
            }
            WM_COMMANDE => {
                p.commandes();
                true
            }
            WM_TIMER => {
                // La minuterie de l'icône (les autres vont au logiciel).
                // SAFETY: minuterie de cette fenêtre.
                let _ = unsafe { KillTimer(Some(p.fenetre), MINUTERIE_ICONE) };
                p.icone_attendue = false;
                p.changer_icone();
                true
            }
            WM_OUVRIR_PARAMETRES => {
                // Relancé alors qu'il tourne déjà : sa fenêtre principale, sinon ses
                // Paramètres.
                p.ouvrir_le_logiciel();
                true
            }
            WM_COPYDATA => {
                // Les arguments d'un second lancement (marque, taille et UTF-8 vérifiés).
                match lire_arguments(lp.0) {
                    Some(args) => {
                        if let Some(f) = p.arguments.as_mut() {
                            f(args);
                        }
                        resultat = 1;
                    }
                    None => resultat = 0,
                }
                true
            }
            WM_MENU_DE_SECOURS => {
                let x = (lp.0 & 0xFFFF) as i16;
                let y = ((lp.0 >> 16) & 0xFFFF) as i16;
                p.menu_de_secours(i32::from(x), i32::from(y));
                true
            }
            WM_SETTINGCHANGE => {
                p.redessiner_icone(false);
                false
            }
            WM_DISPLAYCHANGE => {
                // Windows refait l'affichage : les fenêtres pas encore créées attendent la fin
                // de la reprise (`autres_fenetres::REPRISE`).
                crate::autres_fenetres::affichage_change();
                false
            }
            WM_POWERBROADCAST => {
                // Veille, reprise, écran allumé ou éteint : le dessin des fenêtres attend la fin
                // du réveil de Windows (`crate::reprise`).
                // SAFETY: pour PBT_POWERSETTINGCHANGE, Windows passe dans `lp` un
                // POWERBROADCAST_SETTING valable pendant ce message (lu par `signaler_l_alimentation`).
                unsafe { signaler_l_alimentation(wp.0, lp.0) };
                false
            }
            WM_ENDSESSION => {
                if wp.0 != 0 {
                    // La session se ferme : le programme peut être arrêté dès le retour.
                    p.logiciel.noter("Windows ferme la session.".into());
                    p.liens().tout_fermer();
                    p.fermer();
                }
                true
            }
            WM_CLOSE => {
                p.fermer();
                true
            }
            WM_DESTROY => {
                // SAFETY: fin de la boucle des messages de ce fil.
                unsafe { PostQuitMessage(0) };
                true
            }
            m if m == p.taskbar_created && m != 0 => {
                // L'Explorateur a redémarré : remettre l'icône.
                p.ajouter_icone();
                true
            }
            _ => false,
        }
    });
    if traite {
        LRESULT(resultat)
    } else {
        // SAFETY: traitement par défaut de Windows, avec les paramètres reçus.
        unsafe { DefWindowProcW(h, msg, wp, lp) }
    }
}

/// Crée la fenêtre cachée et l'icône, puis traite les messages jusqu'à « Quitter ».
fn boucle(
    nom: &str,
    logiciel: Arc<dyn Logiciel>,
    poste: Poste,
    messages: Option<Recepteur>,
    arguments: Option<RecepteurArguments>,
) -> Result<(), String> {
    let c = classe(nom);
    let titre = HSTRING::from(nom);
    // SAFETY: module de ce programme ; classe décrite par une structure locale dont les
    // chaînes vivent jusqu'à la fin de la fonction ; sa procédure vit tout le programme ;
    // fenêtre de style simple, jamais montrée.
    let fenetre = unsafe {
        let instance = GetModuleHandleW(PCWSTR::null()).map_err(|e| e.to_string())?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(c.as_ptr()),
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            return Err("classe de fenêtre refusée".into());
        }
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            &c,
            &titre,
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .map_err(|e| e.to_string())?
    };
    poste.liens().fenetre.store(fenetre.0 as isize, Ordering::Release);
    // L'écran allumé ou éteint, par WM_POWERBROADCAST (Windows envoie l'état tout de suite,
    // puis à chaque changement) : pendant un écran éteint, les fenêtres ne dessinent pas
    // (`crate::reprise`). L'inscription vit autant que la fenêtre. Sans elle (refusée), seules
    // la veille et la reprise comptent.
    // SAFETY: fenêtre de ce fil, qui vient d'être créée ; GUID constant.
    let _ = unsafe {
        windows::Win32::System::Power::RegisterPowerSettingNotification(
            HANDLE(fenetre.0),
            &ECRAN_DE_LA_CONSOLE,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        )
    };
    // Surveillé : sa boucle de messages bat à chaque message ; un message vide (WM_NULL) posté par
    // la surveillance le tire de son attente.
    let h_reveil = fenetre.0 as isize;
    let _surveille = crate::sonde::inscrire(
        "icône",
        Some(Box::new(move || {
            // SAFETY: PostMessageW n'attend rien et accepte une fenêtre déjà détruite (il échoue).
            let _ = unsafe { PostMessageW(Some(hwnd(h_reveil)), WM_NULL, WPARAM(0), LPARAM(0)) };
        })),
    );
    // SAFETY: nom constant ; « TaskbarCreated » est diffusé quand l'Explorateur redémarre.
    let taskbar_created = unsafe { RegisterWindowMessageW(windows::core::w!("TaskbarCreated")) };
    let mut p = Principal {
        logiciel: logiciel.clone(),
        poste,
        fenetre,
        taskbar_created,
        icone: None,
        barre_sombre: choix_windows().barre_sombre,
        infobulle: nom.into(),
        clics: Clics::default(),
        au_clic: AuClic::default(),
        messages,
        arguments,
        icone_changee: None,
        icone_attendue: false,
    };
    p.icone = fabriquer_icone(&*logiciel, p.barre_sombre);
    p.ajouter_icone();
    let etat: Box<Etat> = Box::new(RefCell::new(Some(p)));
    // SAFETY: la fenêtre est celle de ce fil ; l'adresse reste valable jusqu'à la remise à
    // 0 ci-dessous, avant que `etat` disparaisse.
    unsafe { SetWindowLongPtrW(fenetre, GWLP_USERDATA, &*etat as *const Etat as isize) };
    let mut m = MSG::default();
    // SAFETY: boucle des messages classique de ce fil, sur un MSG local.
    unsafe {
        crate::sonde::etape("attente d'un message");
        while GetMessageW(&mut m, None, 0, 0).as_bool() {
            crate::sonde::battement();
            crate::sonde::etape("traitement d'un message");
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
            crate::sonde::etape("attente d'un message");
        }
    }
    // SAFETY: plus aucun message ne lira l'état (fenêtre détruite, ou sans état désormais).
    unsafe { SetWindowLongPtrW(fenetre, GWLP_USERDATA, 0) };
    drop(etat);
    Ok(())
}

#[cfg(test)]
mod essais {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Veille, reprise et écran : ce que la bibliothèque lit d'un `WM_POWERBROADCAST`.
    #[test]
    fn veille_reprise_et_ecran() {
        assert_eq!(alimentation(PBT_APMSUSPEND, None), Some(Alimentation::Veille));
        assert_eq!(
            alimentation(PBT_APMRESUMEAUTOMATIC, None),
            Some(Alimentation::Reprise)
        );
        assert_eq!(
            alimentation(PBT_APMRESUMESUSPEND, None),
            Some(Alimentation::Reprise)
        );
        assert_eq!(
            alimentation(PBT_POWERSETTINGCHANGE, Some(0)),
            Some(Alimentation::Ecran { allume: false })
        );
        assert_eq!(
            alimentation(PBT_POWERSETTINGCHANGE, Some(1)),
            Some(Alimentation::Ecran { allume: true })
        );
        // Assombri : allumé.
        assert_eq!(
            alimentation(PBT_POWERSETTINGCHANGE, Some(2)),
            Some(Alimentation::Ecran { allume: true })
        );
        // Un autre réglage, ou un autre événement : rien.
        assert_eq!(alimentation(PBT_POWERSETTINGCHANGE, None), None);
        assert_eq!(alimentation(10, None), None);
    }

    /// Le GUID de l'écran est bien celui de Windows.
    #[test]
    fn guid_de_l_ecran() {
        assert_eq!(
            ECRAN_DE_LA_CONSOLE,
            windows::core::GUID::from_u128(0x6fe69556_704a_47a0_8f24_c28d936fda47)
        );
    }

    /// Les messages de Windows : ceux de la bibliothèque restent à elle ; la fin de session et
    /// les changements de réglages sont vus des deux ; tous les autres (raccourcis globaux,
    /// messages du logiciel) vont au logiciel.
    #[test]
    fn messages_de_windows_au_logiciel() {
        use windows::Win32::UI::WindowsAndMessaging::WM_HOTKEY;
        let barre = 0xC123;
        for m in [
            WM_ICONE,
            WM_COMMANDE,
            WM_OUVRIR_PARAMETRES,
            WM_MENU_DE_SECOURS,
            WM_COPYDATA,
            WM_CLOSE,
            WM_DESTROY,
            barre,
        ] {
            assert_eq!(destinataire(m, 0, barre), Destinataire::Bibliotheque, "{m:#x}");
        }
        for m in [
            WM_QUERYENDSESSION,
            WM_ENDSESSION,
            WM_SETTINGCHANGE,
            WM_DISPLAYCHANGE,
            WM_POWERBROADCAST,
        ] {
            assert_eq!(destinataire(m, 0, barre), Destinataire::LesDeux, "{m:#x}");
        }
        for m in [WM_HOTKEY, WM_APP + 100] {
            assert_eq!(destinataire(m, 0, barre), Destinataire::Logiciel, "{m:#x}");
        }
        // La minuterie de l'icône est à la bibliothèque ; celles du logiciel, à lui.
        assert_eq!(
            destinataire(WM_TIMER, MINUTERIE_ICONE, barre),
            Destinataire::Bibliotheque
        );
        assert_eq!(destinataire(WM_TIMER, 7, barre), Destinataire::Logiciel);
        // Avant « TaskbarCreated » (0 : pas encore connu), 0 n'est pas un message de la barre.
        assert_eq!(destinataire(0, 0, 0), Destinataire::Logiciel);
    }

    /// Les arguments d'un second lancement font l'aller-retour ; tout ce qui ne vient pas de
    /// `coder_arguments` est refusé.
    #[test]
    fn arguments_du_second_lancement() {
        let args: Vec<String> = ["--choisir=2", "un deux", "é"].map(String::from).into();
        let octets = coder_arguments(&args).unwrap();
        assert_eq!(decoder_arguments(&octets), Some(args));
        assert_eq!(coder_arguments(&[]).as_deref(), Some(&[][..]));
        assert_eq!(decoder_arguments(&[]), Some(vec![]));
        // Trop, trop gros, un zéro dedans.
        let trop: Vec<String> = (0..=ARGUMENTS_MAX).map(|i| i.to_string()).collect();
        assert_eq!(coder_arguments(&trop), None);
        assert_eq!(coder_arguments(&["x".repeat(ARGUMENTS_OCTETS_MAX)]), None);
        assert_eq!(coder_arguments(&["a\0b".into()]), None);
        // Reçu : pas de zéro final, UTF-8 invalide, trop long, trop d'arguments.
        assert_eq!(decoder_arguments(b"abc"), None);
        assert_eq!(decoder_arguments(&[0xFF, 0]), None);
        assert_eq!(decoder_arguments(&vec![b'a'; ARGUMENTS_OCTETS_MAX + 1]), None);
        assert_eq!(decoder_arguments(&[0u8; ARGUMENTS_MAX + 1]), None);
    }

    /// Un `WM_COPYDATA` sans la marque, vide ou mal fait n'est pas lu ; avec elle, il l'est.
    #[test]
    fn copie_de_donnees_lue_avec_prudence() {
        let octets = coder_arguments(&["--choisir=3".to_string()]).unwrap();
        let mut copie = COPYDATASTRUCT {
            dwData: MARQUE_ARGUMENTS,
            cbData: octets.len() as u32,
            lpData: octets.as_ptr().cast_mut().cast(),
        };
        assert_eq!(
            lire_arguments(&raw const copie as isize),
            Some(vec!["--choisir=3".to_string()])
        );
        copie.dwData = 7;
        assert_eq!(lire_arguments(&raw const copie as isize), None);
        copie.dwData = MARQUE_ARGUMENTS;
        copie.cbData = (ARGUMENTS_OCTETS_MAX + 1) as u32;
        assert_eq!(lire_arguments(&raw const copie as isize), None);
        copie.cbData = 1;
        copie.lpData = std::ptr::null_mut();
        assert_eq!(lire_arguments(&raw const copie as isize), None);
        assert_eq!(lire_arguments(0), None);
    }

    /// Une fenêtre qui ne répond plus ne retient pas la fermeture au-delà du délai.
    #[test]
    fn fenetre_bloquee_ne_retient_pas_la_fermeture() {
        let (lacher, attend) = std::sync::mpsc::channel::<()>();
        let bloque = std::thread::spawn(move || {
            let _ = attend.recv();
        });
        let debut = Instant::now();
        assert!(!attendre_la_fin(bloque, Duration::from_millis(150)));
        let duree = debut.elapsed();
        assert!(
            duree >= Duration::from_millis(150) && duree < Duration::from_secs(3),
            "{duree:?}"
        );
        let _ = lacher.send(());
        // Un fil qui finit vite est attendu et joint.
        let vite = std::thread::spawn(|| ());
        assert!(attendre_la_fin(vite, Duration::from_secs(5)));
        assert_eq!(ATTENTE_FERMETURE, Duration::from_secs(5));
    }

    /// Un second lancement ne fait rien quand c'est ce qu'on lui a dit, et un seul logiciel
    /// à la fois tient le verrou.
    #[test]
    fn second_lancement_sans_rien_dire() {
        let nom = format!("essai-instance-{}", std::process::id());
        let premier = instance_unique_avec(&nom, &Reveil::Rien).unwrap();
        // Le verrou est pris : le second est prévenu qu'il n'a rien à faire (sans fenêtre à
        // chercher, donc sans rien dire à personne).
        assert!(matches!(instance_unique_avec(&nom, &Reveil::Rien), Err(true)));
        // Sans fenêtre cachée à l'autre bout, les arguments ne partent pas.
        assert!(matches!(
            instance_unique_avec(&nom, &Reveil::Arguments(vec!["--x".into()])),
            Err(false)
        ));
        drop(premier);
        assert!(instance_unique_avec(&nom, &Reveil::Rien).is_ok());
    }

    /// Au plus un changement d'icône par seconde : le suivant attend la fin du délai.
    #[test]
    fn icone_contre_le_clignotement() {
        let t = Instant::now();
        assert_eq!(attente_icone(None, t), None);
        assert_eq!(
            attente_icone(Some(t), t + Duration::from_millis(300)),
            Some(Duration::from_millis(700))
        );
        assert_eq!(attente_icone(Some(t), t + DELAI_ICONE), None);
        assert_eq!(attente_icone(Some(t), t + Duration::from_secs(5)), None);
    }
}
