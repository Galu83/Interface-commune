//! Les autres fenêtres du logiciel, ouvertes et fermées **dans la boucle de fenêtres de la
//! bibliothèque**.
//!
//! winit 0.30 n'accepte qu'une boucle de fenêtres par programme : une deuxième, créée dans un
//! autre fil, échoue (« EventLoop can't be recreated »). La bibliothèque garde donc une seule
//! boucle, toute la vie du logiciel, dont la fenêtre principale est celle du volet (cachée au
//! repos) ; chaque autre fenêtre (les Paramètres, une fenêtre simple, une fenêtre de lecture…)
//! y est une fenêtre de plus d'egui (un « viewport » immédiat), dessinée pendant l'image de la
//! fenêtre principale.
//!
//! Pour un logiciel, depuis n'importe quel fil :
//!
//! ```no_run
//! # fn exemple(fenetres: interface_commune::autres_fenetres::AutresFenetres) {
//! use interface_commune::autres_fenetres::{Cadre, Contenu, Ouverture};
//! struct Bonjour;
//! impl Contenu for Bonjour {
//!     fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
//!         egui::CentralPanel::default().show(ui, |ui| {
//!             if ui.button("Fermer").clicked() {
//!                 cadre.fermer();
//!             }
//!         });
//!     }
//! }
//! fenetres.ouvrir("bonjour", Ouverture::new("Bonjour", [320.0, 160.0]), Bonjour);
//! // … plus tard, depuis n'importe où :
//! fenetres.fermer("bonjour");
//! # }
//! ```
//!
//! Chaque fenêtre s'ouvre **cachée**, se dessine deux fois, puis se montre : jamais une
//! fenêtre vide ni un éclair blanc. Elle suit le thème de Windows (barre de titre sombre,
//! coins arrondis). Elle se ferme par sa croix, Alt+F4, [`Cadre::fermer`] ou
//! [`AutresFenetres::fermer`] ; [`Contenu::fermee`] reçoit alors sa place, à retenir.
//!
//! Refonte, bloc J : le code vit dans `crate::bords::fenetres::autres`, avec les bords
//! Windows des fenêtres ; il est repris ici, aux mêmes chemins.

pub use crate::bords::fenetres::autres::*;
