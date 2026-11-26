//! Les bords Windows (ORGANIGRAMME.md, §3.3 et §10) : le seul endroit où `unsafe` est permis,
//! chaque appel commenté. Les blocs J à O y entrent un à un (PLAN-REFONTE.md, §2) ; leurs types
//! publics restent repris à leurs chemins d'aujourd'hui.
//!
//! - `fenetres` : les boucles des fenêtres, le volet, les Paramètres, les autres fenêtres
//!   (bloc J) ;
//! - `icone` : les icônes près de l'horloge, `horloge::Horloge` et `zone::Zone` (bloc M) ;
//! - `poste` : le guichet des messages de Windows (`horloge::Poste`, bloc K) ;
//! - `reprise` : le dessin des fenêtres attend la fin d'un réveil de Windows ;
//! - `sonde` : la sonde des fils, le chien de garde et le journal (bloc O) ;
//! - `systeme` : fenêtres, écran, échelle, thème, polices, sélecteur de dossier (bloc L) ;
//! - `unique` : une seule copie du logiciel (bloc N) ;
//! - `verification` : chaque caractère affiché existe dans les polices de Windows (bloc L) ;
//! - `verrou` : le seul mécanisme de verrou, qui reprend un verrou empoisonné.

pub(crate) mod fenetres;
#[cfg(windows)]
pub(crate) mod icone;
#[cfg(windows)]
pub(crate) mod poste;
pub(crate) mod reprise;
pub(crate) mod sonde;
pub(crate) mod systeme;
pub(crate) mod unique;
pub(crate) mod verification;
pub(crate) mod verrou;
