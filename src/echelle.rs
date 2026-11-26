//! Une fenêtre glissée d'un écran à l'autre quand les deux n'ont pas la même échelle (125 %,
//! 150 %…) : elle suit la souris, sans sauter ni revenir sur l'écran qu'elle quitte.
//!
//! winit 0.30 répond au message `WM_DPICHANGED` de Windows en posant la fenêtre à un endroit
//! qu'il calcule lui-même, et qu'il « repousse » vers l'écran que la fenêtre quitte : Windows
//! rechange alors l'échelle, et la fenêtre fait des allers-retours (le détail :
//! [`crate::place::place_au_changement_d_echelle`]). La bibliothèque glisse donc sa propre
//! procédure devant celle de winit (une « sous-classe » de Windows) : pendant que winit traite
//! ce message, le déplacement qu'il demande est remplacé par le rectangle que Windows a
//! suggéré. winit garde tout le reste (la nouvelle échelle, qu'egui reçoit comme d'habitude).
//! C'est le correctif de winit (#4341), qui n'est pas encore dans sa version 0.30.
//!
//! Écrit dans `src/bords/systeme/echelle.rs` (bloc L de la refonte), repris ici à son chemin
//! d'aujourd'hui.

pub use crate::bords::systeme::echelle::suivre_windows;
