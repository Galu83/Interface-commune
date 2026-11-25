//! Le seul fichier de couleurs et de durées (règles : interface Windows 11, points 2 et 10).
//! Les valeurs sont celles de Fluent (Windows 11), en clair et en sombre ; un seul accent :
//! celui de Windows. Vert, orange et rouge disent un état, les mêmes partout ; le rouge sert
//! aussi aux actions qui détruisent. Un logiciel n'écrit aucune couleur ailleurs : il prend
//! celles de [`Palette`].
//!
//! Tout est écrit dans le cœur (`ic_coeur::theme`, bloc C) et repris ici, aux mêmes chemins.

pub use ic_coeur::theme::{
    ARRONDI_CARTE, ARRONDI_CONTROLE, ChoixWindows, FERMETURE_FONDU_MS, HAUTEUR_LIGNE, Image,
    OUVERTURE_FONDU_MS, OUVERTURE_OPACITE_DE_DEPART, Palette, Rgb, Surface, TAILLE_EXPLICATION, TAILLE_PETIT,
    TAILLE_SECTION, TAILLE_TEXTE, TAILLE_TITRE, TAILLE_TITRE_VOLET, contraste, couverture, fermeture,
    ouverture, ralenti_a_l_arrivee,
};
