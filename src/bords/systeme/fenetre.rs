//! Les fenêtres de l'interface (egui), vues de Windows : DWM (coins arrondis, barre de titre
//! sombre, voile), la place, le premier plan, les styles (sans focus, sans cadre), l'opacité
//! (fondu), le réveil du volet, la recherche d'une fenêtre d'egui par son titre, et « pas de
//! fantôme ». Chaque `unsafe` est commenté.

use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAK, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    DwmSetWindowAttribute,
};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GWL_EXSTYLE, GWL_STYLE, GetClassNameW, GetWindowLongPtrW, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, HWND_TOPMOST, IsWindow, LWA_ALPHA, SW_HIDE, SW_SHOWNOACTIVATE,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, ShowWindowAsync, WS_CAPTION,
    WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_POPUP, WS_SYSMENU,
    WS_THICKFRAME,
};
use windows::core::BOOL;

use crate::place::{Place, Rect};

/// Une poignée de fenêtre, depuis le nombre que la bibliothèque garde.
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

/// Pose la fenêtre à cette place (gauche, haut, largeur, hauteur, en pixels), sans changer
/// son rang.
pub fn poser(h: isize, p: Place) {
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
pub(crate) const CLASSE_WINIT: &str = "Window Class";

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
