//! Règle de l'interface (point 4) : chaque icône et chaque caractère affiché par la
//! bibliothèque existent dans les polices de Windows. Le texte est pris dans tous ses
//! fichiers : chaque caractère est vérifié, commentaires compris.

use interface_commune::{icones, verification};

const SOURCES: [(&str, &str); 13] = [
    ("surface.rs", include_str!("../src/surface.rs")),
    ("menu.rs", include_str!("../src/menu.rs")),
    ("parametres/mod.rs", include_str!("../src/parametres/mod.rs")),
    (
        "parametres/modele.rs",
        include_str!("../src/parametres/modele.rs"),
    ),
    (
        "parametres/fenetre.rs",
        include_str!("../src/parametres/fenetre.rs"),
    ),
    ("dessin.rs", include_str!("../src/dessin.rs")),
    ("fenetres.rs", include_str!("../src/fenetres.rs")),
    ("horloge.rs", include_str!("../src/horloge.rs")),
    ("textes.rs", include_str!("../src/textes.rs")),
    ("autres_fenetres.rs", include_str!("../src/autres_fenetres.rs")),
    ("toutes_faites.rs", include_str!("../src/toutes_faites.rs")),
    ("briques.rs", include_str!("../src/briques.rs")),
    // Les caractères écrits par leur code : espace insécable, points de suspension.
    ("ajoutés", "\u{a0}…›"),
];

#[test]
fn chaque_icone_existe() {
    match verification::icones_absentes(&icones::TOUTES) {
        Ok(absentes) => assert!(absentes.is_empty(), "{absentes:?}"),
        Err(e) => eprintln!("{e} : test sauté."),
    }
}

#[test]
fn chaque_caractere_existe() {
    match verification::caracteres_absents(&SOURCES) {
        Ok(absents) => assert!(absents.is_empty(), "{absents:?}"),
        Err(e) => eprintln!("{e} : test sauté."),
    }
}
