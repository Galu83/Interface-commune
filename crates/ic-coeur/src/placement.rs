//! Où ouvrir le volet et le petit menu de l'icône, et où rouvrir les Paramètres. Comme les
//! menus de Windows 11 : collé à la barre des tâches, en face de l'icône, en entier sur son
//! écran. Le côté de la barre se déduit de la différence entre l'écran entier et sa zone de
//! travail. Du calcul pur, testé partout.
//!
//! Bloc D : ici depuis la bibliothèque (`interface_commune::place` le reprend). La place
//! retenue compte le temps avec l'instant reçu en paramètre ([`Horodatage`]).

use crate::gestes::Horodatage;

#[cfg(test)]
mod essais;
pub mod plan;
pub mod surface;

/// Un rectangle en pixels : gauche, haut, droite, bas (droite et bas exclus).
pub type Rect = [i32; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cote {
    Bas,
    Haut,
    Gauche,
    Droite,
}

/// Le côté de la barre des tâches. Barre masquée (zone de travail = écran entier) : le bord
/// de l'écran le plus proche de l'icône.
pub fn cote_de_la_barre(icone: Rect, ecran: Rect, travail: Rect) -> Cote {
    if travail[3] < ecran[3] {
        return Cote::Bas;
    }
    if travail[1] > ecran[1] {
        return Cote::Haut;
    }
    if travail[0] > ecran[0] {
        return Cote::Gauche;
    }
    if travail[2] < ecran[2] {
        return Cote::Droite;
    }
    let cx = (i64::from(icone[0]) + i64::from(icone[2])) / 2;
    let cy = (i64::from(icone[1]) + i64::from(icone[3])) / 2;
    let distances = [
        (i64::from(ecran[3]) - cy, Cote::Bas),
        (cy - i64::from(ecran[1]), Cote::Haut),
        (cx - i64::from(ecran[0]), Cote::Gauche),
        (i64::from(ecran[2]) - cx, Cote::Droite),
    ];
    distances
        .iter()
        .min_by_key(|(d, _)| d.abs())
        .map_or(Cote::Bas, |(_, c)| *c)
}

/// La place (gauche, haut, largeur, hauteur) d'une fenêtre de `taille` ouverte depuis
/// `icone`, sur l'écran `ecran` dont la zone de travail est `travail`, à `marge` pixels de la
/// barre et des bords. Une fenêtre trop grande est rapetissée.
pub fn placer(icone: Rect, taille: [i32; 2], ecran: Rect, travail: Rect, marge: i32) -> Rect {
    let z = travail.map(i64::from);
    let (zg, zh) = (z[0], z[1]);
    let (zd, zb) = (z[2].max(zg + 1), z[3].max(zh + 1));
    let m = i64::from(marge.clamp(0, 48));
    let largeur = i64::from(taille[0]).clamp(1, (zd - zg - 2 * m).max(1));
    let hauteur = i64::from(taille[1]).clamp(1, (zb - zh - 2 * m).max(1));
    let i = icone.map(i64::from);
    let centre_x = i[0] + (i[2] - i[0]) / 2;
    let (x, y) = match cote_de_la_barre(icone, ecran, travail) {
        Cote::Bas => (centre_x - largeur / 2, zb - m - hauteur),
        Cote::Haut => (centre_x - largeur / 2, zh + m),
        Cote::Gauche => (zg + m, i[3] - hauteur),
        Cote::Droite => (zd - m - largeur, i[3] - hauteur),
    };
    let x = x.clamp(zg + m, (zd - m - largeur).max(zg + m));
    let y = y.clamp(zh + m, (zb - m - hauteur).max(zh + m));
    let vers_i32 = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    [vers_i32(x), vers_i32(y), vers_i32(largeur), vers_i32(hauteur)]
}

/// La place (gauche, haut, largeur, hauteur) du petit menu (clic droit), comme les menus de
/// Windows : son coin touche le point du clic (`point`), du côté opposé à la barre des
/// tâches (au-dessus du clic quand la barre est en bas), vers la droite s'il y a la place,
/// sinon vers la gauche ; toujours en entier dans la zone de travail, à `marge` pixels des
/// bords. Le côté de la barre se déduit comme pour [`placer`] (`icone`).
pub fn placer_au_point(
    point: [i32; 2],
    icone: Rect,
    taille: [i32; 2],
    ecran: Rect,
    travail: Rect,
    marge: i32,
) -> Rect {
    let z = travail.map(i64::from);
    let (zg, zh) = (z[0], z[1]);
    let (zd, zb) = (z[2].max(zg + 1), z[3].max(zh + 1));
    let m = i64::from(marge.clamp(0, 48));
    let largeur = i64::from(taille[0]).clamp(1, (zd - zg - 2 * m).max(1));
    let hauteur = i64::from(taille[1]).clamp(1, (zb - zh - 2 * m).max(1));
    // Le point, ramené dans la zone de travail (le clic est sur la barre, hors de la zone).
    let px = i64::from(point[0]).clamp(zg, zd);
    let py = i64::from(point[1]).clamp(zh, zb);
    let x = if px + largeur <= zd - m { px } else { px - largeur };
    let y = match cote_de_la_barre(icone, ecran, travail) {
        Cote::Haut => py,
        Cote::Bas => py - hauteur,
        Cote::Gauche | Cote::Droite => {
            if py - hauteur >= zh + m {
                py - hauteur
            } else {
                py
            }
        }
    };
    let x = x.clamp(zg + m, (zd - m - largeur).max(zg + m));
    let y = y.clamp(zh + m, (zb - m - hauteur).max(zh + m));
    let vers_i32 = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    [vers_i32(x), vers_i32(y), vers_i32(largeur), vers_i32(hauteur)]
}

/// La place (gauche, haut, largeur, hauteur) d'une fenêtre de `taille` ouverte **contre
/// l'icône** `icone`, dans la zone de travail `travail`, à `marge` pixels de l'icône et des
/// bords : en face de l'icône, du côté opposé à la barre des tâches (au-dessus quand elle est
/// en bas, à droite quand elle est à gauche…). Une icône dans la zone de travail (barre
/// masquée, icônes cachées sous la flèche) : au-dessus d'elle s'il y a la place, sinon
/// dessous. Toujours en entier dans la zone ; une fenêtre trop grande est rapetissée.
pub fn placer_contre_l_icone(icone: Rect, taille: [i32; 2], travail: Rect, marge: i32) -> Rect {
    let z = travail.map(i64::from);
    let (zg, zh) = (z[0], z[1]);
    let (zd, zb) = (z[2].max(zg + 1), z[3].max(zh + 1));
    let m = i64::from(marge.clamp(0, 48));
    let largeur = i64::from(taille[0]).clamp(1, (zd - zg - 2 * m).max(1));
    let hauteur = i64::from(taille[1]).clamp(1, (zb - zh - 2 * m).max(1));
    let [ig, ih, id, ib] = icone.map(i64::from);
    let (cx, cy) = ((ig + id).div_euclid(2), (ih + ib).div_euclid(2));
    let (x, y) = if cy >= zb {
        // La barre en bas : au-dessus de l'icône, centré sur elle.
        (cx - largeur / 2, ih - m - hauteur)
    } else if cy < zh {
        // En haut : dessous.
        (cx - largeur / 2, ib + m)
    } else if cx < zg {
        // À gauche : à sa droite, le bas au niveau du bas de l'icône.
        (id + m, ib - hauteur)
    } else if cx >= zd {
        // À droite : à sa gauche.
        (ig - m - largeur, ib - hauteur)
    } else if ih - m - hauteur >= zh + m {
        (cx - largeur / 2, ih - m - hauteur)
    } else {
        (cx - largeur / 2, ib + m)
    };
    let x = x.clamp(zg + m, (zd - m - largeur).max(zg + m));
    let y = y.clamp(zh + m, (zb - m - hauteur).max(zh + m));
    let vers_i32 = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    [vers_i32(x), vers_i32(y), vers_i32(largeur), vers_i32(hauteur)]
}

/// L'écart du volet avec la barre et les bords quand le logiciel n'en choisit pas (pixels,
/// quelle que soit l'échelle de l'écran : le comportement d'avant [`PlacementVolet`]).
pub const ECART_VOLET: i32 = 12;

/// Où et comment le volet se place (`Logiciel::placement_volet`). `Default` : comme les volets
/// de Windows 11, collé à la barre des tâches en face de l'icône, à [`ECART_VOLET`] pixels,
/// aussi haut que son contenu. Pour changer un seul point :
/// `PlacementVolet { hauteur_max: Some(600.0), ..Default::default() }`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlacementVolet {
    /// Contre l'icône ([`placer_contre_l_icone`]) plutôt que contre la barre ([`placer`]).
    pub contre_l_icone: bool,
    /// L'écart avec l'icône (ou la barre) et les bords, en points : converti en pixels réels à
    /// l'échelle de l'écran du volet. `None` : [`ECART_VOLET`] pixels.
    pub ecart: Option<f32>,
    /// La hauteur maximale du volet, en points. Au-delà, l'en-tête (titre, roue) et l'état
    /// restent en place et les briques défilent. Le volet ne dépasse jamais non plus la zone de travail de
    /// son écran. `None` : aussi haut que son contenu.
    pub hauteur_max: Option<f32>,
}

/// Des pixels par point utilisables (une échelle absurde vaut 1).
fn echelle_sure(ppp: f32) -> f32 {
    if ppp.is_finite() && ppp > 0.0 { ppp } else { 1.0 }
}

impl PlacementVolet {
    /// L'écart en pixels réels, à l'échelle `ppp` (pixels par point) de l'écran.
    pub fn ecart_px(&self, ppp: f32) -> i32 {
        match self.ecart {
            Some(e) if e.is_finite() => (e.max(0.0) * echelle_sure(ppp)).round().min(48.0) as i32,
            _ => ECART_VOLET,
        }
    }

    /// La place du volet de `taille` (pixels), ouvert depuis `icone`, sur l'écran `ecran` dont
    /// la zone de travail est `travail`, à l'échelle `ppp`.
    pub fn placer(&self, icone: Rect, taille: [i32; 2], ecran: Rect, travail: Rect, ppp: f32) -> Rect {
        let m = self.ecart_px(ppp);
        if self.contre_l_icone {
            placer_contre_l_icone(icone, taille, travail, m)
        } else {
            placer(icone, taille, ecran, travail, m)
        }
    }

    /// La hauteur que le volet ne dépasse pas (points), sur un écran dont la zone de travail
    /// est `travail` (pixels), à l'échelle `ppp` : la hauteur maximale voulue, bornée par la
    /// zone de travail moins les écarts. `None` sans hauteur maximale.
    pub fn hauteur_max_points(&self, travail: Rect, ppp: f32) -> Option<f32> {
        let voulue = self.hauteur_max.filter(|h| h.is_finite())?;
        let ppp = echelle_sure(ppp);
        let zone = (i64::from(travail[3]) - i64::from(travail[1]) - 2 * i64::from(self.ecart_px(ppp))) as f32;
        Some(voulue.min(zone / ppp).max(1.0))
    }
}

/// Une place de fenêtre retenue : gauche, haut, largeur, hauteur, en pixels.
pub type Place = [i32; 4];

/// La place d'une fenêtre est retenue ce temps après son dernier mouvement (et à sa
/// fermeture) : un déplacement à la souris n'écrit pas les réglages à chaque pixel, et un arrêt
/// brutal du logiciel ne perd pas la dernière place.
pub const RETENIR_APRES: std::time::Duration = std::time::Duration::from_secs(1);

/// Ce qu'il faut faire de la place d'une fenêtre à cette image ([`SuiviDePlace`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retenue {
    /// Rien n'a bougé depuis la dernière place retenue.
    Rien,
    /// La fenêtre a bougé : revenir dans ce temps (une image de plus), la place n'est pas
    /// encore retenue.
    Attendre(std::time::Duration),
    /// Elle ne bouge plus depuis [`RETENIR_APRES`] : retenir sa place maintenant.
    Retenir,
}

/// Le suivi de la place d'une fenêtre ouverte, image par image, pour la retenir
/// [`RETENIR_APRES`] après son dernier mouvement. Du calcul pur, testé partout.
/// `T` : l'instant ([`Horodatage`]).
#[derive(Debug, Clone)]
pub struct SuiviDePlace<T> {
    vue: Option<Place>,
    bougee: Option<T>,
}

// À la main : `derive(Default)` demanderait `T: Default`, qu'`Instant` n'a pas.
impl<T> Default for SuiviDePlace<T> {
    fn default() -> Self {
        SuiviDePlace {
            vue: None,
            bougee: None,
        }
    }
}

impl<T: Horodatage> SuiviDePlace<T> {
    /// La place de la fenêtre à cette image (`None` : inconnue, ou fenêtre pas encore montrée).
    /// La première place vue ne compte pas comme un mouvement.
    pub fn voir(&mut self, place: Option<Place>, maintenant: T) -> Retenue {
        if place.is_some() && place != self.vue {
            if self.vue.is_some() {
                self.bougee = Some(maintenant);
            }
            self.vue = place;
        }
        let Some(depuis) = self.bougee else {
            return Retenue::Rien;
        };
        let ecoule = maintenant.depuis_ou_zero(depuis);
        if ecoule >= RETENIR_APRES {
            self.bougee = None;
            Retenue::Retenir
        } else {
            Retenue::Attendre(RETENIR_APRES - ecoule)
        }
    }
}

/// La place des Paramètres à retenir d'une ouverture à l'autre ; `None` si elle n'a pas de
/// sens (fenêtre réduite : Windows la range très loin de l'écran ; taille minuscule).
pub fn place_valable(f: Place) -> Option<Place> {
    let [x, y, l, h] = f;
    if x <= -30_000 || y <= -30_000 || l < 200 || h < 150 {
        return None;
    }
    Some([
        x.clamp(-20_000, 60_000),
        y.clamp(-20_000, 60_000),
        l.min(20_000),
        h.min(20_000),
    ])
}

/// La place retenue, ramenée en entier dans la zone de travail `travail` de l'écran le plus
/// proche (l'écran a pu changer, ou être débranché, depuis la dernière fois) : la fenêtre
/// n'est jamais plus grande que la zone, ni plus petite que `minimum` (largeur, hauteur) si
/// la zone le permet.
pub fn garder_visible(place: Place, travail: Rect, minimum: [i32; 2]) -> Place {
    let z = travail.map(i64::from);
    let (zg, zh) = (z[0], z[1]);
    let (zd, zb) = (z[2].max(zg + 1), z[3].max(zh + 1));
    let p = place.map(i64::from);
    let largeur = p[2].max(i64::from(minimum[0])).clamp(1, zd - zg);
    let hauteur = p[3].max(i64::from(minimum[1])).clamp(1, zb - zh);
    let x = p[0].clamp(zg, zd - largeur);
    let y = p[1].clamp(zh, zb - hauteur);
    let vers_i32 = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    [vers_i32(x), vers_i32(y), vers_i32(largeur), vers_i32(hauteur)]
}

/// Le centre d'un rectangle (gauche, haut, droite, bas), sans débordement, juste aussi avec
/// des coordonnées négatives (un écran au-dessus ou à gauche de l'écran principal).
pub fn centre_de(r: Rect) -> [i32; 2] {
    let c = |a: i32, b: i32| ((i64::from(a) + i64::from(b)).div_euclid(2)) as i32;
    [c(r[0], r[2]), c(r[1], r[3])]
}

/// L'écran (son rang dans `ecrans`, rectangles entiers en pixels) qui contient la plus
/// grande part de `r`, comme `MonitorFromRect` de Windows ; `None` si `r` ne touche aucun
/// écran.
pub fn ecran_de(r: Rect, ecrans: &[Rect]) -> Option<usize> {
    let aire = |e: &Rect| {
        let l = i64::from(r[2].min(e[2])) - i64::from(r[0].max(e[0]));
        let h = i64::from(r[3].min(e[3])) - i64::from(r[1].max(e[1]));
        // Chaque côté tient sur 32 bits : leur produit tient sur 64 bits sans signe.
        if l > 0 && h > 0 { l as u64 * h as u64 } else { 0 }
    };
    let (i, a) = ecrans
        .iter()
        .enumerate()
        .map(|(i, e)| (i, aire(e)))
        .max_by_key(|&(i, a)| (a, std::cmp::Reverse(i)))?;
    (a > 0).then_some(i)
}

/// Où poser la fenêtre quand Windows change son échelle (message `WM_DPICHANGED`), en
/// pixels : le déplacement `demande` (gauche, haut, largeur, hauteur) que winit fait en
/// réponse est remplacé par le rectangle `suggere` (gauche, haut, droite, bas) que Windows
/// joint au message. Les drapeaux du déplacement sont respectés : `sans_place` (`SWP_NOMOVE`)
/// garde la place demandée, `sans_taille` (`SWP_NOSIZE`) la taille demandée.
///
/// Pourquoi : Windows annonce l'échelle de l'écran **vers lequel** la fenêtre part, et pose
/// `suggere` sur cet écran, le curseur à la même place relative. winit 0.30 calcule son propre
/// rectangle, puis le « repousse » tant qu'il n'est pas sur l'écran que donne
/// `MonitorFromWindow` ; or, pendant un glissement, celui-ci répond l'écran que la fenêtre
/// **quitte** (elle y est encore surtout). La fenêtre revient donc sur l'ancien écran avec la
/// nouvelle échelle, Windows rechange l'échelle, et ainsi de suite : elle saute d'un écran à
/// l'autre. Correctif repris de winit (#4341, pour Windows 11 ; pas encore dans la 0.30).
pub fn place_au_changement_d_echelle(
    demande: Place,
    sans_place: bool,
    sans_taille: bool,
    suggere: Rect,
) -> Place {
    let largeur = i64::from(suggere[2]) - i64::from(suggere[0]);
    let hauteur = i64::from(suggere[3]) - i64::from(suggere[1]);
    if largeur < 1 || hauteur < 1 {
        // Un rectangle vide : rien de sûr à poser, winit garde la main.
        return demande;
    }
    let vers_i32 = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    let [x, y] = if sans_place {
        [demande[0], demande[1]]
    } else {
        [suggere[0], suggere[1]]
    };
    let [l, h] = if sans_taille {
        [demande[2], demande[3]]
    } else {
        [vers_i32(largeur), vers_i32(hauteur)]
    };
    [x, y, l, h]
}

/// Le coin (gauche, haut) d'une fenêtre de `taille` (pixels) centrée dans la zone de
/// travail ; une fenêtre plus grande que la zone part de son coin.
pub fn centrer(taille: [i32; 2], travail: Rect) -> [i32; 2] {
    let z = travail.map(i64::from);
    let t = taille.map(i64::from);
    let x = (z[0] + (z[2] - z[0] - t[0]) / 2).max(z[0]);
    let y = (z[1] + (z[3] - z[1] - t[1]) / 2).max(z[1]);
    let vers_i32 = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    [vers_i32(x), vers_i32(y)]
}
