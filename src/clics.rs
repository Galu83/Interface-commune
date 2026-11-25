//! Les gestes sur l'icône près de l'horloge (règles : interface Windows 11, point 10) : clic
//! gauche, le volet ; clic droit, touche menu ou Maj+F10, le petit menu ; double-clic, ouvrir
//! le logiciel ; Entrée ou Espace, le volet ; chaque action part au relâchement du bouton.
//!
//! Le calcul vit dans le cœur (`ic_coeur::gestes::clics`, bloc D) et y est testé sans
//! fenêtre ; ce module le reprend à ses chemins, avec l'instant de la bibliothèque
//! (`std::time::Instant`).

pub use ic_coeur::gestes::clics::{Evenement, Geste, Suite, evenement, suite};

/// Le comportement de l'icône ([`ic_coeur::gestes::clics::Clics`]), compté en `Instant`.
pub type Clics = ic_coeur::gestes::clics::Clics<std::time::Instant>;
