//! Les bords Windows (ORGANIGRAMME.md, §3.3 et §10) : le seul endroit où `unsafe` est permis,
//! chaque appel commenté. Les blocs J à O y entrent un à un (PLAN-REFONTE.md, §2) ; leurs types
//! publics restent repris à leurs chemins d'aujourd'hui.
//!
//! - `poste` : le guichet des messages de Windows (`horloge::Poste`, bloc K) ;
//! - `systeme` : fenêtres, écran, échelle, thème, polices, sélecteur de dossier (bloc L) ;
//! - `verification` : chaque caractère affiché existe dans les polices de Windows (bloc L).

#[cfg(windows)]
pub(crate) mod poste;
pub(crate) mod sonde;
pub(crate) mod systeme;
pub(crate) mod unique;
pub(crate) mod verification;
