//! Le décodage pur de l'icône (bloc M) : ce que Windows dit d'une icône ([`Message`],
//! [`FinDeSession`], [`Alimentation`]), à qui va un message de la fenêtre cachée
//! ([`destinataire`]), le GUID stable des icônes, les lignes du menu gris. Aucun appel à Windows
//! ici : tout est testé sans fenêtre (`essais_zone.rs`, `essais_horloge.rs`).

use windows::Win32::UI::WindowsAndMessaging::{
    PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, PBT_POWERSETTINGCHANGE, WM_APP, WM_CLOSE,
    WM_COPYDATA, WM_DESTROY, WM_DISPLAYCHANGE, WM_ENDSESSION, WM_POWERBROADCAST, WM_QUERYENDSESSION,
    WM_SETTINGCHANGE, WM_TIMER,
};

use crate::bords::unique::WM_OUVRIR_PARAMETRES;
use crate::clics;
use crate::place::Rect;

// Le nombre d'icônes à la fois, l'info-bulle et sa copie dans le tampon de Windows, le délai
// contre le clignotement : calcul pur, dans le cœur (bloc D), repris ici.
pub use ic_coeur::gestes::aiguilleur::ICONES_MAX;
pub use ic_coeur::gestes::icone::DELAI_ICONE;
pub use ic_coeur::gestes::icone::{INFOBULLE_MAX, raccourcir};
pub(crate) use ic_coeur::gestes::icone::{attente_icone, copier};

// ---------------------------------------------------------------------------------------
// Les messages de l'icône seule (`Horloge`)

/// Clic sur l'icône seule (son message de rappel).
pub(crate) const WM_ICONE: u32 = WM_APP + 1;
/// Une commande du logiciel attend dans la file.
pub(crate) const WM_COMMANDE: u32 = WM_APP + 4;
/// Le menu moderne ne peut pas s'ouvrir : le menu de secours à `lparam` (x, y).
pub(crate) const WM_MENU_DE_SECOURS: u32 = WM_APP + 6;
/// L'icône seule garde le numéro 1, sans GUID (C-09) : Windows la retrouve comme avant.
pub(crate) const ID_ICONE: u32 = 1;
/// La minuterie qui redessine l'icône quand un changement arrive trop tôt.
pub(crate) const MINUTERIE_ICONE: usize = 0x1C0E;

/// Un nombre tiré au hasard au lancement, connu de ce seul processus. Les messages internes le
/// portent dans `wparam` : un autre programme qui les posterait à la fenêtre cachée serait
/// ignoré.
pub(crate) fn jeton() -> usize {
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
pub(crate) fn message_permis(msg: u32, wp: usize) -> bool {
    match msg {
        WM_COMMANDE | WM_MENU_DE_SECOURS => wp == jeton(),
        _ => true,
    }
}

/// Qui reçoit un message de la fenêtre cachée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Destinataire {
    /// La bibliothèque seule (ses messages, la fermeture).
    Bibliotheque,
    /// Le logiciel le voit, puis la bibliothèque le traite.
    LesDeux,
    /// Le logiciel (sinon : traitement par défaut de Windows).
    Logiciel,
}

pub(crate) fn destinataire(msg: u32, wp: usize, taskbar_created: u32) -> Destinataire {
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

/// `GUID_CONSOLE_DISPLAY_STATE` : l'écran de la console allumé (1), éteint (0) ou assombri (2).
pub(crate) const ECRAN_DE_LA_CONSOLE: windows::core::GUID =
    windows::core::GUID::from_u128(0x6fe69556_704a_47a0_8f24_c28d936fda47);

/// Ce qu'un `WM_POWERBROADCAST` dit du réveil de Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alimentation {
    Veille,
    Reprise,
    Ecran { allume: bool },
}

/// Lit un `WM_POWERBROADCAST` : `pbt` (son `wParam`), et l'état de l'écran quand c'est un
/// changement de `GUID_CONSOLE_DISPLAY_STATE` (`None` sinon). Pur.
pub(crate) fn alimentation(pbt: u32, ecran: Option<u32>) -> Option<Alimentation> {
    match pbt {
        PBT_APMSUSPEND => Some(Alimentation::Veille),
        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => Some(Alimentation::Reprise),
        PBT_POWERSETTINGCHANGE => ecran.map(|e| Alimentation::Ecran { allume: e != 0 }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------
// Les messages des icônes (toutes)

/// Un bouton de la souris.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bouton {
    Gauche,
    Droit,
    Milieu,
}

/// Ce que Windows dit d'une icône (`NOTIFYICON_VERSION_4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Message {
    /// Un bouton appuyé.
    Appui(Bouton),
    /// Un bouton relâché.
    Relache(Bouton),
    /// Le second appui d'un double-clic (il remplace son `Appui`).
    DoubleClic(Bouton),
    /// `NIN_SELECT` : après chaque clic gauche, et Entrée sur une icône choisie à la souris.
    Selection,
    /// `NIN_KEYSELECT` : Entrée ou Espace sur l'icône choisie au clavier.
    Clavier,
    /// `WM_CONTEXTMENU` : après chaque clic droit, Maj+F10 ou la touche Menu.
    MenuContextuel,
    /// `NIN_BALLOONUSERCLICK` : un clic sur la notification de cette icône.
    Notification,
}

impl Message {
    /// Le message (mot bas du `lparam` du rappel de l'icône), s'il en est un qui compte. Le
    /// survol et l'ouverture des info-bulles n'en sont pas.
    pub fn de(mot_bas: u32) -> Option<Message> {
        const WM_CONTEXTMENU: u32 = 0x007B;
        const WM_LBUTTONDOWN: u32 = 0x0201;
        const WM_LBUTTONUP: u32 = 0x0202;
        const WM_LBUTTONDBLCLK: u32 = 0x0203;
        const WM_RBUTTONDOWN: u32 = 0x0204;
        const WM_RBUTTONUP: u32 = 0x0205;
        const WM_RBUTTONDBLCLK: u32 = 0x0206;
        const WM_MBUTTONDOWN: u32 = 0x0207;
        const WM_MBUTTONUP: u32 = 0x0208;
        const WM_MBUTTONDBLCLK: u32 = 0x0209;
        const NIN_SELECT: u32 = 0x0400;
        const NIN_KEYSELECT: u32 = 0x0401;
        const NIN_BALLOONUSERCLICK: u32 = 0x0405;
        use Bouton::*;
        Some(match mot_bas {
            WM_LBUTTONDOWN => Message::Appui(Gauche),
            WM_RBUTTONDOWN => Message::Appui(Droit),
            WM_MBUTTONDOWN => Message::Appui(Milieu),
            WM_LBUTTONUP => Message::Relache(Gauche),
            WM_RBUTTONUP => Message::Relache(Droit),
            WM_MBUTTONUP => Message::Relache(Milieu),
            WM_LBUTTONDBLCLK => Message::DoubleClic(Gauche),
            WM_RBUTTONDBLCLK => Message::DoubleClic(Droit),
            WM_MBUTTONDBLCLK => Message::DoubleClic(Milieu),
            NIN_SELECT => Message::Selection,
            NIN_KEYSELECT => Message::Clavier,
            WM_CONTEXTMENU => Message::MenuContextuel,
            NIN_BALLOONUSERCLICK => Message::Notification,
            _ => return None,
        })
    }

    /// Le même geste pour [`clics::Clics`] (volet, petit menu, Paramètres), s'il compte.
    pub fn geste(self) -> Option<clics::Evenement> {
        match self {
            Message::Selection => Some(clics::Evenement::Selection),
            Message::Clavier => Some(clics::Evenement::Clavier),
            Message::MenuContextuel => Some(clics::Evenement::MenuContextuel),
            Message::DoubleClic(Bouton::Gauche) => Some(clics::Evenement::DoubleClic),
            Message::Relache(Bouton::Gauche) => Some(clics::Evenement::Relache),
            _ => None,
        }
    }
}

/// Un geste sur une icône, ou un clic sur sa notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evenement {
    /// L'identifiant de l'icône, tel que donné à [`crate::zone::Zone::poser`].
    pub icone: String,
    pub message: Message,
    /// Le point du geste (pixels réels de l'écran) : la souris, ou l'icône au clavier.
    pub ancre: [i32; 2],
    /// L'icône (pixels réels de l'écran) ; à défaut, un petit carré autour de la souris.
    pub rect: Rect,
}

/// La fin de la session Windows (`WM_ENDSESSION` avec `wparam` vrai) : déconnexion, arrêt ou
/// redémarrage de l'ordinateur. **Windows ne laisse que quelques secondes** (selon le réglage
/// du système), puis arrête le programme sans prévenir, parfois dès que la procédure de la
/// fenêtre cachée a rendu la main : le logiciel doit enregistrer ce qui ne l'est pas encore
/// *pendant* le message.
///
/// Deux façons de la lire :
/// - **synchrone, à privilégier** : [`crate::zone::Zone::sur_fin_de_session`] donne une fonction appelée par
///   la procédure de la fenêtre cachée, sur le fil de la zone ; Windows attend son retour
///   (le programme peut être arrêté dès qu'elle rend la main) ;
/// - **après coup** : [`crate::zone::Zone::fin_de_session`], à lire après `DispatchMessageW`. Trop tard pour
///   enregistrer si la session se ferme vraiment : à garder pour un simple signal (arrêter une
///   boucle, ne plus rien proposer à l'utilisateur).
///
/// Un logiciel qui utilise `horloge::Horloge` reçoit le même message dans son récepteur
/// (`horloge::Recepteur`, appelé de façon synchrone) : [`FinDeSession::de`] le décode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FinDeSession {
    /// `ENDSESSION_LOGOFF` : l'utilisateur ferme sa session (et non un arrêt de l'ordinateur).
    pub deconnexion: bool,
    /// `ENDSESSION_CRITICAL` : la session se ferme tout de suite, quoi que fasse le logiciel
    /// (pas d'attente, pas de boîte de dialogue).
    pub critique: bool,
    /// `ENDSESSION_CLOSEAPP` : Windows ferme les programmes pour une mise à jour (le gestionnaire
    /// de redémarrage) ; la session, elle, continue.
    pub application: bool,
}

impl FinDeSession {
    /// Le message décodé : `Some` pour `WM_ENDSESSION` avec `wparam` vrai (la session se ferme
    /// pour de bon), `None` sinon (autre message, ou fermeture annulée : `wparam` faux).
    pub fn de(message: u32, wparam: usize, lparam: isize) -> Option<FinDeSession> {
        const ENDSESSION_CLOSEAPP: u32 = 0x0000_0001;
        const ENDSESSION_CRITICAL: u32 = 0x4000_0000;
        const ENDSESSION_LOGOFF: u32 = 0x8000_0000;
        if message != WM_ENDSESSION || wparam == 0 {
            return None;
        }
        let options = lparam as u32;
        Some(FinDeSession {
            deconnexion: options & ENDSESSION_LOGOFF != 0,
            critique: options & ENDSESSION_CRITICAL != 0,
            application: options & ENDSESSION_CLOSEAPP != 0,
        })
    }
}

/// Le GUID stable d'une icône : tiré de `graine` (propre au logiciel), du chemin du programme
/// (Windows refuse un GUID déjà pris par un autre chemin) et de l'identifiant de l'icône. Un
/// GUID de version 4 ordinaire.
pub fn guid_stable(graine: &str, programme: &str, icone: &str) -> u128 {
    let donnees = format!("{graine}|{}|{icone}", programme.to_lowercase());
    let mut h1: u64 = 0xcbf2_9ce4_8422_2325;
    let mut h2: u64 = 0x6c62_272e_07bb_0142;
    for b in donnees.bytes() {
        h1 = (h1 ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
        h2 = (h2 ^ u64::from(b))
            .wrapping_mul(0x0000_0100_0000_01B3)
            .rotate_left(5);
    }
    let mut g = (u128::from(h1) << 64) | u128::from(h2);
    // Bits de version 4 et de variante.
    g = (g & !(0xF000u128 << 64)) | (0x4000u128 << 64);
    g = (g & !(0xC000u128 << 48)) | (0x8000u128 << 48);
    g
}

/// Ajoute une icône à la zone, même si elle y est déjà. `ajout` : `NIM_ADD` ; `modif` :
/// `NIM_MODIFY` avec les mêmes données ; `version` : `NIM_SETVERSION` (version 4).
///
/// `NIM_ADD` échoue quand l'icône existe déjà : l'Explorateur redémarré (« TaskbarCreated »)
/// qui l'a gardée, ou une icône restée d'une instance précédente arrêtée brutalement. On la
/// modifie alors à la place (image, info-bulle, message de rappel), puis on redemande la
/// version 4, que Windows peut avoir perdue. Vrai si l'ajout ou la modification a réussi ;
/// faux si Windows refuse les deux (la version n'est alors pas redemandée).
pub(crate) fn ajouter_avec_repli(
    ajout: impl FnOnce() -> bool,
    modif: impl FnOnce() -> bool,
    version: impl FnOnce(),
) -> bool {
    if ajout() || modif() {
        version();
        true
    } else {
        false
    }
}

/// Une ligne du menu gris de Windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LigneMenu {
    /// Une ligne à choisir : `cle` revient si elle est choisie. `actif: false` : grisée
    /// (une ligne d'état, par exemple).
    Choix {
        cle: String,
        texte: String,
        coche: bool,
        actif: bool,
    },
    /// Un sous-menu.
    Sous {
        texte: String,
        actif: bool,
        lignes: Vec<LigneMenu>,
    },
    /// Un trait.
    Trait,
}

impl LigneMenu {
    /// Une ligne à choisir, sans case.
    pub fn choix(cle: &str, texte: &str) -> LigneMenu {
        LigneMenu::Choix {
            cle: cle.into(),
            texte: texte.into(),
            coche: false,
            actif: true,
        }
    }

    /// Une ligne grisée (un état, pas une action).
    pub fn etat(texte: &str) -> LigneMenu {
        LigneMenu::Choix {
            cle: String::new(),
            texte: texte.into(),
            coche: false,
            actif: false,
        }
    }
}

/// « & » marque un raccourci clavier dans les menus de Windows : doublé pour s'afficher.
pub fn texte_menu(texte: &str) -> String {
    texte.replace('&', "&&")
}

/// Les clés des lignes à choisir, dans l'ordre des numéros donnés à Windows (à partir de 1).
pub(crate) fn cles(lignes: &[LigneMenu], dest: &mut Vec<String>) {
    for l in lignes {
        match l {
            LigneMenu::Choix { cle, .. } => dest.push(cle.clone()),
            LigneMenu::Sous { lignes, .. } => cles(lignes, dest),
            LigneMenu::Trait => {}
        }
    }
}
