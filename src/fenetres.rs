//! Le fil des fenêtres, sans Windows (tout est testé) : les demandes que l'icône lui envoie,
//! ce que partagent les fils, et sa boucle.
//!
//! Un fil à part tient **la seule boucle de fenêtres du programme** (winit n'en accepte
//! qu'une). Sa fenêtre principale est celle du volet et du petit menu, préparée une fois, au
//! repos, et gardée cachée jusqu'à la fin : un clic la met à jour et la montre, sans rien
//! reconstruire (ni fenêtre, ni OpenGL, ni polices ; règles : interface Windows 11, point 10,
//! « la première ouverture aussi rapide que les suivantes »). Les Paramètres et les autres
//! fenêtres du logiciel s'ouvrent et se ferment dans cette même boucle
//! ([`crate::autres_fenetres`]). Sans elle (polices ou OpenGL absents), le menu gris de
//! Windows sert de secours.
//!
//! **Une fenêtre perdue est recréée.** Une panique pendant la boucle des fenêtres (par
//! exemple eframe qui ne peut plus rendre son contexte OpenGL courant après une mise en
//! veille des écrans) détruit la fenêtre du volet. Sans garde, elle tuait le fil des
//! fenêtres en silence : plus rien ne s'ouvrait au clic. Elle est maintenant attrapée, notée
//! au journal avec sa cause, et la fenêtre est recréée aussitôt ; le clic en cours est refait
//! après la recréation. Des pertes en rafale (plus de [`RECREATIONS_EN_RAFALE`] en
//! [`RAFALE`]) donnent le menu de secours pour ce clic, et la fenêtre n'est recréée qu'au clic
//! suivant (jamais de boucle folle). Si le fil s'arrête quand même, [`GardeDuFil`] fait
//! passer l'icône au menu de secours.
//!
//! Un logiciel qui tient sa propre boucle eframe (au lieu de [`boucle_des_fenetres`]) suit
//! les mêmes règles avec [`superviser_les_fenetres`] : il dit comment lancer sa boucle et quoi
//! faire de ses fenêtres perdues ([`Supervise`]), la supervision compte les pertes
//! ([`Pertes`]) et décide quand les recréer ([`Reprise`]).
//!
//! Refonte, bloc J : le code vit dans `crate::bords::fenetres`, avec les bords
//! Windows des fenêtres ; il est repris ici, aux mêmes chemins.

// La décision d'un clic, la garde, les pertes et leur reprise : calcul pur, dans le cœur
// (bloc D), repris ici à leurs chemins d'avant.
pub use ic_coeur::gestes::ouverture::{
    ANTI_REBOND, Appuis, Clic, GARDE, GARDE_MAX, Garde, LENTE, Mesure, Ouvert, decider_le_clic, est_dehors,
    fermer_sur_appui,
};
pub use ic_coeur::gestes::secours::{Evenement, Fin, Perte, RAFALE, RECREATIONS_EN_RAFALE, Reprise};

pub use crate::bords::fenetres::Pertes;
pub use crate::bords::fenetres::boucle::*;
pub use crate::bords::fenetres::canal::*;
pub use crate::bords::fenetres::liens::*;
pub use crate::bords::fenetres::supervision::*;
