//! Les couleurs dans les Paramètres, sans aucune fenêtre : la rangée « choisir une couleur »
//! (souris, clavier), la carte avec la bulle « Annuler » en option, la pastille d'une carte.
//! Logiciel et valeurs inventés.

#![allow(clippy::expect_used)]

use std::sync::Mutex;

use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, vec2};
use interface_commune::couleurs::{carte_couleur, choisir_couleur, palette, pastille_de_carte};
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::pastilles::Pastille;
use interface_commune::{Communs, Identite, Page, Reglages, Valeur, apparence, polices};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

fn contexte() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    apparence::Apparence::de(Default::default(), Some(false)).appliquer(&ctx);
    ctx
}

fn touche(key: Key) -> Vec<Event> {
    [true, false]
        .into_iter()
        .map(|pressed| Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        })
        .collect()
}

fn bouton(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

/// Les ronds pleins de la rangée et les anneaux de choix (rayon 14,5).
fn ronds(s: &egui::Shape, pleins: &mut Vec<Pos2>, anneaux: &mut Vec<Pos2>) {
    match s {
        egui::Shape::Vec(l) => l.iter().for_each(|s| ronds(s, pleins, anneaux)),
        egui::Shape::Circle(c) if c.fill != egui::Color32::TRANSPARENT => pleins.push(c.center),
        egui::Shape::Circle(c) if (c.radius - 14.5).abs() < 0.1 => anneaux.push(c.center),
        _ => {}
    }
}

/// La rangée seule : une image avec les événements donnés ; rend le choix, les centres des
/// ronds et ceux des anneaux.
fn rangee(
    ctx: &egui::Context,
    events: Vec<Event>,
    choisie: Option<usize>,
) -> (Option<usize>, Vec<Pos2>, Vec<Pos2>) {
    let p = apparence::Apparence::de(Default::default(), Some(false)).palette;
    let mut choix = None;
    let entree = RawInput {
        events,
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 200.0))),
        ..Default::default()
    };
    let mut sortie = ctx.run_ui(entree, |ui| {
        choix = choisir_couleur(ui, &p, &palette(), choisie, "Couleur du PC");
    });
    sortie.textures_delta.clear();
    let (mut pleins, mut anneaux) = (vec![], vec![]);
    for c in sortie.shapes {
        ronds(&c.shape, &mut pleins, &mut anneaux);
    }
    (choix, pleins, anneaux)
}

#[test]
fn la_palette_est_celle_des_pastilles_du_volet() {
    assert_eq!(palette(), apparence::couleurs_pastilles());
}

#[test]
fn un_clic_choisit_et_le_contour_suit_le_choix() {
    let ctx = contexte();
    let _ = rangee(&ctx, vec![], Some(1));
    let (_, pleins, anneaux) = rangee(&ctx, vec![], Some(1));
    assert_eq!(pleins.len(), 6);
    assert_eq!(anneaux.len(), 1);
    assert_eq!(anneaux[0], pleins[1], "le contour entoure la couleur choisie");
    let au = pleins[3];
    let _ = rangee(&ctx, vec![Event::PointerMoved(au)], Some(1));
    let _ = rangee(&ctx, vec![bouton(au, true)], Some(1));
    let (choix, _, _) = rangee(&ctx, vec![bouton(au, false)], Some(1));
    assert_eq!(choix, Some(3));
}

#[test]
fn le_clavier_fleches_puis_entree_ou_espace() {
    for validation in [Key::Enter, Key::Space] {
        let ctx = contexte();
        let _ = rangee(&ctx, vec![], Some(0));
        // Tab entre dans la rangée, sur la première pastille.
        let _ = rangee(&ctx, touche(Key::Tab), Some(0));
        let _ = rangee(&ctx, vec![], Some(0));
        // Deux flèches droite : la troisième. Les flèches ne choisissent pas.
        for _ in 0..2 {
            let (choix, _, _) = rangee(&ctx, touche(Key::ArrowRight), Some(0));
            assert_eq!(choix, None, "une flèche ne choisit pas");
            let _ = rangee(&ctx, vec![], Some(0));
        }
        // Une flèche gauche : la deuxième.
        let _ = rangee(&ctx, touche(Key::ArrowLeft), Some(0));
        let _ = rangee(&ctx, vec![], Some(0));
        let mut choix = None;
        for e in touche(validation) {
            choix = choix.or(rangee(&ctx, vec![e], Some(0)).0);
        }
        let _ = rangee(&ctx, vec![], Some(0));
        assert_eq!(choix, Some(1), "{validation:?}");
    }
}

#[test]
fn les_fleches_s_arretent_aux_bords() {
    let ctx = contexte();
    let _ = rangee(&ctx, touche(Key::Tab), None);
    let _ = rangee(&ctx, vec![], None);
    for _ in 0..3 {
        let _ = rangee(&ctx, touche(Key::ArrowLeft), None);
        let _ = rangee(&ctx, vec![], None);
    }
    let mut choix = None;
    for e in touche(Key::Enter) {
        choix = choix.or(rangee(&ctx, vec![e], None).0);
    }
    assert_eq!(choix, Some(0));
}

/// Un logiciel inventé : une page « Couleurs », deux cartes (avec et sans « Annuler »).
struct Exemple {
    identite: Identite,
    changes: Mutex<Vec<(String, Valeur)>>,
    rendu: Mutex<Vec<Option<usize>>>,
}

impl Reglages for Exemple {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("p", "Couleurs", '\u{E790}')]
    }

    fn dessiner_page(&self, _: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        let avec = carte_couleur(
            ui,
            page,
            '\u{E790}',
            "Couleur du bureau",
            "Choisis une couleur.",
            &palette(),
            0,
            Some(("pc.couleur", "Couleur du bureau")),
        );
        self.rendu.lock().expect("verrou").push(avec);
        let sans = carte_couleur(ui, page, '\u{E790}', "Couleur libre", "", &palette(), 0, None);
        self.rendu.lock().expect("verrou").push(sans);
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, cle: &str, valeur: Valeur) {
        self.changes.lock().expect("verrou").push((cle.into(), valeur));
    }
}

fn exemple() -> Exemple {
    Exemple {
        identite: Identite {
            nom: "Exemple".into(),
            version: "1.0.0".into(),
            licence: "MIT".into(),
            description: "Inventé.".into(),
            icone,
            logo: None,
        },
        changes: Mutex::default(),
        rendu: Mutex::default(),
    }
}

/// Une image des Paramètres ; rend les centres des ronds pleins.
fn image(ctx: &egui::Context, s: &mut Parametres, l: &Exemple, events: Vec<Event>) -> Vec<Pos2> {
    let p = apparence::Apparence::de(Default::default(), Some(false)).palette;
    let entree = RawInput {
        events,
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(920.0, 680.0))),
        ..Default::default()
    };
    let mut sortie = ctx.run_ui(entree, |ui| s.dessiner(ui, &p, l));
    sortie.textures_delta.clear();
    let (mut pleins, mut anneaux) = (vec![], vec![]);
    for c in sortie.shapes {
        ronds(&c.shape, &mut pleins, &mut anneaux);
    }
    pleins
}

/// Clique sur le rond `rang` des ronds de la page.
fn cliquer(ctx: &egui::Context, s: &mut Parametres, l: &Exemple, rang: usize) {
    for _ in 0..3 {
        let _ = image(ctx, s, l, vec![]);
    }
    let pleins = image(ctx, s, l, vec![]);
    assert!(pleins.len() >= 12, "deux rangées de six : {}", pleins.len());
    let au = pleins[rang];
    let _ = image(ctx, s, l, vec![Event::PointerMoved(au)]);
    let _ = image(ctx, s, l, vec![bouton(au, true)]);
    let _ = image(ctx, s, l, vec![bouton(au, false)]);
}

#[test]
fn la_carte_rend_le_choix_et_annuler_remet_l_ancien() {
    let ctx = contexte();
    let l = exemple();
    let mut s = Parametres::new(&l, Some("p"));
    cliquer(&ctx, &mut s, &l, 2);
    assert!(l.rendu.lock().expect("verrou").contains(&Some(2)));
    assert_eq!(
        l.changes.lock().expect("verrou").last(),
        Some(&("pc.couleur".to_string(), Valeur::Nombre(2.0)))
    );
    let defaits = s.annulation.annuler(std::time::Instant::now());
    assert_eq!(defaits, vec![("pc.couleur".to_string(), Valeur::Nombre(0.0))]);
}

#[test]
fn sans_annuler_rien_n_est_signale_a_la_bibliotheque() {
    let ctx = contexte();
    let l = exemple();
    let mut s = Parametres::new(&l, Some("p"));
    cliquer(&ctx, &mut s, &l, 6 + 2);
    assert!(l.rendu.lock().expect("verrou").contains(&Some(2)));
    assert!(l.changes.lock().expect("verrou").is_empty());
    assert!(s.annulation.annuler(std::time::Instant::now()).is_empty());
}

#[test]
fn la_pastille_de_carte_a_la_taille_de_celle_du_volet() {
    let ctx = contexte();
    let p = apparence::Apparence::de(Default::default(), Some(false)).palette;
    let pa = Pastille {
        cle: None,
        lettre: 'F',
        logo: None,
        couleur: palette()[0],
        nom: "Fixe".into(),
        mention: None,
        glyphe: None,
        absent: false,
    };
    let mut taille = vec2(0.0, 0.0);
    let mut sortie = ctx.run_ui(RawInput::default(), |ui| {
        taille = pastille_de_carte(ui, &p, &pa).rect.size();
    });
    sortie.textures_delta.clear();
    assert_eq!(
        taille,
        vec2(
            interface_commune::mesures::PASTILLE_LARGEUR,
            interface_commune::mesures::PASTILLE_HAUTEUR
        )
    );
}
