//! Les polices de Windows (règles : interface Windows 11, points 3 et 4) : Segoe UI Variable
//! (Windows 11), sinon Segoe UI et Segoe UI Semibold (Windows 10) ; Segoe Fluent Icons, sinon
//! Segoe MDL2 Assets. Lues sur le PC, jamais livrées avec un logiciel.
//!
//! Elles sont lues une seule fois, au repos (au démarrage) : jamais au moment d'un clic.
//!
//! Écrit dans `src/bords/systeme/polices.rs` (bloc L de la refonte ; le choix des fichiers
//! dans le cœur, `ic_coeur::theme::polices`), repris ici à son chemin d'aujourd'hui.

pub use crate::bords::systeme::polices::{
    ICONES_FLUENT, ICONES_MDL2, SEGOE, SEGOE_SEMIBOLD, SEGOE_VARIABLE, dossier, icone_disponible, icones,
    lire, polices, polices_ou_vides, semibold, texte, titre,
};
