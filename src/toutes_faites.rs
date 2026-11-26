//! Les fenêtres toutes faites, ouvertes dans la boucle du logiciel ([`AutresFenetres`](crate::autres_fenetres::AutresFenetres)) :
//! - la fenêtre « simple » ([`Simple`]) : titre, texte, nouveautés, état, barre
//!   d'avancement, champ, choix, boutons (avec confirmation) ; sa hauteur suit son contenu,
//!   jusqu'à un plafond ([`HAUTEUR_MAX`]) au-delà duquel on défile ;
//! - la fenêtre de lecture ([`Lecture`]) : un texte lu à voix haute, le mot lu surligné et
//!   suivi ; un clic sur un mot le rend au logiciel (reprendre la lecture là).
//!
//! Reprises de Vocalis (`interface/src/simples.rs`), sans son canal : le logiciel décrit la
//! fenêtre par une structure, la change par sa [`Poignee`], et reçoit ce que l'utilisateur fait
//! par un rappel. Les fonctions de dessin ([`dessiner_simple`], [`dessiner_lecture`]) servent
//! aussi seules, dans une fenêtre du logiciel.
//!
//! Le rappel est appelé pendant le dessin de la fenêtre : il peut changer la fenêtre par sa
//! poignée ([`Poignee::changer`], [`Poignee::fermer`]) et ouvrir ou fermer d'autres
//! fenêtres, mais ne doit pas appeler `AutresFenetres::fermer` sur sa propre clé (la poignée
//! le fait sans attente).
//!
//! Refonte, bloc H : le dessin de ces fenêtres vit dans `crate::dessin::toutes_faites`
//! (`simple`, `prete`, `lecture`) et il est repris ici, aux mêmes chemins. Ce module garde ce
//! qui touche à la boucle : la poignée, l'ouverture, le rappel, la hauteur demandée à Windows.
//!
//! Refonte, bloc J : le code vit dans `crate::bords::fenetres::toutes_faites`, avec les bords
//! Windows des fenêtres ; il est repris ici, aux mêmes chemins.

pub use crate::bords::fenetres::toutes_faites::*;
pub use crate::dessin::toutes_faites::{
    Bouton, Champ, Choix, Clic, EtatLecture, EtatSimple, HAUTEUR_MAX, LARGEUR_SIMPLE, Lecture, PLUS_TARD,
    Simple, Ton, barre_avancement, dessiner_lecture, dessiner_simple, mots,
};
