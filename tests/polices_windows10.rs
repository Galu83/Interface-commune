//! C-92, le repli de Windows 10 : sans Segoe UI Variable ni Segoe Fluent Icons, la
//! bibliothèque prend Segoe UI, Segoe UI Semibold et Segoe MDL2 Assets, et dessine quand même.
//!
//! Un essai seul dans son fichier : les polices sont lues une fois par processus, et l'essai
//! change `WINDIR` pour montrer à la bibliothèque un dossier de polices « Windows 10 » (une
//! copie, dans un dossier temporaire, de trois polices du PC).

#![cfg(windows)]
#![allow(clippy::expect_used, unsafe_code)]

#[allow(dead_code)]
#[path = "references/logiciel.rs"]
mod logiciel;

use std::path::PathBuf;

use interface_commune::capture::{self, Options};
use interface_commune::polices;

#[test]
fn c92_repli_des_polices_de_windows_10() {
    let Some(vrai) = polices::dossier() else {
        eprintln!("Pas de dossier des polices : essai sauté.");
        return;
    };
    let noms = [polices::SEGOE, polices::SEGOE_SEMIBOLD, polices::ICONES_MDL2];
    let Some(octets) = noms
        .iter()
        .map(|n| std::fs::read(vrai.join(n)).ok())
        .collect::<Option<Vec<_>>>()
    else {
        eprintln!("Polices de Windows 10 absentes : essai sauté.");
        return;
    };

    // Un faux Windows 10 : son dossier Fonts n'a que les trois polices d'avant.
    let racine: PathBuf = std::env::temp_dir().join(format!("ic-polices-w10-{}", std::process::id()));
    let fonts = racine.join("Fonts");
    std::fs::create_dir_all(&fonts).expect("dossier temporaire");
    for (n, o) in noms.iter().zip(&octets) {
        std::fs::write(fonts.join(n), o).expect("copie d'une police");
    }
    // SAFETY: le seul essai de ce processus ; aucun autre fil ne lit l'environnement.
    unsafe {
        std::env::set_var("WINDIR", &racine);
        std::env::set_var("SystemRoot", &racine);
    }
    assert_eq!(polices::dossier(), Some(fonts.clone()));

    let d = polices::polices().expect("le repli de Windows 10 doit suffire");
    let donnees = |cle: &str| -> Vec<u8> {
        let f = d.font_data.get(cle).expect("police posée");
        AsRef::<[u8]>::as_ref(&*f.font).to_vec()
    };
    assert_eq!(donnees("texte"), octets[0], "texte : Segoe UI");
    assert_eq!(donnees("semibold"), octets[1], "semibold : Segoe UI Semibold");
    assert_eq!(donnees("titre"), octets[1], "titre : Segoe UI Semibold");
    assert_eq!(donnees("icones"), octets[2], "icônes : Segoe MDL2 Assets");
    // L'engrenage des Paramètres existe dans Segoe MDL2 Assets.
    assert!(polices::icone_disponible('\u{E713}'));

    // Lues une seule fois : le dossier effacé, la bibliothèque garde ce qu'elle a lu…
    std::fs::remove_dir_all(&racine).expect("nettoyage");
    assert!(polices::polices().is_ok());
    // … et dessine avec le repli.
    let image = capture::volet(&logiciel::Fictif::new(), Options::default()).expect("volet");
    assert!(image.largeur > 0 && image.hauteur > 0);
}
