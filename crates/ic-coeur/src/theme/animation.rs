//! Les animations du volet et du petit menu (règles : interface Windows 11, point 10).
//!
//! Les durées viennent de la documentation de Microsoft (« Timing and easing », guide Fluent
//! de WinUI 3, learn.microsoft.com/windows/apps/design/motion/timing-and-easing) :
//! `ControlFasterAnimationDuration` = 83 ms, la plus courte des durées des contrôles de
//! Windows, et la courbe d'entrée « Fast Out, Slow In », cubic-bezier(0, 0, 0, 1) (rapide au
//! départ, ralentie à l'arrivée).
//!
//! Ouvrir doit être vif : le volet et le petit menu apparaissent **à leur place**, sans
//! glisser, en un fondu de 83 ms qui part déjà à 50 % d'opacité. Mesuré image par image
//! (`examples/mesure_ouverture.rs`) : l'ancienne animation (montée de 50 px en 250 ms, fondu
//! de 167 ms parti de 0) ne rendait le volet lisible qu'à ~150 ms du clic, et sa première
//! image montrée était transparente ; la sensation de latence venait d'elle, pas d'un
//! retard avant elle (2 à 7 ms). Désormais lisible dès la première image montrée.
//! Les mêmes valeurs dans tous les projets.

use std::time::Duration;

/// Le volet et le petit menu apparaissent en fondu court, sur place :
/// `ControlFasterAnimationDuration`.
pub const OUVERTURE_FONDU_MS: u32 = 83;
/// L'opacité de la première image montrée (sur 255) : déjà visible, jamais transparente.
pub const OUVERTURE_OPACITE_DE_DEPART: u8 = 128;
/// À la fermeture, le volet comme le petit menu disparaissent en fondu court :
/// `ControlFasterAnimationDuration`.
pub const FERMETURE_FONDU_MS: u32 = 83;

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

/// Une image de l'animation : l'opacité (0 à 255).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Image {
    pub opacite: u8,
    /// L'animation est finie.
    pub finie: bool,
}

/// L'image de l'ouverture `ms` millisecondes après son début. `effets` : « Effets
/// d'animation » de Windows ; coupés, tout est opaque tout de suite.
pub fn ouverture(ms: u32, effets: bool) -> Image {
    if !effets || ms >= OUVERTURE_FONDU_MS {
        return Image {
            opacite: 255,
            finie: true,
        };
    }
    let depart = f32::from(OUVERTURE_OPACITE_DE_DEPART);
    let part = ralenti_a_l_arrivee(ms as f32 / OUVERTURE_FONDU_MS as f32);
    Image {
        opacite: (depart + (255.0 - depart) * part).round() as u8,
        finie: false,
    }
}

/// L'image de la fermeture `ms` millisecondes après son début (un fondu, sur place).
pub fn fermeture(ms: u32, effets: bool) -> Image {
    if !effets || ms >= FERMETURE_FONDU_MS {
        return Image {
            opacite: 0,
            finie: true,
        };
    }
    let reste = 1.0 - ms as f32 / FERMETURE_FONDU_MS as f32;
    Image {
        opacite: (255.0 * reste).round() as u8,
        finie: false,
    }
}

/// Le plus long pas de l'animation d'une image à la suivante : une image d'un écran à 50 Hz.
/// Une image en retard (écran qui fait attendre) avance l'animation d'un pas seulement :
/// l'ouverture se joue toujours en entier, jamais d'un coup à la fin.
pub const PAS_MAX: Duration = Duration::from_millis(20);

/// L'animation jouée après une image de plus, arrivée `ecart` après la précédente : le temps
/// écoulé est donné par l'appelant, jamais lu ici.
pub fn avancer(anime: Duration, ecart: Duration) -> Duration {
    anime + ecart.min(PAS_MAX)
}
