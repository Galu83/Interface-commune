//! L'interface commune des logiciels du même propriétaire, à la façon de Windows 11
//! (règles générales : `interface-windows11.md`, points 1 à 10) :
//!
//! - le thème de Windows suivi en direct (clair ou sombre, accent), la palette Fluent, les
//!   polices Segoe UI Variable et Segoe Fluent Icons, les durées d'animation communes
//!   ([`theme`], [`apparence`], [`polices`]) ;
//! - l'icône près de l'horloge : clic gauche, le volet ; clic droit, le petit menu ;
//!   double-clic, les Paramètres ; clavier ([`clics`], et `horloge` sous Windows) ;
//!   plusieurs icônes à la fois pour un logiciel qui garde sa boucle de messages (`zone`
//!   sous Windows) ;
//! - le volet et le petit menu, dans une seule fenêtre préparée au repos et animée
//!   ([`fenetres`], [`surface`], [`menu`], et `volet` sous Windows) ;
//! - le squelette des Paramètres ([`parametres`]) : en-tête, navigation, pages Général,
//!   Mise à jour et À propos, cartes, interrupteurs, effet immédiat avec « Annuler »,
//!   bandeau de redémarrage, taille retenue, `--parametres <page>`.
//!
//! Rien n'y est propre à un logiciel : le nom, la version, le logo, les pages propres, les
//! actions du petit menu, les réglages rapides du volet viennent du logiciel ; les mots
//! communs, en français ou en anglais, de [`textes`].
//!
//! Deux traits, pour prendre ce dont on a besoin :
//! - [`Reglages`] : ce qu'il faut aux **Paramètres seuls** (un logiciel qui garde son icône,
//!   son menu et sa boucle de fenêtres peut ne prendre qu'eux : [`parametres::Parametres`]
//!   dans sa fenêtre, ou [`autres_fenetres`] et `parametres::fenetre::ouvrir_seuls`) ;
//! - [`Logiciel`] (qui comprend [`Reglages`]) : en plus, l'icône près de l'horloge, le volet
//!   et le petit menu (`horloge::Horloge`).

pub mod apparence;
pub mod autres_fenetres;
pub mod briques {
    //! Les briques du volet : le logiciel décrit ses réglages rapides, la bibliothèque les
    //! dessine (écrites dans `src/dessin/briques.rs`, bloc F).
    pub use crate::dessin::briques::*;
}
pub mod capture;
pub mod clics;
pub mod couleurs {
    //! Les couleurs dans les Paramètres, aux couleurs du volet : pastille de carte, choix de
    //! couleur, carte de couleur (écrites dans `src/dessin/couleurs.rs`, bloc F).
    pub use crate::dessin::couleurs::*;
}
pub mod dessin;
pub mod fenetres;
pub mod icones;
pub mod menu;
pub mod menu_choix {
    //! Le menu flottant de choix, au style des menus de Windows 11 (écrit dans
    //! `src/dessin/menu_choix.rs`, bloc F ; ses touches et sa place se décident dans le cœur).
    pub use crate::dessin::menu_choix::*;
}
pub mod mesures;
pub mod nuancier {
    //! Le nuancier : choisir une couleur d'un clic dans une grille de pastilles (écrit dans
    //! `src/dessin/nuancier.rs`, bloc F).
    pub use crate::dessin::nuancier::*;
}
pub mod parametres;
pub mod pastilles {
    //! Les pastilles d'appareils et le choix illustré du volet (écrits dans
    //! `src/dessin/pastilles.rs`, bloc F ; la marque et le plan des cases dans le cœur).
    pub use crate::dessin::pastilles::*;
}
pub mod place;
pub mod polices;
mod reprise;
pub mod seuils {
    //! La barre à seuils : des zones colorées dont on fait glisser les limites (écrite dans
    //! `src/dessin/seuils.rs`, bloc F ; le déplacement d'une limite dans le cœur).
    pub use crate::dessin::seuils::*;
}
pub mod sonde;
pub mod surface;
pub mod textes;
pub mod theme;
pub mod touches {
    //! Les raccourcis clavier en toutes lettres, comme Windows en français (« Ctrl+Maj+K ») :
    //! [`lisible`] vient du cœur ; la traduction des touches d'egui est dans
    //! `src/dessin/touches.rs` (bloc F).
    pub use crate::dessin::touches::*;
}
pub mod toutes_faites;
pub mod verification;

// Les bords Windows de la refonte (`src/bords/`) : les blocs y entrent un à un, repris à
// leurs chemins d'aujourd'hui (le guichet `Poste` : `horloge::Poste`).
#[cfg(windows)]
#[allow(unsafe_code)]
mod bords;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod echelle;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod horloge;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod systeme;
#[cfg(windows)]
pub mod volet;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod zone;

pub use parametres::modele::{Instantane, Page, Valeur};
pub use textes::{Langue, Textes};
pub use theme::{Palette, Rgb};

// Les deux traits du logiciel (bloc F) : écrits avec le dessin, repris ici à leur chemin.
pub use dessin::traits::{Logiciel, Reglages};

// Le contrat (bloc A) : les types que le logiciel fournit ou reçoit et qui ne dessinent pas
// vivent dans le cœur pur (`ic-coeur`) et sont repris ici, à leur chemin d'aujourd'hui.
pub use ic_coeur::contrat::{
    Communs, Credit, Etat, EtatMiseAJour, FaireLogo, FenetrePrincipale, GenreEtat, Identite, cles,
};
