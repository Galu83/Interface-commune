//! La colonne de navigation des Paramètres : les lignes montrées (toutes, ou les résultats de
//! la recherche), l'espace avant les trois pages communes, et la largeur de la colonne, qui
//! suit le nom du logiciel et les noms des pages (C-80). Les textes sont mesurés par le dessin
//! (bloc H) ; ici, seulement la décision.

use super::recherche::chercher;
use crate::contrat::Page;
use crate::theme::mesures::{
    NAVIGATION_DROITE, NAVIGATION_LARGEUR_MAX, NAVIGATION_LARGEUR_MIN, NAVIGATION_MARGES, NAVIGATION_TEXTE_X,
    TAILLE_LOGO,
};

/// Le champ « Rechercher un paramètre » est montré à partir de ce nombre de pages (règles :
/// plus de six).
pub const PAGES_POUR_RECHERCHE: usize = 7;

/// Le champ « Rechercher un paramètre » est-il montré ?
pub fn recherche_montree(pages: &[Page]) -> bool {
    pages.len() >= PAGES_POUR_RECHERCHE
}

/// Une ligne montrée dans la navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LigneNavigation {
    /// Le rang de la page dans la navigation.
    pub rang: usize,
    /// Un espace la sépare de la ligne d'avant ([`crate::theme::mesures::ESPACE_COMMUNES`]) :
    /// c'est la première des trois pages communes, hors recherche.
    pub espace_avant: bool,
}

/// Les lignes de la navigation : toutes les pages, ou, quand quelque chose est tapé dans
/// « Rechercher un paramètre » (montré dès [`PAGES_POUR_RECHERCHE`] pages), celles qui le
/// contiennent. Les trois pages communes sont séparées des pages propres, sauf pendant une
/// recherche.
pub fn lignes_de_navigation(pages: &[Page], recherche: &str) -> Vec<LigneNavigation> {
    let trouvees =
        (recherche_montree(pages) && !recherche.trim().is_empty()).then(|| chercher(pages, recherche));
    let communes = pages.len().saturating_sub(3);
    (0..pages.len())
        .filter(|i| trouvees.as_ref().is_none_or(|t| t.contains(i)))
        .map(|i| LigneNavigation {
            rang: i,
            espace_avant: i == communes && i > 0 && trouvees.is_none(),
        })
        .collect()
}

/// La largeur de la navigation (points), C-80 : ce qu'il faut à l'en-tête (logo, puis le nom
/// et « Version x.y.z » l'un sous l'autre) et au plus long nom de page pour tenir sur une ligne,
/// marges comprises, arrondi au point du dessus, entre [`NAVIGATION_LARGEUR_MIN`] et
/// [`NAVIGATION_LARGEUR_MAX`]. `nom`, `version` et `titres` : les largeurs mesurées des textes ;
/// `espacement` : l'écart entre le logo et le nom.
pub fn largeur_navigation(
    nom: f32,
    version: f32,
    espacement: f32,
    titres: impl IntoIterator<Item = f32>,
) -> f32 {
    let entete = 8.0 + TAILLE_LOGO + 4.0 + espacement + nom.max(version);
    let lignes = titres
        .into_iter()
        .map(|t| NAVIGATION_TEXTE_X + t + NAVIGATION_DROITE)
        .fold(0.0, f32::max);
    let marges = f32::from(NAVIGATION_MARGES[0]) + f32::from(NAVIGATION_MARGES[1]);
    (entete.max(lignes) + marges)
        .ceil()
        .clamp(NAVIGATION_LARGEUR_MIN, NAVIGATION_LARGEUR_MAX)
}
