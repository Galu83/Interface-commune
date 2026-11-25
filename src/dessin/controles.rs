//! Les contrôles communs à la façon de Windows 11 : lignes, boutons, interrupteurs, champ de
//! raccourci, titres, cartes, section repliable, bandeau, état. Repris un à un par
//! [`crate::dessin`].

mod base;
mod cartes;
mod interrupteur;
mod raccourci;

pub use base::{
    Genre, bouton, bouton_icone, focus, icone, largeur_ligne, ligne, ligne_de, ligne_menu, ligne_menu_grasse,
};
pub use cartes::{
    ETAT_ICONE, bandeau, cadre_carte, carte, carte_chevron, carte_info, carte_large, entete_section, etat,
    etat_long, titre_page, titre_section,
};
pub use interrupteur::{
    RACCOURCI_PARAMETRES, interrupteur, interrupteur_ecrit, largeur_interrupteur_ecrit, roue_parametres,
};
pub use raccourci::{Saisie, champ_raccourci, raccourci_de};
