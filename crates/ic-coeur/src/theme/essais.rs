//! Les essais du thème : contrastes, accent de Windows, courbe et durées de l'animation, pas
//! d'une image en retard, relecture du thème avec une horloge inventée (le temps écoulé est
//! donné, jamais lu), lettre des pastilles, couverture d'une forme. Données inventées : le
//! logiciel « Fictif » et ses couleurs.

use std::time::Duration;

use super::couleurs::{BLANC, NOIR};
use super::*;

/// Le texte est lisible sur tous les fonds, en clair comme en sombre, quel que soit
/// l'accent de Windows (règle : pas de petit texte gris pâle).
#[test]
fn contrastes_suffisants() {
    let accents = [
        None,
        Some(Rgb(0, 120, 215)),
        Some(Rgb(255, 185, 0)),
        Some(Rgb(16, 124, 16)),
        Some(Rgb(232, 17, 35)),
        Some(Rgb(0, 0, 0)),
        Some(Rgb(255, 255, 255)),
    ];
    for sombre in [false, true] {
        for a in accents {
            let p = Palette::de(&ChoixWindows {
                sombre,
                barre_sombre: sombre,
                accent: a,
                nuances: None,
            });
            for fond in [p.fond, p.fond_menu, p.carte, p.survol, p.controle] {
                assert!(contraste(p.texte, fond) >= 7.0, "{p:?}");
                assert!(contraste(p.texte_secondaire, fond) >= 4.5, "{p:?}");
                for etat in [p.vert, p.orange, p.rouge] {
                    assert!(contraste(etat, fond) >= 3.0, "état peu lisible : {p:?}");
                }
            }
            assert!(contraste(p.sur_accent, p.accent) >= 3.0, "{p:?}");
        }
    }
}

#[test]
fn la_palette_de_windows_est_suivie() {
    let n = [
        Rgb(1, 1, 1),
        Rgb(2, 2, 2),
        Rgb(3, 3, 3),
        Rgb(4, 4, 4),
        Rgb(5, 5, 5),
        Rgb(6, 6, 6),
        Rgb(7, 7, 7),
    ];
    let c = |sombre| ChoixWindows {
        sombre,
        barre_sombre: true,
        accent: None,
        nuances: Some(n),
    };
    assert_eq!(Palette::de(&c(true)).accent, Rgb(2, 2, 2));
    assert_eq!(Palette::de(&c(false)).accent, Rgb(5, 5, 5));
}

/// Sans accent donné par Windows, le bleu de Windows 11 ; avec un accent sans ses nuances,
/// l'accent éclairci en sombre et foncé en clair.
#[test]
fn accent_par_defaut_et_accent_seul() {
    let sans = |sombre| ChoixWindows {
        sombre,
        ..ChoixWindows::default()
    };
    assert_eq!(Palette::de(&sans(false)).accent, Rgb::hex(0x005FB8));
    assert_eq!(Palette::de(&sans(true)).accent, Rgb::hex(0x60CDFF));
    let accent = Rgb::hex(0x7B57C9);
    let seul = |sombre| ChoixWindows {
        sombre,
        accent: Some(accent),
        ..ChoixWindows::default()
    };
    assert_eq!(Palette::de(&seul(true)).accent, accent.vers(BLANC, 0.35));
    assert_eq!(Palette::de(&seul(false)).accent, accent.vers(NOIR, 0.15));
}

/// La courbe de Windows : part de 0, arrive à 1, toujours croissante, et déjà bien
/// avancée à mi-temps (rapide au départ, ralentie à l'arrivée).
#[test]
fn courbe_rapide_puis_ralentie() {
    assert!(ralenti_a_l_arrivee(0.0).abs() < 1e-6);
    assert!((ralenti_a_l_arrivee(1.0) - 1.0).abs() < 1e-6);
    assert!(ralenti_a_l_arrivee(0.5) > 0.8, "{}", ralenti_a_l_arrivee(0.5));
    let mut avant = 0.0;
    for i in 1..=100 {
        let y = ralenti_a_l_arrivee(i as f32 / 100.0);
        assert!(y >= avant, "la courbe recule à {i} %");
        avant = y;
    }
    // Hors de 0 à 1 : bornée.
    assert!((ralenti_a_l_arrivee(2.0) - 1.0).abs() < 1e-6);
    assert!(ralenti_a_l_arrivee(-1.0).abs() < 1e-6);
}

/// Les durées de Windows 11 : ouvrir et fermer en 83 ms, la plus courte des durées des
/// contrôles de Windows (une ouverture plus longue se ressent comme une attente).
#[test]
fn durees_de_windows() {
    assert_eq!(OUVERTURE_FONDU_MS, 83);
    assert_eq!(FERMETURE_FONDU_MS, 83);
}

/// L'ouverture est vive : la première image montrée est déjà visible (jamais
/// transparente), lisible (80 % d'opacité) au plus tard à la deuxième image d'un écran à
/// 60 Hz, opaque et finie en 83 ms, toujours croissante.
#[test]
fn ouverture_vive() {
    let debut = ouverture(0, true);
    assert_eq!(debut.opacite, OUVERTURE_OPACITE_DE_DEPART);
    assert!(debut.opacite >= 128, "{}", debut.opacite);
    assert!(!debut.finie);
    assert!(
        ouverture(17, true).opacite >= 204,
        "{}",
        ouverture(17, true).opacite
    );
    let mut avant = 0;
    for ms in 0..=OUVERTURE_FONDU_MS {
        let i = ouverture(ms, true);
        assert!(i.opacite >= avant, "{ms} ms");
        avant = i.opacite;
    }
    let fin = ouverture(OUVERTURE_FONDU_MS, true);
    assert_eq!(fin.opacite, 255);
    assert!(fin.finie);
    // Fermeture : un fondu court.
    assert_eq!(fermeture(0, true).opacite, 255);
    assert!(!fermeture(40, true).finie);
    assert!(fermeture(FERMETURE_FONDU_MS, true).finie);
}

/// « Effets d'animation » coupé dans Windows : aucune animation.
#[test]
fn sans_effets_aucune_animation() {
    let i = ouverture(0, false);
    assert_eq!(i.opacite, 255);
    assert!(i.finie);
    assert!(fermeture(0, false).finie);
}

/// Après un retard de l'écran (une image échangée 300 ms trop tard), l'animation ne saute
/// pas à la fin : elle n'avance que d'un pas, et se joue ensuite en entier.
#[test]
fn un_retard_ne_fait_pas_sauter_l_animation() {
    let retard = avancer(Duration::ZERO, Duration::from_millis(300));
    assert_eq!(retard, PAS_MAX);
    let ms = u32::try_from(retard.as_millis()).unwrap_or(u32::MAX);
    assert!(!ouverture(ms, true).finie);
    // Des images régulières à 60 Hz : finie en 5 ou 6 images, comme prévu.
    let mut anime = Duration::ZERO;
    let mut images = 0;
    while !ouverture(u32::try_from(anime.as_millis()).unwrap_or(u32::MAX), true).finie {
        anime = avancer(anime, Duration::from_micros(16_667));
        images += 1;
    }
    assert!((5..=6).contains(&images), "{images}");
}

/// Le choix de Windows inventé pour les essais de relecture : sombre ou clair, avec un accent.
fn windows(sombre: bool, accent: u32) -> ChoixWindows {
    ChoixWindows {
        sombre,
        barre_sombre: sombre,
        accent: Some(Rgb::hex(accent)),
        nuances: None,
    }
}

/// C-90 : Windows est relu toutes les 2 s, jamais avant ; l'horloge est inventée (le temps
/// écoulé est donné à la fonction).
#[test]
fn relecture_toutes_les_deux_secondes() {
    assert_eq!(RELECTURE, Duration::from_secs(2));
    assert!(!faut_relire(Duration::ZERO));
    assert!(!faut_relire(Duration::from_millis(1_999)));
    assert!(faut_relire(Duration::from_secs(2)));
    assert!(faut_relire(Duration::from_secs(60)));
}

/// C-90 : le logiciel « Fictif » suit un Windows clair ; Windows passe en sombre : la palette
/// change ; relu une seconde fois sans changement : rien ne change.
#[test]
fn le_theme_de_windows_est_suivi() {
    let clair = windows(false, 0x3865F3);
    let sombre = windows(true, 0x3865F3);
    assert_eq!(relu(&clair, clair.clone(), None), None);
    let Some((choix, palette)) = relu(&clair, sombre.clone(), None) else {
        panic!("Windows passé en sombre : la palette doit changer");
    };
    assert_eq!(choix, sombre);
    assert_eq!(palette, Palette::de(&sombre));
    assert!(palette.sombre);
    assert_eq!(relu(&choix, sombre.clone(), None), None);
    // L'accent seul change : la palette suit aussi.
    let autre = windows(true, 0xC0416B);
    let Some((_, palette)) = relu(&sombre, autre.clone(), None) else {
        panic!("l'accent a changé : la palette doit suivre");
    };
    assert_eq!(palette.accent, Palette::de(&autre).accent);
}

/// C-90 : un thème imposé (captures, essais) l'emporte sur Windows ; l'accent reste celui de
/// Windows.
#[test]
fn un_theme_impose_l_emporte() {
    let clair = windows(false, 0x7B57C9);
    let impose_sombre = impose(clair.clone(), Some(true));
    assert!(impose_sombre.sombre);
    assert_eq!(impose_sombre.accent, clair.accent);
    assert_eq!(impose(clair.clone(), None), clair);
    // Imposé sombre, les applications de Windows passent de sombre à clair (la barre des
    // tâches reste sombre) : rien ne change.
    let actuel = impose(windows(true, 0x7B57C9), Some(true));
    let applis_claires = ChoixWindows {
        sombre: false,
        ..windows(true, 0x7B57C9)
    };
    assert_eq!(relu(&actuel, applis_claires, Some(true)), None);
    // La barre des tâches change, elle : le choix suivi change (l'icône près de l'horloge).
    assert!(relu(&actuel, windows(false, 0x7B57C9), Some(true)).is_some());
    // Imposé clair, mais l'accent change : la palette suit l'accent, reste claire.
    let actuel = impose(windows(false, 0x7B57C9), Some(false));
    let Some((choix, palette)) = relu(&actuel, windows(true, 0x857A2A), Some(false)) else {
        panic!("l'accent a changé : la palette doit suivre");
    };
    assert!(!choix.sombre);
    assert!(!palette.sombre);
}

/// La lettre est blanche sur un fond foncé, noire sur un fond clair.
#[test]
fn lettre_selon_le_fond() {
    assert_eq!(couleur_lettre(Rgb(0x30, 0x20, 0x70)), BLANC);
    assert_eq!(couleur_lettre(Rgb(0xF0, 0xE0, 0x90)), NOIR);
    assert_eq!(couleur_lettre(NOIR), BLANC);
    assert_eq!(couleur_lettre(BLANC), NOIR);
}

/// La couverture d'une forme : un pixel entièrement dedans vaut 255, entièrement dehors 0,
/// à moitié dedans 127 (8 points sur 16).
#[test]
fn couverture_d_une_forme() {
    let gauche = |x: f32, _y: f32| x < 1.0;
    assert_eq!(couverture(0, 0, 1.0, gauche), 255);
    assert_eq!(couverture(1, 0, 1.0, gauche), 0);
    assert_eq!(couverture(0, 0, 2.0, |x: f32, _y: f32| x < 0.25), 127);
}
