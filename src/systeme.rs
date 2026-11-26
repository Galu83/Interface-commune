//! Ce qui parle à Windows pour les fenêtres : le thème et l'accent, les coins arrondis et la
//! barre de titre sombre (DWM), la place, l'opacité (fondu), « Effets d'animation », le
//! réveil du fil des fenêtres, le sélecteur de dossier, `DisableProcessWindowsGhosting`.
//!
//! Écrit dans `src/bords/systeme/` (bloc L de la refonte), repris ici à son chemin
//! d'aujourd'hui.

pub(crate) use crate::bords::systeme::fenetre::hwnd;

pub use crate::bords::systeme::dossier::{choisir_dossier, disque_local, dossier_de_depart};
pub use crate::bords::systeme::ecran::{
    bouton_enfonce, centree, ecran_autour, effets_d_animation, position_souris, temps_double_clic,
    zone_de_travail_pres_de,
};
pub use crate::bords::systeme::fenetre::{
    cacher, deplacer, fenetre_du_fil, fenetre_existe, habiller, hwnd_de, montrer_sans_activer, poser,
    premier_plan, rect_fenetre, reveiller, sans_cadre, sans_fantome, sans_focus, taille_fenetre,
    transparente, voiler,
};
pub use crate::bords::systeme::registre::choix_windows;
pub use crate::bords::systeme::reveil::Reveil;
