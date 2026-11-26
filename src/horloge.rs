//! L'icône près de l'horloge et le fil principal (règles : interface Windows 11, point 10) :
//! une fenêtre cachée reçoit les gestes sur l'icône, les notifications cliquées, les
//! changements de thème et le redémarrage de l'Explorateur ; elle envoie au fil des fenêtres
//! le volet, le petit menu ou les Paramètres. Une seule instance par session : relancer le
//! logiciel l'ouvre (sa fenêtre principale s'il en a une, sinon ses Paramètres).
//!
//! Utilisation, sur le fil principal du logiciel :
//!
//! ```no_run
//! # fn exemple(logiciel: std::sync::Arc<dyn interface_commune::Logiciel>) -> Result<(), String> {
//! use interface_commune::horloge::{Horloge, instance_unique};
//! let _verrou = match instance_unique("MonLogiciel") {
//!     Ok(v) => v,
//!     Err(_) => return Ok(()), // déjà lancé : il s'ouvre
//! };
//! let horloge = Horloge::new("MonLogiciel");
//! let poste = horloge.poste(); // à garder : notifications, info-bulle, quitter
//! horloge.lancer(logiciel, None) // rend la main quand le logiciel se ferme
//! # }
//! ```
//!
//! Le code est dans `src/bords/icone/` (bloc M) : l'icône seule est une zone à une icône
//! (`bords/icone/zone.rs`, le même moteur que `zone::Zone`) ; ses messages passent par
//! `bords/icone/horloge.rs`. Ce module en reprend les chemins d'aujourd'hui, sans `unsafe`.

pub use crate::bords::icone::decodage::DELAI_ICONE;
pub use crate::bords::icone::fil::{ATTENTE_FERMETURE, Secours, lancer_le_fil_des_fenetres};
pub use crate::bords::icone::horloge::{Horloge, MessageWindows, Recepteur, RecepteurArguments};
/// Avant toute fenêtre : chaque écran à sa propre échelle (sinon Windows étire tout). Écrit
/// dans `src/bords/systeme/echelle.rs` (bloc L), repris ici à son chemin d'aujourd'hui.
pub use crate::bords::systeme::echelle::reglage_dpi;
// Le contrat (bloc A), repris du cœur à ce chemin.
pub use ic_coeur::contrat::{Niveau, Reveil};
// Le guichet (bloc K), repris des bords à ce chemin.
pub use crate::bords::poste::{AuClic, Poste};
// L'instance unique (bloc N), reprise des bords à ce chemin.
pub use crate::bords::unique::{
    ARGUMENTS_MAX, ARGUMENTS_OCTETS_MAX, Unique, instance_unique, instance_unique_avec,
};
