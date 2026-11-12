//! Le seul fichier de couleurs et de durées (règles : interface Windows 11, points 2 et 10).
//! Les valeurs sont celles de Fluent (Windows 11), en clair et en sombre ; un seul accent :
//! celui de Windows. Vert, orange et rouge disent un état, les mêmes partout ; le rouge sert
//! aussi aux actions qui détruisent. Un logiciel n'écrit aucune couleur ailleurs : il prend
//! celles de [`Palette`].

/// Une couleur opaque : rouge, vert, bleu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Une couleur écrite `0xRRVVBB`.
    pub const fn hex(v: u32) -> Rgb {
        Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }

    /// Luminance relative (WCAG), de 0 à 1.
    pub fn luminance(self) -> f32 {
        let c = |v: u8| {
            let v = f32::from(v) / 255.0;
            if v <= 0.040_45 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * c(self.0) + 0.7152 * c(self.1) + 0.0722 * c(self.2)
    }

    /// Mélange avec `autre`, `t` de 0 (cette couleur) à 1 (l'autre).
    pub fn vers(self, autre: Rgb, t: f32) -> Rgb {
        let m =
            |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * t.clamp(0.0, 1.0)).round() as u8;
        Rgb(m(self.0, autre.0), m(self.1, autre.1), m(self.2, autre.2))
    }
}

/// Contraste entre deux couleurs (WCAG) : 1 à 21 ; 4,5 au moins pour du texte.
pub fn contraste(a: Rgb, b: Rgb) -> f32 {
    let (la, lb) = (a.luminance(), b.luminance());
    let (h, l) = if la > lb { (la, lb) } else { (lb, la) };
    (h + 0.05) / (l + 0.05)
}

const BLANC: Rgb = Rgb(255, 255, 255);
const NOIR: Rgb = Rgb(0, 0, 0);

/// Ce que Windows dit : mode sombre ou clair des applications, et la couleur d'accent
/// (la palette de 7 nuances, de la plus claire à la plus foncée, si Windows la donne).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChoixWindows {
    pub sombre: bool,
    /// Barre des tâches sombre (pour l'icône près de l'horloge).
    pub barre_sombre: bool,
    pub accent: Option<Rgb>,
    pub nuances: Option<[Rgb; 7]>,
}

/// La palette complète d'un mode.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub sombre: bool,
    /// Fond des fenêtres.
    pub fond: Rgb,
    /// Fond du menu de l'icône.
    pub fond_menu: Rgb,
    /// Fond et bord des cartes de réglage.
    pub carte: Rgb,
    pub bord_carte: Rgb,
    /// Fond d'un élément survolé, et appuyé.
    pub survol: Rgb,
    pub appui: Rgb,
    /// Fond des contrôles (boutons, listes, champs).
    pub controle: Rgb,
    pub bord_controle: Rgb,
    pub separateur: Rgb,
    pub texte: Rgb,
    /// Texte secondaire : bien contrasté (jamais gris pâle).
    pub texte_secondaire: Rgb,
    pub texte_desactive: Rgb,
    /// L'accent de Windows, adapté au mode, et le texte posé dessus.
    pub accent: Rgb,
    pub sur_accent: Rgb,
    /// États : prêt, en cours, problème (et actions qui détruisent).
    pub vert: Rgb,
    pub orange: Rgb,
    pub rouge: Rgb,
}

/// Accent par défaut de Windows 11 (bleu), si Windows n'en donne pas.
const ACCENT_CLAIR: Rgb = Rgb::hex(0x005FB8);
const ACCENT_SOMBRE: Rgb = Rgb::hex(0x60CDFF);

impl Palette {
    pub fn de(choix: &ChoixWindows) -> Palette {
        let sombre = choix.sombre;
        // Comme Windows 11 : en clair, la nuance « foncée 1 » de l'accent ; en sombre, la
        // « claire 2 ». Sans palette, l'accent de base éclairci ou foncé.
        let accent = match (choix.nuances, choix.accent) {
            (Some(n), _) => {
                if sombre {
                    n[1]
                } else {
                    n[4]
                }
            }
            (None, Some(a)) => {
                if sombre {
                    a.vers(BLANC, 0.35)
                } else {
                    a.vers(NOIR, 0.15)
                }
            }
            (None, None) => {
                if sombre {
                    ACCENT_SOMBRE
                } else {
                    ACCENT_CLAIR
                }
            }
        };
        let sur_accent = if contraste(accent, NOIR) >= contraste(accent, BLANC) {
            NOIR
        } else {
            BLANC
        };
        if sombre {
            Palette {
                sombre,
                fond: Rgb::hex(0x202020),
                fond_menu: Rgb::hex(0x2C2C2C),
                carte: Rgb::hex(0x2B2B2B),
                bord_carte: Rgb::hex(0x1D1D1D),
                survol: Rgb::hex(0x383838),
                appui: Rgb::hex(0x323232),
                controle: Rgb::hex(0x373737),
                bord_controle: Rgb::hex(0x454545),
                separateur: Rgb::hex(0x404040),
                texte: Rgb::hex(0xFFFFFF),
                texte_secondaire: Rgb::hex(0xCFCFCF),
                texte_desactive: Rgb::hex(0x878787),
                accent,
                sur_accent,
                vert: Rgb::hex(0x6CCB5F),
                orange: Rgb::hex(0xFCE100),
                rouge: Rgb::hex(0xFF99A4),
            }
        } else {
            Palette {
                sombre,
                fond: Rgb::hex(0xF3F3F3),
                fond_menu: Rgb::hex(0xF9F9F9),
                carte: Rgb::hex(0xFBFBFB),
                bord_carte: Rgb::hex(0xE5E5E5),
                survol: Rgb::hex(0xEAEAEA),
                appui: Rgb::hex(0xE0E0E0),
                controle: Rgb::hex(0xFEFEFE),
                bord_controle: Rgb::hex(0xD6D6D6),
                separateur: Rgb::hex(0xE0E0E0),
                texte: Rgb::hex(0x1A1A1A),
                texte_secondaire: Rgb::hex(0x505050),
                texte_desactive: Rgb::hex(0x8A8A8A),
                accent,
                sur_accent,
                vert: Rgb::hex(0x0F7B0F),
                orange: Rgb::hex(0x9D5D00),
                rouge: Rgb::hex(0xC42B1C),
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Le texte et les arrondis (règles : interface Windows 11, points 3, 5 et 8)

/// Titre de page, en points.
pub const TAILLE_TITRE: f32 = 28.0;
/// Le gros titre du volet (l'état, ce qu'on vient y voir).
pub const TAILLE_TITRE_VOLET: f32 = 20.0;
/// Titre de section (semibold).
pub const TAILLE_SECTION: f32 = 15.0;
/// Le texte courant.
pub const TAILLE_TEXTE: f32 = 14.0;
/// Le texte secondaire : jamais plus petit.
pub const TAILLE_PETIT: f32 = 12.0;
/// La hauteur d'une ligne cliquable (menu, volet, navigation) : au moins 32 points.
pub const HAUTEUR_LIGNE: f32 = 36.0;
/// L'arrondi des cartes, et celui des contrôles.
pub const ARRONDI_CARTE: u8 = 8;
pub const ARRONDI_CONTROLE: u8 = 4;

// ---------------------------------------------------------------------------------------
// Les animations du volet et du petit menu (règles : interface Windows 11, point 10)
//
// Les durées sont celles de Windows 11, prises dans la documentation de Microsoft (« Timing
// and easing », guide Fluent de WinUI 3, learn.microsoft.com/windows/apps/design/motion/
// timing-and-easing) : les trois durées standard des contrôles de Windows,
// `ControlNormalAnimationDuration` = 250 ms, `ControlFastAnimationDuration` = 167 ms et
// `ControlFasterAnimationDuration` = 83 ms ; la courbe d'entrée « Fast Out, Slow In »,
// cubic-bezier(0, 0, 0, 1) (rapide au départ, ralentie à l'arrivée). La distance du
// glissement vient du guide des fenêtres surgissantes de Microsoft (« Animating pop-up UI ») :
// 50 pixels, depuis le côté de ce qui les ouvre (ici, la barre des tâches).
// Les mêmes valeurs dans tous les projets.

/// Le volet monte depuis la barre des tâches : `ControlNormalAnimationDuration`.
pub const VOLET_GLISSEMENT_MS: u32 = 250;
/// Il apparaît en fondu pendant ce temps : `ControlFastAnimationDuration`.
pub const VOLET_FONDU_MS: u32 = 167;
/// Le petit menu apparaît en fondu rapide, sans glisser : `ControlFastAnimationDuration`.
pub const MENU_FONDU_MS: u32 = 167;
/// À la fermeture, le volet comme le petit menu disparaissent en fondu court :
/// `ControlFasterAnimationDuration`.
pub const FERMETURE_FONDU_MS: u32 = 83;
/// La distance du glissement du volet, en points (multipliée par l'échelle de l'écran).
pub const VOLET_DECALAGE: f32 = 50.0;

/// La courbe « Fast Out, Slow In » de Windows, cubic-bezier(0, 0, 0, 1) : pour l'avancée `x`
/// du temps (0 à 1), la part du chemin faite (0 à 1). Avec ces points de contrôle, la courbe
/// s'écrit x = t³ et y = 3t² - 2t³ : on retrouve t par la racine cubique.
pub fn ralenti_a_l_arrivee(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    let t = x.cbrt();
    3.0 * t * t - 2.0 * t * t * t
}

/// Ce qui s'ouvre près de l'horloge : le volet (clic gauche) ou le petit menu (clic droit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Volet,
    Menu,
}

/// Une image de l'animation : l'opacité (0 à 255) et le décalage vers la barre des tâches,
/// en points (0 : à sa place).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Image {
    pub opacite: u8,
    pub decalage: f32,
    /// L'animation est finie.
    pub finie: bool,
}

/// L'image de l'ouverture `ms` millisecondes après son début. `effets` : « Effets
/// d'animation » de Windows ; coupés, tout est à sa place tout de suite.
pub fn ouverture(surface: Surface, ms: u32, effets: bool) -> Image {
    if !effets {
        return Image {
            opacite: 255,
            decalage: 0.0,
            finie: true,
        };
    }
    let (fondu, glissement) = match surface {
        Surface::Volet => (VOLET_FONDU_MS, VOLET_GLISSEMENT_MS),
        Surface::Menu => (MENU_FONDU_MS, 0),
    };
    let part = |duree: u32| {
        if duree == 0 {
            1.0
        } else {
            (ms as f32 / duree as f32).clamp(0.0, 1.0)
        }
    };
    let opacite = (255.0 * part(fondu)).round() as u8;
    let decalage = VOLET_DECALAGE * (1.0 - ralenti_a_l_arrivee(part(glissement)));
    let decalage = if glissement == 0 { 0.0 } else { decalage };
    Image {
        opacite,
        decalage,
        finie: ms >= fondu.max(glissement),
    }
}

/// L'image de la fermeture `ms` millisecondes après son début (un fondu, sur place).
pub fn fermeture(ms: u32, effets: bool) -> Image {
    if !effets || ms >= FERMETURE_FONDU_MS {
        return Image {
            opacite: 0,
            decalage: 0.0,
            finie: true,
        };
    }
    let reste = 1.0 - ms as f32 / FERMETURE_FONDU_MS as f32;
    Image {
        opacite: (255.0 * reste).round() as u8,
        decalage: 0.0,
        finie: false,
    }
}

/// La part du pixel (`x`, `y`) dans une forme (0 à 255), en échantillonnant 4×4 points par
/// pixel (bords lissés) : pour dessiner l'icône d'un logiciel sans fichier image. `s` :
/// pixels par unité de la grille du dessin ; `forme` dit si un point de la grille est dans
/// la forme.
pub fn couverture(x: usize, y: usize, s: f32, forme: impl Fn(f32, f32) -> bool) -> u8 {
    let mut n = 0u32;
    for sy in 0..4 {
        for sx in 0..4 {
            let fx = (x as f32 + (sx as f32 + 0.5) / 4.0) / s;
            let fy = (y as f32 + (sy as f32 + 0.5) / 4.0) / s;
            n += u32::from(forme(fx, fy));
        }
    }
    (n * 255 / 16) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le texte est lisible sur tous les fonds, en clair comme en sombre, quel que soit
    /// l'accent de Windows (règle : pas de petit texte gris pâle).
    #[test]
    fn contrastes_suffisants() {
        let accents = [
            None,
            Some(Rgb(0, 120, 215)),
            Some(Rgb(255, 185, 0)),
            Some(Rgb(16, 124, 16)),
            Some(Rgb(232, 17, 35)),
            Some(Rgb(0, 0, 0)),
            Some(Rgb(255, 255, 255)),
        ];
        for sombre in [false, true] {
            for a in accents {
                let p = Palette::de(&ChoixWindows {
                    sombre,
                    barre_sombre: sombre,
                    accent: a,
                    nuances: None,
                });
                for fond in [p.fond, p.fond_menu, p.carte, p.survol, p.controle] {
                    assert!(contraste(p.texte, fond) >= 7.0, "{p:?}");
                    assert!(contraste(p.texte_secondaire, fond) >= 4.5, "{p:?}");
                    for etat in [p.vert, p.orange, p.rouge] {
                        assert!(contraste(etat, fond) >= 3.0, "état peu lisible : {p:?}");
                    }
                }
                assert!(contraste(p.sur_accent, p.accent) >= 3.0, "{p:?}");
            }
        }
    }

    #[test]
    fn la_palette_de_windows_est_suivie() {
        let n = [
            Rgb(1, 1, 1),
            Rgb(2, 2, 2),
            Rgb(3, 3, 3),
            Rgb(4, 4, 4),
            Rgb(5, 5, 5),
            Rgb(6, 6, 6),
            Rgb(7, 7, 7),
        ];
        let c = |sombre| ChoixWindows {
            sombre,
            barre_sombre: true,
            accent: None,
            nuances: Some(n),
        };
        assert_eq!(Palette::de(&c(true)).accent, Rgb(2, 2, 2));
        assert_eq!(Palette::de(&c(false)).accent, Rgb(5, 5, 5));
    }

    /// La courbe de Windows : part de 0, arrive à 1, toujours croissante, et déjà bien
    /// avancée à mi-temps (rapide au départ, ralentie à l'arrivée).
    #[test]
    fn courbe_rapide_puis_ralentie() {
        assert!(ralenti_a_l_arrivee(0.0).abs() < 1e-6);
        assert!((ralenti_a_l_arrivee(1.0) - 1.0).abs() < 1e-6);
        assert!(ralenti_a_l_arrivee(0.5) > 0.8, "{}", ralenti_a_l_arrivee(0.5));
        let mut avant = 0.0;
        for i in 1..=100 {
            let y = ralenti_a_l_arrivee(i as f32 / 100.0);
            assert!(y >= avant, "la courbe recule à {i} %");
            avant = y;
        }
        // Hors de 0 à 1 : bornée.
        assert!((ralenti_a_l_arrivee(2.0) - 1.0).abs() < 1e-6);
        assert!(ralenti_a_l_arrivee(-1.0).abs() < 1e-6);
    }

    /// Les durées de Windows 11 : environ 0,2 s pour ouvrir, environ 0,1 s pour fermer.
    #[test]
    fn durees_de_windows() {
        assert_eq!(VOLET_GLISSEMENT_MS, 250);
        assert_eq!(VOLET_FONDU_MS, 167);
        assert_eq!(MENU_FONDU_MS, 167);
        assert_eq!(FERMETURE_FONDU_MS, 83);
    }

    #[test]
    fn animation_du_volet() {
        let debut = ouverture(Surface::Volet, 0, true);
        assert_eq!(debut.opacite, 0);
        assert!((debut.decalage - VOLET_DECALAGE).abs() < 1e-3);
        assert!(!debut.finie);
        let milieu = ouverture(Surface::Volet, 100, true);
        assert!(milieu.opacite > 100 && milieu.opacite < 255);
        assert!(milieu.decalage > 0.0 && milieu.decalage < VOLET_DECALAGE / 2.0);
        let fin = ouverture(Surface::Volet, VOLET_GLISSEMENT_MS, true);
        assert_eq!(fin.opacite, 255);
        assert!(fin.decalage.abs() < 1e-3);
        assert!(fin.finie);
        // Le petit menu ne glisse pas : un fondu seul.
        let menu = ouverture(Surface::Menu, 50, true);
        assert!(menu.decalage.abs() < 1e-6);
        assert!(menu.opacite > 0 && menu.opacite < 255);
        assert!(ouverture(Surface::Menu, MENU_FONDU_MS, true).finie);
        // Fermeture : un fondu court.
        assert_eq!(fermeture(0, true).opacite, 255);
        assert!(!fermeture(40, true).finie);
        assert!(fermeture(FERMETURE_FONDU_MS, true).finie);
    }

    /// « Effets d'animation » coupé dans Windows : aucune animation.
    #[test]
    fn sans_effets_aucune_animation() {
        for s in [Surface::Volet, Surface::Menu] {
            let i = ouverture(s, 0, false);
            assert_eq!(i.opacite, 255);
            assert!(i.decalage.abs() < 1e-6);
            assert!(i.finie);
        }
        assert!(fermeture(0, false).finie);
    }
}
