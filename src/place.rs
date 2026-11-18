//! Où ouvrir le volet et le petit menu de l'icône, et où rouvrir les Paramètres. Comme les
//! menus de Windows 11 : collé à la barre des tâches, en face de l'icône, en entier sur son
//! écran. Le côté de la barre se déduit de la différence entre l'écran entier et sa zone de
//! travail. Du calcul pur, testé partout.

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

/// Une place de fenêtre retenue : gauche, haut, largeur, hauteur, en pixels.
pub type Place = [i32; 4];

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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Une place retenue sur un écran débranché revient en entier sur l'écran qui reste ;
    /// une fenêtre réduite n'est pas retenue.
    #[test]
    fn place_des_parametres() {
        let travail = [0, 0, 1920, 1040];
        assert_eq!(
            garder_visible([2600, 300, 940, 660], travail, [760, 480]),
            [980, 300, 940, 660]
        );
        assert_eq!(
            garder_visible([-500, -80, 940, 660], travail, [760, 480]),
            [0, 0, 940, 660]
        );
        // Plus grande que l'écran : rapetissée.
        assert_eq!(
            garder_visible([10, 10, 4000, 3000], travail, [760, 480]),
            [0, 0, 1920, 1040]
        );
        // Plus petite que le minimum : agrandie.
        assert_eq!(
            garder_visible([10, 10, 300, 200], travail, [760, 480]),
            [10, 10, 760, 480]
        );
        assert_eq!(place_valable([-32_000, -32_000, 160, 28]), None);
        assert_eq!(place_valable([100, 80, 940, 660]), Some([100, 80, 940, 660]));
    }

    /// Le petit menu : au-dessus du clic quand la barre est en bas, vers la droite, et
    /// retourné vers la gauche contre le bord de l'écran.
    #[test]
    fn petit_menu_au_point() {
        let travail = [0, 0, 2560, 1380];
        let icone = [2200, 1392, 2232, 1428];
        let r = placer_au_point([2210, 1410], icone, [240, 200], ECRAN, travail, 8);
        assert_eq!(r, [2210, 1380 - 8 - 200, 240, 200]);
        let r = placer_au_point([2500, 1410], icone, [240, 200], ECRAN, travail, 8);
        assert_eq!(r[0] + r[2], 2500);
        // Barre en haut : sous le clic.
        let haut = [0, 60, 2560, 1440];
        let r = placer_au_point([2210, 30], [2200, 10, 2232, 50], [240, 200], ECRAN, haut, 8);
        assert_eq!(r[1], 68);
    }

    /// Un écran de 2560 × 1440 avec une barre de 60 px.
    const ECRAN: Rect = [0, 0, 2560, 1440];

    fn dedans(r: Rect, t: Rect) -> bool {
        r[0] >= t[0] && r[1] >= t[1] && r[0] + r[2] <= t[2] && r[1] + r[3] <= t[3]
    }

    #[test]
    fn barre_en_bas() {
        let travail = [0, 0, 2560, 1380];
        let r = placer([2200, 1392, 2232, 1428], [360, 420], ECRAN, travail, 10);
        assert_eq!(r, [2216 - 180, 1380 - 10 - 420, 360, 420]);
        // Icône tout à droite : la fenêtre reste entière.
        let r = placer([2530, 1392, 2556, 1428], [360, 420], ECRAN, travail, 10);
        assert_eq!(r[0] + r[2], 2560 - 10);
    }

    #[test]
    fn les_autres_cotes() {
        let haut = [0, 60, 2560, 1440];
        assert_eq!(cote_de_la_barre([2200, 10, 2232, 50], ECRAN, haut), Cote::Haut);
        assert_eq!(placer([2200, 10, 2232, 50], [360, 420], ECRAN, haut, 10)[1], 70);
        let gauche = [70, 0, 2560, 1440];
        let r = placer([20, 1300, 50, 1330], [360, 420], ECRAN, gauche, 10);
        assert_eq!((r[0], r[1] + r[3]), (80, 1330));
        let droite = [0, 0, 2490, 1440];
        let r = placer([2510, 1300, 2540, 1330], [360, 420], ECRAN, droite, 10);
        assert_eq!(r[0] + r[2], 2480);
    }

    /// Barre masquée : le côté vient de la place de l'icône.
    #[test]
    fn barre_masquee() {
        assert_eq!(
            cote_de_la_barre([1200, 1420, 1230, 1440], ECRAN, ECRAN),
            Cote::Bas
        );
        assert_eq!(cote_de_la_barre([0, 700, 20, 730], ECRAN, ECRAN), Cote::Gauche);
    }

    /// Un écran à gauche de l'écran principal (coordonnées négatives), en 4K.
    #[test]
    fn ecran_a_gauche_en_4k() {
        let ecran = [-3840, -200, 0, 1960];
        let travail = [-3840, -200, 0, 1880];
        let r = placer([-400, 1890, -360, 1950], [720, 840], ecran, travail, 20);
        assert!(dedans(r, travail));
        assert_eq!(r[1] + r[3], 1860);
    }

    #[test]
    fn trop_grand_rapetisse() {
        let travail = [0, 0, 1024, 728];
        let r = placer([900, 735, 930, 765], [360, 5000], [0, 0, 1024, 768], travail, 10);
        assert_eq!(r[3], 708);
        assert!(dedans(r, travail));
    }

    proptest! {
        #[test]
        fn toujours_dans_la_zone(icone in any::<[i32; 4]>(), taille in any::<[i32; 2]>(),
                                 g in -50_000..50_000i32, h in -50_000..50_000i32,
                                 l in 300..8_000i32, ht in 300..8_000i32, barre in 0..80i32, marge in any::<i32>()) {
            let ecran = [g, h, g + l, h + ht];
            let travail = [g, h, g + l, h + ht - barre];
            let r = placer(icone, taille, ecran, travail, marge);
            prop_assert!(r[2] >= 1 && r[3] >= 1);
            prop_assert!(dedans(r, travail));
        }

        #[test]
        fn petit_menu_toujours_dans_la_zone(point in any::<[i32; 2]>(), icone in any::<[i32; 4]>(),
                                            taille in any::<[i32; 2]>(),
                                            g in -50_000..50_000i32, h in -50_000..50_000i32,
                                            l in 300..8_000i32, ht in 300..8_000i32, barre in 0..80i32, marge in any::<i32>()) {
            let ecran = [g, h, g + l, h + ht];
            let travail = [g, h, g + l, h + ht - barre];
            let r = placer_au_point(point, icone, taille, ecran, travail, marge);
            prop_assert!(r[2] >= 1 && r[3] >= 1);
            prop_assert!(dedans(r, travail));
        }

        #[test]
        fn valeurs_absurdes(icone in any::<[i32; 4]>(), taille in any::<[i32; 2]>(),
                            e in any::<[i32; 4]>(), t in any::<[i32; 4]>(), marge in any::<i32>()) {
            let _ = placer(icone, taille, e, t, marge);
        }
    }

    proptest! {
        #[test]
        fn place_retenue_toujours_visible(place in any::<[i32; 4]>(), minimum in any::<[i32; 2]>(),
                                          g in -50_000..50_000i32, h in -50_000..50_000i32,
                                          l in 300..8_000i32, ht in 300..8_000i32) {
            let travail = [g, h, g + l, h + ht];
            let r = garder_visible(place, travail, minimum);
            prop_assert!(r[2] >= 1 && r[3] >= 1);
            prop_assert!(dedans(r, travail));
        }
    }

    /// Un portable (écran principal, 1920 × 1200 à 150 %) sous un écran plus large
    /// (2560 × 1440 à 100 %), décalé vers la gauche : l'écran du haut a un y négatif.
    const BAS: Rect = [0, 0, 1920, 1200];
    const HAUT: Rect = [-320, -1440, 2240, 0];
    const ECRANS: [Rect; 2] = [BAS, HAUT];
    /// La zone de travail de l'écran du haut (barre des tâches de 48 px en bas).
    const TRAVAIL_HAUT: Rect = [-320, -1440, 2240, -48];

    fn rect_de(p: Place) -> Rect {
        [p[0], p[1], p[0] + p[2], p[1] + p[3]]
    }

    #[test]
    fn ecran_d_un_rectangle() {
        assert_eq!(ecran_de([100, -700, 1100, 0], &ECRANS), Some(1));
        assert_eq!(ecran_de([100, -300, 1100, 500], &ECRANS), Some(0));
        assert_eq!(ecran_de([5000, 5000, 5100, 5100], &ECRANS), None);
        assert_eq!(centre_de([533, -498, 1483, 172]), [1008, -163]);
        assert_eq!(centre_de([-321, -1441, -320, -1440]), [-321, -1441]);
        assert_eq!(centre_de([i32::MIN, i32::MIN, i32::MAX, i32::MAX]), [-1, -1]);
    }

    /// Le défaut : les Paramètres glissés du portable (150 %) vers l'écran du haut (100 %),
    /// tenus par leur barre de titre. Fenêtre de 1425 × 1005 px, curseur à (700, 15) dans la
    /// fenêtre. Windows change l'échelle quand la fenêtre passe surtout sur l'écran du haut :
    /// le curseur est en (1000, −488), la fenêtre encore surtout en bas
    /// ([300, −495, 1725, 510] : `MonitorFromWindow` répond « en bas »). Windows suggère
    /// 950 × 670 px (2/3), le curseur toujours à la même place relative. winit 0.30 prend ce
    /// coin avec sa propre taille (952 × 668), voit que ce n'est pas l'écran que donne
    /// `MonitorFromWindow` et descend la fenêtre pixel par pixel jusqu'à ce qu'elle soit surtout
    /// en bas : 165 px sous le curseur, sur le portable mais à l'échelle de l'écran du haut ;
    /// Windows rechange l'échelle, et la fenêtre saute.
    #[test]
    fn glisser_vers_l_ecran_du_haut() {
        let suggere = [533, -498, 1483, 172];
        assert_eq!(ecran_de(suggere, &ECRANS), Some(1), "Windows annonce le haut");
        let winit = [533, -333, 952, 668];
        assert_eq!(
            ecran_de(rect_de(winit), &ECRANS),
            Some(0),
            "winit 0.30 ramenait la fenêtre sur le portable"
        );
        let r = place_au_changement_d_echelle(winit, false, false, suggere);
        assert_eq!(r, [533, -498, 950, 670]);
        assert_eq!(ecran_de(rect_de(r), &ECRANS), Some(1), "elle reste en haut");
        // Le curseur reste dans la barre de titre, à la même place relative.
        let curseur = [1000, -488];
        assert!(r[1] <= curseur[1] && curseur[1] < r[1] + 20);
    }

    /// Et le retour : de l'écran du haut (100 %, 950 × 670) vers le portable (150 %). Le
    /// curseur, à (466, 10) dans la fenêtre, est en (1000, −324) quand la fenêtre passe
    /// surtout sur le portable (elle était encore surtout en haut, en [534, −342, 1484, 328]).
    /// Windows suggère 1425 × 1005, surtout sur le portable ; winit 0.30 la remontait pixel
    /// par pixel jusqu'à ce qu'elle soit surtout en haut, à l'échelle du portable.
    #[test]
    fn glisser_vers_le_portable() {
        let suggere = [301, -339, 1726, 666];
        assert_eq!(ecran_de(suggere, &ECRANS), Some(0));
        let winit = [301, -502, 1428, 1002];
        assert_eq!(ecran_de(rect_de(winit), &ECRANS), Some(1));
        let r = place_au_changement_d_echelle(winit, false, false, suggere);
        assert_eq!(ecran_de(rect_de(r), &ECRANS), Some(0));
        assert_eq!(r, [301, -339, 1425, 1005]);
    }

    /// Les drapeaux du déplacement sont respectés, et un rectangle vide ne change rien.
    #[test]
    fn changement_d_echelle_drapeaux() {
        let suggere = [533, -498, 1483, 172];
        let demande = [10, 20, 300, 400];
        assert_eq!(
            place_au_changement_d_echelle(demande, true, false, suggere),
            [10, 20, 950, 670]
        );
        assert_eq!(
            place_au_changement_d_echelle(demande, false, true, suggere),
            [533, -498, 300, 400]
        );
        assert_eq!(
            place_au_changement_d_echelle(demande, true, true, suggere),
            demande
        );
        assert_eq!(
            place_au_changement_d_echelle(demande, false, false, [5, 5, 5, 90]),
            demande
        );
    }

    /// La place retenue sur l'écran du haut (y négatif) est gardée telle quelle à la
    /// réouverture : valable, sur cet écran, et déjà en entier dans sa zone de travail.
    #[test]
    fn place_retenue_sur_l_ecran_du_haut() {
        let fermee = [533, -1200, 950, 670];
        let p = place_valable(fermee).expect("place valable");
        assert_eq!(p, fermee);
        let centre = centre_de(rect_de(p));
        assert!(centre[1] < 0, "le centre est sur l'écran du haut");
        assert_eq!(ecran_de(rect_de(p), &ECRANS), Some(1));
        assert_eq!(garder_visible(p, TRAVAIL_HAUT, [760, 480]), fermee);
        // Collée en haut de l'écran du haut : rien ne bouge non plus.
        let collee = [-320, -1440, 950, 670];
        assert_eq!(garder_visible(collee, TRAVAIL_HAUT, [760, 480]), collee);
    }

    proptest! {
        /// Le rectangle suggéré est toujours suivi, sauf ce que les drapeaux interdisent.
        #[test]
        fn changement_d_echelle_suit_windows(demande in any::<[i32; 4]>(),
                                             g in -40_000..40_000i32, h in -40_000..40_000i32,
                                             l in 1..10_000i32, ht in 1..10_000i32,
                                             sans_place in any::<bool>(), sans_taille in any::<bool>()) {
            let suggere = [g, h, g + l, h + ht];
            let r = place_au_changement_d_echelle(demande, sans_place, sans_taille, suggere);
            let place = if sans_place { [demande[0], demande[1]] } else { [g, h] };
            let taille = if sans_taille { [demande[2], demande[3]] } else { [l, ht] };
            prop_assert_eq!(r, [place[0], place[1], taille[0], taille[1]]);
        }

        #[test]
        fn changement_d_echelle_valeurs_absurdes(demande in any::<[i32; 4]>(), suggere in any::<[i32; 4]>(),
                                                 a in any::<bool>(), b in any::<bool>()) {
            let _ = place_au_changement_d_echelle(demande, a, b, suggere);
            let _ = ecran_de(suggere, &ECRANS);
            let _ = centre_de(suggere);
        }
    }

    #[test]
    fn centrer_dans_la_zone() {
        assert_eq!(centrer([200, 100], [0, 0, 1000, 500]), [400, 200]);
        assert_eq!(centrer([200, 100], [-1920, 0, -920, 500]), [-1520, 200]);
        assert_eq!(centrer([3000, 900], [0, 0, 1000, 500]), [0, 0]);
    }
}
