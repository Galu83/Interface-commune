//! Les Paramètres à la façon de Windows 11 (règles : interface Windows 11, point 7), le même
//! squelette dans tous les logiciels, dessiné comme ceux de PowerPeek (le modèle) :
//!
//! - barre de titre « Paramètres de <logiciel> » ;
//! - en haut de la navigation : le vrai logo du logiciel (32 points), son nom (16 points,
//!   semibold) et juste dessous « Version x.y.z » (12 points) ;
//! - la navigation à gauche, dont la largeur suit ses textes (250 points au moins, comme
//!   PowerPeek) : lignes de 38 points, icônes, petite barre d'accent sur la page choisie ;
//!   l'Accueil facultatif, les pages propres, puis, séparées par 6 points, toujours Général,
//!   Mise à jour et À propos (juste après, jamais collées au bas de la fenêtre) ;
//! - « Rechercher un paramètre » dès sept pages ;
//! - une carte par réglage, effet immédiat, bulle « Annuler », bandeau de redémarrage ;
//! - « Conserver ces modifications ? » (15 s, retour automatique) pour un réglage qui peut
//!   couper l'écran, la souris, le clavier ou la liaison ([`Contexte::changer_a_confirmer`]) ;
//! - une sous-page s'ouvre par un chevron « › », avec le chemin « Page › Sous-page » ;
//! - la fenêtre retient sa taille et sa place (sous Windows, `fenetre`).
//!
//! Tous les mots viennent de [`crate::textes`] (français ou anglais) ou du logiciel. Le
//! dessin ne dépend pas de la fenêtre : [`Parametres::dessiner`] se teste sans rien ouvrir,
//! et un logiciel qui a sa propre boucle de fenêtres peut le dessiner dans la sienne.
//!
//! Refonte, bloc H : le dessin vit dans `crate::dessin::parametres` (un fichier par partie) ;
//! il est repris ici, aux mêmes chemins. Ce module garde la fenêtre (`fenetre`), le modèle
//! (`modele`) et la seule lecture de Windows du dessin : les « effets d'animation ».

#[cfg(windows)]
pub mod fenetre;
pub mod modele;

use crate::Reglages;
use crate::theme::Palette;

pub use crate::dessin::parametres::{Contexte, Parametres};

/// La clé de la fenêtre des Paramètres parmi les autres fenêtres de la boucle
/// ([`crate::autres_fenetres`]) : réservée.
pub const CLE_FENETRE: &str = "parametres";

// Les mesures, écrites une seule fois dans `crate::mesures`.
pub use crate::mesures::{
    ESPACE_COMMUNES, NAVIGATION_LARGEUR_MAX as LARGEUR_NAVIGATION_MAX,
    NAVIGATION_LARGEUR_MIN as LARGEUR_NAVIGATION_MIN, NAVIGATION_LIGNE as LIGNE_NAVIGATION,
    PARAMETRES_TAILLE as TAILLE, PARAMETRES_TAILLE_MIN as TAILLE_MIN, RAYON_ALERTE, TAILLE_LOGO, TAILLE_NOM,
};

/// La présentation de la page « Mise à jour » vient du cœur (bloc E) ; ces noms restent ici
/// pour les logiciels qui les appellent depuis `interface_commune::parametres`.
pub use ic_coeur::parametres::{
    TonMiseAJour, ligne_grise_mise_a_jour, presentation_mise_a_jour, texte_mise_a_jour,
};

impl Parametres {
    /// Toute la fenêtre : navigation et page. Lit les « effets d'animation » de Windows (la
    /// rotation des chevrons des sections repliables en dépend), puis dessine.
    pub fn dessiner(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        self.dessiner_avec(ui, p, l, effets_d_animation());
    }
}

/// « Effets d'animation » de Windows ; hors Windows, toujours oui.
fn effets_d_animation() -> bool {
    #[cfg(windows)]
    {
        crate::systeme::effets_d_animation()
    }
    #[cfg(not(windows))]
    {
        true
    }
}
