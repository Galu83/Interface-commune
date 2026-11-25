//! Les gestes et les secours, en calcul pur (bloc D) : ce qu'un geste sur l'icône ouvre, une
//! seule fois pour l'icône unique et pour les icônes d'une zone ; l'ouverture et la fermeture
//! du volet et du petit menu ; les fenêtres perdues et leur reprise ; l'info-bulle ; le petit
//! menu. L'instant arrive en paramètre ([`Horodatage`]) : le cœur ne lit jamais l'heure.
//!
//! - [`clics`] : le geste reconnu (double-clic, Entrée en double) et sa suite ;
//! - [`aiguilleur`] : le suivi par icône et l'ordre unique qui en découle ;
//! - [`ouverture`] : la décision d'un clic, l'anti-rebond, la garde, le clic en dehors ;
//! - [`secours`] : les pertes de fenêtres, la rafale, la reprise ;
//! - [`icone`] : l'info-bulle et le délai contre le clignotement ;
//! - [`menu`] : les lignes du petit menu et ses actions (`Action`, venue du contrat).

pub mod aiguilleur;
pub mod clics;
pub mod icone;
pub mod menu;
pub mod ouverture;
pub mod secours;
mod temps;

pub use temps::Horodatage;
