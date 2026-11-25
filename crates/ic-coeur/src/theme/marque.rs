//! Ce qu'un carré coloré montre (bloc F) : le logo de l'appareil si la police d'icônes l'a,
//! sinon sa lettre ; et le texte lisible sur la couleur d'une zone. Le dessin vit dans
//! `interface_commune::pastilles` et `interface_commune::seuils`.

use super::couleurs::{Rgb, contraste};

/// Ce qu'un carré montre : le logo (glyphe), ou la lettre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marque {
    Logo(char),
    Lettre(char),
}

/// Le logo si la police d'icônes l'a (`disponible`), sinon la lettre, sinon rien : jamais un
/// carré vide à la place d'un glyphe absent. La police est passée en paramètre.
pub fn marque(logo: Option<char>, lettre: Option<char>, disponible: impl Fn(char) -> bool) -> Option<Marque> {
    logo.filter(|&g| disponible(g))
        .map(Marque::Logo)
        .or_else(|| lettre.map(Marque::Lettre))
}

/// Un texte lisible (noir ou blanc) sur la couleur d'une zone : pour écrire dans une zone ou
/// sur une pastille de sa couleur. À contraste égal, le noir.
pub fn texte_sur(zone: Rgb) -> Rgb {
    let (noir, blanc) = (Rgb(0, 0, 0), Rgb(255, 255, 255));
    if contraste(zone, noir) >= contraste(zone, blanc) {
        noir
    } else {
        blanc
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    #[test]
    fn le_logo_si_la_police_l_a_sinon_la_lettre() {
        let tout = |_: char| true;
        let rien = |_: char| false;
        assert_eq!(
            marque(Some('\u{E7F4}'), Some('A'), tout),
            Some(Marque::Logo('\u{E7F4}'))
        );
        assert_eq!(
            marque(Some('\u{E7F4}'), Some('A'), rien),
            Some(Marque::Lettre('A'))
        );
        assert_eq!(marque(None, Some('A'), tout), Some(Marque::Lettre('A')));
        assert_eq!(marque(Some('\u{E7F4}'), None, rien), None);
        assert_eq!(marque(None, None, tout), None);
    }

    #[test]
    fn le_texte_sur_une_zone_est_lisible() {
        assert_eq!(texte_sur(Rgb(255, 255, 0)), Rgb(0, 0, 0));
        assert_eq!(texte_sur(Rgb(0, 40, 160)), Rgb(255, 255, 255));
    }
}
