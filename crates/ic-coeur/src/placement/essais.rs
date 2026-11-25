//! Les essais du placement (venus de `interface_commune::place`, bloc D).

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
    let propose = [533, -333, 952, 668];
    assert_eq!(
        ecran_de(rect_de(propose), &ECRANS),
        Some(0),
        "la bibliothèque de fenêtres 0.30 ramenait la fenêtre sur le portable"
    );
    let r = place_au_changement_d_echelle(propose, false, false, suggere);
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
    let propose = [301, -502, 1428, 1002];
    assert_eq!(ecran_de(rect_de(propose), &ECRANS), Some(1));
    let r = place_au_changement_d_echelle(propose, false, false, suggere);
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

/// Contre l'icône : au-dessus d'elle quand la barre est en bas, centré sur elle ; à sa
/// droite quand la barre est à gauche ; au-dessus ou dessous une icône cachée sous la
/// flèche (dans la zone de travail) ; toujours en entier dans la zone.
#[test]
fn contre_l_icone() {
    // Barre en bas (zone de travail jusqu'à 1032), icône à droite.
    let travail = [0, 0, 1920, 1032];
    let icone = [1700, 1040, 1724, 1072];
    assert_eq!(
        placer_contre_l_icone(icone, [360, 400], travail, 8),
        [1712 - 180, 1032 - 8 - 400, 360, 400]
    );
    // Contre le bord droit : ramené dans la zone.
    let r = placer_contre_l_icone([1900, 1040, 1916, 1072], [360, 400], travail, 8);
    assert_eq!(r[0], 1920 - 8 - 360);
    // Barre à gauche (zone de travail à partir de 48) : à droite de l'icône, bas aligné.
    let gauche = [48, 0, 1920, 1080];
    assert_eq!(
        placer_contre_l_icone([8, 900, 40, 932], [360, 400], gauche, 8),
        [48 + 8, 932 - 400, 360, 400]
    );
    // Barre à droite : à gauche de l'icône.
    let droite = [0, 0, 1872, 1080];
    assert_eq!(
        placer_contre_l_icone([1880, 900, 1912, 932], [360, 400], droite, 8),
        [1872 - 8 - 360, 932 - 400, 360, 400]
    );
    // Barre en haut : dessous.
    let haut = [0, 48, 1920, 1080];
    assert_eq!(
        placer_contre_l_icone([1700, 8, 1724, 40], [360, 400], haut, 8)[1],
        48 + 8
    );
    // Icône dans la zone de travail (sous la flèche) : au-dessus s'il y a la place…
    assert_eq!(
        placer_contre_l_icone([1500, 900, 1524, 924], [360, 400], travail, 8)[1],
        900 - 8 - 400
    );
    // … sinon dessous.
    assert_eq!(
        placer_contre_l_icone([1500, 100, 1524, 124], [360, 400], travail, 8)[1],
        132
    );
    // Plus haut que la zone : rapetissé.
    assert_eq!(
        placer_contre_l_icone(icone, [360, 5000], travail, 8)[3],
        1032 - 16
    );
}

/// Le placement par défaut est celui d'avant les options (collé à la barre, 12 pixels) ;
/// l'écart choisi est en points, converti à l'échelle de l'écran.
#[test]
fn placement_du_volet() {
    let travail = [0, 0, 2560, 1380];
    let icone = [2200, 1392, 2232, 1428];
    let defaut = PlacementVolet::default();
    assert_eq!(defaut.ecart_px(1.5), ECART_VOLET);
    assert_eq!(
        defaut.placer(icone, [480, 600], ECRAN, travail, 1.5),
        placer(icone, [480, 600], ECRAN, travail, 12)
    );
    assert_eq!(defaut.hauteur_max_points(travail, 1.5), None);
    let pp = PlacementVolet {
        contre_l_icone: true,
        ecart: Some(8.0),
        hauteur_max: Some(600.0),
    };
    assert_eq!(pp.ecart_px(1.5), 12);
    assert_eq!(pp.ecart_px(2.0), 16);
    assert_eq!(
        pp.placer(icone, [480, 600], ECRAN, travail, 2.0),
        placer_contre_l_icone(icone, [480, 600], travail, 16)
    );
    // 600 points tiennent sur un écran de 1380 pixels à l'échelle 1,5 (≈ 900 points)…
    assert_eq!(pp.hauteur_max_points(travail, 1.5), Some(600.0));
    // … pas sur un petit écran : bornés à la zone moins les écarts.
    let petit = [0, 0, 1366, 728];
    assert_eq!(pp.hauteur_max_points(petit, 1.5), Some((728.0 - 24.0) / 1.5));
    // Valeurs absurdes : rien ne casse.
    let absurde = PlacementVolet {
        contre_l_icone: false,
        ecart: Some(f32::NAN),
        hauteur_max: Some(f32::INFINITY),
    };
    assert_eq!(absurde.ecart_px(f32::NAN), ECART_VOLET);
    assert_eq!(absurde.hauteur_max_points(travail, 0.0), None);
    let enorme = PlacementVolet {
        ecart: Some(1e9),
        hauteur_max: Some(-5.0),
        ..Default::default()
    };
    assert_eq!(enorme.ecart_px(1.0), 48);
    assert_eq!(enorme.hauteur_max_points(travail, 1.0), Some(1.0));
}

/// La place est retenue 1 s après le dernier mouvement, pas pendant ; la première place vue
/// n'est pas un mouvement ; une place inconnue ne change rien.
#[test]
fn place_retenue_apres_un_deplacement() {
    let t0 = std::time::Duration::from_secs(1_000);
    let ms = |n: u64| t0 + std::time::Duration::from_millis(n);
    let mut s = SuiviDePlace::<std::time::Duration>::default();
    assert_eq!(s.voir(None, t0), Retenue::Rien);
    assert_eq!(s.voir(Some([10, 10, 900, 600]), ms(10)), Retenue::Rien);
    assert_eq!(s.voir(Some([10, 10, 900, 600]), ms(500)), Retenue::Rien);
    // Elle bouge, puis encore : l'attente repart du dernier mouvement.
    assert_eq!(
        s.voir(Some([20, 10, 900, 600]), ms(600)),
        Retenue::Attendre(RETENIR_APRES)
    );
    assert_eq!(
        s.voir(Some([40, 12, 900, 600]), ms(900)),
        Retenue::Attendre(RETENIR_APRES)
    );
    assert_eq!(
        s.voir(None, ms(1_400)),
        Retenue::Attendre(std::time::Duration::from_millis(500))
    );
    assert_eq!(s.voir(Some([40, 12, 900, 600]), ms(1_900)), Retenue::Retenir);
    // Retenue une seule fois.
    assert_eq!(s.voir(Some([40, 12, 900, 600]), ms(3_000)), Retenue::Rien);
}

proptest! {
    /// Contre l'icône : toujours en entier dans la zone de travail (s'il y tient), quelles
    /// que soient l'icône et la taille.
    #[test]
    fn contre_l_icone_toujours_dans_la_zone(ig in -5_000..5_000i32, ih in -5_000..5_000i32,
                                            l in 1..3_000i32, h in 1..3_000i32, m in 0..48i32) {
        let travail = [0, 0, 1920, 1032];
        let r = placer_contre_l_icone([ig, ih, ig + 24, ih + 24], [l, h], travail, m);
        prop_assert!(r[0] >= m && r[1] >= m);
        prop_assert!(r[0] + r[2] <= 1920 - m && r[1] + r[3] <= 1032 - m);
    }

    #[test]
    fn contre_l_icone_valeurs_absurdes(icone in any::<[i32; 4]>(), taille in any::<[i32; 2]>(),
                                       travail in any::<[i32; 4]>(), m in any::<i32>()) {
        let _ = placer_contre_l_icone(icone, taille, travail, m);
    }
}
