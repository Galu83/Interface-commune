//! Le thème suivi en direct (règles : interface Windows 11, point 1) : le mode clair ou
//! sombre et l'accent de Windows, relus toutes les 2 secondes (quelques valeurs du registre,
//! rien de lent), appliqués à egui dès qu'ils changent. Quand relire, et ce qui change,
//! se décide dans le cœur (`ic_coeur::theme`, bloc C).
//!
//! Écrit dans `src/bords/systeme/apparence.rs` (lire Windows) et `src/dessin/apparence.rs`
//! (les couleurs données à egui), bloc L de la refonte ; repris ici à son chemin d'aujourd'hui.

pub use crate::bords::systeme::apparence::{
    Apparence, RELECTURE, choix_windows, couleur, couleurs_pastilles,
};
