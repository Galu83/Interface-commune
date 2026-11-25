//! Plusieurs icônes près de l'horloge, pour un logiciel qui garde **sa propre boucle de
//! messages** (règles : interface Windows 11, point 10). Une fenêtre cachée, créée sur le fil
//! qui appelle [`Zone::creer`], reçoit les messages des icônes ; la boucle du logiciel
//! (`GetMessageW` / `DispatchMessageW`) les lui donne, puis le logiciel lit
//! [`Zone::evenements`].
//!
//! Chaque icône a :
//! - un **identifiant stable** choisi par le logiciel (« souris-1234 », « principale ») : son
//!   GUID en est tiré (avec le chemin du programme), pour que Windows retienne d'une fois sur
//!   l'autre si l'utilisateur la montre dans la barre des tâches ;
//! - son image et son info-bulle, changées en marche ;
//! - ses propres notifications.
//!
//! Elle peut être ajoutée ou retirée à tout moment. Quand l'Explorateur redémarre, la zone
//! remet seule toutes les icônes. Quand Windows change l'affichage (`WM_DISPLAYCHANGE`), elle
//! le signale à [`crate::autres_fenetres::affichage_change`] : les fenêtres pas encore créées
//! attendent alors 1 s.
//!
//! Si une icône existe déjà quand la zone l'ajoute (Explorateur redémarré, reste d'une
//! instance arrêtée brutalement), elle est modifiée à la place : l'ajout ne se perd pas.
//!
//! Quand la session Windows se ferme (déconnexion, arrêt), le logiciel n'a que quelques secondes
//! pour enregistrer : [`Zone::sur_fin_de_session`] lui donne la main **pendant** le message
//! (voir [`FinDeSession`]).
//!
//! Un clic dit **quelle icône** et **où elle est** ([`Evenement`]). Le menu gris de Windows
//! ([`Zone::menu_windows`]), avec cases cochées et sous-menus, sert de secours quand les
//! fenêtres modernes ne peuvent pas s'ouvrir.
//!
//! Un logiciel qui n'a qu'une icône et laisse la bibliothèque tout faire prend plutôt
//! `horloge::Horloge`.
//!
//! **Le volet, le petit menu et les Paramètres communs** pour ces icônes :
//! [`FenetresCommunes`] lance le fil des fenêtres de la bibliothèque (le même que celui de
//! `Horloge`) ; chaque geste lu par [`Zone::evenements`] lui est donné
//! ([`FenetresCommunes::geste`]) et passe par les mécanismes communs (`fenetres::Demandes`,
//! `fenetres::Liens`) : clic gauche, le volet ; clic droit, le petit menu ; double-clic, les
//! Paramètres ; un second clic referme ; rien ne se rouvre juste après une fermeture.
//!
//! ```no_run
//! use interface_commune::zone::{Image, Zone};
//! let mut zone = Zone::creer("MonLogiciel", "MonLogiciel").map_err(|e| e.to_string())?;
//! let image = Image { taille: 16, bgra: vec![0; 16 * 16 * 4] };
//! zone.poser("souris", &image, "Souris — 80 %");
//! // Dans la boucle de messages du logiciel, après DispatchMessageW :
//! for e in zone.evenements() {
//!     println!("{} : {:?} à {:?}", e.icone, e.message, e.rect);
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! Chaque `unsafe` est commenté.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS, DeleteObject,
    HGDIOBJ,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    NIF_GUID, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_ERROR, NIIF_INFO,
    NIIF_RESPECT_QUIET_TIME, NIIF_WARNING, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFYICON_VERSION_4, NOTIFYICONDATAW, NOTIFYICONIDENTIFIER, Shell_NotifyIconGetRect, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, GWLP_USERDATA, GetCursorPos, GetWindowLongPtrW, HICON, HMENU, ICONINFO,
    MENU_ITEM_FLAGS, MF_CHECKED, MF_GRAYED, MF_POPUP, MF_SEPARATOR, MF_STRING, PostMessageW, RegisterClassW,
    RegisterWindowMessageW, SetForegroundWindow, SetWindowLongPtrW, TPM_BOTTOMALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TrackPopupMenu, WINDOW_EX_STYLE, WM_APP, WM_DISPLAYCHANGE, WM_ENDSESSION, WM_NULL,
    WNDCLASSW, WS_OVERLAPPED,
};
use windows::core::{GUID, HSTRING, PCWSTR};

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::Logiciel;
use crate::clics;
use crate::fenetres::{self, Aiguillage, Demande, Demandes, Liens, envoyer_le_clic};
use crate::horloge::{Niveau, Secours};
use crate::place::Rect;
use ic_coeur::gestes::aiguilleur::{
    Ordre, SuiviDesIcones, fermer_avant_la_fenetre_principale, ordre, point_de_secours,
};

/// Le premier message de rappel des icônes : chaque icône a le sien (`PREMIER_MESSAGE + n`),
/// ce qui dit sans erreur de quelle icône vient un geste.
const PREMIER_MESSAGE: u32 = WM_APP + 0x200;
// Le nombre d'icônes à la fois, l'info-bulle et sa copie dans le tampon de Windows : calcul
// pur, dans le cœur (bloc D), repris ici à leurs chemins d'avant.
pub use ic_coeur::gestes::aiguilleur::ICONES_MAX;
pub(crate) use ic_coeur::gestes::icone::copier;
pub use ic_coeur::gestes::icone::{INFOBULLE_MAX, raccourcir};

// ---------------------------------------------------------------------------------------
// Calcul pur

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
    /// L'identifiant de l'icône, tel que donné à [`Zone::poser`].
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
/// - **synchrone, à privilégier** : [`Zone::sur_fin_de_session`] donne une fonction appelée par
///   la procédure de la fenêtre cachée, sur le fil de la zone ; Windows attend son retour
///   (le programme peut être arrêté dès qu'elle rend la main) ;
/// - **après coup** : [`Zone::fin_de_session`], à lire après `DispatchMessageW`. Trop tard pour
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
fn cles(lignes: &[LigneMenu], dest: &mut Vec<String>) {
    for l in lignes {
        match l {
            LigneMenu::Choix { cle, .. } => dest.push(cle.clone()),
            LigneMenu::Sous { lignes, .. } => cles(lignes, dest),
            LigneMenu::Trait => {}
        }
    }
}

// ---------------------------------------------------------------------------------------
// Les images

/// Une image d'icône : `taille` × `taille` pixels en BGRA (alpha non prémultiplié), lignes
/// de haut en bas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub taille: u32,
    pub bgra: Vec<u8>,
}

impl Image {
    /// Depuis des pixels RVBA (l'ordre de la plupart des bibliothèques d'images).
    pub fn depuis_rvba(taille: u32, mut rvba: Vec<u8>) -> Image {
        for p in rvba.as_chunks_mut::<4>().0 {
            p.swap(0, 2);
        }
        Image { taille, bgra: rvba }
    }
}

/// Fait une icône de Windows depuis des pixels BGRA (`taille`² × 4 octets).
pub(crate) fn icone_windows(taille: i32, px: &[u8]) -> Option<HICON> {
    if taille <= 0 || px.len() != (taille as usize) * (taille as usize) * 4 {
        return None;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: taille,
            // Négatif : lignes de haut en bas.
            biHeight: -taille,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    // SAFETY: `info` décrit une image 32 bits de `taille`² pixels ; Windows alloue `bits`
    // (taille × taille × 4 octets), où l'on copie exactement `px` (taille vérifiée plus
    // haut). Les deux images sont détruites après CreateIconIndirect, qui en fait sa copie.
    unsafe {
        let couleur = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        if bits.is_null() {
            let _ = DeleteObject(HGDIOBJ(couleur.0));
            return None;
        }
        std::ptr::copy_nonoverlapping(px.as_ptr(), bits.cast::<u8>(), px.len());
        let masque_octets = vec![0u8; (taille as usize).div_ceil(16) * 2 * taille as usize];
        let masque = CreateBitmap(taille, taille, 1, 1, Some(masque_octets.as_ptr().cast()));
        let ii = ICONINFO {
            fIcon: true.into(),
            hbmMask: masque,
            hbmColor: couleur,
            ..Default::default()
        };
        let icone = CreateIconIndirect(&ii).ok();
        let _ = DeleteObject(HGDIOBJ(couleur.0));
        let _ = DeleteObject(HGDIOBJ(masque.0));
        icone
    }
}

// ---------------------------------------------------------------------------------------
// La zone

/// Une icône posée.
struct Posee {
    guid: u128,
    message: u32,
    hicon: HICON,
    infobulle: String,
    /// Windows a accepté l'icône (sinon : réessayée au prochain `poser`).
    dans_la_barre: bool,
}

/// Ce que le logiciel fait de la fin de session ([`Zone::sur_fin_de_session`]).
type SurFinDeSession = Box<dyn FnMut(&FinDeSession)>;

/// Ce que partagent la zone et la procédure de sa fenêtre (même fil).
#[derive(Default)]
struct Partage {
    evenements: VecDeque<Evenement>,
    /// Le message de rappel de chaque icône → son identifiant et son GUID.
    rappels: BTreeMap<u32, (String, u128)>,
    taskbar_created: u32,
    /// L'Explorateur a redémarré : toutes les icônes sont à remettre.
    a_remettre: bool,
    /// La session se ferme (lu par [`Zone::fin_de_session`]).
    fin_de_session: Option<FinDeSession>,
    /// Ce que le logiciel fait tout de suite de la fin de session ([`Zone::sur_fin_de_session`]).
    sur_fin_de_session: Option<SurFinDeSession>,
}

/// Plusieurs icônes près de l'horloge, sur le fil qui l'a créée (voir le module).
pub struct Zone {
    fenetre: HWND,
    graine: String,
    programme: String,
    icones: BTreeMap<String, Posee>,
    partage: Box<RefCell<Partage>>,
}

impl Zone {
    /// Crée la fenêtre cachée des icônes sur ce fil. `nom` : le nom court du logiciel (sa
    /// classe de fenêtre en est tirée) ; `graine` : ce qui rend ses GUID uniques (gardée d'une
    /// version à l'autre, pour que Windows retienne les icônes montrées).
    pub fn creer(nom: &str, graine: &str) -> Result<Zone, String> {
        let classe = HSTRING::from(format!("{nom}-icones"));
        let programme = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        // SAFETY: module de ce programme ; classe décrite par une structure locale dont les
        // chaînes vivent pendant les appels ; sa procédure vit tout le programme ; fenêtre de
        // style simple, jamais montrée (pas `HWND_MESSAGE` : elle doit recevoir
        // « TaskbarCreated », diffusé aux seules fenêtres ordinaires).
        let fenetre = unsafe {
            let instance = GetModuleHandleW(PCWSTR::null()).map_err(|e| e.to_string())?;
            let wc = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(classe.as_ptr()),
                ..Default::default()
            };
            // Déjà enregistrée (une deuxième zone dans le même programme) : sans conséquence.
            let _ = RegisterClassW(&wc);
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                &classe,
                &classe,
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
        let partage = Box::new(RefCell::new(Partage {
            // SAFETY: nom constant ; « TaskbarCreated » est diffusé quand l'Explorateur
            // redémarre.
            taskbar_created: unsafe { RegisterWindowMessageW(windows::core::w!("TaskbarCreated")) },
            ..Default::default()
        }));
        // SAFETY: fenêtre de ce fil ; l'adresse de `partage` (dans une boîte, donc fixe)
        // reste valable jusqu'à `Drop`, qui la remet à 0 avant de détruire la fenêtre.
        unsafe {
            SetWindowLongPtrW(
                fenetre,
                GWLP_USERDATA,
                &*partage as *const RefCell<Partage> as isize,
            )
        };
        Ok(Zone {
            fenetre,
            graine: graine.into(),
            programme,
            icones: BTreeMap::new(),
            partage,
        })
    }

    /// La fenêtre cachée (son adresse) : pour y poster des messages, par exemple.
    pub fn fenetre(&self) -> isize {
        self.fenetre.0 as isize
    }

    /// Les identifiants des icônes posées.
    pub fn icones(&self) -> impl Iterator<Item = &str> {
        self.icones.keys().map(String::as_str)
    }

    /// Le GUID de l'icône `id` (le même d'un lancement à l'autre).
    pub fn guid(&self, id: &str) -> u128 {
        guid_stable(&self.graine, &self.programme, id)
    }

    fn donnees(&self, guid: u128) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.fenetre,
            uFlags: NIF_GUID,
            guidItem: GUID::from_u128(guid),
            ..Default::default()
        }
    }

    /// Ajoute l'icône `id`, ou change son image et son info-bulle. Rend faux si Windows l'a
    /// refusée (l'Explorateur redémarre…) : la redemander plus tard.
    pub fn poser(&mut self, id: &str, image: &Image, infobulle: &str) -> bool {
        self.remettre_si_besoin();
        let Some(hicon) = icone_windows(image.taille as i32, &image.bgra) else {
            return false;
        };
        let infobulle = raccourcir(infobulle, INFOBULLE_MAX);
        if let Some(p) = self.icones.get(id) {
            let mut d = self.donnees(p.guid);
            d.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
            d.hIcon = hicon;
            copier(&mut d.szTip, &infobulle);
            let ok = if p.dans_la_barre {
                // SAFETY: `d` est local, complet et de la taille annoncée.
                unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) }.as_bool()
            } else {
                self.ajouter(&mut d, p.message)
            };
            let ancienne = match self.icones.get_mut(id) {
                Some(p) => {
                    p.dans_la_barre = ok;
                    p.infobulle = infobulle;
                    std::mem::replace(&mut p.hicon, hicon)
                }
                None => hicon,
            };
            // SAFETY: l'ancienne icône n'est plus utilisée (remplacée juste avant).
            let _ = unsafe { DestroyIcon(ancienne) };
            return ok;
        }
        let Some(message) = self.message_libre() else {
            // SAFETY: icône créée ci-dessus, jamais donnée à Windows.
            let _ = unsafe { DestroyIcon(hicon) };
            return false;
        };
        let guid = self.guid(id);
        // Une icône restée d'un programme arrêté brutalement a le même GUID : retirée avant.
        let reste = self.donnees(guid);
        // SAFETY: comme ci-dessus ; échoue sans conséquence s'il n'y a rien à retirer.
        let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &reste) };
        let mut d = self.donnees(guid);
        d.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        d.hIcon = hicon;
        copier(&mut d.szTip, &infobulle);
        let ok = self.ajouter(&mut d, message);
        if let Ok(mut p) = self.partage.try_borrow_mut() {
            p.rappels.insert(message, (id.to_string(), guid));
        }
        self.icones.insert(
            id.to_string(),
            Posee {
                guid,
                message,
                hicon,
                infobulle,
                dans_la_barre: ok,
            },
        );
        ok
    }

    /// `NIM_ADD` puis la version 4 (gestes complets, point du geste dans `wparam`) ; si
    /// l'icône existe déjà, `NIM_MODIFY` à la place (voir [`ajouter_avec_repli`]).
    fn ajouter(&self, d: &mut NOTIFYICONDATAW, message: u32) -> bool {
        d.uFlags |= NIF_MESSAGE;
        d.uCallbackMessage = message;
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let d: &NOTIFYICONDATAW = d;
        ajouter_avec_repli(
            // SAFETY (les trois appels) : `d` est local, complet et de la taille annoncée.
            || unsafe { Shell_NotifyIconW(NIM_ADD, d) }.as_bool(),
            || unsafe { Shell_NotifyIconW(NIM_MODIFY, d) }.as_bool(),
            || {
                let _ = unsafe { Shell_NotifyIconW(NIM_SETVERSION, d) };
            },
        )
    }

    fn message_libre(&self) -> Option<u32> {
        let pris: Vec<u32> = self.icones.values().map(|p| p.message).collect();
        (0..ICONES_MAX as u32)
            .map(|n| PREMIER_MESSAGE + n)
            .find(|m| !pris.contains(m))
    }

    /// Retire l'icône `id` (rien si elle n'est pas posée).
    pub fn retirer(&mut self, id: &str) {
        let Some(p) = self.icones.remove(id) else {
            return;
        };
        let d = self.donnees(p.guid);
        // SAFETY: retire l'icône de cette fenêtre, puis détruit son image.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &d);
            let _ = DestroyIcon(p.hicon);
        }
        if let Ok(mut s) = self.partage.try_borrow_mut() {
            s.rappels.remove(&p.message);
        }
    }

    /// Une notification posée sur l'icône `id` (titre, texte, niveau). Windows la garde
    /// silencieuse quand « Ne pas déranger » est actif. Un clic dessus revient en
    /// [`Message::Notification`] de cette icône. Rend faux si l'icône n'est pas posée ou
    /// si Windows refuse.
    pub fn notifier(&self, id: &str, titre: &str, texte: &str, niveau: Niveau) -> bool {
        let Some(p) = self.icones.get(id) else {
            return false;
        };
        let mut d = self.donnees(p.guid);
        d.uFlags |= NIF_INFO;
        copier(&mut d.szInfoTitle, titre);
        copier(&mut d.szInfo, texte);
        d.dwInfoFlags = match niveau {
            Niveau::Information => NIIF_INFO,
            Niveau::Avertissement => NIIF_WARNING,
            Niveau::Erreur => NIIF_ERROR,
        } | NIIF_RESPECT_QUIET_TIME;
        // SAFETY: comme `poser`.
        unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) }.as_bool()
    }

    /// Le rectangle de l'icône `id` (pixels réels), si Windows le donne.
    pub fn rect(&self, id: &str) -> Option<Rect> {
        rect_icone(self.fenetre, self.icones.get(id)?.guid)
    }

    /// Les gestes et clics de notification reçus depuis le dernier appel, dans l'ordre. À
    /// appeler après chaque `DispatchMessageW` de la boucle du logiciel. Remet aussi les
    /// icônes si l'Explorateur a redémarré.
    pub fn evenements(&mut self) -> Vec<Evenement> {
        self.remettre_si_besoin();
        match self.partage.try_borrow_mut() {
            Ok(mut p) => p.evenements.drain(..).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Ce que le logiciel fait **tout de suite** quand la session Windows se ferme
    /// ([`FinDeSession`]) : `f` est appelée par la procédure de la fenêtre cachée, sur ce fil,
    /// pendant `WM_ENDSESSION`, et Windows attend son retour. C'est là qu'il faut enregistrer
    /// ce qui ne l'est pas encore (rapidement : l'arrêt n'attend que quelques secondes), sans
    /// rien ouvrir ni attendre l'utilisateur. Remplace la fonction précédente, s'il y en avait
    /// une.
    ///
    /// `f` s'exécute pendant `DispatchMessageW` : elle ne touche pas à la zone (qui est
    /// empruntée par la boucle du logiciel) ; elle appelle ce que le logiciel partage déjà
    /// avec sa boucle (un `Rc<RefCell<…>>`, un fichier à écrire…).
    pub fn sur_fin_de_session(&mut self, f: impl FnMut(&FinDeSession) + 'static) {
        if let Ok(mut p) = self.partage.try_borrow_mut() {
            p.sur_fin_de_session = Some(Box::new(f));
        }
    }

    /// La fin de session reçue depuis le dernier appel, s'il y en a eu une (`None` ensuite).
    /// À lire après `DispatchMessageW`, comme [`Zone::evenements`] : c'est un simple signal,
    /// **trop tardif pour enregistrer** (voir [`FinDeSession`] et [`Zone::sur_fin_de_session`]).
    pub fn fin_de_session(&mut self) -> Option<FinDeSession> {
        self.partage.try_borrow_mut().ok()?.fin_de_session.take()
    }

    /// L'Explorateur a redémarré : chaque icône est remise, avec son image et son info-bulle.
    fn remettre_si_besoin(&mut self) {
        let remettre = self
            .partage
            .try_borrow_mut()
            .map(|mut p| std::mem::take(&mut p.a_remettre))
            .unwrap_or(false);
        if !remettre {
            return;
        }
        let ids: Vec<String> = self.icones.keys().cloned().collect();
        for id in ids {
            let Some(p) = self.icones.get(&id) else { continue };
            let mut d = self.donnees(p.guid);
            d.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
            d.hIcon = p.hicon;
            copier(&mut d.szTip, &p.infobulle);
            let ok = self.ajouter(&mut d, p.message);
            if let Some(p) = self.icones.get_mut(&id) {
                p.dans_la_barre = ok;
            }
        }
    }

    /// Le menu gris de Windows au point `point` (pixels réels), jusqu'au choix : la clé de
    /// la ligne choisie, ou `None`. Les « & » des textes sont doublés ici. La boucle des
    /// messages tourne pendant que le menu est ouvert (les gestes arrivent ensuite par
    /// [`Zone::evenements`]).
    pub fn menu_windows(&self, point: [i32; 2], lignes: &[LigneMenu]) -> Option<String> {
        let mut toutes = Vec::new();
        cles(lignes, &mut toutes);
        // SAFETY: menus créés, remplis de chaînes vivantes pendant chaque appel, affichés
        // puis détruits ici (DestroyMenu détruit aussi les sous-menus) ; la fenêtre est celle
        // de ce fil.
        let choix = unsafe {
            let Ok(m) = CreatePopupMenu() else {
                return None;
            };
            let mut n = 0usize;
            remplir(m, lignes, &mut n);
            let _ = SetForegroundWindow(self.fenetre);
            let _imbriquee = crate::sonde::imbriquee("TrackPopupMenu (menu de la zone)");
            let c = TrackPopupMenu(
                m,
                TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                point[0],
                point[1],
                None,
                self.fenetre,
                None,
            )
            .0 as usize;
            // Recommandé par Windows : que le menu se ferme bien au clic suivant ailleurs.
            let _ = PostMessageW(Some(self.fenetre), WM_NULL, WPARAM(0), LPARAM(0));
            let _ = DestroyMenu(m);
            c
        };
        let cle = toutes.get(choix.checked_sub(1)?)?;
        (!cle.is_empty()).then(|| cle.clone())
    }
}

/// Remplit un menu de Windows ; les lignes à choisir sont numérotées à partir de 1 dans
/// l'ordre de [`cles`].
///
/// # Safety
/// `m` est un menu vivant de ce fil.
unsafe fn remplir(m: HMENU, lignes: &[LigneMenu], n: &mut usize) {
    for l in lignes {
        match l {
            LigneMenu::Trait => {
                // SAFETY: voir la fonction.
                let _ = unsafe { AppendMenuW(m, MF_SEPARATOR, 0, PCWSTR::null()) };
            }
            LigneMenu::Choix {
                texte, coche, actif, ..
            } => {
                *n += 1;
                let mut f: MENU_ITEM_FLAGS = MF_STRING;
                if *coche {
                    f |= MF_CHECKED;
                }
                if !*actif {
                    f |= MF_GRAYED;
                }
                // SAFETY: voir la fonction ; la chaîne vit pendant l'appel.
                let _ = unsafe { AppendMenuW(m, f, *n, &HSTRING::from(texte_menu(texte))) };
            }
            LigneMenu::Sous { texte, actif, lignes } => {
                // SAFETY: sous-menu créé ici, rattaché à `m` (détruit avec lui).
                let Ok(s) = (unsafe { CreatePopupMenu() }) else {
                    continue;
                };
                // SAFETY: `s` est vivant.
                unsafe { remplir(s, lignes, n) };
                let mut f: MENU_ITEM_FLAGS = MF_STRING | MF_POPUP;
                if !*actif {
                    f |= MF_GRAYED;
                }
                // SAFETY: voir la fonction.
                let _ = unsafe { AppendMenuW(m, f, s.0 as usize, &HSTRING::from(texte_menu(texte))) };
            }
        }
    }
}

/// Le rectangle de l'icône `guid` de la fenêtre `h` (pixels réels).
fn rect_icone(h: HWND, guid: u128) -> Option<Rect> {
    let id = NOTIFYICONIDENTIFIER {
        cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
        hWnd: h,
        guidItem: GUID::from_u128(guid),
        ..Default::default()
    };
    // SAFETY: `id` local et complet.
    unsafe { Shell_NotifyIconGetRect(&id) }
        .ok()
        .map(|r| [r.left, r.top, r.right, r.bottom])
}

impl Drop for Zone {
    fn drop(&mut self) {
        let ids: Vec<String> = self.icones.keys().cloned().collect();
        for id in ids {
            self.retirer(&id);
        }
        // SAFETY: plus aucun message ne lira l'état partagé, puis la fenêtre de ce fil est
        // détruite.
        unsafe {
            SetWindowLongPtrW(self.fenetre, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.fenetre);
        }
    }
}

/// Note la fin de session, puis appelle la fonction du logiciel ([`Zone::sur_fin_de_session`]).
/// La fonction est sortie de l'état partagé pendant l'appel : si elle appelle la zone, rien
/// n'est emprunté deux fois ; elle est remise après, sauf si elle en a posé une autre.
fn signaler_fin_de_session(partage: Option<&RefCell<Partage>>, fin: FinDeSession) {
    let Some(partage) = partage else { return };
    let mut f = {
        let Ok(mut p) = partage.try_borrow_mut() else {
            return;
        };
        p.fin_de_session = Some(fin);
        p.sur_fin_de_session.take()
    };
    if let Some(f) = f.as_mut() {
        f(&fin);
    }
    if let Ok(mut p) = partage.try_borrow_mut()
        && p.sur_fin_de_session.is_none()
    {
        p.sur_fin_de_session = f;
    }
}

extern "system" fn procedure(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_DISPLAYCHANGE {
        // Windows refait l'affichage (écran retiré, rendu, autre résolution) : les fenêtres pas
        // encore créées du logiciel attendent la fin de la reprise (1 s). Un logiciel qui n'a
        // pas sa propre procédure de fenêtre en profite aussi. Traitement par défaut ensuite.
        crate::autres_fenetres::affichage_change();
    }
    // SAFETY: la valeur rangée est 0 ou l'adresse du `Partage` de la zone, valable jusqu'à
    // ce que `Drop` la remette à 0 ; les messages arrivent sur ce même fil.
    let partage = unsafe { GetWindowLongPtrW(h, GWLP_USERDATA) } as *const RefCell<Partage>;
    if let Some(fin) = FinDeSession::de(msg, wp.0, lp.0) {
        // La session se ferme : le logiciel enregistre maintenant, avant que Windows ne
        // reprenne la main. Zéro : message traité.
        // SAFETY: voir ci-dessus.
        signaler_fin_de_session(unsafe { partage.as_ref() }, fin);
        return LRESULT(0);
    }
    // SAFETY: voir ci-dessus.
    if let Some(p) = unsafe { partage.as_ref() }
        && let Ok(mut p) = p.try_borrow_mut()
    {
        if msg == p.taskbar_created && msg != 0 {
            p.a_remettre = true;
            return LRESULT(0);
        }
        if let Some((id, guid)) = p.rappels.get(&msg).cloned() {
            if let Some(message) = Message::de((lp.0 as u32) & 0xFFFF) {
                // Avec la version 4, `wparam` porte le point du geste (x, y).
                let ancre = [
                    i32::from(wp.0 as u16 as i16),
                    i32::from((wp.0 >> 16) as u16 as i16),
                ];
                let rect = rect_icone(h, guid).unwrap_or_else(|| {
                    let mut s = POINT::default();
                    // SAFETY: POINT local.
                    let _ = unsafe { GetCursorPos(&mut s) };
                    [s.x - 8, s.y - 8, s.x + 8, s.y + 8]
                });
                // Bornée : un logiciel qui ne lirait jamais les événements ne remplit pas la
                // mémoire.
                if p.evenements.len() < 256 {
                    p.evenements.push_back(Evenement {
                        icone: id,
                        message,
                        ancre,
                        rect,
                    });
                }
            }
            return LRESULT(0);
        }
    }
    // SAFETY: traitement par défaut de Windows, avec les paramètres reçus.
    unsafe { DefWindowProcW(h, msg, wp, lp) }
}

// ---------------------------------------------------------------------------------------
// Le volet, le petit menu et les Paramètres communs pour les icônes de la zone

/// Ce que le logiciel fait d'un geste sur une de ses icônes, une fois passé par les
/// mécanismes communs ([`Aiguilleur::recevoir`], [`FenetresCommunes::geste`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuiteDuGeste {
    /// Rien : la bibliothèque s'en occupe (volet, petit menu, Paramètres), ou le geste ne
    /// compte pas (survol, clic du milieu…).
    Rien,
    /// Les fenêtres modernes sont impossibles : le menu de secours de Windows
    /// ([`Zone::menu_windows`]) au point `point` (pixels réels), pour l'icône `icone`.
    Secours { icone: String, point: [i32; 2] },
    /// Un clic sur la notification de cette icône : au logiciel de faire ce qu'elle annonçait.
    Notification(String),
}

/// L'aiguillage des gestes des icônes d'une zone vers le fil des fenêtres commun, sans
/// Windows ni fenêtre (testé) : le suivi des clics par icône et l'ordre qui en découle, les
/// mêmes que pour l'icône unique (`ic_coeur::gestes::aiguilleur`), puis
/// [`fenetres::envoyer_le_clic`]. [`FenetresCommunes`] en tient un.
pub struct Aiguilleur {
    liens: Arc<Liens>,
    demandes: Demandes,
    clics: SuiviDesIcones<Instant>,
    /// Le logiciel, pour sa fenêtre principale (`Logiciel::fenetre_principale`) ; sans lui,
    /// le double-clic ouvre les Paramètres et le clic gauche le volet.
    logiciel: Option<Arc<dyn Logiciel>>,
}

impl Aiguilleur {
    pub fn new(liens: Arc<Liens>, demandes: Demandes) -> Aiguilleur {
        Aiguilleur {
            liens,
            demandes,
            clics: SuiviDesIcones::default(),
            logiciel: None,
        }
    }

    /// Avec le logiciel : sa fenêtre principale, s'il en a une, s'ouvre au double-clic (et au
    /// clic gauche avec `FenetrePrincipale::clic_gauche`).
    pub fn avec_logiciel(mut self, logiciel: Arc<dyn Logiciel>) -> Aiguilleur {
        self.logiciel = Some(logiciel);
        self
    }

    /// Le geste `e`, reçu à `maintenant` ; `double` : le temps d'un double-clic de Windows.
    pub fn recevoir(&mut self, e: &Evenement, maintenant: Instant, double: Duration) -> SuiteDuGeste {
        if e.message == Message::Notification {
            return SuiteDuGeste::Notification(e.icone.clone());
        }
        let Some(g) = e.message.geste() else {
            return SuiteDuGeste::Rien;
        };
        let g = self.clics.recevoir(&e.icone, g, maintenant, double);
        let principale = self.logiciel.as_ref().and_then(|l| l.fenetre_principale());
        let (surface, ancre) = match ordre(g, principale, e.ancre) {
            Ordre::Rien => return SuiteDuGeste::Rien,
            Ordre::Parametres => {
                // Double-clic : le volet ouvert par le premier clic lit la demande des
                // Paramètres, se cache, puis leur laisse la place.
                self.liens.montrer_page(&self.demandes, None);
                return SuiteDuGeste::Rien;
            }
            Ordre::FenetrePrincipale => {
                // Le volet ou le petit menu ouvert par le premier clic lui laisse la place.
                if fermer_avant_la_fenetre_principale(self.liens.ouvert()) {
                    self.liens.fermer_la_fenetre();
                }
                if let Some(l) = &self.logiciel {
                    l.ouvrir_fenetre_principale();
                }
                return SuiteDuGeste::Rien;
            }
            Ordre::Surface { surface, ancre } => (surface, ancre),
        };
        match envoyer_le_clic(&self.liens, &self.demandes, surface, e.rect, ancre, maintenant) {
            Aiguillage::Rien => SuiteDuGeste::Rien,
            Aiguillage::Envoye => {
                // La fenêtre prête se montre tout de suite, encore invisible.
                crate::systeme::reveiller(self.liens.surface_prete.load(Ordering::Acquire));
                SuiteDuGeste::Rien
            }
            Aiguillage::Secours => SuiteDuGeste::Secours {
                icone: e.icone.clone(),
                point: point_de_secours(ancre, e.rect),
            },
        }
    }
}

/// Le volet, le petit menu et les Paramètres de la bibliothèque pour un logiciel qui garde sa
/// [`Zone`] et sa boucle de messages : le fil des fenêtres commun
/// (`horloge::lancer_le_fil_des_fenetres`), et l'aiguillage des gestes vers lui
/// ([`Aiguilleur`]). Le volet est le même quelle que soit l'icône cliquée ; il se place contre
/// l'icône du geste (`Logiciel::placement_volet`).
///
/// ```no_run
/// # use std::sync::Arc;
/// # fn essai(logiciel: Arc<dyn interface_commune::Logiciel>) -> Result<(), String> {
/// use interface_commune::zone::{FenetresCommunes, SuiteDuGeste, Zone};
/// let mut zone = Zone::creer("MonLogiciel", "MonLogiciel")?;
/// // Le menu de secours est demandé sur le fil des fenêtres : le transmettre au fil de la
/// // zone (un message posté à `zone.fenetre()`), qui l'ouvre avec `Zone::menu_windows`.
/// let mut communes = FenetresCommunes::lancer(&zone, logiciel, None, Box::new(|_demande| {}))?;
/// // Dans la boucle de messages du logiciel, après DispatchMessageW :
/// for e in zone.evenements() {
///     match communes.geste(&e) {
///         SuiteDuGeste::Rien => {}
///         SuiteDuGeste::Secours { icone, point } => { /* zone.menu_windows(point, …) */ }
///         SuiteDuGeste::Notification(icone) => { /* ce que la bulle annonçait */ }
///     }
/// }
/// // « Quitter » : les fenêtres se ferment (5 s au plus), puis la zone.
/// communes.fermer();
/// # Ok(()) }
/// ```
pub struct FenetresCommunes {
    aiguilleur: Aiguilleur,
    liens: Arc<Liens>,
    demandes: Demandes,
    logiciel: Arc<dyn Logiciel>,
    fil: Option<std::thread::JoinHandle<()>>,
}

impl FenetresCommunes {
    /// Lance le fil des fenêtres (le volet préparé au repos, caché). `parametres` : les
    /// Paramètres à ouvrir tout de suite (`--parametres <page>` : `Some(page)`). `secours` :
    /// voir `horloge::lancer_le_fil_des_fenetres` (appelé sur le fil des fenêtres, il transmet
    /// seulement la demande au fil de la zone).
    pub fn lancer(
        zone: &Zone,
        logiciel: Arc<dyn Logiciel>,
        parametres: Option<Option<String>>,
        secours: Secours,
    ) -> Result<FenetresCommunes, String> {
        let (demandes, reception) = fenetres::canal();
        let liens = Arc::new(Liens::default());
        liens.fenetre.store(zone.fenetre(), Ordering::Release);
        if let Some(page) = parametres {
            let _ = demandes.send(Demande::Parametres(page));
        }
        // La sonde des fils écrit dans le journal du logiciel, comme avec `Horloge`.
        let pour_la_sonde = logiciel.clone();
        crate::sonde::brancher_le_journal(move |ligne| pour_la_sonde.noter(ligne));
        let fil =
            crate::horloge::lancer_le_fil_des_fenetres(logiciel.clone(), liens.clone(), reception, secours)?;
        Ok(FenetresCommunes {
            aiguilleur: Aiguilleur::new(liens.clone(), demandes.clone()).avec_logiciel(logiciel.clone()),
            liens,
            demandes,
            logiciel,
            fil: Some(fil),
        })
    }

    /// Un geste lu par [`Zone::evenements`] : le volet, le petit menu ou les Paramètres
    /// s'ouvrent (ou se ferment) par les mécanismes communs ; rend ce qui reste au logiciel.
    pub fn geste(&mut self, e: &Evenement) -> SuiteDuGeste {
        self.aiguilleur
            .recevoir(e, Instant::now(), crate::systeme::temps_double_clic())
    }

    /// Ce que partagent le fil des fenêtres et le logiciel (`Liens::redessiner` quand l'état
    /// change, `Liens::autres` pour ses autres fenêtres…).
    pub fn liens(&self) -> &Arc<Liens> {
        &self.liens
    }

    /// L'envoi des demandes au fil des fenêtres.
    pub fn demandes(&self) -> &Demandes {
        &self.demandes
    }

    /// Ouvre les Paramètres à cette page (une notification, une demande du logiciel) ;
    /// ouverts, ils passent devant et changent de page.
    pub fn ouvrir_parametres(&self, page: Option<String>) {
        self.liens.montrer_page(&self.demandes, page);
    }

    /// Ouvre le logiciel (un second lancement, par exemple) : sa fenêtre principale s'il en a
    /// une (`Logiciel::fenetre_principale`), sinon ses Paramètres.
    pub fn ouvrir_le_logiciel(&self) {
        if self.logiciel.fenetre_principale().is_some() {
            self.logiciel.ouvrir_fenetre_principale();
        } else {
            self.ouvrir_parametres(None);
        }
    }

    /// L'état du logiciel a changé : le volet et les Paramètres ouverts se redessinent.
    pub fn redessiner(&self) {
        self.liens.redessiner();
    }

    /// Ferme les fenêtres et attend la fin de leur fil (`horloge::ATTENTE_FERMETURE` au
    /// plus) ; faux s'il a fallu partir sans l'attendre (c'est noté au journal).
    pub fn fermer(mut self) -> bool {
        self.arreter()
    }

    fn arreter(&mut self) -> bool {
        self.liens.tout_fermer();
        let _ = self.demandes.send(Demande::Quitter);
        let Some(fil) = self.fil.take() else {
            return true;
        };
        let fini = crate::horloge::attendre_la_fin(fil, crate::horloge::ATTENTE_FERMETURE);
        if !fini {
            self.logiciel
                .noter("Une fenêtre ne répond plus : le logiciel se ferme sans l'attendre.".into());
        }
        fini
    }
}

impl Drop for FenetresCommunes {
    fn drop(&mut self) {
        if self.fil.is_some() {
            self.arreter();
        }
    }
}

#[cfg(test)]
mod essais {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::theme::Surface;

    /// Chaque message de Windows qui compte, et lui seul.
    #[test]
    fn messages_de_l_icone() {
        assert_eq!(Message::de(0x0201), Some(Message::Appui(Bouton::Gauche)));
        assert_eq!(Message::de(0x0205), Some(Message::Relache(Bouton::Droit)));
        assert_eq!(Message::de(0x0203), Some(Message::DoubleClic(Bouton::Gauche)));
        assert_eq!(Message::de(0x0400), Some(Message::Selection));
        assert_eq!(Message::de(0x0401), Some(Message::Clavier));
        assert_eq!(Message::de(0x007B), Some(Message::MenuContextuel));
        assert_eq!(Message::de(0x0405), Some(Message::Notification));
        // Survol, ouverture d'info-bulle : rien.
        assert_eq!(Message::de(0x0200), None);
        assert_eq!(Message::de(0x0406), None);
    }

    /// Les gestes du volet et du petit menu (`clics`) se tirent des messages.
    #[test]
    fn gestes_pour_les_clics_communs() {
        assert_eq!(Message::Selection.geste(), Some(clics::Evenement::Selection));
        assert_eq!(
            Message::Relache(Bouton::Gauche).geste(),
            Some(clics::Evenement::Relache)
        );
        assert_eq!(Message::Relache(Bouton::Droit).geste(), None);
        assert_eq!(Message::Notification.geste(), None);
    }

    /// Le GUID ne change ni d'un lancement à l'autre ni avec la casse du chemin ; il change
    /// avec l'icône, le chemin et la graine ; c'est un GUID de version 4.
    #[test]
    fn guid_stable_et_unique() {
        let a = guid_stable("Essai", r"C:\Programmes\Essai\essai.exe", "souris");
        assert_eq!(
            a,
            guid_stable("Essai", r"c:\programmes\essai\ESSAI.exe", "souris")
        );
        assert_ne!(
            a,
            guid_stable("Essai", r"C:\Programmes\Essai\essai.exe", "clavier")
        );
        assert_ne!(a, guid_stable("Essai", r"D:\Essai\essai.exe", "souris"));
        assert_ne!(
            a,
            guid_stable("Autre", r"C:\Programmes\Essai\essai.exe", "souris")
        );
        assert_eq!((a >> 76) & 0xF, 4, "version 4");
        assert_eq!((a >> 62) & 0x3, 2, "variante");
        // Figé : un changement ferait oublier à Windows les icônes montrées.
        assert_eq!(a, 0xc70b_4318_db8d_4747_8c46_ffab_623b_aed8);
    }

    /// Les numéros donnés à Windows suivent l'ordre des lignes, sous-menus compris ; les
    /// « & » sont doublés.
    #[test]
    fn menu_windows_numerote_les_choix() {
        let lignes = vec![
            LigneMenu::etat("Souris — 80 %"),
            LigneMenu::Trait,
            LigneMenu::Sous {
                texte: "Styles".into(),
                actif: true,
                lignes: vec![
                    LigneMenu::Choix {
                        cle: "style:pixel".into(),
                        texte: "Pixel".into(),
                        coche: true,
                        actif: true,
                    },
                    LigneMenu::choix("style:lisse", "Lisse"),
                ],
            },
            LigneMenu::choix("quitter", "Quitter"),
        ];
        let mut c = Vec::new();
        cles(&lignes, &mut c);
        assert_eq!(c, ["", "style:pixel", "style:lisse", "quitter"]);
        assert_eq!(texte_menu("Ventes & achats"), "Ventes && achats");
    }

    #[test]
    fn image_depuis_rvba() {
        let i = Image::depuis_rvba(1, vec![1, 2, 3, 4]);
        assert_eq!(i.bgra, [3, 2, 1, 4]);
    }

    /// L'ajout d'une icône : `NIM_ADD` seul s'il réussit ; sinon `NIM_MODIFY` (icône déjà là) ;
    /// la version 4 est redemandée après l'un ou l'autre, jamais si les deux échouent.
    #[test]
    fn ajout_avec_repli_sur_la_modification() {
        use std::cell::Cell;
        let essai = |ajout: bool, modif: bool| {
            let (a, m, v) = (Cell::new(0), Cell::new(0), Cell::new(0));
            let ok = ajouter_avec_repli(
                || {
                    a.set(a.get() + 1);
                    ajout
                },
                || {
                    m.set(m.get() + 1);
                    modif
                },
                || v.set(v.get() + 1),
            );
            (ok, a.get(), m.get(), v.get())
        };
        // Ajout réussi : pas de modification.
        assert_eq!(essai(true, false), (true, 1, 0, 1));
        // L'icône existe déjà : la modification la remet, puis la version 4.
        assert_eq!(essai(false, true), (true, 1, 1, 1));
        // Windows refuse tout : faux, et pas de version à redemander.
        assert_eq!(essai(false, false), (false, 1, 1, 0));
    }

    /// `WM_ENDSESSION` n'est une fin de session que si `wparam` est vrai ; les options de
    /// `lparam` sont décodées, tout autre message est ignoré.
    #[test]
    fn fin_de_session_decodee() {
        const WM_QUERYENDSESSION: u32 = 0x0011;
        assert_eq!(
            FinDeSession::de(WM_ENDSESSION, 1, 0),
            Some(FinDeSession::default())
        );
        // Fermeture annulée par un autre logiciel : rien à enregistrer.
        assert_eq!(FinDeSession::de(WM_ENDSESSION, 0, 0), None);
        assert_eq!(FinDeSession::de(WM_QUERYENDSESSION, 1, 0), None);
        assert_eq!(FinDeSession::de(WM_DISPLAYCHANGE, 1, 0), None);
        let deconnexion = FinDeSession::de(WM_ENDSESSION, 1, 0x8000_0000_u32 as isize).unwrap();
        assert_eq!(
            deconnexion,
            FinDeSession {
                deconnexion: true,
                ..Default::default()
            }
        );
        let tout = FinDeSession::de(WM_ENDSESSION, 1, 0xC000_0001_u32 as isize).unwrap();
        assert!(tout.deconnexion && tout.critique && tout.application);
        // Les bits hauts d'un `lparam` de 64 bits ne comptent pas.
        let large = (1isize << 40) | 0x4000_0000;
        let c = FinDeSession::de(WM_ENDSESSION, 1, large).unwrap();
        assert!(c.critique && !c.deconnexion && !c.application);
    }

    /// La fenêtre cachée, pour de vrai : `WM_ENDSESSION` (envoyé, comme Windows le fait) appelle
    /// la fonction du logiciel **avant le retour** du message, avec les options ; il reste lisible
    /// ensuite par `fin_de_session` ; une fermeture annulée (`wparam` faux) n'appelle rien ; la
    /// fonction est gardée d'un message à l'autre ; une fonction qui appelle la zone ne bloque rien.
    #[test]
    fn fin_de_session_recue_par_la_fenetre_cachee() {
        use std::rc::Rc;
        use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
        let mut z = Zone::creer("EssaiZoneFin", "Essai").unwrap();
        assert_eq!(z.fin_de_session(), None);
        let vus: Rc<RefCell<Vec<FinDeSession>>> = Rc::default();
        let notes = vus.clone();
        z.sur_fin_de_session(move |f| notes.borrow_mut().push(*f));
        let fenetre = z.fenetre;
        let envoyer = |wp: usize, lp: isize| {
            // SAFETY: fenêtre de ce fil, message sans pointeur ; la procédure rend la main avant
            // le retour de `SendMessageW`.
            unsafe { SendMessageW(fenetre, WM_ENDSESSION, Some(WPARAM(wp)), Some(LPARAM(lp))) }
        };
        // Annulée : rien.
        assert_eq!(envoyer(0, 0).0, 0);
        assert!(vus.borrow().is_empty());
        assert_eq!(z.fin_de_session(), None);
        // Déconnexion : la fonction a déjà tout vu au retour du message.
        assert_eq!(envoyer(1, 0x8000_0000_u32 as isize).0, 0, "message traité");
        assert_eq!(
            *vus.borrow(),
            [FinDeSession {
                deconnexion: true,
                ..Default::default()
            }]
        );
        assert_eq!(z.fin_de_session().map(|f| f.deconnexion), Some(true));
        assert_eq!(z.fin_de_session(), None, "lue une seule fois");
        // La fonction est gardée : un arrêt, ensuite.
        envoyer(1, 0);
        assert_eq!(vus.borrow().len(), 2);
        // Sans fonction du logiciel : le signal reste lisible, sans erreur.
        let mut z2 = Zone::creer("EssaiZoneFin2", "Essai").unwrap();
        // SAFETY: comme ci-dessus, pour la seconde fenêtre.
        unsafe { SendMessageW(z2.fenetre, WM_ENDSESSION, Some(WPARAM(1)), Some(LPARAM(0))) };
        assert!(z2.fin_de_session().is_some());
    }

    /// La fenêtre cachée, pour de vrai, sans rien montrer : aucune icône n'est donnée à
    /// Windows (les rappels sont inscrits à la main), des gestes lui sont postés comme
    /// Windows le ferait. Chacun revient avec son icône et son point, dans l'ordre ; un
    /// message d'une icône inconnue et le survol sont ignorés ; « TaskbarCreated » demande
    /// de remettre les icônes.
    #[test]
    fn gestes_recus_par_la_fenetre_cachee() {
        use windows::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
        };
        let mut z = Zone::creer("EssaiZoneGestes", "Essai").unwrap();
        {
            let mut p = z.partage.borrow_mut();
            p.rappels.insert(PREMIER_MESSAGE, ("souris".into(), 1));
            p.rappels.insert(PREMIER_MESSAGE + 1, ("clavier".into(), 2));
        }
        let point = |x: u16, y: u16| WPARAM(usize::from(x) | (usize::from(y) << 16));
        let poster = |msg: u32, wp: WPARAM, lp: isize| {
            // SAFETY: fenêtre de ce fil, message sans pointeur.
            unsafe { PostMessageW(Some(z.fenetre), msg, wp, LPARAM(lp)) }.unwrap();
        };
        poster(PREMIER_MESSAGE, point(100, 200), 0x0400); // NIN_SELECT
        poster(PREMIER_MESSAGE + 1, point(300, 400), 0x007B); // WM_CONTEXTMENU
        poster(PREMIER_MESSAGE, point(1, 1), 0x0200); // survol : ignoré
        poster(PREMIER_MESSAGE + 7, point(1, 1), 0x0400); // icône inconnue : ignorée
        poster(PREMIER_MESSAGE + 1, point(5, 6), 0x0405); // clic sur la notification
        let barre = z.partage.borrow().taskbar_created;
        poster(barre, WPARAM(0), 0);
        let mut m = MSG::default();
        // SAFETY: boucle des messages de ce fil, sur un MSG local.
        unsafe {
            while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&m);
                DispatchMessageW(&m);
            }
        }
        assert!(z.partage.borrow().a_remettre, "Explorateur redémarré");
        let e = z.evenements();
        assert!(!z.partage.borrow().a_remettre, "icônes remises");
        let vu: Vec<(&str, Message, [i32; 2])> =
            e.iter().map(|e| (e.icone.as_str(), e.message, e.ancre)).collect();
        assert_eq!(
            vu,
            [
                ("souris", Message::Selection, [100, 200]),
                ("clavier", Message::MenuContextuel, [300, 400]),
                ("clavier", Message::Notification, [5, 6]),
            ]
        );
        // Sans icône dans la barre, la place est un petit carré autour de la souris.
        assert!(e.iter().all(|e| e.rect[2] - e.rect[0] == 16));
        assert!(z.evenements().is_empty());
    }

    /// Les gestes des icônes d'une zone passent par les mécanismes communs, comme l'icône
    /// unique : clic gauche, le volet contre l'icône ; un second clic le referme ; clic droit,
    /// le petit menu au point du clic ; double-clic, les Paramètres (et le clic qui le suit ne
    /// rouvre rien) ; la bulle revient au logiciel ; sans fenêtres modernes, le menu de
    /// secours ; le fil des fenêtres parti, le secours pour de bon. Sans aucune fenêtre.
    #[test]
    fn gestes_par_les_mecanismes_communs() {
        use crate::fenetres::Ouvert;
        let (demandes, reception) = fenetres::canal();
        let liens = Arc::new(Liens::default());
        let mut a = Aiguilleur::new(liens.clone(), demandes);
        let t0 = Instant::now();
        let t = |ms: u64| t0 + Duration::from_millis(ms);
        let double = Duration::from_millis(500);
        let geste = |icone: &str, message: Message| Evenement {
            icone: icone.into(),
            message,
            ancre: [1500, 1050],
            rect: [1490, 1040, 1514, 1072],
        };
        assert_eq!(
            a.recevoir(&geste("souris", Message::Selection), t(0), double),
            SuiteDuGeste::Rien
        );
        assert!(matches!(
            reception.prendre(),
            Ok(Some(Demande::Volet {
                icone: [1490, 1040, 1514, 1072],
                ..
            }))
        ));
        // Le volet est montré : un second clic le referme, sans nouvelle demande.
        liens.mettre(Ouvert::Surface(Surface::Volet));
        assert_eq!(
            a.recevoir(&geste("souris", Message::Selection), t(1_000), double),
            SuiteDuGeste::Rien
        );
        assert!(liens.fermer.load(Ordering::Acquire));
        assert_eq!(reception.prendre(), Ok(None));
        liens.mettre(Ouvert::Rien);
        // Clic droit sur une autre icône : le petit menu au point du clic.
        a.recevoir(&geste("clavier", Message::MenuContextuel), t(2_000), double);
        assert!(matches!(
            reception.prendre(),
            Ok(Some(Demande::Menu {
                ancre: [1500, 1050],
                ..
            }))
        ));
        // Le survol et le clic du milieu ne comptent pas.
        assert_eq!(
            a.recevoir(&geste("souris", Message::Appui(Bouton::Milieu)), t(2_100), double),
            SuiteDuGeste::Rien
        );
        assert_eq!(reception.prendre(), Ok(None));
        // La bulle.
        assert_eq!(
            a.recevoir(&geste("souris", Message::Notification), t(2_200), double),
            SuiteDuGeste::Notification("souris".into())
        );
        // Double-clic : les Paramètres ; le NIN_SELECT qui suit ne rouvre pas le volet.
        a.recevoir(
            &geste("souris", Message::DoubleClic(Bouton::Gauche)),
            t(3_000),
            double,
        );
        a.recevoir(
            &geste("souris", Message::Relache(Bouton::Gauche)),
            t(3_050),
            double,
        );
        assert_eq!(reception.prendre(), Ok(Some(Demande::Parametres(None))));
        a.recevoir(&geste("souris", Message::Selection), t(3_060), double);
        assert_eq!(reception.prendre(), Ok(None));
        // Sans fenêtres modernes : le menu de secours, au point du clic droit ou à l'icône.
        liens.egui_impossible.store(true, Ordering::Release);
        assert_eq!(
            a.recevoir(&geste("clavier", Message::MenuContextuel), t(5_000), double),
            SuiteDuGeste::Secours {
                icone: "clavier".into(),
                point: [1500, 1050]
            }
        );
        assert_eq!(
            a.recevoir(&geste("souris", Message::Selection), t(6_000), double),
            SuiteDuGeste::Secours {
                icone: "souris".into(),
                point: [1490, 1040]
            }
        );
        // Le fil des fenêtres parti : le secours, pour de bon.
        liens.egui_impossible.store(false, Ordering::Release);
        drop(reception);
        assert!(matches!(
            a.recevoir(&geste("souris", Message::Selection), t(8_000), double),
            SuiteDuGeste::Secours { .. }
        ));
        assert!(liens.egui_impossible.load(Ordering::Acquire));
    }

    /// Un logiciel d'essai, avec ou sans fenêtre principale ; il compte ses ouvertures.
    struct AvecFenetre {
        identite: crate::Identite,
        principale: Option<crate::FenetrePrincipale>,
        ouvertures: std::sync::atomic::AtomicUsize,
    }

    impl crate::Reglages for AvecFenetre {
        fn identite(&self) -> &crate::Identite {
            &self.identite
        }
        fn action(&self, _: &str) {}
        fn rechercher_mise_a_jour(&self) {}
        fn communs(&self) -> crate::Communs {
            crate::Communs::default()
        }
        fn changer(&self, _: &str, _: crate::Valeur) {}
    }

    impl Logiciel for AvecFenetre {
        fn etat(&self) -> crate::Etat {
            crate::Etat {
                genre: crate::GenreEtat::Pret,
                titre: "Exemple".into(),
                ligne: "Prêt".into(),
                detail: None,
            }
        }
        fn quitter(&self) {}
        fn noter(&self, _: String) {}
        fn fenetre_principale(&self) -> Option<crate::FenetrePrincipale> {
            self.principale
        }
        fn ouvrir_fenetre_principale(&self) {
            self.ouvertures.fetch_add(1, Ordering::AcqRel);
        }
    }

    fn avec_fenetre(principale: Option<crate::FenetrePrincipale>) -> Arc<AvecFenetre> {
        Arc::new(AvecFenetre {
            identite: crate::Identite {
                nom: "Exemple".into(),
                version: "0.0.0".into(),
                licence: "MIT".into(),
                description: "Essai.".into(),
                icone: |t, _| vec![255; (t * t * 4) as usize],
                logo: None,
            },
            principale,
            ouvertures: Default::default(),
        })
    }

    /// Avec une fenêtre principale : le double-clic l'ouvre (le volet du premier clic se
    /// ferme), pas les Paramètres ; le clic gauche ouvre le volet. Avec `clic_gauche` : le
    /// clic gauche l'ouvre aussi. Sans : rien ne change (les Paramètres au double-clic).
    #[test]
    fn double_clic_la_fenetre_principale() {
        use crate::fenetres::Ouvert;
        let t0 = Instant::now();
        let t = |ms: u64| t0 + Duration::from_millis(ms);
        let double = Duration::from_millis(500);
        let geste = |message: Message| Evenement {
            icone: "principale".into(),
            message,
            ancre: [1500, 1050],
            rect: [1490, 1040, 1514, 1072],
        };
        let double_clic = |a: &mut Aiguilleur, debut: u64| {
            a.recevoir(&geste(Message::DoubleClic(Bouton::Gauche)), t(debut), double);
            a.recevoir(&geste(Message::Relache(Bouton::Gauche)), t(debut + 50), double);
            a.recevoir(&geste(Message::Selection), t(debut + 60), double);
        };

        // Fenêtre principale, clic gauche : le volet ; double-clic : la fenêtre principale.
        let l = avec_fenetre(Some(crate::FenetrePrincipale::default()));
        let (demandes, reception) = fenetres::canal();
        let liens = Arc::new(Liens::default());
        let mut a = Aiguilleur::new(liens.clone(), demandes).avec_logiciel(l.clone());
        a.recevoir(&geste(Message::Selection), t(0), double);
        assert!(matches!(reception.prendre(), Ok(Some(Demande::Volet { .. }))));
        liens.mettre(Ouvert::Surface(Surface::Volet));
        double_clic(&mut a, 150);
        assert_eq!(l.ouvertures.load(Ordering::Acquire), 1);
        assert!(liens.fermer.load(Ordering::Acquire));
        assert_eq!(reception.prendre(), Ok(None));

        // `clic_gauche` : le clic gauche ouvre la fenêtre principale, sans volet.
        let l = avec_fenetre(Some(crate::FenetrePrincipale { clic_gauche: true }));
        let (demandes, reception) = fenetres::canal();
        let mut a = Aiguilleur::new(Arc::new(Liens::default()), demandes).avec_logiciel(l.clone());
        a.recevoir(&geste(Message::Selection), t(0), double);
        assert_eq!(l.ouvertures.load(Ordering::Acquire), 1);
        assert_eq!(reception.prendre(), Ok(None));
        // Le clic droit garde le petit menu.
        a.recevoir(&geste(Message::MenuContextuel), t(2_000), double);
        assert!(matches!(reception.prendre(), Ok(Some(Demande::Menu { .. }))));

        // Sans fenêtre principale : le double-clic ouvre les Paramètres, comme avant.
        let l = avec_fenetre(None);
        let (demandes, reception) = fenetres::canal();
        let mut a = Aiguilleur::new(Arc::new(Liens::default()), demandes).avec_logiciel(l.clone());
        double_clic(&mut a, 0);
        assert_eq!(l.ouvertures.load(Ordering::Acquire), 0);
        assert_eq!(reception.prendre(), Ok(Some(Demande::Parametres(None))));
    }
}
