//! Le squelette des Paramètres en calcul pur, sans fenêtre (tout est testé) : l'ordre des
//! pages et leurs clés, la page demandée sur la ligne de commande (`--parametres <page>`), la
//! recherche d'un paramètre, la bulle « Annuler » après un changement, et « Conserver ces
//! modifications ? » (retour automatique) pour ce qui peut couper l'écran, la souris, le
//! clavier ou la liaison (règles : interface Windows 11, point 7).
//!
//! Refonte, bloc E : tout vit dans le cœur (`ic_coeur::parametres`), avec ses essais sur une
//! fausse horloge ; repris ici aux mêmes chemins. Les types qui gardent un instant
//! (`Annulation`, `Confirmation`…) y sont génériques ; ici, ils gardent un `Instant`, comme
//! avant.

// Le contrat (bloc A), repris du cœur à ce chemin.
pub use ic_coeur::contrat::{Instantane, Page, Valeur};
pub use ic_coeur::parametres::{
    A_PROPOS, ACCUEIL, AConfirmer, Annulable, Annulation, Changement, Confirmation, DUREE_ANNULER,
    DUREE_CONSERVER, GENERAL, MISE_A_JOUR, PAGES_POUR_RECHERCHE, Retour, chercher, navigation, page_accueil,
    page_demandee, pages_communes, rang_de_page, section_trouvee, simplifier,
};
