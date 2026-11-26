//! La fenêtre des Paramètres (sous Windows) : « Paramètres de `<logiciel>` », coins arrondis et
//! barre de titre sombre par Windows, aucune animation à nous (Windows anime déjà les
//! fenêtres), taille et place retenues d'une ouverture à l'autre et toujours en entier sur un
//! écran, taille minimale. La place est retenue à la fermeture, et aussi 1 s après chaque
//! déplacement ([`crate::place::SuiviDePlace`]) : un arrêt brutal ne la perd pas. Celle d'une
//! fenêtre réduite ou agrandie ne l'est jamais (sa place d'avant reste retenue).
//!
//! Trois façons de l'ouvrir :
//! - avec l'icône de la bibliothèque (`horloge`) : dans sa boucle, à côté du volet
//!   ([`ouvrir_dans`]) ;
//! - dans la boucle d'un logiciel qui a la sienne : ses [`AutresFenetres`](crate::autres_fenetres::AutresFenetres) dessinées par sa
//!   fenêtre principale ([`ouvrir_dans_boucle`]), ou [`super::Parametres::dessiner`] dans sa
//!   propre fenêtre ;
//! - seuls, comme fenêtre principale d'une boucle ([`ouvrir_seuls`]) : un logiciel sans autre
//!   fenêtre, ou le secours quand la fenêtre du volet n'a pas pu être préparée.
//!
//! Refonte, bloc J : le code vit dans `crate::bords::fenetres::parametres`, avec les bords
//! Windows des fenêtres ; il est repris ici, aux mêmes chemins.

pub use crate::bords::fenetres::parametres::*;
