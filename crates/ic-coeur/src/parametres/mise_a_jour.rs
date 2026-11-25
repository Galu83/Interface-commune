//! La présentation de la page « Mise à jour », à la façon de Windows Update : de l'état
//! ([`EtatMiseAJour`]) viennent une seule icône (coche = à jour, flèche = mise à jour
//! disponible, triangle = problème), sa couleur, le titre, la ligne grise dessous, le bouton de
//! la carte, et ce que montre la carte du dossier des installateurs. Le dessin (bloc H) ne fait
//! que poser ces décisions.

use crate::contrat::{Communs, EtatMiseAJour};
use crate::textes::{Textes, remplir};
use crate::theme::{Palette, Rgb, icones};

/// La couleur d'une icône d'état de la mise à jour, avant de choisir dans la palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonMiseAJour {
    Neutre,
    Bon,
    Occupe,
    Accent,
    Probleme,
}

/// Ce que dit la page « Mise à jour » d'un état : l'icône, sa couleur et le titre.
pub fn presentation_mise_a_jour(e: &EtatMiseAJour, t: &Textes) -> (char, TonMiseAJour, String) {
    use TonMiseAJour::*;
    match e {
        EtatMiseAJour::Inconnu => (icones::MISE_A_JOUR, Neutre, t.maj_pas_cherche.into()),
        EtatMiseAJour::AJour => (icones::COCHE, Bon, t.maj_a_jour.into()),
        EtatMiseAJour::Recherche => (icones::MISE_A_JOUR, Occupe, t.maj_recherche.into()),
        EtatMiseAJour::Prete(v) => (icones::INSTALLER, Accent, remplir(t.maj_prete, &[v])),
        EtatMiseAJour::Erreur(_) => (icones::ATTENTION, Probleme, t.maj_erreur.into()),
        EtatMiseAJour::Coupee => (icones::A_PROPOS, Neutre, t.maj_coupee.into()),
        EtatMiseAJour::Information(e) => (icones::A_PROPOS, Neutre, e.clone()),
    }
}

/// La ligne grise sous le titre : la raison d'une erreur, la version installée, puis la
/// dernière recherche, séparées par « · ».
pub fn ligne_grise_mise_a_jour(
    e: &EtatMiseAJour,
    version: &str,
    derniere: Option<&str>,
    t: &Textes,
) -> String {
    let mut morceaux: Vec<String> = Vec::new();
    if let EtatMiseAJour::Erreur(raison) = e {
        morceaux.push(raison.clone());
    }
    morceaux.push(remplir(t.version_actuelle, &[version]));
    if let Some(d) = derniere {
        morceaux.push(remplir(t.derniere_recherche, &[d]));
    }
    morceaux.join(" · ")
}

/// Le texte de l'état de la mise à jour, et sa couleur dans la palette.
pub fn texte_mise_a_jour(e: &EtatMiseAJour, p: &Palette, t: &Textes) -> (char, Rgb, String) {
    let (glyphe, ton, texte) = presentation_mise_a_jour(e, t);
    let c = match ton {
        TonMiseAJour::Neutre => p.texte_secondaire,
        TonMiseAJour::Bon => p.vert,
        TonMiseAJour::Occupe => p.orange,
        TonMiseAJour::Accent => p.accent,
        TonMiseAJour::Probleme => p.rouge,
    };
    (glyphe, c, texte)
}

/// L'explication de la carte « Rechercher les mises à jour » : rien quand elles sont actives,
/// car la carte « Dossier des installateurs » suit et dit déjà où (pas de redite) ; « Coupé… »
/// quand elles sont coupées (la carte du dossier n'est alors pas affichée).
pub fn explication_recherches(t: &Textes, actif: bool) -> &'static str {
    if actif { "" } else { t.recherches_coupees }
}

/// Le bouton de la carte d'état, à droite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoutonMiseAJour {
    /// « Installer maintenant » : une version est prête (action
    /// [`crate::contrat::cles::INSTALLER`]).
    Installer,
    /// « Rechercher des mises à jour » (`Reglages::rechercher_mise_a_jour`).
    Rechercher,
}

/// Le bouton de la carte d'état : « Installer maintenant » quand une version est prête, sinon
/// « Rechercher », sauf si les recherches sont coupées (alors aucun).
pub fn bouton_mise_a_jour(c: &Communs) -> Option<BoutonMiseAJour> {
    match c.mise_a_jour {
        EtatMiseAJour::Prete(_) => Some(BoutonMiseAJour::Installer),
        _ if c.recherches != Some(false) => Some(BoutonMiseAJour::Rechercher),
        _ => None,
    }
}

/// Ce que montre la carte « Dossier des installateurs ».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarteDossier {
    /// « Choisir… » (action [`crate::contrat::cles::DOSSIER`]) : le dossier n'est pas fixe.
    pub choisir: bool,
    /// « Automatique » (action [`crate::contrat::cles::DOSSIER_AUTO`]) : le dossier peut être
    /// trouvé tout seul et ne l'est pas encore.
    pub automatique: bool,
}

/// La carte du dossier des installateurs ; `None` quand les recherches sont coupées (elle ne
/// servirait à rien).
pub fn carte_dossier(c: &Communs) -> Option<CarteDossier> {
    (c.recherches != Some(false)).then_some(CarteDossier {
        choisir: !c.dossier_fixe,
        automatique: !c.dossier_fixe && c.dossier_automatique == Some(false),
    })
}
