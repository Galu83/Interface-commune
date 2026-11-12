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
use std::time::Instant;

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
    NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION, NOTIFYICON_VERSION_4, NOTIFYICONDATAW,
    NOTIFYICONIDENTIFIER, Shell_NotifyIconGetRect, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon,
    DestroyMenu, DestroyWindow, DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW, HICON, ICONINFO,
    MF_SEPARATOR, MF_STRING, MSG, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
    SM_CXSMICON, SetForegroundWindow, TPM_BOTTOMALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu,
    TranslateMessage, WINDOW_EX_STYLE, WM_APP, WM_CLOSE, WM_DESTROY, WM_ENDSESSION, WM_SETTINGCHANGE,
    WNDCLASSW, WS_OVERLAPPED,
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
#[derive(Debug, Clone, PartialEq, Eq)]
enum Commande {
    Notifier {
        texte: String,
        probleme: bool,
        page: Option<String>,
    },
    Infobulle(String),
    Icone,
    Quitter,
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
            probleme,
            page: page.map(str::to_string),
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
}

/// L'icône près de l'horloge, avant son lancement.
pub struct Horloge {
    nom: String,
    poste: Poste,
    reception: Reception,
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
        }
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
        } = self;
        if let Some(page) = parametres {
            let _ = poste.demandes.send(Demande::Parametres(page));
        }
        let (l2, liens2) = (logiciel.clone(), poste.liens.clone());
        let fil = std::thread::Builder::new()
            .name("fenêtres".into())
            .spawn(move || fil_des_fenetres(l2, liens2, reception))
            .map_err(|e| e.to_string())?;
        let r = boucle(&nom, logiciel, poste.clone());
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
        crate::parametres::fenetre::ouvrir(
            self.logiciel.clone(),
            self.liens.clone(),
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
    let px = (logiciel.identite().icone)(taille as u32, barre_sombre);
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
    /// La page des Paramètres de la dernière notification (un clic dessus l'ouvre).
    page_bulle: Option<String>,
}

thread_local! {
    static PRINCIPAL: RefCell<Option<Principal>> = const { RefCell::new(None) };
}

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
                    probleme,
                    page,
                } => {
                    self.page_bulle = page;
                    let mut d = donnees_icone(self.fenetre);
                    d.uFlags = NIF_INFO;
                    copier(&mut d.szInfoTitle, &self.logiciel.identite().nom);
                    copier(&mut d.szInfo, &texte);
                    d.dwInfoFlags = if probleme { NIIF_ERROR } else { NIIF_INFO } | NIIF_RESPECT_QUIET_TIME;
                    // SAFETY: comme `ajouter_icone`.
                    let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &d) };
                }
                Commande::Infobulle(t) => {
                    self.infobulle = t;
                    self.modifier_icone();
                }
                Commande::Icone => self.redessiner_icone(true),
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

    /// Le menu gris de Windows : seulement en secours, si le menu moderne ne peut pas
    /// s'ouvrir (carte graphique sans OpenGL, polices absentes). Les mêmes lignes, dans le
    /// même ordre.
    fn menu_de_secours(&mut self, x: i32, y: i32) {
        let lignes = menu::lignes(&self.logiciel.actions_menu(), &self.logiciel.lignes_en_plus());
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
            // WM_DESTROY arrive pendant que `PRINCIPAL` est emprunté : finir ici.
            PostQuitMessage(0);
        }
    }
}

extern "system" fn procedure(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    // Les messages arrivent sur le fil principal, où vit `PRINCIPAL`. Pendant la création
    // de la fenêtre, `PRINCIPAL` est encore vide : traitement par défaut.
    let traite = PRINCIPAL.with(|p| {
        let Ok(mut p) = p.try_borrow_mut() else {
            return false;
        };
        let Some(p) = p.as_mut() else { return false };
        if !message_permis(msg, wp.0) {
            // Message interne sans le jeton : venu d'un autre programme, ignoré.
            return true;
        }
        match msg {
            WM_ICONE => {
                if (lp.0 as u32) & 0xFFFF == NIN_BALLOONUSERCLICK {
                    let page = p.page_bulle.clone();
                    p.ouvrir_parametres(page);
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
        LRESULT(0)
    } else {
        // SAFETY: traitement par défaut de Windows, avec les paramètres reçus.
        unsafe { DefWindowProcW(h, msg, wp, lp) }
    }
}

/// Crée la fenêtre cachée et l'icône, puis traite les messages jusqu'à « Quitter ».
fn boucle(nom: &str, logiciel: Arc<dyn Logiciel>, poste: Poste) -> Result<(), String> {
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
        page_bulle: None,
    };
    p.icone = fabriquer_icone(&*logiciel, p.barre_sombre);
    p.ajouter_icone();
    PRINCIPAL.with(|c| *c.borrow_mut() = Some(p));
    let mut m = MSG::default();
    // SAFETY: boucle des messages classique de ce fil, sur un MSG local.
    unsafe {
        while GetMessageW(&mut m, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
    PRINCIPAL.with(|c| c.borrow_mut().take());
    Ok(())
}
