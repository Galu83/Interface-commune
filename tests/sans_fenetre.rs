//! Le volet, le petit menu et les Paramètres mis en page par egui seul, sans aucune fenêtre
//! (rien ne s'ouvre chez le propriétaire pendant les essais), avec un logiciel inventé.

// Dans un essai, un verrou empoisonné doit faire échouer l'essai.
#![allow(clippy::expect_used)]

use std::sync::Mutex;

use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::surface::{self, LARGEUR, LARGEUR_MAX, LARGEUR_MENU};
use interface_commune::theme::Surface;
use interface_commune::{
    Communs, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Palette, Valeur, apparence, dessin,
    menu, polices,
};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

/// Un logiciel inventé : « Démo », deux pages propres, une action propre au petit menu.
struct Demo {
    identite: Identite,
    changes: Mutex<Vec<(String, Valeur)>>,
    actions: Mutex<Vec<String>>,
}

impl Demo {
    fn new() -> Demo {
        Demo {
            identite: Identite {
                nom: "Démo".into(),
                version: "1.2.3".into(),
                licence: "MIT".into(),
                description: "Un logiciel inventé pour les essais.".into(),
                icone,
                glyphe: '\u{E7F4}',
            },
            changes: Mutex::default(),
            actions: Mutex::default(),
        }
    }
}

impl Logiciel for Demo {
    fn identite(&self) -> &Identite {
        &self.identite
    }

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

    fn action(&self, cle: &str) {
        self.actions.lock().expect("verrou").push(cle.into());
    }

    fn rechercher_mise_a_jour(&self) {}

    fn quitter(&self) {}

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

    fn noter(&self, _: String) {}
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
