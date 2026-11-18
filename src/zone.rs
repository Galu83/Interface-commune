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
    TPM_RIGHTBUTTON, TrackPopupMenu, WINDOW_EX_STYLE, WM_APP, WM_DISPLAYCHANGE, WM_NULL, WNDCLASSW,
    WS_OVERLAPPED,
};
use windows::core::{GUID, HSTRING, PCWSTR};

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::Logiciel;
use crate::clics::{self, Geste};
use crate::fenetres::{self, Aiguillage, Demande, Demandes, Liens, envoyer_le_clic};
use crate::horloge::{Niveau, Secours};
use crate::place::Rect;
use crate::theme::Surface;

/// Le premier message de rappel des icônes : chaque icône a le sien (`PREMIER_MESSAGE + n`),
/// ce qui dit sans erreur de quelle icône vient un geste.
const PREMIER_MESSAGE: u32 = WM_APP + 0x200;
/// Le nombre d'icônes à la fois (largement assez : la zone de notification n'en montre pas
/// autant).
pub const ICONES_MAX: usize = 256;
/// Caractères d'une info-bulle gardés par Windows (127 et le zéro final).
pub const INFOBULLE_MAX: usize = 127;

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

/// Le texte raccourci à `max` caractères, « … » compris.
pub fn raccourcir(texte: &str, max: usize) -> String {
    if texte.chars().count() <= max {
        texte.to_string()
    } else {
        texte.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    }
}

/// Copie un texte dans un tampon fixe de Windows, coupé pour garder son zéro final.
pub(crate) fn copier(dest: &mut [u16], texte: &str) {
    let mut n = 0;
    for (d, c) in dest.iter_mut().zip(texte.encode_utf16()) {
        *d = c;
        n += 1;
    }
    let fin = n.min(dest.len().saturating_sub(1));
    if let Some(d) = dest.get_mut(fin) {
        *d = 0;
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

/// Ce que partagent la zone et la procédure de sa fenêtre (même fil).
#[derive(Default)]
struct Partage {
    evenements: VecDeque<Evenement>,
    /// Le message de rappel de chaque icône → son identifiant et son GUID.
    rappels: BTreeMap<u32, (String, u128)>,
    taskbar_created: u32,
    /// L'Explorateur a redémarré : toutes les icônes sont à remettre.
    a_remettre: bool,
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

    /// `NIM_ADD` puis la version 4 (gestes complets, point du geste dans `wparam`).
    fn ajouter(&self, d: &mut NOTIFYICONDATAW, message: u32) -> bool {
        d.uFlags |= NIF_MESSAGE;
        d.uCallbackMessage = message;
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        // SAFETY: `d` est local, complet et de la taille annoncée.
        unsafe {
            let ok = Shell_NotifyIconW(NIM_ADD, d).as_bool();
            let _ = Shell_NotifyIconW(NIM_SETVERSION, d);
            ok
        }
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
/// Windows ni fenêtre (testé) : un suivi des clics par icône (`clics::Clics` : double-clic,
/// Entrée en double…), puis [`fenetres::envoyer_le_clic`]. [`FenetresCommunes`] en tient un.
pub struct Aiguilleur {
    liens: Arc<Liens>,
    demandes: Demandes,
    clics: BTreeMap<String, clics::Clics>,
}

impl Aiguilleur {
    pub fn new(liens: Arc<Liens>, demandes: Demandes) -> Aiguilleur {
        Aiguilleur {
            liens,
            demandes,
            clics: BTreeMap::new(),
        }
    }

    /// Le geste `e`, reçu à `maintenant` ; `double` : le temps d'un double-clic de Windows.
    pub fn recevoir(&mut self, e: &Evenement, maintenant: Instant, double: Duration) -> SuiteDuGeste {
        if e.message == Message::Notification {
            return SuiteDuGeste::Notification(e.icone.clone());
        }
        let Some(g) = e.message.geste() else {
            return SuiteDuGeste::Rien;
        };
        if self.clics.len() >= ICONES_MAX && !self.clics.contains_key(&e.icone) {
            // Des icônes retirées depuis longtemps : leur suivi est oublié.
            self.clics.clear();
        }
        let suivi = self.clics.entry(e.icone.clone()).or_default();
        let (surface, ancre) = match suivi.recevoir(g, maintenant, double) {
            Geste::Rien => return SuiteDuGeste::Rien,
            Geste::Parametres => {
                // Double-clic : le volet ouvert par le premier clic lit la demande des
                // Paramètres, se cache, puis leur laisse la place.
                self.liens.montrer_page(&self.demandes, None);
                return SuiteDuGeste::Rien;
            }
            Geste::Volet => (Surface::Volet, None),
            Geste::Menu => (Surface::Menu, Some(e.ancre)),
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
                point: ancre.unwrap_or([e.rect[0], e.rect[1]]),
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
            aiguilleur: Aiguilleur::new(liens.clone(), demandes.clone()),
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

    #[test]
    fn info_bulle_raccourcie() {
        assert_eq!(raccourcir("Souris", 10), "Souris");
        assert_eq!(raccourcir("Souris MX Master", 8), "Souris …");
        assert_eq!(
            raccourcir(&"é".repeat(200), INFOBULLE_MAX).chars().count(),
            INFOBULLE_MAX
        );
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
}
