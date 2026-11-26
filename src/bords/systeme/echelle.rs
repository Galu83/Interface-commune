//! Une fenêtre glissée d'un écran à l'autre quand les deux n'ont pas la même échelle (125 %,
//! 150 %…) : elle suit la souris, sans sauter ni revenir sur l'écran qu'elle quitte.
//!
//! winit 0.30 répond au message `WM_DPICHANGED` de Windows en posant la fenêtre à un endroit
//! qu'il calcule lui-même, et qu'il « repousse » vers l'écran que la fenêtre quitte : Windows
//! rechange alors l'échelle, et la fenêtre fait des allers-retours (le détail :
//! [`crate::place::place_au_changement_d_echelle`]). La bibliothèque glisse donc sa propre
//! procédure devant celle de winit (une « sous-classe » de Windows) : pendant que winit traite
//! ce message, le déplacement qu'il demande est remplacé par le rectangle que Windows a
//! suggéré. winit garde tout le reste (la nouvelle échelle, qu'egui reçoit comme d'habitude).
//! C'est le correctif de winit (#4341), qui n'est pas encore dans sa version 0.30.
//!
//! Et, avant toute fenêtre, le réglage d'échelle du processus (C-113) : chaque écran à sa
//! propre échelle ([`reglage_dpi`]). Chaque `unsafe` est commenté.

use std::cell::Cell;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    SWP_NOMOVE, SWP_NOSIZE, WINDOWPOS, WM_DPICHANGED, WM_NCDESTROY, WM_WINDOWPOSCHANGING,
};

use crate::place::{Rect, place_au_changement_d_echelle};

/// Avant toute fenêtre : chaque écran à sa propre échelle (sinon Windows étire tout).
pub fn reglage_dpi() {
    // SAFETY: simple appel sans pointeur ; un échec (déjà réglé) est sans conséquence.
    let _ = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// Le numéro de notre sous-classe (une fenêtre peut en avoir plusieurs).
const ID: usize = 0x4543_4845;

// ÉTAT GLOBAL : la procédure de sous-classe ne reçoit rien d'autre que le message ;
// WM_DPICHANGED pose le rectangle suggéré, WM_WINDOWPOSCHANGING (envoyé pendant l'appel, sur le
// même fil) le lit. Un par fil, rendu à sa valeur d'avant à la fin du message.
thread_local! {
    /// Le changement d'échelle en cours sur ce fil : la fenêtre et le rectangle suggéré.
    static EN_COURS: Cell<Option<(isize, Rect)>> = const { Cell::new(None) };
}

/// Une fenêtre de ce fil suit désormais le rectangle de Windows quand son échelle change.
/// À appeler sur le fil de la fenêtre, une fois qu'elle existe ; sans effet de plus si c'est
/// déjà fait. Vrai si Windows a accepté.
pub fn suivre_windows(h: isize) -> bool {
    // SAFETY: fenêtre de ce fil (Windows refuse sinon) ; la procédure est une fonction
    // statique, retirée par elle-même quand la fenêtre est détruite.
    unsafe { SetWindowSubclass(HWND(h as *mut _), Some(procedure), ID, 0) }.as_bool()
}

/// Notre procédure, appelée avant celle de winit pour chaque message de la fenêtre.
unsafe extern "system" fn procedure(
    h: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _donnees: usize,
) -> LRESULT {
    match message {
        WM_DPICHANGED if lparam.0 != 0 => {
            // SAFETY: pour WM_DPICHANGED, `lparam` pointe sur le RECT suggéré par Windows,
            // valable pendant le message.
            let r = unsafe { *(lparam.0 as *const RECT) };
            let suggere = [r.left, r.top, r.right, r.bottom];
            let avant = EN_COURS.replace(Some((h.0 as isize, suggere)));
            // SAFETY: on passe le message tel quel à winit.
            let resultat = unsafe { DefSubclassProc(h, message, wparam, lparam) };
            EN_COURS.set(avant);
            resultat
        }
        WM_WINDOWPOSCHANGING if lparam.0 != 0 => {
            if let Some((fenetre, suggere)) = EN_COURS.get()
                && fenetre == h.0 as isize
            {
                // SAFETY: pour WM_WINDOWPOSCHANGING, `lparam` pointe sur le WINDOWPOS que
                // Windows va appliquer, modifiable pendant le message.
                let pos = unsafe { &mut *(lparam.0 as *mut WINDOWPOS) };
                let [x, y, cx, cy] = place_au_changement_d_echelle(
                    [pos.x, pos.y, pos.cx, pos.cy],
                    pos.flags.contains(SWP_NOMOVE),
                    pos.flags.contains(SWP_NOSIZE),
                    suggere,
                );
                (pos.x, pos.y, pos.cx, pos.cy) = (x, y, cx, cy);
            }
            // SAFETY: on passe le message (corrigé) à winit.
            unsafe { DefSubclassProc(h, message, wparam, lparam) }
        }
        WM_NCDESTROY => {
            // SAFETY: la fenêtre est encore valable pendant WM_NCDESTROY ; on retire notre
            // propre sous-classe.
            unsafe {
                let _ = RemoveWindowSubclass(h, Some(procedure), ID);
                DefSubclassProc(h, message, wparam, lparam)
            }
        }
        // SAFETY: tout le reste passe tel quel à winit.
        _ => unsafe { DefSubclassProc(h, message, wparam, lparam) },
    }
}
