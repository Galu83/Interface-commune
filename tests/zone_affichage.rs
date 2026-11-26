//! La fenêtre cachée de la zone de notification reçoit `WM_DISPLAYCHANGE` (Windows change
//! l'affichage) : les fenêtres pas encore créées du logiciel attendent alors la fin de la
//! reprise ([`autres_fenetres::REPRISE`]). Aucune fenêtre n'est montrée ; aucune icône n'est
//! donnée à Windows. Un seul essai dans ce fichier : l'heure du changement d'affichage est
//! gardée pour tout le programme.

#![cfg(windows)]
// Dans un essai, un échec doit arrêter l'essai (`expect`) ; le message et sa boucle
// passent par Windows (`unsafe`).
#![allow(clippy::expect_used, unsafe_code)]

use std::time::Duration;

use interface_commune::autres_fenetres::{REPRISE, attente_avant_creation, depuis_le_changement};
use interface_commune::zone::Zone;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, PostMessageW, TranslateMessage, WM_DISPLAYCHANGE,
};

#[test]
fn la_zone_signale_le_changement_d_affichage() {
    let zone = Zone::creer("EssaiZoneAffichage", "Essai").expect("zone");
    assert!(
        depuis_le_changement().is_none(),
        "aucun changement avant le message"
    );

    // SAFETY: fenêtre de ce fil, message sans pointeur (profondeur de couleur, taille).
    unsafe {
        PostMessageW(
            Some(HWND(zone.fenetre() as *mut core::ffi::c_void)),
            WM_DISPLAYCHANGE,
            WPARAM(32),
            LPARAM(0),
        )
    }
    .expect("message posté");
    let mut m = MSG::default();
    // SAFETY: boucle des messages de ce fil, sur un MSG local.
    unsafe {
        while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }

    let depuis = depuis_le_changement().expect("le changement d'affichage est noté");
    assert!(depuis < Duration::from_secs(5), "{depuis:?}");
    // Une fenêtre pas encore créée attend la reprise.
    let attente = attente_avant_creation(Some(depuis), None, None);
    assert!(attente.is_some_and(|a| a <= REPRISE), "{attente:?}");
}
