//! Le seul fichier de couleurs (règles : interface Windows 11, points 2 et 10). Les valeurs
//! sont celles de Fluent (Windows 11), en clair et en sombre ; un seul accent : celui de
//! Windows. Vert, orange et rouge disent un état, les mêmes partout ; le rouge sert aussi aux
//! actions qui détruisent. Un logiciel n'écrit aucune couleur ailleurs : il prend celles de
//! [`Palette`].

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

pub(super) const BLANC: Rgb = Rgb(255, 255, 255);
pub(super) const NOIR: Rgb = Rgb(0, 0, 0);

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
        let accent = accent(choix);
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

/// L'accent du mode, comme Windows 11 : en clair, la nuance « foncée 1 » de l'accent ; en
/// sombre, la « claire 2 ». Sans palette, l'accent de base éclairci ou foncé ; sans accent, le
/// bleu de Windows.
fn accent(choix: &ChoixWindows) -> Rgb {
    let sombre = choix.sombre;
    match (choix.nuances, choix.accent) {
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
    }
}

/// Les 10 couleurs des pastilles et des cases : lisibles en clair comme en sombre (3 contre
/// 1 au moins avec le fond des deux modes, lettre à 4,5 contre 1), sans vert, orange, rouge
/// ni bleu d'accent, qui disent un état ou l'accent de Windows, et à un écart ΔE2000 d'au
/// moins 12 l'une de l'autre (contrôlé par `theme/essais_pastilles.rs`). Un logiciel prend la
/// suivante pour chaque appareil. Les 6 premières ne bougent jamais : un rang déjà enregistré
/// garde sa couleur ; les suivantes s'ajoutent derrière.
pub fn couleurs_pastilles() -> [Rgb; 10] {
    [
        Rgb::hex(0x7B57C9), // violet
        Rgb::hex(0xB03A9C), // magenta
        Rgb::hex(0x1D7F86), // sarcelle foncé
        Rgb::hex(0x94604F), // brun
        Rgb::hex(0x667A94), // ardoise
        Rgb::hex(0x857A2A), // olive
        Rgb::hex(0x3865F3), // indigo
        Rgb::hex(0xC0416B), // framboise
        Rgb::hex(0x8E679E), // prune
        Rgb::hex(0x7C6D61), // taupe
    ]
}

/// La lettre lisible sur ce fond : blanche, ou noire si le fond est clair (contraste
/// calculé, le plus fort des deux).
pub fn couleur_lettre(fond: Rgb) -> Rgb {
    if contraste(fond, BLANC) >= contraste(fond, NOIR) {
        BLANC
    } else {
        NOIR
    }
}
