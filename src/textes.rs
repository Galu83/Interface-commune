//! Tous les mots que la bibliothèque écrit elle-même (petit menu, volet, Paramètres), en
//! français et en anglais. Aucun texte en dur : un logiciel choisit une langue
//! ([`Reglages::textes`](crate::Reglages::textes)), ou donne les siens en partant d'une des
//! deux (`Textes { quitter: "…", ..textes::FRANCAIS }`).
//!
//! Les modèles qui contiennent `{}` reçoivent leurs valeurs dans l'ordre ([`remplir`]).
//!
//! Refonte, bloc B : le catalogue vit dans le cœur (`ic_coeur::textes`), repris ici à ses
//! chemins d'aujourd'hui ; ses essais sont dans `crates/ic-coeur/src/textes/essais.rs`.

pub use ic_coeur::textes::{ANGLAIS, FRANCAIS, Langue, Textes, remplir};
