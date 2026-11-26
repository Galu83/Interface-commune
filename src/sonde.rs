//! La sonde des fils : dit au journal quand un fil du logiciel reste pris (l'icône qui ne
//! réagit plus, le volet qui ne s'ouvre plus) et où il est pris.
//!
//! Trois choses, toutes écrites au journal du logiciel par [`crate::Logiciel::noter`] :
//! - **les clics** de l'icône (gauche, droit, double), avec l'heure ([`clic`]) ;
//! - **les boucles imbriquées** : un appel qui fait tourner sa propre boucle de messages
//!   (menu de Windows, sélecteur de dossier…) est entouré de [`imbriquee`] : une ligne au
//!   début (le nom), une à la fin (la durée) ;
//! - **le chien de garde** : un seul fil de surveillance pour tous les fils inscrits
//!   ([`inscrire`]). Chaque fil a un compteur de battements ([`battement`]) qui avance à chaque
//!   tour de SA boucle. Toutes les 5 s, la surveillance lit les compteurs ; un compteur immobile
//!   depuis 15 s donne une ligne (une seule par blocage), puis une ligne quand le fil repart,
//!   avec la durée totale. La ligne dit la **dernière étape** du fil ([`etape`]) et, s'il est
//!   dans une boucle imbriquée, laquelle et depuis combien de temps.
//!
//! Les lignes ne contiennent jamais rien de l'utilisateur (frappes, textes, chemins) : seulement
//! des noms écrits dans le code (`&'static str`).
//!
//! Écrite en deux étages (bloc O) : la décision « bloqué », la veille du PC et le texte des
//! lignes dans le cœur (`ic_coeur::gestes::sonde`, testé à fausse horloge) ; les compteurs, le
//! journal et le fil du chien de garde dans les bords (`src/bords/sonde/`). Ce module les
//! reprend à leur chemin d'avant.
//!
//! # Utilisation par un projet
//!
//! La bibliothèque inscrit ses propres fils (l'icône, les fenêtres) et branche le journal
//! d'elle-même. Un projet peut seulement changer les délais, **avant** `Horloge::lancer` :
//!
//! ```no_run
//! use std::time::Duration;
//! interface_commune::sonde::regler(interface_commune::sonde::Delais {
//!     pas: Duration::from_secs(2),
//!     blocage: Duration::from_secs(10),
//! });
//! ```
//!
//! Un fil du projet qui a sa propre boucle s'inscrit pareil : `let _fil = sonde::inscrire(
//! "mon fil", None);` puis `sonde::battement()` à chaque tour, `sonde::etape("…")` avant un
//! appel à risque et `let _g = sonde::imbriquee("…");` autour d'un appel à boucle propre.

pub(crate) use crate::bords::sonde::noter;
pub use crate::bords::sonde::{
    Creation, Delais, Fil, Imbriquee, Inscription, SEUIL_DE_CREATION, Surveillance, battement,
    brancher_le_journal, clic, creation, est_une_veille, etape, etape_du_fil, imbriquee, inscrire,
    lieu_de_la_derniere_panique, regler, retenir_les_paniques,
};
