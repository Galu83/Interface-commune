//! Les essais de l'icône seule (`horloge`), repris de l'ancien `horloge.rs` (bloc M) : le
//! décodage pur, sans Windows ni fenêtre.

#![allow(clippy::unwrap_used)]

use std::time::{Duration, Instant};

use windows::Win32::UI::WindowsAndMessaging::{
    PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, PBT_POWERSETTINGCHANGE, WM_APP, WM_CLOSE,
    WM_COPYDATA, WM_DESTROY, WM_DISPLAYCHANGE, WM_ENDSESSION, WM_POWERBROADCAST, WM_QUERYENDSESSION,
    WM_SETTINGCHANGE, WM_TIMER,
};

use super::decodage::{
    Alimentation, Destinataire, ECRAN_DE_LA_CONSOLE, MINUTERIE_ICONE, WM_COMMANDE, WM_ICONE,
    WM_MENU_DE_SECOURS, alimentation, destinataire,
};
use super::fil::{ATTENTE_FERMETURE, attendre_la_fin};
use crate::bords::unique::WM_OUVRIR_PARAMETRES;

/// Veille, reprise et écran : ce que la bibliothèque lit d'un `WM_POWERBROADCAST`.
#[test]
fn veille_reprise_et_ecran() {
    assert_eq!(alimentation(PBT_APMSUSPEND, None), Some(Alimentation::Veille));
    assert_eq!(
        alimentation(PBT_APMRESUMEAUTOMATIC, None),
        Some(Alimentation::Reprise)
    );
    assert_eq!(
        alimentation(PBT_APMRESUMESUSPEND, None),
        Some(Alimentation::Reprise)
    );
    assert_eq!(
        alimentation(PBT_POWERSETTINGCHANGE, Some(0)),
        Some(Alimentation::Ecran { allume: false })
    );
    assert_eq!(
        alimentation(PBT_POWERSETTINGCHANGE, Some(1)),
        Some(Alimentation::Ecran { allume: true })
    );
    // Assombri : allumé.
    assert_eq!(
        alimentation(PBT_POWERSETTINGCHANGE, Some(2)),
        Some(Alimentation::Ecran { allume: true })
    );
    // Un autre réglage, ou un autre événement : rien.
    assert_eq!(alimentation(PBT_POWERSETTINGCHANGE, None), None);
    assert_eq!(alimentation(10, None), None);
}

/// Le GUID de l'écran est bien celui de Windows.
#[test]
fn guid_de_l_ecran() {
    assert_eq!(
        ECRAN_DE_LA_CONSOLE,
        windows::core::GUID::from_u128(0x6fe69556_704a_47a0_8f24_c28d936fda47)
    );
}

/// Les messages de Windows : ceux de la bibliothèque restent à elle ; la fin de session et
/// les changements de réglages sont vus des deux ; tous les autres (raccourcis globaux,
/// messages du logiciel) vont au logiciel.
#[test]
fn messages_de_windows_au_logiciel() {
    use windows::Win32::UI::WindowsAndMessaging::WM_HOTKEY;
    let barre = 0xC123;
    for m in [
        WM_ICONE,
        WM_COMMANDE,
        WM_OUVRIR_PARAMETRES,
        WM_MENU_DE_SECOURS,
        WM_COPYDATA,
        WM_CLOSE,
        WM_DESTROY,
        barre,
    ] {
        assert_eq!(destinataire(m, 0, barre), Destinataire::Bibliotheque, "{m:#x}");
    }
    for m in [
        WM_QUERYENDSESSION,
        WM_ENDSESSION,
        WM_SETTINGCHANGE,
        WM_DISPLAYCHANGE,
        WM_POWERBROADCAST,
    ] {
        assert_eq!(destinataire(m, 0, barre), Destinataire::LesDeux, "{m:#x}");
    }
    for m in [WM_HOTKEY, WM_APP + 100] {
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

/// Une fenêtre qui ne répond plus ne retient pas la fermeture au-delà du délai.
#[test]
fn fenetre_bloquee_ne_retient_pas_la_fermeture() {
    let (lacher, attend) = std::sync::mpsc::channel::<()>();
    let bloque = std::thread::spawn(move || {
        let _ = attend.recv();
    });
    let debut = Instant::now();
    assert!(!attendre_la_fin(bloque, Duration::from_millis(150)));
    let duree = debut.elapsed();
    assert!(
        duree >= Duration::from_millis(150) && duree < Duration::from_secs(3),
        "{duree:?}"
    );
    let _ = lacher.send(());
    // Un fil qui finit vite est attendu et joint.
    let vite = std::thread::spawn(|| ());
    assert!(attendre_la_fin(vite, Duration::from_secs(5)));
    assert_eq!(ATTENTE_FERMETURE, Duration::from_secs(5));
}
