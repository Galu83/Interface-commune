//! Le volet, le petit menu et les Paramètres mis en page par egui seul, sans aucune fenêtre
//! (rien ne s'ouvre chez le propriétaire pendant les essais), avec un logiciel inventé.

// Dans un essai, un verrou empoisonné doit faire échouer l'essai.
#![allow(clippy::expect_used)]

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::surface::{self, LARGEUR, LARGEUR_MAX, LARGEUR_MENU};
use interface_commune::theme::Surface;
use interface_commune::{
    Communs, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Palette, Reglages, Textes, Valeur,
    apparence, dessin, menu, polices, textes,
};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

/// Un logo inventé : un carré bleu, à toutes les tailles.
fn logo(t: u32) -> Option<Vec<u8>> {
    Some([0x20, 0x60, 0xD0, 0xFF].repeat((t * t) as usize))
}

fn identite() -> Identite {
    Identite {
        nom: "Démo".into(),
        version: "1.2.3".into(),
        licence: "MIT".into(),
        description: "Un logiciel inventé pour les essais.".into(),
        icone,
        logo: Some(logo),
    }
}

/// Un logiciel inventé : « Démo », deux pages propres, une action propre au petit menu.
struct Demo {
    identite: Identite,
    anglais: AtomicBool,
    changes: Mutex<Vec<(String, Valeur)>>,
    actions: Mutex<Vec<String>>,
}

impl Demo {
    fn new() -> Demo {
        Demo {
            identite: identite(),
            anglais: AtomicBool::new(false),
            changes: Mutex::default(),
            actions: Mutex::default(),
        }
    }
}

impl Reglages for Demo {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn textes(&self) -> &Textes {
        if self.anglais.load(Ordering::Relaxed) {
            &textes::ANGLAIS
        } else {
            &textes::FRANCAIS
        }
    }

    fn action(&self, cle: &str) {
        self.actions.lock().expect("verrou").push(cle.into());
    }

    fn rechercher_mise_a_jour(&self) {}

    fn accueil(&self) -> bool {
        true
    }

    fn pages(&self) -> Vec<Page> {
        vec![
            Page::new("ecrans", "Écrans", '\u{E7F4}'),
            Page::new("dispositions", "Dispositions", '\u{E8A9}'),
        ]
    }

    fn dessiner_page(&self, cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        ui.label(format!("Page {cle}"));
        page.redemarrage_necessaire("Le changement de carte graphique demande un redémarrage.");
    }

    fn communs(&self) -> Communs {
        Communs {
            demarrage: Some(true),
            langue: Some(("Français".into(), vec!["Français".into(), "English".into()])),
            notifications: Some(false),
            mise_a_jour: EtatMiseAJour::Prete("1.3.0".into()),
            dossier_installateurs: "D:\\Logiciels\\Installateurs".into(),
            nouveautes: vec!["Une nouveauté inventée.".into()],
            credits: vec![interface_commune::Credit {
                nom: "egui".into(),
                licence: "MIT OR Apache-2.0".into(),
            }],
        }
    }

    fn changer(&self, cle: &str, valeur: Valeur) {
        self.changes.lock().expect("verrou").push((cle.into(), valeur));
    }
}

impl Logiciel for Demo {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Bureau du salon".into(),
            ligne: "Prêt".into(),
            detail: Some("Une explication assez longue pour passer à la ligne dans le volet, sans jamais être coupée au bord droit de la fenêtre.".into()),
        }
    }

    fn dessiner_rapides(&self, ui: &mut egui::Ui, p: &Palette) -> Option<String> {
        dessin::ligne(ui, p, '\u{E7F4}', p.texte, "Écran seul", Some("Ctrl+Alt+F1"))
            .clicked()
            .then(|| "ecran".into())
    }

    fn actions_menu(&self) -> Vec<menu::Action> {
        vec![menu::Action::new("relire", '\u{E72C}', "Relire les écrans")]
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}

/// Un logiciel qui ne prend que les Paramètres : ni volet, ni petit menu, ni icône.
struct Seul {
    identite: Identite,
}

impl Reglages for Seul {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn textes(&self) -> &Textes {
        &textes::ANGLAIS
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Une image egui sans fenêtre : rend les textes dessinés.
fn image(
    taille: egui::Vec2,
    ppp: f32,
    mut dessin: impl FnMut(&mut egui::Ui, &Palette),
) -> Vec<(String, egui::Rect)> {
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    let a = apparence::Apparence::de(Default::default(), Some(false));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let mut textes = Vec::new();
    for _ in 0..3 {
        let mut entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, taille)),
            ..Default::default()
        };
        entree
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(ppp);
        let mut s = ctx.run_ui(entree, |ui| dessin(ui, &p));
        s.textures_delta.clear();
        textes.clear();
        for c in s.shapes {
            formes(&c.shape, &mut textes);
        }
    }
    textes
}

fn formes(s: &egui::Shape, textes: &mut Vec<(String, egui::Rect)>) {
    match s {
        egui::Shape::Vec(v) => v.iter().for_each(|s| formes(s, textes)),
        egui::Shape::Text(t) => textes.push((t.galley.text().to_string(), t.visual_bounding_rect())),
        _ => {}
    }
}

/// Le volet et le petit menu tiennent dans leur largeur à 100, 125, 150 et 200 %, et le petit
/// menu dessine ses lignes dans l'ordre des règles.
#[test]
fn volet_et_petit_menu_sans_fenetre() {
    let demo = Demo::new();
    for ppp in [1.0, 1.25, 1.5, 2.0] {
        let mut taille = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 1200.0), ppp, |ui, p| {
            taille = surface::dessiner(ui, p, Surface::Volet, &demo).1;
        });
        assert!((LARGEUR..=LARGEUR_MAX).contains(&taille.x), "{ppp} : {taille:?}");
        if polices::polices().is_ok() {
            for (t, r) in &textes {
                assert!(
                    r.max.x <= taille.x + 0.5,
                    "{ppp} : « {t} » dépasse ({r:?}, {taille:?})"
                );
            }
            assert!(textes.iter().any(|(t, _)| t == "Démo 1.2.3"));
        }
        let mut taille = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 1200.0), ppp, |ui, p| {
            taille = surface::dessiner(ui, p, Surface::Menu, &demo).1;
        });
        assert!(
            (LARGEUR_MENU..=LARGEUR_MAX).contains(&taille.x),
            "{ppp} : {taille:?}"
        );
        if polices::polices().is_ok() {
            let ordre: Vec<&str> = textes
                .iter()
                .map(|(t, _)| t.as_str())
                .filter(|t| t.chars().any(|c| c.is_alphabetic()))
                .collect();
            assert_eq!(
                ordre,
                [
                    "Relire les écrans",
                    "Paramètres",
                    "Rechercher une mise à jour",
                    "Ouvrir le journal",
                    "Quitter"
                ]
            );
        }
    }
}

/// Chaque page des Paramètres se dessine sans fenêtre ; l'en-tête montre le nom et
/// « Version x.y.z » ; une page qui demande un redémarrage montre le bandeau.
#[test]
fn parametres_sans_fenetre() {
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("mise_a_jour"));
    assert_eq!(s.cle(), "mise_a_jour");
    let cles: Vec<String> = s.pages.iter().map(|p| p.cle.clone()).collect();
    assert_eq!(
        cles,
        [
            "accueil",
            "ecrans",
            "dispositions",
            "general",
            "mise_a_jour",
            "a_propos"
        ]
    );
    for cle in &cles {
        s.aller_a(cle);
        let textes = image(egui::vec2(940.0, 660.0), 1.0, |ui, p| s.dessiner(ui, p, &demo));
        if polices::polices().is_ok() {
            let tous: Vec<&str> = textes.iter().map(|(t, _)| t.as_str()).collect();
            assert!(tous.contains(&"Démo"), "{cle} : {tous:?}");
            assert!(tous.contains(&"Version 1.2.3"), "{cle} : {tous:?}");
            assert!(tous.contains(&"Mise à jour"), "{cle} : {tous:?}");
        }
    }
    assert!(s.redemarrage.is_some(), "le bandeau de redémarrage est demandé");
}

/// Effet immédiat : un changement part tout de suite au logiciel ; « Annuler » lui rend la
/// valeur d'avant.
#[test]
fn effet_immediat_et_annuler_vers_le_logiciel() {
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, None);
    s.changer(
        &demo,
        "general.demarrage",
        "Démarrer avec Windows",
        Valeur::Booleen(true),
        Valeur::Booleen(false),
    );
    assert_eq!(
        demo.changes.lock().expect("verrou").last(),
        Some(&("general.demarrage".to_string(), Valeur::Booleen(false)))
    );
    let (cle, avant) = s
        .annulation
        .annuler(std::time::Instant::now())
        .expect("bulle Annuler");
    demo.changer(&cle, avant);
    assert_eq!(
        demo.changes.lock().expect("verrou").last(),
        Some(&("general.demarrage".to_string(), Valeur::Booleen(true)))
    );
    assert!(demo.actions.lock().expect("verrou").is_empty());
}

/// Ce qu'une image dessine : les textes (avec leur taille de police) et les images.
#[derive(Debug, Clone)]
enum Dessine {
    Texte {
        texte: String,
        rect: egui::Rect,
        taille: f32,
    },
    Image(egui::Rect),
}

fn dessins(s: &egui::Shape, v: &mut Vec<Dessine>) {
    match s {
        egui::Shape::Vec(l) => l.iter().for_each(|s| dessins(s, v)),
        egui::Shape::Text(t) => v.push(Dessine::Texte {
            texte: t.galley.text().to_string(),
            rect: egui::Rect::from_min_size(t.pos, t.galley.size()),
            taille: t
                .galley
                .job
                .sections
                .first()
                .map_or(0.0, |s| s.format.font_id.size),
        }),
        egui::Shape::Mesh(m) if m.texture_id != egui::TextureId::default() => {
            v.push(Dessine::Image(m.calc_bounds()))
        }
        _ => {}
    }
}

/// Une image des Paramètres, sans fenêtre (trois passes, la dernière est rendue).
fn image_parametres(s: &mut Parametres, l: &dyn Reglages) -> Vec<Dessine> {
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    let a = apparence::Apparence::de(Default::default(), Some(false));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let mut v = Vec::new();
    for _ in 0..3 {
        let entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(940.0, 660.0),
            )),
            ..Default::default()
        };
        let mut sortie = ctx.run_ui(entree, |ui| s.dessiner(ui, &p, l));
        sortie.textures_delta.clear();
        v.clear();
        for c in sortie.shapes {
            dessins(&c.shape, &mut v);
        }
    }
    v
}

fn texte<'a>(v: &'a [Dessine], cherche: &str) -> Option<(&'a egui::Rect, f32)> {
    v.iter().find_map(|d| match d {
        Dessine::Texte {
            texte, rect, taille, ..
        } if texte == cherche => Some((rect, *taille)),
        _ => None,
    })
}

/// Un logiciel qui ne prend que les Paramètres (trait `Reglages` seul), en anglais : tous
/// les mots communs sont en anglais, aucun texte français imposé.
#[test]
fn parametres_seuls_en_anglais() {
    let seul = Seul { identite: identite() };
    assert_eq!(Parametres::titre_fenetre(&seul), "Démo Settings");
    let mut s = Parametres::new(&seul, None);
    let titres: Vec<&str> = s.pages.iter().map(|p| p.titre.as_str()).collect();
    assert_eq!(titres, ["General", "Update", "About"]);
    for cle in ["general", "mise_a_jour", "a_propos"] {
        s.aller_a(cle);
        let v = image_parametres(&mut s, &seul);
        if polices::polices().is_err() {
            return;
        }
        let tous: Vec<&str> = v
            .iter()
            .filter_map(|d| match d {
                Dessine::Texte { texte, .. } => Some(texte.as_str()),
                Dessine::Image(_) => None,
            })
            .collect();
        for francais in [
            "Général",
            "Mise à jour",
            "À propos",
            "Ouvrir le journal",
            "Rechercher une mise à jour",
            "Dossier des installateurs",
        ] {
            assert!(!tous.contains(&francais), "{cle} : « {francais} » dans {tous:?}");
        }
        assert!(tous.contains(&"Version 1.2.3"), "{cle} : {tous:?}");
    }
}

/// L'en-tête rend celui de PowerPeek : le vrai logo (une image de 32 points), le nom en 16
/// points, la version en 12 ; des lignes de navigation de 38 points ; Général, Mise à jour et
/// À propos juste après les pages du logiciel, 6 points plus bas, jamais au bas de la
/// fenêtre ; une colonne de 250 points au moins, plus large pour un nom de page long.
#[test]
fn entete_et_navigation_comme_powerpeek() {
    if polices::polices().is_err() {
        return;
    }
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("ecrans"));
    let v = image_parametres(&mut s, &demo);
    let logo = v
        .iter()
        .find_map(|d| match d {
            Dessine::Image(r) => Some(*r),
            Dessine::Texte { .. } => None,
        })
        .expect("le logo est une image");
    assert!(
        (logo.width() - 32.0).abs() < 0.5 && (logo.height() - 32.0).abs() < 0.5,
        "{logo:?}"
    );
    let (nom, taille_nom) = texte(&v, "Démo").expect("le nom");
    assert_eq!(taille_nom, 16.0);
    assert!(nom.left() > logo.right(), "le nom est à droite du logo");
    let (version, taille_version) = texte(&v, "Version 1.2.3").expect("la version");
    assert_eq!(taille_version, 12.0);
    assert!(version.top() >= nom.bottom() - 1.0, "la version est sous le nom");
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    let pas = y("Dispositions") - y("Écrans");
    let espacement = 2.0 + 8.0;
    assert!(
        (pas - 38.0 - espacement).abs() < 0.6,
        "pas entre deux lignes : {pas}"
    );
    let saut = y("Général") - y("Dispositions");
    assert!(
        (saut - pas - 6.0).abs() < 0.6,
        "6 points avant Général : {saut} / {pas}"
    );
    assert!((y("Mise à jour") - y("Général") - pas).abs() < 0.6);
    assert!(
        (y("À propos") - y("Mise à jour") - pas).abs() < 0.6 && y("À propos") < 660.0 - 150.0,
        "les pages communes ne sont pas au bas"
    );
    let largeur = |s: &Parametres, l: &dyn Reglages| {
        let ctx = egui::Context::default();
        ctx.set_fonts(polices::polices_ou_vides());
        let mut w = 0.0;
        for _ in 0..2 {
            let mut o = ctx.run_ui(egui::RawInput::default(), |ui| w = s.largeur_navigation(ui, l));
            o.textures_delta.clear();
        }
        w
    };
    assert_eq!(
        largeur(&s, &demo),
        interface_commune::parametres::LARGEUR_NAVIGATION_MIN
    );
    let longue = Longue(Demo::new());
    let s2 = Parametres::new(&longue, None);
    let w = largeur(&s2, &longue);
    assert!(
        w > interface_commune::parametres::LARGEUR_NAVIGATION_MIN
            && w <= interface_commune::parametres::LARGEUR_NAVIGATION_MAX,
        "{w}"
    );
}

/// Un logiciel aux noms de pages longs.
struct Longue(Demo);

impl Reglages for Longue {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new(
            "longue",
            "Une page au nom vraiment très, très long",
            '\u{E7F4}',
        )]
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// La langue change pendant que les Paramètres sont ouverts : la navigation suit, sur la
/// même page.
#[test]
fn la_langue_change_en_direct() {
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("a_propos"));
    assert_eq!(s.pages[s.page].titre, "À propos");
    demo.anglais.store(true, Ordering::Relaxed);
    let _ = image_parametres(&mut s, &demo);
    assert_eq!(s.cle(), "a_propos");
    assert_eq!(s.pages[s.page].titre, "About");
    assert_eq!(Parametres::titre_fenetre(&demo), "Démo Settings");
}
