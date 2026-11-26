//! Règle de l'interface (point 4) : chaque icône et chaque caractère affiché par la
//! bibliothèque existent dans les polices de Windows. Le texte est pris dans tous ses
//! fichiers : chaque caractère est vérifié, commentaires compris.

use interface_commune::{icones, verification};

const SOURCES: [(&str, &str); 35] = [
    ("dessin/surface.rs", include_str!("../src/dessin/surface.rs")),
    ("dessin/volet.rs", include_str!("../src/dessin/volet.rs")),
    (
        "dessin/volet/entete.rs",
        include_str!("../src/dessin/volet/entete.rs"),
    ),
    (
        "dessin/volet/clavier.rs",
        include_str!("../src/dessin/volet/clavier.rs"),
    ),
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
    (
        "dessin/controles/base.rs",
        include_str!("../src/dessin/controles/base.rs"),
    ),
    (
        "dessin/controles/cartes.rs",
        include_str!("../src/dessin/controles/cartes.rs"),
    ),
    (
        "dessin/controles/interrupteur.rs",
        include_str!("../src/dessin/controles/interrupteur.rs"),
    ),
    (
        "dessin/controles/raccourci.rs",
        include_str!("../src/dessin/controles/raccourci.rs"),
    ),
    ("fenetres.rs", include_str!("../src/fenetres.rs")),
    ("horloge.rs", include_str!("../src/horloge.rs")),
    ("textes.rs", include_str!("../src/textes.rs")),
    ("autres_fenetres.rs", include_str!("../src/autres_fenetres.rs")),
    ("toutes_faites.rs", include_str!("../src/toutes_faites.rs")),
    ("dessin/briques.rs", include_str!("../src/dessin/briques.rs")),
    (
        "dessin/briques/appareil.rs",
        include_str!("../src/dessin/briques/appareil.rs"),
    ),
    (
        "dessin/menu_choix.rs",
        include_str!("../src/dessin/menu_choix.rs"),
    ),
    (
        "dessin/parametres.rs",
        include_str!("../src/dessin/parametres.rs"),
    ),
    (
        "dessin/parametres/a_propos.rs",
        include_str!("../src/dessin/parametres/a_propos.rs"),
    ),
    (
        "dessin/parametres/bulles.rs",
        include_str!("../src/dessin/parametres/bulles.rs"),
    ),
    (
        "dessin/parametres/contexte.rs",
        include_str!("../src/dessin/parametres/contexte.rs"),
    ),
    (
        "dessin/parametres/general.rs",
        include_str!("../src/dessin/parametres/general.rs"),
    ),
    (
        "dessin/parametres/gestes.rs",
        include_str!("../src/dessin/parametres/gestes.rs"),
    ),
    (
        "dessin/parametres/mise_a_jour.rs",
        include_str!("../src/dessin/parametres/mise_a_jour.rs"),
    ),
    (
        "dessin/parametres/navigation.rs",
        include_str!("../src/dessin/parametres/navigation.rs"),
    ),
    (
        "dessin/toutes_faites.rs",
        include_str!("../src/dessin/toutes_faites.rs"),
    ),
    (
        "dessin/toutes_faites/essais.rs",
        include_str!("../src/dessin/toutes_faites/essais.rs"),
    ),
    (
        "dessin/toutes_faites/lecture.rs",
        include_str!("../src/dessin/toutes_faites/lecture.rs"),
    ),
    (
        "dessin/toutes_faites/prete.rs",
        include_str!("../src/dessin/toutes_faites/prete.rs"),
    ),
    (
        "dessin/toutes_faites/simple.rs",
        include_str!("../src/dessin/toutes_faites/simple.rs"),
    ),
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
