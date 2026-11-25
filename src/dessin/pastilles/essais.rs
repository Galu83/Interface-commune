//! Les essais des pastilles et du choix illustré (sortis de `pastilles.rs`, bloc F : plus de
//! 200 lignes).

use super::*;
use crate::briques::{self, Brique};
use crate::theme::ChoixWindows;

fn pa(cle: Option<&str>, lettre: char, nom: &str) -> Pastille {
    Pastille {
        cle: cle.map(Into::into),
        lettre,
        logo: None,
        couleur: couleurs_pastilles()[0],
        nom: nom.into(),
        mention: None,
        glyphe: None,
        absent: false,
    }
}

fn case_ecran(l: char) -> Case {
    Case {
        lettre: Some(l),
        couleur: Some(couleurs_pastilles()[0]),
        ..Default::default()
    }
}

fn avec_place(place: Option<[i32; 4]>) -> Case {
    Case {
        place,
        ..case_ecran('A')
    }
}

fn zone() -> Rect {
    Rect::from_min_max(pos2(10.0, 4.0), pos2(115.0, 48.0))
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

/// Le plan ne sert que si TOUTES les cases ont une place valide.
#[test]
fn plan_ou_rangee() {
    let ok = [Some([0, 0, 100, 100]), Some([100, 0, 100, 100])];
    let c: Vec<Case> = ok.iter().map(|p| avec_place(*p)).collect();
    assert_eq!(places_valides(&c).map(|v| v.len()), Some(2));
    for faux in [
        None,
        Some([0, 0, 0, 100]),
        Some([0, 0, 100, 0]),
        Some([0, 0, -1, 5]),
    ] {
        let c = vec![avec_place(ok[0]), avec_place(faux)];
        assert_eq!(places_valides(&c), None, "{faux:?}");
    }
    assert_eq!(places_valides(&[]), None);
}

/// La plus petite case fait-elle moins de 12 points ? Alors seulement la couleur.
#[test]
fn plan_petite_case_en_couleur_seule() {
    let r = plan_des_cases(&cas_proprietaire(), zone());
    assert_eq!(
        genre_du_plan(&r),
        Genre::Plan,
        "le cas du propriétaire garde ses lettres"
    );
    // Un petit écran à côté d'un très grand : sa case tombe sous 12 points.
    let r = plan_des_cases(&[[0, 0, 3840, 2160], [3840, 0, 800, 480]], zone());
    assert!(r[1].height() < PLAN_CASE_H_MIN);
    assert_eq!(genre_du_plan(&r), Genre::PlanCouleur);
    // Les proportions ne sont pas faussées : la case garde son rapport 5/3.
    assert!(proche(
        (r[1].width() + 2.0) / (r[1].height() + 2.0),
        800.0 / 480.0
    ));
}

/// Le plan se dessine dans le volet (3 cases, puis une place manquante : la rangée, puis
/// 5 cases : 4 au plus), sans changer la hauteur de la ligne.
#[test]
fn plan_dans_le_volet() {
    let p = Palette::de(&ChoixWindows::default());
    let ctx = egui::Context::default();
    let cas = |cases: Vec<Case>| {
        let mut h = 0.0;
        let mut sortie = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(320.0, 300.0))),
                ..Default::default()
            },
            |ui| {
                let h0 = ui.cursor().top();
                dessiner_choix(ui, &p, "c", "Choix", Some("sous"), &cases, None, true, None);
                h = ui.cursor().top() - h0;
            },
        );
        sortie.textures_delta.clear();
        h
    };
    let places = cas_proprietaire();
    let trois: Vec<Case> = places.iter().map(|q| avec_place(Some(*q))).collect();
    // La référence : la rangée d'aujourd'hui (sans place).
    let rangee = cas(vec![case_ecran('A'); 3]);
    assert!(
        (ILLUSTRE_LIGNE..=ILLUSTRE_LIGNE + 8.0).contains(&rangee),
        "{rangee}"
    );
    assert_eq!(cas(trois.clone()), rangee);
    let mut manque = trois.clone();
    manque[2].place = None;
    assert_eq!(cas(manque), rangee);
    let cinq: Vec<Case> = (0..5).map(|i| avec_place(Some([i * 100, 0, 100, 100]))).collect();
    assert_eq!(cas(cinq), rangee);
}

/// Le texte ne bouge pas, avec ou sans plan : la zone du plan garde la largeur d'aujourd'hui.
#[test]
fn plan_ne_deplace_pas_le_texte() {
    assert!(ILLUSTRE_MARGE + ILLUSTRE_LARGEUR <= texte_x());
}

fn liste() -> Vec<Brique> {
    vec![
        Brique::Pastilles {
            titre: Some("Ordinateurs".into()),
            pastilles: vec![
                pa(Some("fixe"), 'F', "Fixe"),
                pa(None, 'P', "Portable"),
                pa(Some("tour"), 'T', "Tour"),
            ],
        },
        Brique::ChoixIllustre {
            cle: "bureau".into(),
            texte: "Bureau".into(),
            sous_ligne: Some("Fixe à gauche".into()),
            illustration: vec![case_ecran('F'), case_ecran('P')],
            raccourci: Some("Ctrl+Alt+Maj+F1".into()),
            actif: true,
            etiquette_active: Some("Affiché".into()),
        },
    ]
}

/// Une image de la liste ; rend le geste et les hauteurs des deux briques.
fn image(ctx: &egui::Context, evts: Vec<egui::Event>) -> (Option<Geste>, [f32; 2]) {
    let p = Palette::de(&ChoixWindows::default());
    let entree = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(320.0, 600.0))),
        events: evts,
        time: Some(ctx.input(|i| i.time) + 0.1),
        ..Default::default()
    };
    let mut rendu = (None, [0.0; 2]);
    let mut sortie = ctx.run_ui(entree, |ui| {
        let l = liste();
        let h0 = ui.cursor().top();
        let mut g = briques::dessiner(ui, &p, &l[..1]);
        let h1 = ui.cursor().top();
        g = g.or(briques::dessiner(ui, &p, &l[1..]));
        rendu = (g, [h1 - h0, ui.cursor().top() - h1]);
    });
    sortie.textures_delta.clear();
    rendu
}

fn clic(ctx: &egui::Context, pos: egui::Pos2) -> Option<Geste> {
    let bouton = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    image(ctx, vec![egui::Event::PointerMoved(pos)]);
    image(ctx, vec![bouton(true)]);
    image(ctx, vec![bouton(false)]).0
}

/// 3 pastilles : 2 colonnes, donc 2 rangées ; le choix illustré a sa hauteur propre.
#[test]
fn hauteurs() {
    let ctx = egui::Context::default();
    let (_, h) = image(&ctx, vec![]);
    assert!(
        (h[0] - (VOLET_SECTION + 2.0 * VOLET_LIGNE + 2.0 * 2.0)).abs() <= 8.0,
        "{h:?}"
    );
    assert!((h[1] - ILLUSTRE_LIGNE).abs() <= 8.0, "{h:?}");
    assert_eq!(rangees(1), 1);
    assert_eq!(rangees(2), 1);
    assert_eq!(rangees(3), 2);
    assert_eq!(rangees(4), 2);
}

/// Une pastille avec clé rend sa clé ; sans clé, un clic ne fait rien ; le choix illustré
/// rend la sienne.
#[test]
fn gestes() {
    let ctx = egui::Context::default();
    image(&ctx, vec![]);
    let y = VOLET_SECTION + 2.0 + VOLET_LIGNE / 2.0;
    assert_eq!(clic(&ctx, pos2(40.0, y)), Some(Geste::Action("fixe".into())));
    assert_eq!(clic(&ctx, pos2(200.0, y)), None, "pastille sans clé");
    // Rangée 2, colonne 1 : « Tour ».
    assert_eq!(
        clic(&ctx, pos2(40.0, y + VOLET_LIGNE)),
        Some(Geste::Action("tour".into()))
    );
    let (_, h) = image(&ctx, vec![]);
    let y_choix = VOLET_SECTION + 2.0 + 2.0 * VOLET_LIGNE + 2.0 + 20.0;
    assert!(h[0] > 0.0);
    assert_eq!(
        clic(&ctx, pos2(200.0, y_choix)),
        Some(Geste::Action("bureau".into()))
    );
}

/// Les formes dessinées pour un choix illustré de ces cases (rangée, ou plan avec places).
fn formes_du_choix(cases: Vec<Case>) -> Vec<egui::epaint::ClippedShape> {
    let ctx = egui::Context::default();
    ctx.set_fonts(crate::polices::polices_ou_vides());
    let p = Palette::de(&ChoixWindows::default());
    let l = vec![Brique::ChoixIllustre {
        cle: "c".into(),
        texte: "Choix".into(),
        sous_ligne: None,
        illustration: cases,
        raccourci: None,
        actif: false,
        etiquette_active: None,
    }];
    let mut formes = Vec::new();
    for _ in 0..2 {
        let entree = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(320.0, 200.0))),
            ..Default::default()
        };
        let mut sortie = ctx.run_ui(entree, |ui| {
            briques::dessiner(ui, &p, &l);
        });
        sortie.textures_delta.clear();
        formes = sortie.shapes;
    }
    formes
}

/// Un écran à la fois débranché et en veille se dessine comme un écran débranché, dans la
/// rangée comme dans le plan ; et ce dessin n'est pas celui de la veille.
#[test]
fn debranche_l_emporte_sur_la_veille() {
    let places = [Some([0, 0, 1920, 1080]), Some([1920, 0, 1920, 1080])];
    // Sans places : la rangée ; avec : le plan des écrans.
    for place in [[None, None], places] {
        let choix = |veille: bool, debranche: bool| {
            let mut b = case_ecran('B');
            b.veille = veille;
            b.debranche = debranche;
            vec![
                Case {
                    place: place[0],
                    ..case_ecran('A')
                },
                Case { place: place[1], ..b },
            ]
        };
        let les_deux = formes_du_choix(choix(true, true));
        assert_eq!(les_deux, formes_du_choix(choix(false, true)), "{place:?}");
        assert_ne!(les_deux, formes_du_choix(choix(true, false)), "{place:?}");
        assert_ne!(les_deux, formes_du_choix(choix(false, false)), "{place:?}");
    }
}

/// La largeur de l'illustration est fixe : 1 ou 4 cases, le texte est au même endroit.
#[test]
fn texte_au_meme_endroit() {
    assert!(texte_x() >= ILLUSTRE_LARGEUR);
    let ctx = egui::Context::default();
    let mut largeur = 0.0;
    let mut sortie = ctx.run_ui(Default::default(), |ui| {
        largeur = briques::largeur(ui, &liste());
    });
    sortie.textures_delta.clear();
    assert!(largeur >= texte_x() + LIGNE_MARGE_DROITE, "{largeur}");
}
