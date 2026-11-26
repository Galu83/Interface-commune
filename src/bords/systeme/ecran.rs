//! L'écran et la souris, et deux réglages de Windows (C-112) : l'écran autour d'un point et sa
//! zone de travail, la place de la souris et ses boutons, « Effets d'animation », le temps du
//! double-clic. Des lectures sans fenêtre, rien de lent. Chaque `unsafe` est commenté.

use std::time::Duration;

use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetDoubleClickTime, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};
use windows::core::BOOL;

use crate::place::Rect;

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

/// La zone de travail de l'écran le plus proche de ce rectangle (gauche, haut, droite, bas).
pub fn zone_de_travail_pres_de(r: Rect) -> Option<Rect> {
    let [x, y] = crate::place::centre_de(r);
    ecran_autour(x, y).map(|(_, t)| t)
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

/// Le temps du double-clic réglé dans Windows (borné par le cœur :
/// [`ic_coeur::gestes::clics::temps_double_clic`]).
pub fn temps_double_clic() -> Duration {
    // SAFETY: simple lecture d'un réglage de Windows.
    let ms = unsafe { GetDoubleClickTime() };
    ic_coeur::gestes::clics::temps_double_clic(ms)
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
