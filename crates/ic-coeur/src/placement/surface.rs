//! La mise en page du volet et du petit menu, en calcul pur (bloc G de la refonte) : la
//! largeur de la surface d'après ses textes mesurés, la hauteur permise, la place de la roue
//! et du titre dans l'en-tête, la place de la version à droite du titre, la place laissée aux
//! briques qui défilent. Le dessin (`interface_commune::surface`) mesure les textes avec ses
//! polices, appelle ces fonctions et dessine ce qu'elles rendent, sans rien décider.
//!
//! Les calculs sont faits dans le même ordre qu'avant leur sortie du dessin : le rendu reste
//! identique au point près (images de référence).

use crate::placement::plan::Rectangle;
use crate::theme::mesures::{
    LARGEUR_MAX, MENU_LARGEUR, MENU_MARGE, VOLET_ENTETE_DROITE, VOLET_ENTETE_ECART, VOLET_LARGEUR,
    VOLET_MARGE, VOLET_MARGE_ETAT, VOLET_ROUE,
};
use crate::theme::{Surface, TAILLE_TEXTE};

/// L'écart entre le gros titre et sa version (points).
pub const ECART_VERSION: f32 = 6.0;
/// La marge au-dessus de l'en-tête du volet (points).
pub const ENTETE_HAUT: f32 = 4.0;
/// La marge sous l'état du volet (points).
pub const ETAT_BAS: f32 = 6.0;
/// L'espace entre l'état et la première brique (points).
pub const AVANT_BRIQUES: f32 = 2.0;
/// Le retrait du trait de séparation à chaque bout (points).
pub const RETRAIT_SEPARATEUR: f32 = 4.0;
/// L'opacité de la poignée de défilement au repos : elle se voit sans survol, sinon rien
/// n'indique que la liste des briques continue.
pub const OPACITE_POIGNEE: f32 = 0.55;

/// La marge autour du contenu de la surface (points).
pub fn marge(s: Surface) -> f32 {
    match s {
        Surface::Volet => VOLET_MARGE,
        Surface::Menu => MENU_MARGE,
    }
}

/// La largeur minimale de la surface (points).
pub fn largeur_min(s: Surface) -> f32 {
    match s {
        Surface::Volet => VOLET_LARGEUR,
        Surface::Menu => MENU_LARGEUR,
    }
}

/// La largeur de la surface (points) : `besoin` (ce que demandent ses textes, sans les
/// marges), plus ses marges, arrondie au point au-dessus, entre la largeur minimale et
/// [`LARGEUR_MAX`]. Au-delà, les phrases passent à la ligne (C-25).
pub fn largeur(s: Surface, besoin: f32) -> f32 {
    (besoin + 2.0 * marge(s))
        .ceil()
        .clamp(largeur_min(s), LARGEUR_MAX)
}

/// La version à écrire à droite du gros titre : aucune si elle est vide.
pub fn version_montree(version: &str) -> Option<&str> {
    (!version.is_empty()).then_some(version)
}

/// Les largeurs mesurées des textes du volet (points), pour [`besoin_du_volet`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MesuresDuVolet {
    /// Le gros titre, sur une ligne.
    pub titre: f32,
    /// La version, si elle est montrée ([`version_montree`]).
    pub version: Option<f32>,
    /// L'icône de l'état.
    pub icone_etat: f32,
    /// L'écart entre l'icône de l'état et sa ligne.
    pub ecart_etat: f32,
    /// La ligne d'état, sur une ligne.
    pub ligne_etat: f32,
    /// La plus large des briques.
    pub briques: f32,
    /// La plus large des briques du pied.
    pub pied: f32,
}

/// Ce que demande le volet (points, sans ses marges) : l'en-tête (titre, version, écart, roue,
/// marge de droite), la ligne d'état avec son icône, les briques et le pied ; la plus grande
/// de ces largeurs.
pub fn besoin_du_volet(m: &MesuresDuVolet) -> f32 {
    let mut titre_v = m.titre;
    if let Some(v) = m.version {
        titre_v += ECART_VERSION + v;
    }
    let mut b = VOLET_MARGE_ETAT + titre_v + VOLET_ENTETE_ECART + VOLET_ROUE + VOLET_ENTETE_DROITE;
    b = b.max(2.0 * VOLET_MARGE_ETAT + m.icone_etat + m.ecart_etat + m.ligne_etat);
    b = b.max(m.briques);
    b.max(m.pied)
}

/// Ce que demande le petit menu (points, sans ses marges) : sa ligne la plus large.
pub fn besoin_du_menu(lignes: impl IntoIterator<Item = f32>) -> f32 {
    lignes.into_iter().fold(0.0, f32::max)
}

/// La hauteur permise à la surface (points, marges comprises) : seul le volet en a une, et
/// seulement si elle est finie ; jamais moins d'un point.
pub fn hauteur_max(s: Surface, demandee: Option<f32>) -> Option<f32> {
    match s {
        Surface::Volet => demandee.filter(|h| h.is_finite()).map(|h| h.max(1.0)),
        Surface::Menu => None,
    }
}

/// La hauteur permise au contenu, sans les marges de la surface.
pub fn hauteur_max_du_contenu(s: Surface, hauteur_max: Option<f32>) -> Option<f32> {
    hauteur_max.map(|h| h - 2.0 * marge(s))
}

/// La hauteur voulue pour la fenêtre (points) : la hauteur dessinée, arrondie au point
/// au-dessus, sans dépasser la hauteur permise (arrondie de même).
pub fn hauteur_rendue(dessinee: f32, hauteur_max: Option<f32>) -> f32 {
    let mut hauteur = dessinee.ceil();
    if let Some(max) = hauteur_max {
        hauteur = hauteur.min(max.ceil());
    }
    hauteur
}

/// La hauteur où les briques défilent (points) : ce qui reste du contenu permis (`max`) sous
/// l'en-tête et l'état (`deja`), au-dessus du pied (`pied`) ; au moins deux lignes de texte.
pub fn place_des_briques(max: f32, deja: f32, pied: f32) -> f32 {
    (max - deja - pied).max(TAILLE_TEXTE * 2.0)
}

/// L'en-tête du volet : la roue des Paramètres et la place du titre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Entete {
    /// La ligne entière : toute la largeur, la hauteur de la roue.
    pub ligne: Rectangle,
    /// La roue, carrée, collée à droite.
    pub roue: Rectangle,
    /// Le titre et sa version : à gauche, centrés sur la hauteur de la roue, sans jamais
    /// passer sous elle (un écart d'au moins `VOLET_ENTETE_ECART`).
    pub titre: Rectangle,
}

/// La mise en page de l'en-tête dont la ligne commence en `coin`, large de `largeur`, pour
/// un titre haut de `hauteur_titre` (points).
pub fn entete(coin: [f32; 2], largeur: f32, hauteur_titre: f32) -> Entete {
    let ligne = Rectangle::de_min_taille(coin, [largeur, VOLET_ROUE]);
    let roue = Rectangle::de_min_taille(
        [ligne.droite() - VOLET_ROUE, ligne.haut()],
        [VOLET_ROUE, VOLET_ROUE],
    );
    let haut = ligne.haut() + ((VOLET_ROUE - hauteur_titre) / 2.0).max(0.0);
    let place = (largeur - VOLET_ROUE - VOLET_ENTETE_ECART).max(1.0);
    let titre = Rectangle::de_min_taille([ligne.gauche(), haut], [place, hauteur_titre]);
    Entete { ligne, roue, titre }
}

/// La largeur laissée au gros titre (points) dans une place de `place` points : ce qui reste
/// après la version et son écart ; le titre se coupe par « … » au-delà.
pub fn place_du_titre(place: f32, version: Option<f32>) -> f32 {
    let reserve = version.map_or(0.0, |v| ECART_VERSION + v);
    (place - reserve).max(1.0)
}

/// Le coin haut gauche de la version (points), à droite du titre dessiné dans `titre`, sur
/// sa ligne de base. `largeur_titre` : le titre entier mesuré (coupé, il ne prend que la
/// largeur de `titre`) ; `place` : la largeur de
/// l'en-tête pour le titre et sa version ; `base_titre`, `base_version` : la distance du
/// haut de chaque texte à sa ligne de base. La version reste entière : si le titre est
/// coupé, elle se colle au bord droit de la place.
pub fn coin_de_la_version(
    titre: Rectangle,
    largeur_titre: f32,
    place: f32,
    largeur_version: f32,
    (base_titre, base_version): (f32, f32),
) -> [f32; 2] {
    let x = (titre.gauche() + largeur_titre.min(titre.largeur()) + ECART_VERSION)
        .min(titre.gauche() + place - largeur_version);
    let y = titre.haut() + base_titre - base_version;
    [x, y]
}

#[cfg(test)]
mod essais {
    use super::*;

    /// La largeur suit les textes, entre la largeur minimale et la largeur maximale (C-25).
    #[test]
    fn largeur_entre_min_et_max() {
        assert_eq!(largeur(Surface::Volet, 0.0), VOLET_LARGEUR);
        assert_eq!(largeur(Surface::Menu, 0.0), MENU_LARGEUR);
        assert_eq!(largeur(Surface::Volet, 10_000.0), LARGEUR_MAX);
        // Marges comprises, arrondie au point au-dessus.
        let l = largeur(Surface::Volet, 340.2);
        assert_eq!(l, (340.2_f32 + 2.0 * VOLET_MARGE).ceil());
    }

    #[test]
    fn version_vide_jamais_montree() {
        assert_eq!(version_montree(""), None);
        assert_eq!(version_montree("1.2.3"), Some("1.2.3"));
    }

    /// Le plus large l'emporte : l'en-tête, l'état, les briques ou le pied.
    #[test]
    fn besoin_du_volet_le_plus_large() {
        let base = MesuresDuVolet::default();
        let entete = VOLET_MARGE_ETAT + VOLET_ENTETE_ECART + VOLET_ROUE + VOLET_ENTETE_DROITE;
        assert_eq!(besoin_du_volet(&base), entete.max(2.0 * VOLET_MARGE_ETAT));
        let avec_version = MesuresDuVolet {
            titre: 100.0,
            version: Some(30.0),
            ..base
        };
        assert_eq!(
            besoin_du_volet(&avec_version),
            entete + 100.0 + ECART_VERSION + 30.0
        );
        let briques = MesuresDuVolet {
            briques: 500.0,
            pied: 450.0,
            ..base
        };
        assert_eq!(besoin_du_volet(&briques), 500.0);
        let etat = MesuresDuVolet {
            icone_etat: 18.0,
            ecart_etat: 8.0,
            ligne_etat: 300.0,
            ..base
        };
        assert_eq!(
            besoin_du_volet(&etat),
            2.0 * VOLET_MARGE_ETAT + 18.0 + 8.0 + 300.0
        );
    }

    #[test]
    fn besoin_du_menu_sa_ligne_la_plus_large() {
        assert_eq!(besoin_du_menu([120.0, 210.5, 90.0]), 210.5);
        assert_eq!(besoin_du_menu([]), 0.0);
    }

    /// Seul le volet a une hauteur maximale, finie et d'au moins un point.
    #[test]
    fn hauteur_permise() {
        assert_eq!(hauteur_max(Surface::Menu, Some(300.0)), None);
        assert_eq!(hauteur_max(Surface::Volet, None), None);
        assert_eq!(hauteur_max(Surface::Volet, Some(f32::INFINITY)), None);
        assert_eq!(hauteur_max(Surface::Volet, Some(f32::NAN)), None);
        assert_eq!(hauteur_max(Surface::Volet, Some(-5.0)), Some(1.0));
        assert_eq!(hauteur_max(Surface::Volet, Some(500.0)), Some(500.0));
        assert_eq!(
            hauteur_max_du_contenu(Surface::Volet, Some(500.0)),
            Some(500.0 - 2.0 * VOLET_MARGE)
        );
        assert_eq!(hauteur_rendue(120.2, None), 121.0);
        assert_eq!(hauteur_rendue(820.0, Some(600.4)), 601.0);
    }

    /// Les briques gardent au moins deux lignes de texte, même sans place.
    #[test]
    fn briques_au_moins_deux_lignes() {
        assert_eq!(place_des_briques(600.0, 100.0, 50.0), 450.0);
        assert_eq!(place_des_briques(100.0, 100.0, 50.0), TAILLE_TEXTE * 2.0);
    }

    /// La roue est collée à droite ; le titre ne passe jamais sous elle.
    #[test]
    fn entete_roue_a_droite_titre_a_gauche() {
        let e = entete([10.0, 20.0], 300.0, 26.0);
        assert_eq!(e.roue.droite(), 310.0);
        assert_eq!(e.roue.largeur(), VOLET_ROUE);
        assert_eq!(e.titre.gauche(), 10.0);
        assert!(e.titre.droite() + VOLET_ENTETE_ECART <= e.roue.gauche());
        // Centré sur la hauteur de la roue.
        assert_eq!(e.titre.haut(), 20.0 + (VOLET_ROUE - 26.0) / 2.0);
        // Un titre plus haut que la roue part du haut de la ligne.
        assert_eq!(entete([0.0, 0.0], 300.0, 50.0).titre.haut(), 0.0);
        // Aucune place : un point, jamais moins.
        assert_eq!(entete([0.0, 0.0], 10.0, 26.0).titre.largeur(), 1.0);
    }

    /// La version suit le titre ; si le titre est coupé, elle reste entière au bord droit.
    #[test]
    fn version_a_droite_du_titre() {
        assert_eq!(place_du_titre(200.0, None), 200.0);
        assert_eq!(place_du_titre(200.0, Some(40.0)), 200.0 - ECART_VERSION - 40.0);
        let titre = Rectangle::de_min_taille([10.0, 5.0], [150.0, 26.0]);
        let [x, y] = coin_de_la_version(titre, 80.0, 200.0, 40.0, (20.0, 11.0));
        assert_eq!(x, 10.0 + 80.0 + ECART_VERSION);
        assert_eq!(y, 5.0 + 20.0 - 11.0);
        // Titre coupé (plus large que sa place) : la version suit ce qui en est dessiné.
        let [x, _] = coin_de_la_version(titre, 900.0, 200.0, 40.0, (20.0, 11.0));
        assert_eq!(x, 10.0 + 150.0 + ECART_VERSION);
        // Sans assez de place : la version reste entière, collée au bord droit.
        let [x, _] = coin_de_la_version(titre, 900.0, 180.0, 40.0, (20.0, 11.0));
        assert_eq!(x, 10.0 + 180.0 - 40.0);
    }
}
