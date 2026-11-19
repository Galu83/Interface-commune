//! Les mesures communes du volet, du petit menu et des Paramètres (points), écrites une seule
//! fois : tous les logiciels ont le même dessin (règles : interface Windows 11, points 6 et
//! 7). Le petit menu prend celles des menus de Windows 11 ; les Paramètres, celles de
//! PowerPeek ; le volet, celles du volet réseau de Windows 11.

// ---- Le petit menu (menus de Windows 11) ---------------------------------------------

/// Hauteur d'une ligne du petit menu.
pub const MENU_LIGNE: f32 = 32.0;
/// Écart entre deux lignes du petit menu.
pub const MENU_ECART: f32 = 2.0;
/// Marge autour des lignes du petit menu.
pub const MENU_MARGE: f32 = 4.0;
/// Largeur du petit menu, au moins.
pub const MENU_LARGEUR: f32 = 240.0;

// ---- Les lignes (petit menu et briques du volet) --------------------------------------

/// Centre de l'icône d'une ligne, depuis son bord gauche ; taille de l'icône.
pub const LIGNE_ICONE_X: f32 = 20.0;
pub const LIGNE_ICONE: f32 = 16.0;
/// Où commence le texte d'une ligne, sa marge à droite, l'écart minimal entre le texte et
/// l'aide de droite (un raccourci, un pourcentage).
pub const LIGNE_TEXTE_X: f32 = 42.0;
pub const LIGNE_MARGE_DROITE: f32 = 12.0;
pub const LIGNE_ECART: f32 = 16.0;
/// Arrondi du fond survolé d'une ligne.
pub const LIGNE_ARRONDI: u8 = 4;
/// Hauteur d'un séparateur (un trait d'un point au milieu).
pub const SEPARATEUR: f32 = 9.0;

// ---- Le volet (volet réseau de Windows 11) --------------------------------------------

/// Largeur du volet, au moins ; volet et petit menu, au plus.
pub const VOLET_LARGEUR: f32 = 320.0;
pub const LARGEUR_MAX: f32 = 400.0;
/// Marge autour du contenu du volet.
pub const VOLET_MARGE: f32 = 6.0;
/// Marge à gauche et à droite du bloc de l'état, en haut du volet.
pub const VOLET_MARGE_ETAT: f32 = 14.0;
/// Hauteur d'une brique du volet (interrupteur, choix, appareil, action), et du choix
/// segmenté.
pub const VOLET_LIGNE: f32 = 36.0;
pub const VOLET_SEGMENTS: f32 = 32.0;
/// Écart entre deux briques.
pub const VOLET_ECART: f32 = 2.0;
/// Hauteur d'un titre de section, et d'une ligne de la liste des raccourcis.
pub const VOLET_SECTION: f32 = 28.0;
pub const VOLET_RACCOURCI: f32 = 24.0;
/// Hauteur de la barre d'un appareil, et de sa sous-ligne.
pub const VOLET_BARRE: f32 = 4.0;
pub const VOLET_SOUS_LIGNE: f32 = 18.0;
/// La tuile de l'image d'un appareil (son icône dessinée par le logiciel), et la hauteur du
/// petit bouton de sa sous-ligne (« Oublier »).
pub const VOLET_TUILE: f32 = 44.0;
pub const VOLET_BOUTON: f32 = 22.0;
/// Une pastille d'appareil (rangée « Ordinateurs ») : la taille de son carré coloré.
pub const PASTILLE_LARGEUR: f32 = 26.0;
pub const PASTILLE_HAUTEUR: f32 = 20.0;
/// Un choix illustré : sa hauteur (deux lignes), la taille d'une case d'écran de son
/// illustration, l'écart entre deux cases, le nombre de cases au plus, et la largeur fixe de
/// l'illustration (le texte commence au même endroit d'une ligne à l'autre).
pub const ILLUSTRE_LIGNE: f32 = 52.0;
pub const ILLUSTRE_CASE_L: f32 = 24.0;
pub const ILLUSTRE_CASE_H: f32 = 18.0;
pub const ILLUSTRE_CASE_ECART: f32 = 3.0;
pub const ILLUSTRE_CASES_MAX: usize = 4;
pub const ILLUSTRE_LARGEUR: f32 =
    ILLUSTRE_CASES_MAX as f32 * ILLUSTRE_CASE_L + (ILLUSTRE_CASES_MAX - 1) as f32 * ILLUSTRE_CASE_ECART;
/// La barre d'accent à gauche d'un choix illustré actif : largeur et hauteur.
pub const ILLUSTRE_BARRE: [f32; 2] = [3.0, 18.0];

/// La roue dentée des Paramètres, en haut à droite du volet, sur la ligne du titre : un bouton
/// carré ; la marge à droite de cette ligne ; l'écart minimal entre le titre (ou la version)
/// et la roue.
pub const VOLET_ROUE: f32 = 36.0;
pub const VOLET_ENTETE_DROITE: f32 = 6.0;
pub const VOLET_ENTETE_ECART: f32 = 8.0;

// ---- Les Paramètres (PowerPeek) -------------------------------------------------------

/// Taille de la fenêtre au premier lancement, et taille minimale.
pub const PARAMETRES_TAILLE: [f32; 2] = [920.0, 680.0];
pub const PARAMETRES_TAILLE_MIN: [f32; 2] = [700.0, 500.0];
/// Largeur de la navigation : au moins, au plus.
pub const NAVIGATION_LARGEUR_MIN: f32 = 250.0;
pub const NAVIGATION_LARGEUR_MAX: f32 = 340.0;
/// Les marges de la navigation (gauche, droite, haut, bas).
pub const NAVIGATION_MARGES: [i8; 4] = [10, 6, 18, 12];
/// Une ligne de la navigation : hauteur, début du texte, place gardée à droite.
pub const NAVIGATION_LIGNE: f32 = 38.0;
pub const NAVIGATION_TEXTE_X: f32 = 46.0;
pub const NAVIGATION_DROITE: f32 = 32.0;
/// Rayon du point d'alerte (à 16 points du bord droit).
pub const RAYON_ALERTE: f32 = 4.0;
/// Le logo et le nom en haut de la navigation.
pub const TAILLE_LOGO: f32 = 32.0;
pub const TAILLE_NOM: f32 = 16.0;
/// L'espace entre les pages du logiciel et les trois pages communes.
pub const ESPACE_COMMUNES: f32 = 6.0;
