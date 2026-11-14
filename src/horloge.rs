//! L'icône près de l'horloge et le fil principal (règles : interface Windows 11, point 10) :
//! une fenêtre cachée reçoit les gestes sur l'icône, les notifications cliquées, les
//! changements de thème et le redémarrage de l'Explorateur ; elle envoie au fil des fenêtres
//! le volet, le petit menu ou les Paramètres. Une seule instance par session : relancer le
//! logiciel ouvre ses Paramètres.
//!
//! Utilisation, sur le fil principal du logiciel :
//!
//! ```no_run
//! # fn exemple(logiciel: std::sync::Arc<dyn interface_commune::Logiciel>) -> Result<(), String> {
//! use interface_commune::horloge::{Horloge, instance_unique};
//! let _verrou = match instance_unique("MonLogiciel") {
//!     Ok(v) => v,
//!     Err(_) => return Ok(()), // déjà lancé : ses Paramètres s'ouvrent
//! };
//! let horloge = Horloge::new("MonLogiciel");
//! let poste = horloge.poste(); // à garder : notifications, info-bulle, quitter
//! horloge.lancer(logiciel, None) // rend la main quand le logiciel se ferme
//! # }
//! ```
//!
//! Chaque `unsafe` est commenté.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, POINT, WPARAM,
};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS, DeleteObject,
    HGDIOBJ,
};
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
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, DispatchMessageW, FindWindowW, GWLP_USERDATA, GetCursorPos, GetMessageW,
    GetWindowLongPtrW, HICON, ICONINFO, KillTimer, MF_SEPARATOR, MF_STRING, MSG, PostMessageW,
    PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SM_CXSMICON, SetForegroundWindow, SetTimer,
    SetWindowLongPtrW, TPM_BOTTOMALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage,
    WINDOW_EX_STYLE, WM_APP, WM_CLOSE, WM_DESTROY, WM_ENDSESSION, WM_QUERYENDSESSION, WM_SETTINGCHANGE,
    WM_TIMER, WNDCLASSW, WS_OVERLAPPED,
};
use windows::core::{HSTRING, PCWSTR};

use crate::Logiciel;
use crate::clics::{Clics, Geste, evenement};
use crate::fenetres::{self, Demande, Demandes, Liens, Ouvert, Ouvreur, Reception};
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

/// Une seule instance par session et par logiciel (`nom`, sans espace ni barre oblique). Si
/// le logiciel tourne déjà : lui demande d'ouvrir ses Paramètres et rend `Err(true)` ;
/// `Err(false)` si sa fenêtre est introuvable (il démarre ou se ferme).
pub fn instance_unique(nom: &str) -> Result<Unique, bool> {
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
            let c = classe(nom);
            // SAFETY: nom de classe vivant pendant l'appel ; on ne fait que poster un message
            // à la fenêtre trouvée.
            let Ok(f) = (unsafe { FindWindowW(&c, PCWSTR::null()) }) else {
                return Err(false);
            };
            // SAFETY: fenêtre trouvée juste avant ; PostMessageW n'attend rien.
            let poste = unsafe { PostMessageW(Some(f), WM_OUVRIR_PARAMETRES, WPARAM(0), LPARAM(0)) };
            Err(poste.is_ok())
        }
    }
}

/// Une commande du logiciel au fil principal.
enum Commande {
    Notifier {
        texte: String,
        niveau: Niveau,
        au_clic: AuClic,
    },
    Infobulle(String),
    Icone,
    Quitter,
    /// Du code du logiciel, à faire sur le fil de la fenêtre cachée (son adresse en
    /// paramètre) : `RegisterHotKey`…
    Executer(Box<dyn FnOnce(isize) + Send>),
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
/// `WM_QUERYENDSESSION`, `WM_ENDSESSION` et `WM_SETTINGCHANGE`. Il ne voit jamais les
/// messages propres à la bibliothèque.
pub type Recepteur = Box<dyn FnMut(&MessageWindows) -> Option<isize>>;

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

fn destinataire(msg: u32, wp: usize, taskbar_created: u32) -> Destinataire {
    match msg {
        WM_TIMER if wp == MINUTERIE_ICONE => Destinataire::Bibliotheque,
        WM_ICONE | WM_COMMANDE | WM_OUVRIR_PARAMETRES | WM_MENU_DE_SECOURS | WM_CLOSE | WM_DESTROY => {
            Destinataire::Bibliotheque
        }
        m if m == taskbar_created && m != 0 => Destinataire::Bibliotheque,
        WM_QUERYENDSESSION | WM_ENDSESSION | WM_SETTINGCHANGE => Destinataire::LesDeux,
        _ => Destinataire::Logiciel,
    }
}

/// Le niveau d'une notification : son icône dans la bulle de Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Niveau {
    #[default]
    Information,
    Avertissement,
    Erreur,
}

/// Ce que fait un clic sur une notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuClic {
    /// Ouvre les Paramètres (à cette page).
    Parametres(Option<String>),
    /// Rend cette action au logiciel (`Reglages::action`) : par exemple, montrer sa propre
    /// fenêtre.
    Action(String),
}

impl Default for AuClic {
    fn default() -> Self {
        AuClic::Parametres(None)
    }
}

/// De quoi parler à l'icône depuis n'importe quel fil (à cloner et garder par le logiciel).
#[derive(Clone)]
pub struct Poste {
    liens: Arc<Liens>,
    demandes: Demandes,
    file: Arc<Mutex<VecDeque<Commande>>>,
}

impl Poste {
    fn envoyer(&self, c: Commande) {
        if let Ok(mut f) = self.file.lock() {
            // Bornée : un logiciel qui enverrait sans fin ne remplit pas la mémoire.
            if f.len() < 64 {
                f.push_back(c);
            }
        }
        let h = self.liens.fenetre.load(Ordering::Acquire);
        if h != 0 {
            // SAFETY: la fenêtre est celle du logiciel ; PostMessageW n'attend rien et
            // accepte une fenêtre déjà détruite (il échoue simplement).
            let _ = unsafe { PostMessageW(Some(hwnd(h)), WM_COMMANDE, WPARAM(jeton()), LPARAM(0)) };
        }
    }

    /// Une notification près de l'horloge ; un clic dessus ouvre cette page des Paramètres.
    pub fn notifier(&self, texte: &str, probleme: bool, page: Option<&str>) {
        self.envoyer(Commande::Notifier {
            texte: texte.into(),
            niveau: if probleme {
                Niveau::Erreur
            } else {
                Niveau::Information
            },
            au_clic: AuClic::Parametres(page.map(str::to_string)),
        });
    }

    /// Une notification près de l'horloge, à ce niveau ; un clic dessus fait `au_clic`.
    pub fn notifier_avec(&self, texte: &str, niveau: Niveau, au_clic: AuClic) {
        self.envoyer(Commande::Notifier {
            texte: texte.into(),
            niveau,
            au_clic,
        });
    }

    /// Fait ce code du logiciel sur le fil de la fenêtre cachée de l'icône, qui le reçoit en
    /// paramètre (son adresse) : par exemple `RegisterHotKey`, qui exige le fil de la fenêtre.
    /// Les messages arrivent ensuite au récepteur ([`Horloge::recevoir_messages`]).
    pub fn sur_le_fil_principal(&self, f: impl FnOnce(isize) + Send + 'static) {
        self.envoyer(Commande::Executer(Box::new(f)));
    }

    /// Une notification près de l'horloge ; un clic dessus rend cette action au logiciel
    /// (`Reglages::action`), pour revenir à sa propre fenêtre par exemple.
    pub fn notifier_action(&self, texte: &str, probleme: bool, action: &str) {
        self.envoyer(Commande::Notifier {
            texte: texte.into(),
            niveau: if probleme {
                Niveau::Erreur
            } else {
                Niveau::Information
            },
            au_clic: AuClic::Action(action.into()),
        });
    }

    /// Le texte qui s'affiche au survol de l'icône.
    pub fn infobulle(&self, texte: &str) {
        self.envoyer(Commande::Infobulle(texte.into()));
    }

    /// L'icône doit être redessinée (son état a changé).
    pub fn redessiner_icone(&self) {
        self.envoyer(Commande::Icone);
    }

    /// L'état a changé : le volet ou les Paramètres ouverts se redessinent.
    pub fn redessiner(&self) {
        self.liens.redessiner();
    }

    /// Ouvre les Paramètres à cette page (une notification, une demande du logiciel).
    pub fn ouvrir_parametres(&self, page: Option<&str>) {
        self.liens.montrer_page(&self.demandes, page.map(str::to_string));
    }

    /// Ferme tout : les fenêtres, l'icône, puis rend la main à `Horloge::lancer`.
    pub fn quitter(&self) {
        self.liens.tout_fermer();
        self.envoyer(Commande::Quitter);
    }

    pub fn liens(&self) -> &Arc<Liens> {
        &self.liens
    }

    /// Les autres fenêtres du logiciel (une fenêtre simple, une fenêtre de lecture…),
    /// ouvertes et fermées dans la boucle de la bibliothèque, depuis n'importe quel fil (voir
    /// [`crate::autres_fenetres`]).
    pub fn fenetres(&self) -> crate::autres_fenetres::AutresFenetres {
        self.liens.autres.clone()
    }
}

/// L'icône près de l'horloge, avant son lancement.
pub struct Horloge {
    nom: String,
    poste: Poste,
    reception: Reception,
    messages: Option<Recepteur>,
}

impl Horloge {
    /// `nom` : le nom court du logiciel, le même que pour [`instance_unique`].
    pub fn new(nom: &str) -> Horloge {
        let (demandes, reception) = fenetres::canal();
        Horloge {
            nom: nom.into(),
            poste: Poste {
                liens: Arc::new(Liens::default()),
                demandes,
                file: Arc::default(),
            },
            reception,
            messages: None,
        }
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
        } = self;
        if let Some(page) = parametres {
            let _ = poste.demandes.send(Demande::Parametres(page));
        }
        let (l2, liens2) = (logiciel.clone(), poste.liens.clone());
        let fil = std::thread::Builder::new()
            .name("fenêtres".into())
            .spawn(move || fil_des_fenetres(l2, liens2, reception))
            .map_err(|e| e.to_string())?;
        let r = boucle(&nom, logiciel, poste.clone(), messages);
        poste.liens.tout_fermer();
        let _ = poste.demandes.send(Demande::Quitter);
        let _ = fil.join();
        r
    }
}

// ---------------------------------------------------------------------------------------
// Le fil des fenêtres

/// Les vraies fenêtres (egui).
struct Fenetres {
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    polices: egui::FontDefinitions,
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
        let point = match d {
            Demande::Volet { icone, .. } => [icone[0], icone[1]],
            Demande::Menu { ancre, .. } => *ancre,
            _ => return,
        };
        let lp = (i64::from(point[0]) & 0xFFFF) | ((i64::from(point[1]) & 0xFFFF) << 16);
        let h = self.liens.fenetre.load(Ordering::Acquire);
        if h != 0 {
            // SAFETY: comme `Poste::envoyer`.
            let _ = unsafe {
                PostMessageW(
                    Some(hwnd(h)),
                    WM_MENU_DE_SECOURS,
                    WPARAM(jeton()),
                    LPARAM(lp as isize),
                )
            };
        }
    }

    fn noter(&mut self, ligne: String) {
        self.logiciel.noter(ligne);
    }
}

fn fil_des_fenetres(logiciel: Arc<dyn Logiciel>, liens: Arc<Liens>, reception: Reception) {
    match crate::polices::polices() {
        Ok(polices) => {
            let mut f = Fenetres {
                logiciel,
                liens: liens.clone(),
                polices,
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
    if px.len() != (taille * taille * 4) as usize {
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

fn copier(dest: &mut [u16], texte: &str) {
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
    /// Le dernier changement d'icône, et un changement qui attend la fin du délai.
    icone_changee: Option<Instant>,
    icone_attendue: bool,
}

/// L'état du fil principal, rangé dans la fenêtre cachée (`GWLP_USERDATA`) : aucune
/// variable globale. Vide pendant la création de la fenêtre.
type Etat = RefCell<Option<Principal>>;

impl Principal {
    fn liens(&self) -> &Liens {
        &self.poste.liens
    }

    fn ajouter_icone(&mut self) {
        let mut d = donnees_icone(self.fenetre);
        d.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
        d.uCallbackMessage = WM_ICONE;
        d.hIcon = self.icone.unwrap_or_default();
        copier(&mut d.szTip, &self.infobulle);
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        // SAFETY: `d` est local, complet et de la taille annoncée.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &d);
            let _ = Shell_NotifyIconW(NIM_SETVERSION, &d);
        }
    }

    fn modifier_icone(&self) {
        let mut d = donnees_icone(self.fenetre);
        d.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        d.hIcon = self.icone.unwrap_or_default();
        copier(&mut d.szTip, &self.infobulle);
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
        let file: Vec<Commande> = self
            .poste
            .file
            .lock()
            .map(|mut f| f.drain(..).collect())
            .unwrap_or_default();
        for c in file {
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
        let clic = Instant::now();
        match self.clics.recevoir(e, clic, temps_double_clic()) {
            Geste::Rien => {}
            Geste::Volet => self.ouvrir(Surface::Volet, None, clic),
            Geste::Menu => self.ouvrir(Surface::Menu, Some(ancre), clic),
            // Double-clic : le volet ouvert par le premier clic lit la demande des
            // Paramètres, se cache, puis laisse la place.
            Geste::Parametres => self.ouvrir_parametres(None),
        }
    }

    /// Ouvre le volet ou le petit menu ; un nouveau clic quand il est ouvert le referme.
    fn ouvrir(&mut self, surface: Surface, ancre: Option<[i32; 2]>, clic: Instant) {
        let icone = self.rect_icone();
        let liens = self.liens();
        if liens.egui_impossible.load(Ordering::Acquire) {
            let [x, y] = ancre.unwrap_or([icone[0], icone[1]]);
            self.menu_de_secours(x, y);
            return;
        }
        match liens.ouvert() {
            Ouvert::Surface(s) if s == surface => liens.fermer_la_fenetre(),
            Ouvert::Parametres => liens.mettre_devant(),
            // Le clic sur l'icône vient de la fermer (perte du focus) : ne pas la rouvrir.
            Ouvert::Rien if liens.juste_fermee(surface) => {}
            Ouvert::Rien | Ouvert::Surface(_) => {
                let d = match ancre {
                    None => Demande::Volet { icone, clic },
                    Some(ancre) => Demande::Menu { icone, ancre, clic },
                };
                let _ = self.poste.demandes.send(d);
                // La fenêtre prête se montre tout de suite, encore invisible.
                reveiller(liens.surface_prete.load(Ordering::Acquire));
            }
        }
    }

    fn ouvrir_parametres(&self, page: Option<String>) {
        self.liens().montrer_page(&self.poste.demandes, page);
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
        let lignes = menu::lignes(l.textes(), &l.actions_menu(), &l.lignes_en_plus());
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
                    }
                }
            }
            let _ = SetForegroundWindow(self.fenetre);
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
                // Relancé alors qu'il tourne déjà : ses Paramètres.
                p.ouvrir_parametres(None);
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
    poste.liens.fenetre.store(fenetre.0 as isize, Ordering::Release);
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
        while GetMessageW(&mut m, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
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

    /// Les messages de Windows : ceux de la bibliothèque restent à elle ; la fin de session et
    /// les changements de réglages sont vus des deux ; tous les autres (raccourcis globaux,
    /// messages du logiciel) vont au logiciel.
    #[test]
    fn messages_de_windows_au_logiciel() {
        use windows::Win32::UI::WindowsAndMessaging::{WM_HOTKEY, WM_POWERBROADCAST};
        let barre = 0xC123;
        for m in [
            WM_ICONE,
            WM_COMMANDE,
            WM_OUVRIR_PARAMETRES,
            WM_MENU_DE_SECOURS,
            WM_CLOSE,
            WM_DESTROY,
            barre,
        ] {
            assert_eq!(destinataire(m, 0, barre), Destinataire::Bibliotheque, "{m:#x}");
        }
        for m in [WM_QUERYENDSESSION, WM_ENDSESSION, WM_SETTINGCHANGE] {
            assert_eq!(destinataire(m, 0, barre), Destinataire::LesDeux, "{m:#x}");
        }
        for m in [WM_HOTKEY, WM_POWERBROADCAST, WM_APP + 100] {
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

    /// Trois niveaux de notification.
    #[test]
    fn notification_a_trois_niveaux() {
        let p = Horloge::new("essai-niveaux").poste();
        p.notifier("Info.", false, None);
        p.notifier_avec("Attention.", Niveau::Avertissement, AuClic::Action("voir".into()));
        p.notifier("Erreur.", true, None);
        let niveaux: Vec<Niveau> = p
            .file
            .lock()
            .unwrap()
            .iter()
            .filter_map(|c| match c {
                Commande::Notifier { niveau, .. } => Some(*niveau),
                _ => None,
            })
            .collect();
        assert_eq!(
            niveaux,
            [Niveau::Information, Niveau::Avertissement, Niveau::Erreur]
        );
    }

    /// Du code du logiciel attend son tour sur le fil de la fenêtre cachée.
    #[test]
    fn code_pour_le_fil_principal() {
        let p = Horloge::new("essai-fil").poste();
        p.sur_le_fil_principal(|_| {});
        assert!(matches!(
            p.file.lock().unwrap().front(),
            Some(Commande::Executer(_))
        ));
    }

    /// Une notification ouvre les Paramètres à sa page, ou rend une action au logiciel.
    #[test]
    fn clic_sur_une_notification() {
        let h = Horloge::new("essai-notification");
        let p = h.poste();
        p.notifier("Mise à jour prête.", false, Some("mise_a_jour"));
        p.notifier_action("Lecture finie.", false, "montrer");
        let clics: Vec<AuClic> = p
            .file
            .lock()
            .unwrap()
            .iter()
            .filter_map(|c| match c {
                Commande::Notifier { au_clic, .. } => Some(au_clic.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            clics,
            vec![
                AuClic::Parametres(Some("mise_a_jour".into())),
                AuClic::Action("montrer".into())
            ]
        );
    }
}
