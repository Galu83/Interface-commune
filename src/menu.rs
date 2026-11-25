//! Le petit menu de l'icône (clic droit) : toujours dans le même ordre, avec les mêmes mots et
//! les mêmes icônes dans tous les logiciels (règles : interface Windows 11, point 10).
//!
//! Le calcul vit dans le cœur (`ic_coeur::gestes::menu`, bloc D), avec `Action` ; ce module
//! le reprend à ses chemins.

pub use ic_coeur::gestes::menu::{
    ACTIONS_PROPRES_MAX, Action, Choix, LIGNES_EN_PLUS_MAX, Ligne, lignes, lignes_avec,
};
