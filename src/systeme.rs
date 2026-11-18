//! Ce qui parle à Windows pour les fenêtres : le thème et l'accent, les coins arrondis et la
//! barre de titre sombre (DWM), la place, l'opacité (fondu), « Effets d'animation », le
//! réveil du fil des fenêtres, le sélecteur de dossier, `DisableProcessWindowsGhosting`.
//! Chaque `unsafe` est commenté.

use std::path::{Path, PathBuf};
use std::time::Duration;

use windows::Win32::Foundation::{COLORREF, CloseHandle, HANDLE, HWND, LPARAM, POINT, RECT, WAIT_FAILED};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAK, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::Storage::FileSystem::GetDriveTypeW;
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, RRF_RT_REG_BINARY, RRF_RT_REG_DWORD, RegGetValueW,
};
use windows::Win32::System::Threading::{CreateEventW, GetCurrentThreadId, SetEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetDoubleClickTime, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GWL_EXSTYLE, GWL_STYLE, GetClassNameW, GetCursorPos, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, HWND_TOPMOST, IsWindow, LWA_ALPHA, MSG,
    MsgWaitForMultipleObjects, PM_NOREMOVE, PM_QS_SENDMESSAGE, PeekMessageW, QS_SENDMESSAGE,
    SPI_GETCLIENTAREAANIMATION, SW_HIDE, SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, ShowWindowAsync,
    SystemParametersInfoW, WS_CAPTION, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX,
    WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
};
use windows::core::{BOOL, HSTRING, PCWSTR, w};

use crate::place::Rect;
use crate::theme::{ChoixWindows, Rgb};

pub(crate) fn hwnd(v: isize) -> HWND {
    HWND(v as *mut core::ffi::c_void)
}

/// La fenêtre egui (HWND) d'une image, pour DWM et la place.
pub fn hwnd_de(frame: &eframe::Frame) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match frame.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get()),
        _ => None,
    }
}

fn lire_dword(cle: PCWSTR, valeur: PCWSTR) -> Option<u32> {
    let mut v = 0u32;
    let mut taille = 4u32;
    // SAFETY: tampon local de 4 octets annoncé comme tel ; RRF_RT_REG_DWORD garantit le type.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            cle,
            valeur,
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut v).cast()),
            Some(&mut taille),
        )
    };
    r.is_ok().then_some(v)
}

fn lire_octets(cle: PCWSTR, valeur: PCWSTR) -> Option<Vec<u8>> {
    let mut tampon = [0u8; 64];
    let mut taille = tampon.len() as u32;
    // SAFETY: tampon local dont la taille est passée ; Windows n'écrit pas au-delà.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            cle,
            valeur,
            RRF_RT_REG_BINARY,
            None,
            Some(tampon.as_mut_ptr().cast()),
            Some(&mut taille),
        )
    };
    r.is_ok()
        .then(|| tampon[..(taille as usize).min(tampon.len())].to_vec())
}

/// Mode sombre ou clair, barre des tâches, accent : ce que Windows a choisi.
pub fn choix_windows() -> ChoixWindows {
    let theme = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let sombre = lire_dword(theme, w!("AppsUseLightTheme")) == Some(0);
    let barre_sombre = lire_dword(theme, w!("SystemUsesLightTheme")) != Some(1);
    // AccentPalette : 8 couleurs RGBA, de la plus claire à la plus foncée (la 8e ne sert pas).
    let nuances = lire_octets(
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent"),
        w!("AccentPalette"),
    )
    .filter(|o| o.len() >= 28)
    .map(|o| std::array::from_fn(|i| Rgb(o[i * 4], o[i * 4 + 1], o[i * 4 + 2])));
    // AccentColor : 0xAABBGGRR.
    let accent = lire_dword(w!("Software\\Microsoft\\Windows\\DWM"), w!("AccentColor"))
        .map(|v| Rgb(v as u8, (v >> 8) as u8, (v >> 16) as u8));
    ChoixWindows {
        sombre,
        barre_sombre,
        accent,
        nuances,
    }
}

// ---------------------------------------------------------------------------------------
// Fenêtres de l'interface (egui) : DWM, place, premier plan

/// Coins arrondis (Windows 11) et barre de titre sombre ou claire.
pub fn habiller(h: isize, sombre: bool) {
    let coins = DWMWCP_ROUND;
    let sombre = BOOL::from(sombre);
    // SAFETY: valeurs locales passées avec leur taille exacte ; une version de Windows qui
    // ne connaît pas l'attribut renvoie une erreur sans effet.
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd(h),
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&raw const coins).cast(),
            size_of_val(&coins) as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd(h),
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&raw const sombre).cast(),
            size_of_val(&sombre) as u32,
        );
    }
}

/// L'écran entier et sa zone de travail (sans la barre des tâches), autour d'un point.
pub fn ecran_autour(x: i32, y: i32) -> Option<(Rect, Rect)> {
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: `info` est local et `cbSize` est réglé comme GetMonitorInfoW le demande.
    let ok = unsafe {
        let m = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        !m.is_invalid() && GetMonitorInfoW(m, &mut info).as_bool()
    };
    let r = |r: RECT| [r.left, r.top, r.right, r.bottom];
    ok.then(|| (r(info.rcMonitor), r(info.rcWork)))
}

/// La taille de la fenêtre en pixels.
pub fn taille_fenetre(h: isize) -> Option<[i32; 2]> {
    let mut r = RECT::default();
    // SAFETY: `r` est local ; la fenêtre est celle d'egui, vivante pendant l'image.
    unsafe { GetWindowRect(hwnd(h), &mut r) }.ok()?;
    Some([r.right - r.left, r.bottom - r.top])
}

/// Déplace la fenêtre en (x, y), en pixels, au premier plan des fenêtres « toujours
/// visibles » (le menu).
pub fn deplacer(h: isize, x: i32, y: i32, dessus: bool) {
    let apres = dessus.then_some(HWND_TOPMOST);
    let mut drapeaux = SWP_NOSIZE | SWP_NOACTIVATE;
    if !dessus {
        drapeaux |= SWP_NOZORDER;
    }
    // SAFETY: fenêtre d'egui vivante ; simples valeurs.
    let _ = unsafe { SetWindowPos(hwnd(h), apres, x, y, 0, 0, drapeaux) };
}

/// Voile la fenêtre (le compositeur ne la dessine plus à l'écran, même montrée) ou lève le
/// voile. Une fenêtre préparée d'avance reste voilée jusqu'à son ouverture : eframe 0.37
/// montre d'office une fenêtre à sa première image, même créée cachée, et winit retire alors
/// son opacité 0. Rend `false` si Windows refuse.
pub fn voiler(h: isize, voile: bool) -> bool {
    let v = BOOL::from(voile);
    // SAFETY: valeur locale passée avec sa taille exacte ; fenêtre d'egui vivante.
    unsafe {
        DwmSetWindowAttribute(
            hwnd(h),
            DWMWA_CLOAK,
            (&raw const v).cast(),
            size_of_val(&v) as u32,
        )
    }
    .is_ok()
}

/// Met la fenêtre au premier plan.
pub fn premier_plan(h: isize) {
    // SAFETY: fenêtre d'egui vivante ; Windows refuse sans dommage s'il n'est pas d'accord.
    let _ = unsafe { SetForegroundWindow(hwnd(h)) };
}

/// La fenêtre ne prend plus jamais le premier plan (`WS_EX_NOACTIVATE`) : un clic dessus
/// laisse le clavier à l'application d'en dessous. `outil` : aussi `WS_EX_TOOLWINDOW` (ni
/// bouton dans la barre des tâches, ni Alt+Tab). Vrai si Windows a gardé le style.
pub fn sans_focus(h: isize, outil: bool) -> bool {
    let voulu = WS_EX_NOACTIVATE.0 as isize | if outil { WS_EX_TOOLWINDOW.0 as isize } else { 0 };
    // SAFETY: lecture d'un style ; une poignée périmée renvoie 0 sans effet.
    let avant = unsafe { GetWindowLongPtrW(hwnd(h), GWL_EXSTYLE) };
    if avant & voulu != voulu {
        // SAFETY: écriture d'un style sur une fenêtre de ce processus (sinon Windows refuse).
        unsafe { SetWindowLongPtrW(hwnd(h), GWL_EXSTYLE, avant | voulu) };
    }
    // SAFETY: même lecture que ci-dessus.
    let apres = unsafe { GetWindowLongPtrW(hwnd(h), GWL_EXSTYLE) };
    apres & voulu == voulu
}

/// Retire la barre de titre et le cadre de Windows (la fenêtre devient un simple rectangle) :
/// `Decorations(false)` de winit ne le fait pas toujours après la création, et un autre
/// réglage de winit (boutons, visibilité) peut les remettre ; l'appel est sans effet quand
/// c'est déjà fait, donc à refaire à chaque image cachée. Vrai si la fenêtre est sans cadre.
pub fn sans_cadre(h: isize) -> bool {
    let cadre = (WS_CAPTION | WS_THICKFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX).0 as isize;
    // SAFETY: lecture d'un style ; une poignée périmée renvoie 0 sans effet.
    let avant = unsafe { GetWindowLongPtrW(hwnd(h), GWL_STYLE) };
    let apres = (avant & !cadre) | WS_POPUP.0 as isize;
    if apres != avant {
        // SAFETY: écriture d'un style sur une fenêtre de ce processus (sinon Windows refuse),
        // puis recalcul du cadre sans bouger, ni changer la taille, ni prendre le premier plan.
        unsafe {
            SetWindowLongPtrW(hwnd(h), GWL_STYLE, apres);
            let _ = SetWindowPos(
                hwnd(h),
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }
    // SAFETY: même lecture que ci-dessus.
    let fini = unsafe { GetWindowLongPtrW(hwnd(h), GWL_STYLE) };
    fini & cadre == 0
}

/// Montre la fenêtre sans la mettre au premier plan (`SW_SHOWNOACTIVATE`) : la commande
/// `Visible(true)` de winit utilise `SW_SHOW`, qui l'active et prend le clavier à
/// l'application où l'on écrit, même avec `WS_EX_NOACTIVATE`.
pub fn montrer_sans_activer(h: isize) {
    // SAFETY: simples valeurs ; une poignée périmée est refusée par Windows, sans effet.
    let _ = unsafe { ShowWindow(hwnd(h), SW_SHOWNOACTIVATE) };
}

/// Cache la fenêtre tout de suite (sans attendre l'image suivante d'egui) : une fenêtre
/// cachée n'est plus dessinée.
pub fn cacher(h: isize) {
    // SAFETY: fenêtre d'egui vivante, sur son propre fil ; ShowWindow ne fait que la cacher.
    let _ = unsafe { ShowWindow(hwnd(h), SW_HIDE) };
}

/// Le rectangle de la fenêtre, en pixels (gauche, haut, droite, bas).
pub fn rect_fenetre(h: isize) -> Option<Rect> {
    let mut r = RECT::default();
    // SAFETY: `r` est local ; la fenêtre est celle d'egui, vivante pendant l'image.
    unsafe { GetWindowRect(hwnd(h), &mut r) }.ok()?;
    Some([r.left, r.top, r.right, r.bottom])
}

/// L'opacité de toute la fenêtre (0 : invisible, 255 : opaque), par Windows lui-même
/// (fenêtre « en couches », opacité constante : elle garde ses coins arrondis et son ombre).
/// Sert au fondu du volet et du petit menu, et à les préparer invisibles. `false` si Windows
/// refuse.
pub fn transparente(h: isize, opacite: u8) -> bool {
    // SAFETY: fenêtre d'egui vivante, sur son propre fil ; le style étendu est relu puis
    // complété d'un seul drapeau ; l'opacité est une simple valeur.
    unsafe {
        let style = GetWindowLongPtrW(hwnd(h), GWL_EXSTYLE);
        let couches = WS_EX_LAYERED.0 as isize;
        if style & couches == 0 {
            SetWindowLongPtrW(hwnd(h), GWL_EXSTYLE, style | couches);
        }
        SetLayeredWindowAttributes(hwnd(h), COLORREF(0), opacite, LWA_ALPHA).is_ok()
    }
}

/// Depuis le fil principal, au clic : montre la fenêtre du volet, prête et encore
/// invisible (opacité 0), sans l'activer et sans attendre son fil. Elle reçoit aussitôt de
/// quoi se dessiner : le fil des fenêtres la met en place et l'anime sans attendre son tour
/// (une fenêtre cachée n'est réveillée par egui qu'après 100 ms). Rien si la fenêtre
/// n'existe plus ou n'est pas au logiciel.
pub fn reveiller(h: isize) {
    if h == 0 {
        return;
    }
    // SAFETY: on vérifie que la fenêtre existe et appartient à ce processus avant de la
    // montrer ; `ShowWindowAsync` ne fait que poster la demande à son fil.
    unsafe {
        let f = hwnd(h);
        if !IsWindow(Some(f)).as_bool() {
            return;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(f, Some(&mut pid));
        if pid != std::process::id() {
            return;
        }
        let _ = ShowWindowAsync(f, SW_SHOWNOACTIVATE);
    }
}

/// « Effets d'animation » de Windows (Paramètres > Accessibilité > Effets visuels) : coupés,
/// le volet et le petit menu s'ouvrent et se ferment sans animation. Une valeur que Windows
/// garde en mémoire (rien de lent).
pub fn effets_d_animation() -> bool {
    let mut actifs = BOOL::from(true);
    // SAFETY: `actifs` est un BOOL local, ce que SPI_GETCLIENTAREAANIMATION remplit.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&raw mut actifs).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    ok.is_err() || actifs.as_bool()
}

/// Le temps du double-clic réglé dans Windows.
pub fn temps_double_clic() -> Duration {
    // SAFETY: simple lecture d'un réglage de Windows.
    let ms = unsafe { GetDoubleClickTime() };
    Duration::from_millis(u64::from(ms.clamp(100, 5_000)))
}

/// Le réveil du fil des fenêtres (un événement Windows). Entre deux fenêtres, ce fil attend
/// une demande ; il doit pourtant répondre aux messages que Windows envoie à la fenêtre
/// cachée que winit lui laisse (« Winit Thread Event Target ») : sinon Windows le croit bloqué,
/// et à la fermeture de session il arrête le logiciel de force (« s'est bloquée »).
pub struct Reveil(isize);

impl Reveil {
    pub fn new() -> Self {
        // SAFETY: événement anonyme à remise automatique, fermé par `Drop`.
        let h = unsafe { CreateEventW(None, false, false, PCWSTR::null()) };
        Reveil(h.map_or(0, |h| h.0 as isize))
    }

    /// Lève le fil qui attend (une demande vient d'être envoyée).
    pub fn sonner(&self) {
        if self.0 != 0 {
            // SAFETY: événement créé par `new`, ouvert jusqu'à `Drop`.
            let _ = unsafe { SetEvent(HANDLE(self.0 as *mut _)) };
        }
    }

    /// Attend que le réveil sonne (au plus `ms`), en traitant au passage les messages
    /// *envoyés* aux fenêtres de ce fil (fin de session, « répond-il ? » de Windows). Les
    /// messages *postés* restent dans la file pour la prochaine fenêtre (winit les traite).
    pub fn attendre(&self, ms: u32) {
        let poignee = [HANDLE(self.0 as *mut _)];
        let (poignees, ms): (Option<&[HANDLE]>, u32) = if self.0 != 0 {
            (Some(&poignee), ms)
        } else {
            // Sans événement : on regarde la file souvent.
            (None, ms.min(50))
        };
        let mut m = MSG::default();
        // SAFETY: événement valide (ou aucun) ; `PeekMessageW` sur un MSG local, limité aux
        // messages envoyés, n'en retire aucun de la file.
        let resultat = unsafe {
            let r = MsgWaitForMultipleObjects(poignees, false, ms, QS_SENDMESSAGE);
            let _ = PeekMessageW(&mut m, None, 0, 0, PM_NOREMOVE | PM_QS_SENDMESSAGE);
            r
        };
        if resultat == WAIT_FAILED {
            // L'attente a échoué (poignée refusée) : une courte pause, pour ne jamais
            // tourner en boucle à 100 % du processeur.
            std::thread::sleep(Duration::from_millis(u64::from(ms.min(50))));
        }
    }
}

impl Default for Reveil {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Reveil {
    fn drop(&mut self) {
        if self.0 != 0 {
            // SAFETY: événement créé par `new`, fermé une seule fois.
            let _ = unsafe { CloseHandle(HANDLE(self.0 as *mut _)) };
        }
    }
}

/// La zone de travail de l'écran le plus proche de ce rectangle (gauche, haut, droite, bas).
pub fn zone_de_travail_pres_de(r: Rect) -> Option<Rect> {
    let [x, y] = crate::place::centre_de(r);
    ecran_autour(x, y).map(|(_, t)| t)
}

/// Pose la fenêtre à cette place (gauche, haut, largeur, hauteur, en pixels), sans changer
/// son rang.
pub fn poser(h: isize, p: crate::place::Place) {
    // SAFETY: fenêtre d'egui vivante ; simples valeurs.
    let _ = unsafe {
        SetWindowPos(
            hwnd(h),
            None,
            p[0],
            p[1],
            p[2],
            p[3],
            SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
}

/// La classe des fenêtres de winit (toutes celles d'egui). Les fenêtres cachées que le pilote
/// OpenGL crée dans le programme (« NVOGLDC invisible », « __wglDummyWindowFodder ») sont d'une
/// autre classe : elles ne sont jamais prises pour une fenêtre du logiciel.
const CLASSE_WINIT: &str = "Window Class";

/// La fenêtre de ce fil qui porte ce titre, de la classe de winit, et que `prises` ne compte pas
/// déjà (deux fenêtres du même titre ne se confondent pas). Les autres fenêtres d'egui
/// (« viewports ») ne donnent pas leur HWND : on les retrouve par leur titre, parmi les seules
/// fenêtres du fil des fenêtres. Jamais une fenêtre qu'on n'a pas créée n'est rendue.
pub fn fenetre_du_fil(titre: &str, prises: &[isize]) -> Option<isize> {
    struct Recherche<'a> {
        titre: Vec<u16>,
        prises: &'a [isize],
        trouvee: isize,
    }
    extern "system" fn chaque(h: HWND, lp: LPARAM) -> BOOL {
        // SAFETY: `lp` est le `&mut Recherche` donné à EnumThreadWindows plus bas, vivant et
        // utilisé nulle part ailleurs pendant l'appel ; le titre est lu dans un tampon local.
        let r = unsafe { &mut *(lp.0 as *mut Recherche<'_>) };
        if r.prises.contains(&(h.0 as isize)) {
            return BOOL::from(true);
        }
        let mut classe = [0u16; 64];
        // SAFETY: fenêtre donnée par Windows ; tampon local de la taille annoncée.
        let c = unsafe { GetClassNameW(h, &mut classe) };
        if String::from_utf16_lossy(classe.get(..c.max(0) as usize).unwrap_or(&[])) != CLASSE_WINIT {
            return BOOL::from(true);
        }
        let mut tampon = [0u16; 256];
        // SAFETY: fenêtre donnée par Windows ; tampon local de la taille annoncée.
        let n = unsafe { GetWindowTextW(h, &mut tampon) };
        if n > 0 && tampon.get(..n as usize) == Some(&r.titre[..]) {
            r.trouvee = h.0 as isize;
            return BOOL::from(false);
        }
        BOOL::from(true)
    }
    let mut r = Recherche {
        titre: titre.encode_utf16().collect(),
        prises,
        trouvee: 0,
    };
    // SAFETY: `chaque` ne garde pas le pointeur au-delà de l'appel (EnumThreadWindows rend la
    // main une fois les fenêtres vues) ; `r` vit jusque-là.
    let _ = unsafe { EnumThreadWindows(GetCurrentThreadId(), Some(chaque), LPARAM((&raw mut r) as isize)) };
    (r.trouvee != 0).then_some(r.trouvee)
}

/// La fenêtre existe-t-elle encore ? (Une fenêtre détruite : non.)
pub fn fenetre_existe(h: isize) -> bool {
    // SAFETY: simple question à Windows sur une poignée, même périmée.
    unsafe { IsWindow(Some(hwnd(h))) }.as_bool()
}

/// La position de la souris sur l'écran, en pixels.
pub fn position_souris() -> Option<[i32; 2]> {
    let mut p = POINT::default();
    // SAFETY: un POINT local.
    unsafe { GetCursorPos(&mut p) }.ok()?;
    Some([p.x, p.y])
}

/// Un bouton de la souris (gauche, droit ou milieu) est-il enfoncé en ce moment ? État réel
/// des boutons, sans dépendre du focus ni d'une fenêtre : lisible depuis n'importe quel fil,
/// sans crochet système.
pub fn bouton_enfonce() -> bool {
    [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON].into_iter().any(|b| {
        // SAFETY: simple lecture de l'état d'une touche virtuelle ; le bit de poids fort
        // (valeur négative) dit « enfoncée ».
        let etat = unsafe { GetAsyncKeyState(i32::from(b.0)) };
        etat < 0
    })
}

/// La place (gauche, haut) d'une fenêtre de cette taille (pixels), centrée sur la zone de
/// travail de l'écran de la souris.
pub fn centree(taille: [i32; 2]) -> Option<[i32; 2]> {
    let [x, y] = position_souris()?;
    let (_, t) = ecran_autour(x, y)?;
    Some(crate::place::centrer(taille, t))
}

// ---------------------------------------------------------------------------------------
// Le sélecteur de dossier

/// Le sélecteur de dossier de Windows (« Choisir… ») : le dossier choisi, ou `None` si
/// l'utilisateur annule. `proprietaire` : la fenêtre (HWND) au-dessus de laquelle il s'ouvre,
/// qu'il bloque tant qu'il est ouvert (pour les Paramètres :
/// [`crate::parametres::fenetre::choisir_dossier`]) ; `depart` : le dossier montré d'abord,
/// s'il est sur un disque de ce PC ([`dossier_de_depart`]). Seulement des dossiers du disque
/// (pas de bibliothèque ni d'emplacement virtuel). À appeler sur le fil des fenêtres : la
/// boîte tient ses propres messages pendant qu'elle est ouverte.
pub fn choisir_dossier(proprietaire: Option<isize>, depart: Option<&Path>, titre: &str) -> Option<PathBuf> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
        CoInitializeEx, CoTaskMemFree, CoUninitialize,
    };
    use windows::Win32::UI::Shell::{
        FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog, IShellItem,
        SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
    };
    // SAFETY: COM est initialisé pour ce fil (et libéré à la fin s'il l'a été ici) ; la
    // boîte de dialogue et les éléments sont des objets COM comptés par le crate `windows`,
    // libérés à la fin de leur portée ; les chaînes passées vivent pendant chaque appel ; le
    // texte rendu par `GetDisplayName` est copié, puis libéré une seule fois par
    // `CoTaskMemFree`. La fenêtre propriétaire est celle du logiciel, vivante pendant l'appel.
    unsafe {
        let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).is_ok();
        let choisi = (|| {
            let dialogue: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
            let options = dialogue.GetOptions().ok()?;
            dialogue
                .SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)
                .ok()?;
            let _ = dialogue.SetTitle(&HSTRING::from(titre));
            // Toujours un dossier de départ, sur un disque de ce PC : sans lui, Windows
            // rouvrirait de lui-même le dernier dossier choisi, peut-être un partage réseau
            // muet, qui ferait attendre ce fil de 20 à 60 s.
            if let Some(d) = dossier_de_depart(depart) {
                let element: Option<IShellItem> =
                    SHCreateItemFromParsingName(&HSTRING::from(d.as_os_str()), None).ok();
                if let Some(e) = element {
                    let _ = dialogue.SetFolder(&e);
                }
            }
            // Une erreur si l'utilisateur annule.
            let _imbriquee = crate::sonde::imbriquee("sélecteur de dossier");
            dialogue.Show(proprietaire.map(hwnd)).ok()?;
            let texte = dialogue
                .GetResult()
                .ok()?
                .GetDisplayName(SIGDN_FILESYSPATH)
                .ok()?;
            let chemin = texte.to_string().ok();
            CoTaskMemFree(Some(texte.0 as *const _));
            chemin.map(PathBuf::from)
        })();
        if com {
            CoUninitialize();
        }
        choisi
    }
}

/// Le dossier montré d'abord par le sélecteur : `voulu` s'il est sur un disque de ce PC,
/// sinon le profil de l'utilisateur, sinon la racine du disque du système. Toujours un
/// dossier local et sûr (jamais un partage réseau, qui peut faire attendre le fil).
pub fn dossier_de_depart(voulu: Option<&Path>) -> Option<PathBuf> {
    let profil = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let systeme = std::env::var_os("SystemDrive").map(|d| {
        let mut r = d.to_string_lossy().into_owned();
        r.push('\\');
        PathBuf::from(r)
    });
    voulu
        .map(Path::to_path_buf)
        .into_iter()
        .chain(profil)
        .chain(systeme)
        .find(|d| disque_local(d) && d.is_dir())
}

/// Un chemin sur un disque fixe de ce PC (`C:\…`) : le seul genre de dossier regardé depuis
/// le fil des fenêtres. Jamais un chemin réseau ou de périphérique (`\\…`, `//…`), ni un
/// lecteur réseau (`Z:\…` relié à un partage, qui peut ne pas répondre), ni un lecteur
/// amovible ou de CD (qui peut se faire attendre), ni un chemin qui passe par un point de
/// redirection (lien symbolique ou jonction, qui peut mener à un partage réseau).
pub fn disque_local(d: &Path) -> bool {
    /// `DRIVE_FIXED` de Windows (un disque du PC).
    const DRIVE_FIXED: u32 = 3;
    let Some(racine) = racine_de_lecteur(d) else {
        return false;
    };
    // SAFETY: chaîne terminée par zéro, vivante pendant l'appel. Pour une racine de lecteur
    // (`Z:\`), Windows répond d'après sa table des lecteurs, sans attendre le réseau
    // (`tests::disque_local_repond_vite`).
    let fixe = unsafe { GetDriveTypeW(&HSTRING::from(racine)) == DRIVE_FIXED };
    fixe && sans_redirection(d)
}

/// Aucun dossier du chemin (de la racine vers le bout) n'est un point de redirection : lien
/// symbolique ou jonction. Chacun est regardé sans être suivi (`symlink_metadata`), et on
/// s'arrête au premier : on ne passe jamais au travers. Un dossier qui n'existe pas (encore)
/// arrête la recherche sans refus ; une autre erreur compte comme une redirection.
fn sans_redirection(d: &Path) -> bool {
    let mut parties: Vec<&Path> = d.ancestors().collect();
    parties.reverse();
    // La racine (`C:\`) elle-même n'est jamais une redirection.
    for p in parties.into_iter().skip(1) {
        match std::fs::symlink_metadata(p) {
            // Rust range parmi les liens les jonctions et les liens symboliques, pas les
            // fichiers de OneDrive à la demande.
            Ok(m) if m.file_type().is_symlink() => return false,
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
            Err(_) => return false,
        }
    }
    true
}

/// `C:\` pour `C:\…` ou `C:/…` ; `None` pour tout autre chemin (réseau, relatif…).
fn racine_de_lecteur(d: &Path) -> Option<String> {
    let b = d.to_str()?.as_bytes();
    (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'))
        .then(|| format!("{}:\\", char::from(b[0])))
}

// ---------------------------------------------------------------------------------------
// Des fenêtres qui ne « gèlent » pas

/// Avant toute fenêtre : Windows ne remplace jamais une fenêtre du logiciel par son « fantôme »
/// (cadre gris, « (Ne répond pas) », bouton qui ferme le logiciel de force), pour tout le
/// processus. Pour un logiciel qui fait un travail long pendant que sa fenêtre est ouverte
/// (un changement d'écran, un pilote de carte graphique qui retient le dessin) : ce n'est pas
/// un blocage, et la fermer de force arrêterait le travail en cours. Facultatif :
/// [`crate::horloge::Horloge::sans_fantome`] l'appelle au bon moment.
pub fn sans_fantome() {
    use windows::Win32::UI::WindowsAndMessaging::DisableProcessWindowsGhosting;
    // SAFETY: simple appel sans paramètre, pour tout le processus.
    unsafe { DisableProcessWindowsGhosting() };
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use std::time::Instant;
    use windows::Win32::Foundation::{LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WINDOW_EX_STYLE, WNDCLASSW,
        WS_OVERLAPPED,
    };

    extern "system" fn procedure(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        // SAFETY: les paramètres viennent de Windows et sont rendus tels quels à la procédure par défaut.
        unsafe { DefWindowProcW(h, m, w, l) }
    }

    /// Une fenêtre jamais montrée, de cette classe, avec ce titre, sur le fil qui appelle.
    fn cachee(classe: &str, titre: &str) -> isize {
        let c: Vec<u16> = classe.encode_utf16().chain(std::iter::once(0)).collect();
        let t: Vec<u16> = titre.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: modules et textes locaux terminés par 0, vivants pendant les appels ; la classe
        // n'a besoin que de la procédure par défaut ; la fenêtre est détruite par l'essai.
        unsafe {
            let instance = GetModuleHandleW(None).unwrap();
            let classe_w = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance.into(),
                lpszClassName: PCWSTR(c.as_ptr()),
                ..Default::default()
            };
            // Déjà enregistrée par un autre essai : sans importance.
            let _ = RegisterClassW(&classe_w);
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PCWSTR(c.as_ptr()),
                PCWSTR(t.as_ptr()),
                WS_OVERLAPPED,
                0,
                0,
                100,
                50,
                None,
                None,
                Some(instance.into()),
                None,
            )
            .unwrap()
            .0 as isize
        }
    }

    /// `fenetre_du_fil` ne rend jamais une fenêtre d'une autre classe que celle de winit (ex. « NVOGLDC
    /// invisible », créée cachée par le pilote OpenGL de NVIDIA dans chaque programme), même au
    /// même titre ; deux fenêtres de winit du même titre ne se confondent pas (`prises`).
    /// Les fenêtres ne sont jamais montrées.
    #[test]
    fn fenetre_du_fil_ne_rend_que_les_fenetres_de_winit_pas_prises() {
        let titre = "Titre d'essai de la fenetre du fil";
        // L'étrangère est créée la première, puis deux fenêtres de winit au même titre.
        let etrangere = cachee("STATIC", titre);
        let premiere = cachee(CLASSE_WINIT, titre);
        let deuxieme = cachee(CLASSE_WINIT, titre);
        assert!(etrangere != 0 && premiere != 0 && deuxieme != 0);
        let a = fenetre_du_fil(titre, &[]).expect("une fenêtre de winit");
        assert_ne!(a, etrangere, "jamais la fenêtre étrangère");
        assert!(a == premiere || a == deuxieme);
        let b = fenetre_du_fil(titre, &[a]).expect("l'autre fenêtre de winit");
        assert_ne!(b, etrangere, "jamais la fenêtre étrangère");
        assert!(b != a && (b == premiere || b == deuxieme));
        assert_eq!(fenetre_du_fil(titre, &[a, b]), None, "les deux prises : rien");
        assert_eq!(fenetre_du_fil("Un autre titre", &[]), None);
        for h in [etrangere, premiere, deuxieme] {
            // SAFETY: fenêtres créées ci-dessus, sur ce fil.
            let _ = unsafe { DestroyWindow(hwnd(h)) };
        }
    }

    /// Le sélecteur ne regarde jamais un chemin réseau depuis le fil des fenêtres : seulement
    /// un disque fixe de ce PC.
    #[test]
    fn chemins_reseau_reconnus() {
        for reseau in [
            r"\\serveur\partage",
            r"\\?\C:\x",
            r"\\.\C:\x",
            "//serveur/partage",
            r"Installateurs\x",
            "C:",
            "",
        ] {
            assert_eq!(racine_de_lecteur(Path::new(reseau)), None, "{reseau}");
            assert!(!disque_local(Path::new(reseau)), "{reseau}");
        }
        assert_eq!(
            racine_de_lecteur(Path::new(r"Z:\Installateurs")).as_deref(),
            Some(r"Z:\")
        );
        assert_eq!(
            racine_de_lecteur(Path::new("d:/Installateurs")).as_deref(),
            Some(r"d:\")
        );
        // Le disque du système est un disque fixe.
        let systeme = std::env::var("SystemRoot").expect("SystemRoot");
        assert!(disque_local(Path::new(&systeme)), "{systeme}");
    }

    /// Un chemin qui passe par un lien ou une jonction n'est pas regardé comme local : il peut
    /// mener à un partage réseau. Aucune fenêtre ne s'ouvre pendant l'essai.
    #[test]
    fn redirection_jamais_locale() {
        use std::os::windows::process::CommandExt;
        /// `CREATE_NO_WINDOW` : aucune fenêtre ne s'ouvre.
        const SANS_FENETRE: u32 = 0x0800_0000;
        let d = std::env::temp_dir().join(format!("interface-commune-jonction-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let cible = d.join("cible");
        std::fs::create_dir_all(cible.join("dedans")).expect("dossier");
        let jonction = d.join("jonction");
        let fait = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&jonction)
            .arg(&cible)
            .creation_flags(SANS_FENETRE)
            .output()
            .expect("mklink");
        assert!(fait.status.success(), "{fait:?}");
        assert!(disque_local(&cible.join("dedans")), "un vrai dossier");
        assert!(disque_local(&d.join("pas-encore-cree")), "pas encore créé");
        assert!(!disque_local(&jonction), "la jonction elle-même");
        assert!(
            !disque_local(&jonction.join("dedans")),
            "au travers de la jonction"
        );
        // Le sélecteur part alors du profil de l'utilisateur.
        let profil = PathBuf::from(std::env::var("USERPROFILE").expect("profil"));
        assert_eq!(dossier_de_depart(Some(&jonction)), Some(profil.clone()));
        assert_eq!(
            dossier_de_depart(Some(Path::new(r"\\serveur\partage"))),
            Some(profil.clone())
        );
        assert_eq!(dossier_de_depart(None), Some(profil));
        assert_eq!(dossier_de_depart(Some(&cible)), Some(cible.clone()));
        // Retirer la jonction ne touche pas à sa cible.
        std::fs::remove_dir(&jonction).expect("jonction retirée");
        assert!(cible.join("dedans").is_dir());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Savoir si un lecteur est un disque fixe n'attend pas le réseau : toutes les lettres,
    /// lecteurs réseau compris (même injoignables), en bien moins d'une seconde.
    #[test]
    fn disque_local_repond_vite() {
        let debut = Instant::now();
        for lettre in 'A'..='Z' {
            let _ = disque_local(Path::new(&format!(r"{lettre}:\Installateurs")));
        }
        let duree = debut.elapsed();
        assert!(duree < Duration::from_secs(1), "{duree:?}");
    }
}
