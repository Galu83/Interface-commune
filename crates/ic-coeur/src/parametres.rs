//! Bloc E, le modèle des Paramètres (`docs/ORGANIGRAMME.md`, §3.1) : tout ce que les
//! Paramètres décident, sans rien dessiner. L'instant arrive en paramètre
//! ([`crate::gestes::Horodatage`]) : le cœur ne lit jamais l'heure.
//!
//! - [`pages`] : la liste des pages (Accueil facultatif, pages du logiciel, puis Général, Mise
//!   à jour, À propos), la page demandée (`--parametres <page>`, une notification), les pages
//!   arrivées après l'ouverture ([`Suivi`], [`page_suivie`]) ;
//! - [`recherche`] : « Rechercher un paramètre », accents et majuscules sans importance, les
//!   sections repliables trouvées ;
//! - [`colonne`] : la colonne de navigation, ses lignes montrées (recherche dès 7 pages, espace avant les pages
//!   communes), la largeur qui suit les noms des pages (C-80) ;
//! - [`annuler`] : la bulle « Annuler » (C-70 : 6 s, deux changements de suite gardent l'« avant »
//!   du premier, un brouillon revient entier) ;
//! - [`conserver`] : « Conserver ces modifications ? » (15 s, une seule à la fois) ;
//! - [`envois`] : ce que les gestes rendent au logiciel (`changer`, `remettre`), dans l'ordre,
//!   et le bandeau « Redémarrer maintenant » (C-78) ;
//! - [`mise_a_jour`] : la présentation de la page « Mise à jour » (état → icône, couleur, titre,
//!   ligne grise, bouton, carte du dossier).
//!
//! Ce qui n'est pas ici : le dessin (bloc H), lire l'heure, enregistrer (le logiciel).
//! `interface-commune` reprend ces éléments à leurs chemins d'aujourd'hui
//! (`parametres::modele`, `parametres::presentation_mise_a_jour`…).

pub mod annuler;
pub mod colonne;
pub mod conserver;
pub mod envois;
#[cfg(test)]
mod essais;
#[cfg(test)]
mod essais_gestes;
#[cfg(test)]
mod essais_mise_a_jour;
pub mod mise_a_jour;
pub mod pages;
pub mod recherche;

pub use annuler::{Annulable, Annulation, Changement, DUREE_ANNULER};
pub use colonne::{
    LigneNavigation, PAGES_POUR_RECHERCHE, largeur_navigation, lignes_de_navigation, recherche_montree,
};
pub use conserver::{AConfirmer, Confirmation, DUREE_CONSERVER, Retour};
pub use envois::{Envoi, noter_redemarrage};
pub use mise_a_jour::{
    BoutonMiseAJour, CarteDossier, TonMiseAJour, bouton_mise_a_jour, carte_dossier, explication_recherches,
    ligne_grise_mise_a_jour, presentation_mise_a_jour, texte_mise_a_jour,
};
pub use pages::{
    A_PROPOS, ACCUEIL, GENERAL, MISE_A_JOUR, PageSuivie, Suivi, navigation, page_accueil, page_demandee,
    page_suivie, pages_communes, rang_de_cle, rang_de_page,
};
pub use recherche::{chercher, section_trouvee, simplifier};
