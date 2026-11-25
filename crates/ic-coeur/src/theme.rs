//! Bloc C, le thème et les mesures (`docs/ORGANIGRAMME.md`, §3.1) : des valeurs, et une seule
//! décision, celle de suivre Windows.
//!
//! - la palette Fluent claire et sombre, les couleurs d'état, l'accent ([`Palette`]) ;
//! - les 10 couleurs des pastilles ([`couleurs_pastilles`]) et la lettre lisible sur chacune
//!   ([`couleur_lettre`]) ;
//! - la taille du texte et les arrondis ([`TAILLE_TEXTE`], [`ARRONDI_CARTE`]…) ;
//! - les mesures du volet, du petit menu et des Paramètres ([`mesures`]) ;
//! - les glyphes des icônes communes ([`icones`]) ; ce qu'un carré coloré montre, logo ou
//!   lettre ([`marque`]), et le texte lisible sur une zone ([`texte_sur`]) ;
//! - l'animation d'ouverture et de fermeture, et sa courbe ([`ouverture`], [`fermeture`],
//!   [`avancer`]) ;
//! - **suivre Windows** : relire le clair ou sombre et l'accent toutes les 2 s, et changer la
//!   palette dès qu'ils changent, sauf thème imposé (C-90, [`faut_relire`], [`relu`]).
//!
//! Ce qui n'est pas ici : lire Windows (bloc L), charger les polices (bloc L), donner ces
//! couleurs à egui (bloc F, `interface_commune::apparence`). L'instant n'est jamais lu ici : le
//! temps écoulé arrive en paramètre.
//!
//! `interface-commune` reprend ces éléments à leurs chemins d'aujourd'hui (`theme`, `mesures`,
//! `icones`, `apparence::RELECTURE`, `apparence::couleurs_pastilles`, `couleurs::palette`,
//! `pastilles::couleur_lettre`).

mod animation;
mod couleurs;
#[cfg(test)]
mod essais;
#[cfg(test)]
mod essais_pastilles;
mod formes;
pub mod icones;
mod marque;
pub mod mesures;
mod relecture;
mod tailles;

pub use animation::{
    FERMETURE_FONDU_MS, Image, OUVERTURE_FONDU_MS, OUVERTURE_OPACITE_DE_DEPART, PAS_MAX, Surface, avancer,
    fermeture, ouverture, ralenti_a_l_arrivee,
};
pub use couleurs::{ChoixWindows, Palette, Rgb, contraste, couleur_lettre, couleurs_pastilles};
pub use formes::couverture;
pub use marque::{Marque, marque, texte_sur};
pub use relecture::{RELECTURE, faut_relire, impose, relu};
pub use tailles::{
    ARRONDI_CARTE, ARRONDI_CONTROLE, HAUTEUR_LIGNE, TAILLE_EXPLICATION, TAILLE_PETIT, TAILLE_SECTION,
    TAILLE_TEXTE, TAILLE_TITRE, TAILLE_TITRE_VOLET,
};
