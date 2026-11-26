//! Le dessin des fenêtres toutes faites (bloc H de la refonte), sans fenêtre : la fenêtre
//! « simple » (`simple`), la fenêtre « <Logiciel> x.y.z est prête » faite avec elle
//! (`prete`), et la fenêtre de lecture (`lecture`). Ce module n'ouvre rien : les fenêtres de
//! la boucle, leur poignée et leur rappel restent dans [`crate::toutes_faites`], qui reprend
//! tout ceci aux mêmes chemins. Il ne lit pas Windows.

mod lecture;
mod prete;
mod simple;

#[cfg(test)]
pub(crate) mod essais;

pub use lecture::{EtatLecture, Lecture, dessiner_lecture, mots};
pub use prete::PLUS_TARD;
pub use simple::{
    Bouton, Champ, Choix, Clic, EtatSimple, HAUTEUR_MAX, LARGEUR_SIMPLE, Simple, Ton, barre_avancement,
    dessiner_simple,
};
