//! Le **dessin** (bloc F de la refonte, `docs/ORGANIGRAMME.md` §3.2) : tout ce qui passe par
//! egui seul, sans jamais appeler Windows. Les éléments dessinés à la façon de Windows 11 :
//! ligne de menu, interrupteur, carte de réglage, boutons, titres. Zones cliquables d'au moins
//! 32 points, contour visible sur l'élément qui a le focus du clavier (règles : interface
//! Windows 11, points 5 et 8).
//!
//! Les contrôles se lisent ici (`dessin::bouton`, `dessin::carte`…). Les autres parties du
//! dessin gardent leur chemin d'aujourd'hui, à la racine de la bibliothèque : [`crate::briques`],
//! [`crate::pastilles`], [`crate::menu_choix`], [`crate::seuils`], [`crate::nuancier`],
//! [`crate::couleurs`], [`crate::touches`], [`crate::surface`] (le volet et le petit menu, bloc G),
//! `apparence::couleur`, et les traits
//! [`crate::Reglages`] et [`crate::Logiciel`]. Ce qui décide (les touches du menu de choix, le
//! déplacement des seuils, la marque d'une pastille, le plan des cases) vit dans le cœur
//! (`ic-coeur`).

pub(crate) mod apparence;
pub(crate) mod briques;
mod controles;
pub(crate) mod couleur;
pub(crate) mod couleurs;
pub(crate) mod menu_choix;
pub(crate) mod nuancier;
pub(crate) mod pastilles;
pub(crate) mod seuils;
pub(crate) mod surface;
pub(crate) mod touches;
pub(crate) mod traits;
pub(crate) mod volet;

pub use controles::*;
// Les espaces insécables de la typographie française (« Coupé : … ») : du texte pur, dans le
// cœur depuis le bloc B de la refonte, repris ici à son chemin d'aujourd'hui.
pub use ic_coeur::textes::insecables;
