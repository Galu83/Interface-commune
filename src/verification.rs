//! Le contrôle des polices (règles : interface Windows 11, point 4) : chaque icône et chaque
//! caractère affiché existent dans les polices de Windows. La bibliothèque s'en sert pour ses
//! propres textes ; chaque logiciel s'en sert, dans ses tests, pour les siens :
//!
//! ```no_run
//! use interface_commune::verification;
//! // Les icônes propres au logiciel, et les fichiers d'où vient son texte affiché.
//! let icones = ['\u{E7F4}', '\u{E8A9}'];
//! let sources = [("menu.rs", "… le texte du fichier …")];
//! assert_eq!(verification::icones_absentes(&icones), Ok(Vec::new()));
//! assert_eq!(verification::caracteres_absents(&sources), Ok(Vec::new()));
//! ```
//!
//! Hors de Windows (polices absentes), les contrôles rendent `Err` : le test le dit et saute.
//!
//! Écrit dans les bords (`src/bords/verification.rs`, bloc O) : il lit les polices du disque.
//! Ce module le reprend à son chemin d'avant.

pub use crate::bords::verification::{Absent, caracteres_absents, contient, icones_absentes, logos_absents};
