//! Les **boucles des fenêtres** (bloc J de la refonte, `docs/ORGANIGRAMME.md` §3.3 J) : la
//! seule boucle de fenêtres du programme et tout ce qui s'y ouvre. La décision d'un clic, la
//! garde et la reprise après une perte se calculent dans le cœur
//! (`ic_coeur::gestes::ouverture` et `ic_coeur::gestes::secours`, bloc D) ; ici, ce qui fait
//! tourner les fenêtres.
//!
//! - [`liens`] : les demandes de l'icône, l'aiguillage d'un clic, ce que partagent les fils ;
//! - [`canal`] : le canal des demandes et le réveil du fil des fenêtres ;
//! - [`boucle`] : la boucle de la bibliothèque (`boucle_des_fenetres`, `Ouvreur`), la fenêtre
//!   perdue recréée, le dernier filet du fil ;
//! - [`supervision`] : la façon 2, une boucle tenue par le logiciel (`superviser_les_fenetres`,
//!   `Supervise`) ;
//! - [`autres`] : les autres fenêtres dans la même boucle (`AutresFenetres`), une panique par
//!   fenêtre attrapée (C-62) ;
//! - `volet` : la fenêtre du volet et du petit menu (sous Windows) ;
//! - `parametres` : la fenêtre des Paramètres, dans l'une ou l'autre boucle (sous Windows) ;
//! - [`toutes_faites`] : la poignée et l'ouverture des fenêtres toutes faites ;
//! - `faux_moteur` (essais) : le faux moteur qui tient la place d'eframe et de Windows.
//!
//! Les chemins publics d'avant restent : `fenetres`, `autres_fenetres`, `volet`,
//! `parametres::fenetre` et `toutes_faites` reprennent ces modules.

use std::time::Instant;

pub(crate) mod autres;
pub(crate) mod boucle;
pub(crate) mod canal;
pub(crate) mod liens;
#[cfg(windows)]
pub(crate) mod parametres;
pub(crate) mod supervision;
pub(crate) mod toutes_faites;
#[cfg(windows)]
pub(crate) mod volet;

#[cfg(test)]
mod essais;
#[cfg(test)]
mod faux_moteur;

/// Les pertes récentes des fenêtres, à l'heure de Windows ([`ic_coeur::gestes::secours::Pertes`]).
pub type Pertes = ic_coeur::gestes::secours::Pertes<Instant>;
