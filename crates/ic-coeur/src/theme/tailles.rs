//! Le texte et les arrondis (règles : interface Windows 11, points 3, 5 et 8).

/// Titre de page, en points.
pub const TAILLE_TITRE: f32 = 28.0;
/// Le gros titre du volet (le nom du logiciel).
pub const TAILLE_TITRE_VOLET: f32 = 20.0;
/// Titre de section (semibold).
pub const TAILLE_SECTION: f32 = 15.0;
/// Le texte courant.
pub const TAILLE_TEXTE: f32 = 14.0;
/// Le texte secondaire (légendes, précisions grises) : jamais plus petit.
pub const TAILLE_PETIT: f32 = 12.0;
/// L'explication et la valeur d'une carte : 14 px comme le texte courant, en gris (audit
/// KVMSoft, C2 : le 12 px est réservé aux légendes et précisions).
pub const TAILLE_EXPLICATION: f32 = 14.0;
/// La hauteur d'une ligne cliquable (menu, volet, navigation) : au moins 32 points.
pub const HAUTEUR_LIGNE: f32 = 36.0;
/// L'arrondi des cartes, et celui des contrôles.
pub const ARRONDI_CARTE: u8 = 8;
pub const ARRONDI_CONTROLE: u8 = 4;
