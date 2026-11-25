//! Les places dessinées en points (nombres à virgule) : le plan des écrans dans une ligne de
//! pastilles, et la place d'un menu de choix sous (ou sur) sa case. Le rectangle
//! [`Rectangle`] fait les mêmes calculs que celui d'egui, dans le même ordre : la
//! bibliothèque (`interface_commune::pastilles` et `menu_choix`) convertit à l'entrée et à la
//! sortie, et le dessin reste identique au point près.

/// Un rectangle en points : son coin haut gauche `min` et son coin bas droit `max`
/// (`[x, y]`). Une largeur ou une hauteur peut être négative (rectangle vide).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Rectangle {
    /// Le rectangle qui ne contient rien (l'opposé de tout l'espace).
    pub const RIEN: Rectangle = Rectangle {
        min: [f32::INFINITY, f32::INFINITY],
        max: [f32::NEG_INFINITY, f32::NEG_INFINITY],
    };

    pub fn de_min_max(min: [f32; 2], max: [f32; 2]) -> Rectangle {
        Rectangle { min, max }
    }

    pub fn de_min_taille(min: [f32; 2], taille: [f32; 2]) -> Rectangle {
        Rectangle {
            min,
            max: [min[0] + taille[0], min[1] + taille[1]],
        }
    }

    pub fn largeur(&self) -> f32 {
        self.max[0] - self.min[0]
    }

    pub fn hauteur(&self) -> f32 {
        self.max[1] - self.min[1]
    }

    pub fn taille(&self) -> [f32; 2] {
        [self.largeur(), self.hauteur()]
    }

    pub fn gauche(&self) -> f32 {
        self.min[0]
    }

    pub fn droite(&self) -> f32 {
        self.max[0]
    }

    pub fn haut(&self) -> f32 {
        self.min[1]
    }

    pub fn bas(&self) -> f32 {
        self.max[1]
    }

    pub fn centre(&self) -> [f32; 2] {
        [
            (self.min[0] + self.max[0]) / 2.0,
            (self.min[1] + self.max[1]) / 2.0,
        ]
    }

    /// Réduit de `marge` de chaque côté (le centre reste).
    pub fn reduit(self, marge: f32) -> Rectangle {
        Rectangle {
            min: [self.min[0] + marge, self.min[1] + marge],
            max: [self.max[0] - marge, self.max[1] - marge],
        }
    }

    /// Le plus petit rectangle qui contient les deux.
    pub fn union(self, autre: Rectangle) -> Rectangle {
        Rectangle {
            min: [self.min[0].min(autre.min[0]), self.min[1].min(autre.min[1])],
            max: [self.max[0].max(autre.max[0]), self.max[1].max(autre.max[1])],
        }
    }

    fn contient_le_point(&self, p: [f32; 2]) -> bool {
        self.min[0] <= p[0] && p[0] <= self.max[0] && self.min[1] <= p[1] && p[1] <= self.max[1]
    }

    /// `autre` est-il en entier dedans (bords compris) ?
    pub fn contient(&self, autre: Rectangle) -> bool {
        self.contient_le_point(autre.min) && self.contient_le_point(autre.max)
    }

    /// Les deux se touchent-ils (bords compris) ?
    pub fn touche(&self, autre: Rectangle) -> bool {
        self.min[0] <= autre.max[0]
            && autre.min[0] <= self.max[0]
            && self.min[1] <= autre.max[1]
            && autre.min[1] <= self.max[1]
    }
}

/// Le plan des écrans : chaque place (`[x, y, largeur, hauteur]` en pixels du bureau, valeurs
/// quelconques) devient un rectangle dans `zone`. L'ensemble est mis à l'échelle (le plus petit
/// des deux rapports : les proportions sont gardées) et centré dans `zone` ; chaque case est
/// réduite de 1 point de chaque côté, pour que deux écrans collés ne se touchent pas.
/// Une place sans surface (largeur ou hauteur ≤ 0), une liste vide ou une zone sans surface :
/// une liste vide.
pub fn plan_des_cases(places: &[[i32; 4]], zone: Rectangle) -> Vec<Rectangle> {
    if places.is_empty()
        || places.iter().any(|p| p[2] <= 0 || p[3] <= 0)
        || zone.largeur() <= 0.0
        || zone.hauteur() <= 0.0
    {
        return Vec::new();
    }
    // L'enveloppe de tous les écrans (en f64 : des pixels de bureau, sans dépassement).
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in places {
        let (x, y) = (f64::from(p[0]), f64::from(p[1]));
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x + f64::from(p[2]));
        y1 = y1.max(y + f64::from(p[3]));
    }
    let (largeur, hauteur) = (x1 - x0, y1 - y0);
    let echelle = (f64::from(zone.largeur()) / largeur).min(f64::from(zone.hauteur()) / hauteur);
    let [cx_zone, cy_zone] = zone.centre();
    let decalage_x = f64::from(cx_zone) - largeur * echelle / 2.0;
    let decalage_y = f64::from(cy_zone) - hauteur * echelle / 2.0;
    places
        .iter()
        .map(|p| {
            let gauche = decalage_x + (f64::from(p[0]) - x0) * echelle;
            let haut = decalage_y + (f64::from(p[1]) - y0) * echelle;
            let droite = gauche + f64::from(p[2]) * echelle;
            let bas = haut + f64::from(p[3]) * echelle;
            // Réduit de 1 point de chaque côté, sans jamais inverser la case.
            let (cx, cy) = ((gauche + droite) / 2.0, (haut + bas) / 2.0);
            let demi_l = ((droite - gauche) / 2.0 - 1.0).max(0.0);
            let demi_h = ((bas - haut) / 2.0 - 1.0).max(0.0);
            Rectangle::de_min_max(
                [(cx - demi_l) as f32, (cy - demi_h) as f32],
                [(cx + demi_l) as f32, (cy + demi_h) as f32],
            )
        })
        .collect()
}

/// Le vide entre la case qui ouvre un menu de choix et le menu.
pub const ECART_ANCRE: f32 = 4.0;
/// Le vide gardé entre un menu de choix et le bord de la fenêtre.
pub const MARGE_FENETRE: f32 = 4.0;

/// Où poser un menu de choix de `taille` (`[largeur, hauteur]`) pour la case `ancre`, dans
/// `fenetre` : **sous la case** s'il y a la place, sinon **au-dessus**, sinon du côté qui en a
/// le plus (la hauteur est alors réduite, le menu défile). Aligné sur le bord gauche de la
/// case, ramené à gauche s'il dépasse à droite ; jamais hors de la fenêtre (marge
/// [`MARGE_FENETRE`]).
pub fn placer_le_menu(ancre: Rectangle, taille: [f32; 2], fenetre: Rectangle) -> Rectangle {
    let zone = fenetre.reduit(MARGE_FENETRE);
    let largeur = taille[0].min(zone.largeur()).max(0.0);
    let dessous = (zone.bas() - ancre.bas() - ECART_ANCRE).max(0.0);
    let dessus = (ancre.haut() - zone.haut() - ECART_ANCRE).max(0.0);
    let (en_dessous, hauteur) = if taille[1] <= dessous {
        (true, taille[1])
    } else if taille[1] <= dessus {
        (false, taille[1])
    } else if dessous >= dessus {
        (true, dessous.min(zone.hauteur()))
    } else {
        (false, dessus.min(zone.hauteur()))
    };
    let haut = if en_dessous {
        ancre.bas() + ECART_ANCRE
    } else {
        ancre.haut() - ECART_ANCRE - hauteur
    };
    let gauche = ancre.gauche().min(zone.droite() - largeur).max(zone.gauche());
    let haut = haut.min(zone.bas() - hauteur).max(zone.haut());
    Rectangle::de_min_taille([gauche, haut], [largeur, hauteur])
}

#[cfg(test)]
mod essais {
    use super::*;

    // ---- Le plan des écrans (venus de `interface_commune::pastilles`) ----

    fn zone() -> Rectangle {
        Rectangle::de_min_max([10.0, 4.0], [115.0, 48.0])
    }

    /// Le cas du propriétaire (valeurs inventées) : un fixe 2560×1440 à gauche, un portable
    /// 2560×1440 à droite, et sous lui l'écran intégré, plus petit, 1920×1200.
    fn cas_proprietaire() -> Vec<[i32; 4]> {
        vec![
            [0, 0, 2560, 1440],
            [2560, 0, 2560, 1440],
            [2560, 1440, 1920, 1200],
        ]
    }

    fn proche(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    /// Le plan garde les proportions : un seul facteur d'échelle pour la largeur et la hauteur.
    #[test]
    fn plan_garde_les_proportions() {
        let r = plan_des_cases(&cas_proprietaire(), zone());
        assert_eq!(r.len(), 3);
        // Les deux grands écrans : même taille, 16/9 (à 2 points de réduction près).
        assert!(proche(r[0].largeur(), r[1].largeur()));
        assert!(proche(r[0].hauteur(), r[1].hauteur()));
        assert!(proche(
            (r[0].largeur() + 2.0) / (r[0].hauteur() + 2.0),
            16.0 / 9.0
        ));
        // L'écran intégré : 1920×1200 = 16/10, plus petit, aligné à gauche de l'écran du dessus.
        assert!(proche(
            (r[2].largeur() + 2.0) / (r[2].hauteur() + 2.0),
            16.0 / 10.0
        ));
        assert!(r[2].largeur() < r[1].largeur() && r[2].hauteur() < r[1].hauteur());
        assert!(proche(r[2].gauche(), r[1].gauche()));
        // Il est sous l'écran du dessus, avec 2 points de vide.
        assert!(proche(r[2].haut() - r[1].bas(), 2.0));
        // Le portable est collé à droite du fixe : 2 points de vide.
        assert!(proche(r[1].gauche() - r[0].droite(), 2.0));
    }

    /// L'ensemble est centré dans la zone, et tient dedans (hauteur limitante ici).
    #[test]
    fn plan_centre_dans_la_zone() {
        let r = plan_des_cases(&cas_proprietaire(), zone());
        let tout = r.iter().fold(r[0], |a, b| a.union(*b));
        // Le vide de 1 point de chaque côté est symétrique : le centre est celui de la zone.
        assert!(proche(tout.centre()[0], zone().centre()[0]), "{tout:?}");
        assert!(proche(tout.centre()[1], zone().centre()[1]), "{tout:?}");
        assert!(zone().contient(tout));
        // Ici la hauteur limite : l'ensemble la remplit (au vide de 1 point près).
        assert!(proche(tout.hauteur() + 2.0, zone().hauteur()), "{tout:?}");
        assert!(tout.largeur() + 2.0 < zone().largeur());
    }

    /// La largeur peut limiter aussi : deux écrans côte à côte, dans une zone basse et large.
    #[test]
    fn plan_limite_par_la_largeur() {
        let r = plan_des_cases(&[[0, 0, 2000, 500], [2000, 0, 2000, 500]], zone());
        let tout = r[0].union(r[1]);
        assert!(proche(tout.largeur() + 2.0, zone().largeur()), "{tout:?}");
        assert!(tout.hauteur() + 2.0 < zone().hauteur());
        assert!(proche(tout.centre()[1], zone().centre()[1]));
    }

    /// Des coordonnées négatives et un décalage quelconque donnent le même plan.
    #[test]
    fn plan_independant_de_l_origine() {
        let a = plan_des_cases(&cas_proprietaire(), zone());
        let decale: Vec<[i32; 4]> = cas_proprietaire()
            .iter()
            .map(|p| [p[0] - 7000, p[1] - 3000, p[2], p[3]])
            .collect();
        let b = plan_des_cases(&decale, zone());
        for (x, y) in a.iter().zip(&b) {
            assert!(
                proche(x.gauche(), y.gauche()) && proche(x.haut(), y.haut()),
                "{x:?} {y:?}"
            );
            assert!(proche(x.largeur(), y.largeur()) && proche(x.hauteur(), y.hauteur()));
        }
    }

    /// Un seul écran remplit la zone en hauteur ; deux écrans collés ne se touchent pas.
    #[test]
    fn plan_un_ecran_et_ecrans_colles() {
        let r = plan_des_cases(&[[-1920, 0, 1920, 1080]], zone());
        assert_eq!(r.len(), 1);
        assert!(proche(r[0].centre()[0], zone().centre()[0]));
        assert!(proche(r[0].hauteur() + 2.0, zone().hauteur()));
        let d = plan_des_cases(&[[0, 0, 1920, 1080], [1920, 0, 1920, 1080]], zone());
        assert!(d[1].gauche() - d[0].droite() >= 2.0 - 0.01);
        assert!(!d[0].touche(d[1]));
    }

    /// Une place sans surface, une liste vide ou une zone vide : rien à dessiner.
    #[test]
    fn plan_refuse_le_vide() {
        assert!(plan_des_cases(&[], zone()).is_empty());
        assert!(plan_des_cases(&[[0, 0, 0, 100]], zone()).is_empty());
        assert!(plan_des_cases(&[[0, 0, 100, 100], [100, 0, 100, -5]], zone()).is_empty());
        assert!(plan_des_cases(&[[0, 0, 100, 100]], Rectangle::RIEN).is_empty());
    }

    // ---- La place d'un menu de choix (venus de `interface_commune::menu_choix`) ----

    fn fenetre() -> Rectangle {
        Rectangle::de_min_taille([0.0, 0.0], [400.0, 300.0])
    }

    #[test]
    fn le_menu_s_ouvre_sous_la_case_s_il_y_a_la_place() {
        let ancre = Rectangle::de_min_taille([20.0, 20.0], [100.0, 30.0]);
        let r = placer_le_menu(ancre, [200.0, 100.0], fenetre());
        assert_eq!(r.min, [20.0, 50.0 + ECART_ANCRE]);
        assert_eq!(r.taille(), [200.0, 100.0]);
    }

    #[test]
    fn le_menu_passe_au_dessus_quand_le_dessous_est_trop_juste() {
        let ancre = Rectangle::de_min_taille([20.0, 240.0], [100.0, 30.0]);
        let r = placer_le_menu(ancre, [200.0, 120.0], fenetre());
        assert_eq!(r.bas(), 240.0 - ECART_ANCRE);
        assert_eq!(r.hauteur(), 120.0);
    }

    #[test]
    fn le_menu_ne_sort_jamais_de_la_fenetre() {
        let zone = fenetre().reduit(MARGE_FENETRE);
        // À droite, en bas, en haut, à gauche ; petit et grand menu.
        for (x, y) in [
            (380.0, 20.0),
            (380.0, 280.0),
            (-30.0, 150.0),
            (200.0, 0.0),
            (0.0, 295.0),
        ] {
            let ancre = Rectangle::de_min_taille([x, y], [60.0, 24.0]);
            for taille in [[120.0, 60.0], [260.0, 200.0], [500.0, 900.0]] {
                let r = placer_le_menu(ancre, taille, fenetre());
                assert!(zone.contient(r), "{ancre:?} {taille:?} -> {r:?}");
            }
        }
    }

    #[test]
    fn trop_haut_des_deux_cotes_le_menu_prend_le_cote_le_plus_grand() {
        let ancre = Rectangle::de_min_taille([20.0, 200.0], [100.0, 30.0]);
        let r = placer_le_menu(ancre, [200.0, 400.0], fenetre());
        // Plus de place au-dessus (≈192) qu'au-dessous (≈62) : au-dessus, réduit.
        assert!(r.bas() <= ancre.haut());
        assert!(r.hauteur() < 400.0 && r.hauteur() > 100.0, "{r:?}");
    }

    /// Le rectangle compte comme celui d'egui : réduction, union, contenance, contact.
    #[test]
    fn rectangle_comme_egui() {
        let r = Rectangle::de_min_taille([10.0, 20.0], [30.0, 40.0]);
        assert_eq!(r.max, [40.0, 60.0]);
        assert_eq!(r.centre(), [25.0, 40.0]);
        assert_eq!(r.reduit(5.0), Rectangle::de_min_max([15.0, 25.0], [35.0, 55.0]));
        let u = r.union(Rectangle::de_min_max([0.0, 50.0], [5.0, 70.0]));
        assert_eq!(u, Rectangle::de_min_max([0.0, 20.0], [40.0, 70.0]));
        assert!(u.contient(r) && !r.contient(u));
        assert!(r.touche(Rectangle::de_min_max([40.0, 60.0], [50.0, 70.0])));
        assert!(!r.touche(Rectangle::de_min_max([40.1, 60.0], [50.0, 70.0])));
        assert!(Rectangle::RIEN.largeur() < 0.0 && Rectangle::RIEN.hauteur() < 0.0);
    }
}
