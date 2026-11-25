//! La palette des pastilles : couleurs distinctes à l'œil (écart ΔE2000), lettre lisible,
//! visibles sur le fond clair comme sombre, loin des couleurs d'état et de l'accent, et les
//! six premiers rangs inchangés. L'écart ΔE2000 est écrit ici, en Rust seul, et contrôlé sur
//! les paires de référence publiées par Sharma, Wu et Dalal (2005).

use super::{ChoixWindows, Palette, Rgb, contraste, couleur_lettre, couleurs_pastilles};

/// L'écart le plus petit permis entre deux couleurs de la palette (ΔE2000).
const ECART_MIN: f64 = 12.0;

/// Une couleur sRGB en CIELAB (illuminant D65, observateur 2°).
fn lab(c: Rgb) -> [f64; 3] {
    let lin = |v: u8| {
        let v = f64::from(v) / 255.0;
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(c.0), lin(c.1), lin(c.2));
    let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
    let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175_0 * b;
    let z = 0.019_333_9 * r + 0.119_192_0 * g + 0.950_304_1 * b;
    let f = |t: f64| {
        if t > 216.0 / 24_389.0 {
            t.cbrt()
        } else {
            (24_389.0 / 27.0 * t + 16.0) / 116.0
        }
    };
    let (fx, fy, fz) = (f(x / 0.950_47), f(y), f(z / 1.088_83));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// L'écart de couleur CIEDE2000 entre deux couleurs CIELAB (kL = kC = kH = 1), comme le
/// décrivent Sharma, Wu et Dalal (2005) : environ 1 = juste visible côte à côte.
fn delta_e_2000(a: [f64; 3], b: [f64; 3]) -> f64 {
    let [l1, a1, b1] = a;
    let [l2, a2, b2] = b;
    let c_moy = (a1.hypot(b1) + a2.hypot(b2)) / 2.0;
    let septieme = |c: f64| (c.powi(7) / (c.powi(7) + 25f64.powi(7))).sqrt();
    let g = 0.5 * (1.0 - septieme(c_moy));
    let (a1p, a2p) = ((1.0 + g) * a1, (1.0 + g) * a2);
    let (c1p, c2p) = (a1p.hypot(b1), a2p.hypot(b2));
    // Les teintes en degrés, de 0 à 360 (0 pour un gris).
    let teinte = |b: f64, a: f64| {
        if b == 0.0 && a == 0.0 {
            0.0
        } else {
            b.atan2(a).to_degrees().rem_euclid(360.0)
        }
    };
    let (h1, h2) = (teinte(b1, a1p), teinte(b2, a2p));
    let gris = c1p * c2p == 0.0;
    let dh = if gris {
        0.0
    } else if h2 - h1 > 180.0 {
        h2 - h1 - 360.0
    } else if h2 - h1 < -180.0 {
        h2 - h1 + 360.0
    } else {
        h2 - h1
    };
    let d_l = l2 - l1;
    let d_c = c2p - c1p;
    let d_h = 2.0 * (c1p * c2p).sqrt() * (dh / 2.0).to_radians().sin();
    let l_moy = (l1 + l2) / 2.0;
    let cp_moy = (c1p + c2p) / 2.0;
    let h_moy = if gris {
        h1 + h2
    } else if (h1 - h2).abs() <= 180.0 {
        (h1 + h2) / 2.0
    } else if h1 + h2 < 360.0 {
        (h1 + h2 + 360.0) / 2.0
    } else {
        (h1 + h2 - 360.0) / 2.0
    };
    let cos = |deg: f64| deg.to_radians().cos();
    let t = 1.0 - 0.17 * cos(h_moy - 30.0) + 0.24 * cos(2.0 * h_moy) + 0.32 * cos(3.0 * h_moy + 6.0)
        - 0.20 * cos(4.0 * h_moy - 63.0);
    let d_theta = 30.0 * (-((h_moy - 275.0) / 25.0).powi(2)).exp();
    let r_c = 2.0 * septieme(cp_moy);
    let s_l = 1.0 + 0.015 * (l_moy - 50.0).powi(2) / (20.0 + (l_moy - 50.0).powi(2)).sqrt();
    let s_c = 1.0 + 0.045 * cp_moy;
    let s_h = 1.0 + 0.015 * cp_moy * t;
    let r_t = -(2.0 * d_theta).to_radians().sin() * r_c;
    let (ql, qc, qh) = (d_l / s_l, d_c / s_c, d_h / s_h);
    (ql * ql + qc * qc + qh * qh + r_t * qc * qh).sqrt()
}

/// Les 34 paires de référence de Sharma, Wu et Dalal (2005) : L*, a*, b* de chaque couleur,
/// puis l'écart publié (arrondi à 4 décimales).
const SHARMA: [[f64; 7]; 34] = [
    [50.0, 2.6772, -79.7751, 50.0, 0.0, -82.7485, 2.0425],
    [50.0, 3.1571, -77.2803, 50.0, 0.0, -82.7485, 2.8615],
    [50.0, 2.8361, -74.0200, 50.0, 0.0, -82.7485, 3.4412],
    [50.0, -1.3802, -84.2814, 50.0, 0.0, -82.7485, 1.0000],
    [50.0, -1.1848, -84.8006, 50.0, 0.0, -82.7485, 1.0000],
    [50.0, -0.9009, -85.5211, 50.0, 0.0, -82.7485, 1.0000],
    [50.0, 0.0, 0.0, 50.0, -1.0, 2.0, 2.3669],
    [50.0, -1.0, 2.0, 50.0, 0.0, 0.0, 2.3669],
    [50.0, 2.49, -0.001, 50.0, -2.49, 0.0009, 7.1792],
    [50.0, 2.49, -0.001, 50.0, -2.49, 0.0010, 7.1792],
    [50.0, 2.49, -0.001, 50.0, -2.49, 0.0011, 7.2195],
    [50.0, 2.49, -0.001, 50.0, -2.49, 0.0012, 7.2195],
    [50.0, -0.001, 2.49, 50.0, 0.0009, -2.49, 4.8045],
    [50.0, -0.001, 2.49, 50.0, 0.0010, -2.49, 4.8045],
    [50.0, -0.001, 2.49, 50.0, 0.0011, -2.49, 4.7461],
    [50.0, 2.5, 0.0, 50.0, 0.0, -2.5, 4.3065],
    [50.0, 2.5, 0.0, 73.0, 25.0, -18.0, 27.1492],
    [50.0, 2.5, 0.0, 61.0, -5.0, 29.0, 22.8977],
    [50.0, 2.5, 0.0, 56.0, -27.0, -3.0, 31.9030],
    [50.0, 2.5, 0.0, 58.0, 24.0, 15.0, 19.4535],
    [50.0, 2.5, 0.0, 50.0, 3.1736, 0.5854, 1.0000],
    [50.0, 2.5, 0.0, 50.0, 3.2972, 0.0, 1.0000],
    [50.0, 2.5, 0.0, 50.0, 1.8634, 0.5757, 1.0000],
    [50.0, 2.5, 0.0, 50.0, 3.2592, 0.3350, 1.0000],
    [60.2574, -34.0099, 36.2677, 60.4626, -34.1751, 39.4387, 1.2644],
    [63.0109, -31.0961, -5.8663, 62.8187, -29.7946, -4.0864, 1.2630],
    [61.2901, 3.7196, -5.3901, 61.4292, 2.2480, -4.9620, 1.8731],
    [35.0831, -44.1164, 3.7933, 35.0232, -40.0716, 1.5901, 1.8645],
    [22.7233, 20.0904, -46.6940, 23.0331, 14.9730, -42.5619, 2.0373],
    [36.4612, 47.8580, 18.3852, 36.2715, 50.5065, 21.2231, 1.4146],
    [90.8027, -2.0831, 1.4410, 91.1528, -1.6435, 0.0447, 1.4441],
    [90.9257, -0.5406, -0.9208, 88.6381, -0.8985, -0.7239, 1.5381],
    [6.7747, -0.2908, -2.4247, 5.8714, -0.0985, -2.2286, 0.6377],
    [2.0776, 0.0795, -1.1350, 0.9033, -0.0636, -0.5514, 0.9082],
];

/// ΔE2000 rend les 34 écarts publiés, dans les deux sens (l'écart est symétrique).
#[test]
fn delta_e_2000_donne_les_ecarts_de_sharma() {
    for [l1, a1, b1, l2, a2, b2, attendu] in SHARMA {
        for (x, y) in [([l1, a1, b1], [l2, a2, b2]), ([l2, a2, b2], [l1, a1, b1])] {
            let e = delta_e_2000(x, y);
            assert!(
                (e - attendu).abs() < 1e-4,
                "{x:?} {y:?} : {e} au lieu de {attendu}"
            );
        }
    }
    assert_eq!(delta_e_2000([50.0, 10.0, -20.0], [50.0, 10.0, -20.0]), 0.0);
}

/// Le passage en CIELAB : le blanc, le noir et le rouge pur (valeurs connues).
#[test]
fn lab_des_couleurs_connues() {
    let proche = |a: [f64; 3], b: [f64; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
    assert!(
        proche(lab(Rgb(255, 255, 255)), [100.0, 0.0, 0.0]),
        "{:?}",
        lab(Rgb(255, 255, 255))
    );
    assert!(proche(lab(Rgb(0, 0, 0)), [0.0, 0.0, 0.0]));
    assert!(
        proche(lab(Rgb(255, 0, 0)), [53.24, 80.09, 67.20]),
        "{:?}",
        lab(Rgb(255, 0, 0))
    );
}

/// Les six premiers rangs gardent leur couleur : un rang déjà enregistré par un logiciel ne
/// change pas de couleur.
#[test]
fn les_six_premiers_rangs_ne_changent_pas() {
    let six = [0x7B57C9, 0xB03A9C, 0x1D7F86, 0x94604F, 0x667A94, 0x857A2A].map(Rgb::hex);
    assert_eq!(couleurs_pastilles()[..6], six);
}

/// Toute la palette : distincte (ΔE2000 d'au moins 12 entre toutes les paires), lettre lisible
/// (4,5), visible sur le fond clair comme sombre (3), et loin du vert, de l'orange, du rouge
/// et de l'accent bleu de Windows.
#[test]
fn palette_distincte_et_lisible() {
    let c = couleurs_pastilles();
    for (i, a) in c.iter().enumerate() {
        assert!(!c[..i].contains(a), "{a:?} en double");
        assert!(
            contraste(*a, couleur_lettre(*a)) >= 4.5,
            "{a:?} : lettre illisible"
        );
        for sombre in [false, true] {
            let p = Palette::de(&ChoixWindows {
                sombre,
                ..Default::default()
            });
            assert!(
                contraste(*a, p.fond) >= 3.0,
                "{a:?} sur le fond (sombre : {sombre})"
            );
            for etat in [p.vert, p.orange, p.rouge, p.accent] {
                let d = (i32::from(a.0) - i32::from(etat.0)).abs()
                    + (i32::from(a.1) - i32::from(etat.1)).abs()
                    + (i32::from(a.2) - i32::from(etat.2)).abs();
                assert!(d > 90, "{a:?} trop près de {etat:?}");
            }
        }
        for b in &c[..i] {
            let e = delta_e_2000(lab(*a), lab(*b));
            assert!(e >= ECART_MIN, "{a:?} et {b:?} trop proches : ΔE2000 = {e:.2}");
        }
    }
}
