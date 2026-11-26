//! Les icônes près de l'horloge (bloc M) : un seul moteur pour `horloge::Horloge` (l'icône
//! seule, la boucle de la bibliothèque) et `zone::Zone` (plusieurs icônes, la boucle du
//! logiciel) ; une seule fenêtre cachée, un seul menu de secours.
//!
//! - `zone` : la fenêtre cachée, les icônes (`Shell_NotifyIcon`), leur remise quand
//!   l'Explorateur redémarre, les notifications, le menu gris de Windows ;
//! - `horloge` : l'icône seule sur cette zone (commandes du logiciel, menu de secours,
//!   instance unique, veille, boucle des messages) ;
//! - `decodage` : le calcul pur (messages, gestes, fin de session, GUID, lignes du menu) ;
//! - `image` : les images d'icône de Windows ;
//! - `communes` : l'aiguillage des gestes vers le fil des fenêtres, `FenetresCommunes` ;
//! - `fil` : le fil des fenêtres commun et son secours.
//!
//! Chaque `unsafe` est commenté (`// SAFETY:`).

pub(crate) mod communes;
pub(crate) mod decodage;
pub(crate) mod fil;
pub(crate) mod horloge;
pub(crate) mod image;
pub(crate) mod zone;

#[cfg(test)]
mod essais_horloge;
