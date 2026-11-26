//! Des captures du volet, du petit menu et des Paramètres **pour la documentation**, sans
//! ouvrir aucune fenêtre : egui met la page en forme, puis un petit rendu logiciel
//! (triangles, polices d'egui) dessine l'image, enregistrée en PNG. Rien ne passe par
//! OpenGL ni par l'écran : cela se lance partout, pendant les essais, et le résultat ne
//! dépend ni de la carte graphique ni de l'échelle de l'écran.
//!
//! Le logiciel donne ce qu'il montre : pour la documentation, un logiciel qui parle de
//! données inventées (règle n° 1 : jamais celles du propriétaire). Les polices Segoe de
//! Windows sont nécessaires (sans elles, [`Erreur`] : une capture sans texte ne servirait à
//! rien).
//!
//! ```no_run
//! # fn exemple(logiciel: &dyn interface_commune::Logiciel) -> Result<(), String> {
//! use interface_commune::capture::{self, Options};
//! // Tout : le volet, le petit menu et chaque page des Paramètres, en clair et en sombre.
//! let faites = capture::toutes(logiciel, std::path::Path::new("docs/captures"), 1.0)?;
//! // Ou une seule image :
//! let page = capture::parametres(logiciel, Some("general"), Options::default())?;
//! std::fs::write("general.png", page.png()?).map_err(|e| e.to_string())?;
//! # let _ = faites;
//! # Ok(())
//! # }
//! ```
//!
//! Limite : un rendu simplifié (une texture échantillonnée au plus proche, sans correction
//! de gamma) : les images sont fidèles à la mise en page et aux couleurs, pas au pixel près
//! à ce qu'affiche la carte graphique.

//!
//! Refonte, bloc I : écrit dans `src/dessin/capture.rs` (ce qu'on capture, la mise en page
//! par egui) et `src/dessin/capture/toile.rs` (le rendu logiciel), et repris ici à son chemin
//! d'aujourd'hui. Les captures réutilisent le dessin rangé aux blocs F, G et H : le volet et
//! le petit menu (`surface`), les Paramètres, les fenêtres toutes faites et tout dessin.

pub use crate::dessin::capture::*;
