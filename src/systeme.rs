//! Ce qui parle à Windows pour les fenêtres : le thème et l'accent, les coins arrondis et la
//! barre de titre sombre (DWM), la place, l'opacité (fondu), « Effets d'animation », le
//! réveil du fil des fenêtres. Chaque `unsafe` est commenté.

use std::time::Duration;

use windows::Win32::Foundation::{COLORREF, CloseHandle, HANDLE, HWND, LPARAM, POINT, RECT, WAIT_FAILED};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAK, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, RRF_RT_REG_BINARY, RRF_RT_REG_DWORD, RegGetValueW,
};
use windows::Win32::System::Threading::{CreateEventW, GetCurrentThreadId, SetEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GWL_EXSTYLE, GetCursorPos, GetWindowLongPtrW, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, HWND_TOPMOST, IsWindow, LWA_ALPHA, MSG, MsgWaitForMultipleObjects, PM_NOREMOVE,
    PM_QS_SENDMESSAGE, PeekMessageW, QS_SENDMESSAGE, SPI_GETCLIENTAREAANIMATION, SW_HIDE, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, ShowWindowAsync,
    SystemParametersInfoW, WS_EX_LAYERED,
};
use windows::core::{BOOL, PCWSTR, w};

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
    ecran_autour(r[0] / 2 + r[2] / 2, r[1] / 2 + r[3] / 2).map(|(_, t)| t)
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

/// La fenêtre de ce fil qui porte ce titre. Les autres fenêtres d'egui (« viewports ») ne
/// donnent pas leur HWND : on les retrouve par leur titre, parmi les seules fenêtres du fil
/// des fenêtres.
pub fn fenetre_du_fil(titre: &str) -> Option<isize> {
    struct Recherche {
        titre: Vec<u16>,
        trouvee: isize,
    }
    extern "system" fn chaque(h: HWND, lp: LPARAM) -> BOOL {
        // SAFETY: `lp` est le `&mut Recherche` donné à EnumThreadWindows plus bas, vivant et
        // utilisé nulle part ailleurs pendant l'appel ; le titre est lu dans un tampon local.
        let r = unsafe { &mut *(lp.0 as *mut Recherche) };
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
        trouvee: 0,
    };
    // SAFETY: `chaque` ne garde pas le pointeur au-delà de l'appel (EnumThreadWindows rend la
    // main une fois les fenêtres vues) ; `r` vit jusque-là.
    let _ = unsafe { EnumThreadWindows(GetCurrentThreadId(), Some(chaque), LPARAM((&raw mut r) as isize)) };
    (r.trouvee != 0).then_some(r.trouvee)
}

/// La place (gauche, haut) d'une fenêtre de cette taille (pixels), centrée sur la zone de
/// travail de l'écran de la souris.
pub fn centree(taille: [i32; 2]) -> Option<[i32; 2]> {
    let mut p = POINT::default();
    // SAFETY: un POINT local.
    unsafe { GetCursorPos(&mut p) }.ok()?;
    let (_, t) = ecran_autour(p.x, p.y)?;
    Some(crate::place::centrer(taille, t))
}
