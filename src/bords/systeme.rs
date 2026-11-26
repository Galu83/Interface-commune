//! Le **système Windows** (bloc L de la refonte, `docs/ORGANIGRAMME.md` §3.3) : ce qui parle à
//! Windows pour les fenêtres, l'écran, le thème, les polices et le sélecteur de dossier.
//! Chaque `unsafe` est commenté. Ce qui se décide sans Windows vit dans le cœur : le sens des
//! valeurs du registre (`ic_coeur::theme::choix_du_registre`), le choix des polices
//! (`ic_coeur::theme::polices`), les bornes du double-clic (`ic_coeur::gestes::clics`), la
//! place au changement d'échelle (`ic_coeur::placement`).
//!
//! Les chemins publics d'aujourd'hui restent : `systeme`, `echelle`, `apparence`, `polices`
//! et `verification` reprennent ces modules.
//!
//! - [`fenetre`] : DWM, place, premier plan, styles, opacité, réveil du volet, recherche d'une
//!   fenêtre d'egui, « pas de fantôme » ;
//! - [`ecran`] : l'écran, la souris, « Effets d'animation », le double-clic (C-112) ;
//! - [`echelle`] : le réglage d'échelle du processus (C-113) et le correctif du changement
//!   d'échelle de winit ;
//! - [`cachee`] : la seule fabrique de fenêtre cachée (icône unique, zone d'icônes, essais) ;
//! - [`reveil`] : le réveil du fil des fenêtres ;
//! - [`dossier`] : le sélecteur de dossier, jamais sur un chemin lent (C-111) ;
//! - [`registre`] : le thème lu dans le registre ;
//! - [`apparence`] : le thème suivi en direct ; [`polices`] : les polices de Windows (C-92).

pub(crate) mod apparence;
pub(crate) mod polices;

#[cfg(windows)]
pub(crate) mod cachee;
#[cfg(windows)]
pub(crate) mod dossier;
#[cfg(windows)]
pub(crate) mod echelle;
#[cfg(windows)]
pub(crate) mod ecran;
#[cfg(windows)]
pub(crate) mod fenetre;
#[cfg(windows)]
pub(crate) mod registre;
#[cfg(windows)]
pub(crate) mod reveil;

#[cfg(all(test, windows))]
mod essais;
