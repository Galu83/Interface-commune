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
    /// Un gros titre à la place du nom « Bureau du salon » (essai des noms longs).
    titre: Mutex<Option<String>>,
}

impl Demo {
    fn new() -> Demo {
        Demo {
            identite: identite(),
            anglais: AtomicBool::new(false),
            changes: Mutex::default(),
            actions: Mutex::default(),
            titre: Mutex::default(),
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

    fn dessiner_general(&self, ui: &mut egui::Ui, page: &mut Contexte) {
        dessin::carte(
            ui,
            page.palette,
            '\u{E7F4}',
            "Carte propre",
            "Un réglage inventé du logiciel.",
            |_| {},
        );
    }

    fn communs(&self) -> Communs {
        Communs {
            demarrage: Some(true),
            langue: Some(("Français".into(), vec!["Français".into(), "English".into()])),
            notifications: Some(false),
            mise_a_jour: EtatMiseAJour::Prete("1.3.0".into()),
            dossier_installateurs: "D:\\Logiciels\\Installateurs".into(),
            dossier_fixe: false,
            nouveautes: vec!["Une nouveauté inventée.".into()],
            credits: vec![interface_commune::Credit {
                nom: "egui".into(),
                licence: "MIT OR Apache-2.0".into(),
            }],
            ..Default::default()
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
            titre: self
                .titre
                .lock()
                .expect("verrou")
                .clone()
                .unwrap_or_else(|| "Bureau du salon".into()),
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
        egui::Shape::Text(t) => textes.push((
            t.galley.text().to_string(),
            egui::Rect::from_min_size(t.pos, t.galley.size()),
        )),
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
            // La version est petite, à droite du gros titre, sur la même ligne ; elle n'est pas
            // répétée ailleurs.
            let (_, nom) = textes
                .iter()
                .find(|(t, _)| t == "Bureau du salon")
                .expect("le titre");
            let (_, version) = textes.iter().find(|(t, _)| t == "1.2.3").expect("la version");
            assert!(
                version.min.x > nom.max.x,
                "{ppp} : à droite du titre ({version:?} {nom:?})"
            );
            assert!(version.height() < nom.height(), "{ppp} : plus petite");
            assert!(
                version.center().y > nom.center().y && version.max.y <= nom.max.y + 1.0,
                "{ppp} : posée sur la ligne de base ({version:?} {nom:?})"
            );
            assert!(!textes.iter().any(|(t, _)| t.contains("Démo 1.2.3")), "{ppp}");
            // La roue des Paramètres est en haut à droite, sur la ligne du titre, au centre de
            // sa hauteur ; rien en dessous des réglages (plus de ligne du bas).
            let roue = textes
                .iter()
                .find(|(t, _)| t == interface_commune::icones::REGLAGES.to_string().as_str())
                .map(|(_, r)| *r)
                .expect("la roue");
            assert!(
                roue.center().x > taille.x - 36.0,
                "{ppp} : à droite ({roue:?} {taille:?})"
            );
            assert!(
                (roue.center().y - nom.center().y).abs() < 6.0,
                "{ppp} : sur la ligne du titre ({roue:?} {nom:?})"
            );
            let dernier = textes.iter().map(|(_, r)| r.max.y).fold(0.0, f32::max);
            assert!(
                roue.max.y < dernier - 100.0,
                "{ppp} : pas en bas ({roue:?} {dernier})"
            );
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
        let textes = image(egui::vec2(920.0, 680.0), 1.0, |ui, p| s.dessiner(ui, p, &demo));
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
    for (cle, avant) in s.annulation.annuler(std::time::Instant::now()) {
        demo.changer(&cle, avant);
    }
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
    /// Un rond plein (centre, rayon).
    Rond(egui::Pos2, f32),
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
        egui::Shape::Circle(c) => v.push(Dessine::Rond(c.center, c.radius)),
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
                egui::vec2(920.0, 680.0),
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
                _ => None,
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
            _ => None,
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
        (y("À propos") - y("Mise à jour") - pas).abs() < 0.6 && y("À propos") < 680.0 - 150.0,
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

/// La page Général montre les cartes propres du logiciel, après les réglages communs et
/// avant « Journal » ; un dossier fixe n'a pas de bouton « Choisir… ».
#[test]
fn carte_propre_dans_general_et_dossier_fixe() {
    if polices::polices().is_err() {
        return;
    }
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("general"));
    let v = image_parametres(&mut s, &demo);
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    assert!(y("Notifications") < y("Carte propre"));
    assert!(y("Carte propre") < y("Journal"));
    s.aller_a("mise_a_jour");
    let v = image_parametres(&mut s, &demo);
    assert!(texte(&v, "Choisir…").is_some());
    let fixe = DossierFixe(Demo::new());
    let mut s = Parametres::new(&fixe, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &fixe);
    assert!(texte(&v, "Dossier des installateurs").is_some());
    assert!(texte(&v, "Choisir…").is_none());
}

/// Un logiciel dont le dossier des installateurs ne se choisit pas.
struct DossierFixe(Demo);

impl Reglages for DossierFixe {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        Communs {
            dossier_fixe: true,
            ..self.0.communs()
        }
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Un point d'alerte sur la ligne « Mise à jour » quand une version est prête (par défaut),
/// sur une page propre quand le logiciel le demande, nulle part ailleurs.
#[test]
fn point_d_alerte_dans_la_navigation() {
    if polices::polices().is_err() {
        return;
    }
    let ronds = |l: &dyn Reglages| {
        let mut s = Parametres::new(l, None);
        let v = image_parametres(&mut s, l);
        let lignes: Vec<(String, f32)> = ["Écrans", "Dispositions", "Général", "Mise à jour", "À propos"]
            .iter()
            .map(|t| (t.to_string(), texte(&v, t).expect(t).0.center().y))
            .collect();
        v.iter()
            .filter_map(|d| match d {
                Dessine::Rond(c, r) if (*r - interface_commune::parametres::RAYON_ALERTE).abs() < 0.01 => {
                    lignes
                        .iter()
                        .find(|(_, y)| (c.y - y).abs() < 4.0)
                        .map(|(t, _)| t.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let demo = Demo::new();
    assert_eq!(ronds(&demo), ["Mise à jour"]);
    let alerte = Alerte(Demo::new());
    assert_eq!(ronds(&alerte), ["Dispositions"]);
}

/// Un logiciel qui signale une de ses pages (et pas la mise à jour).
struct Alerte(Demo);

impl Reglages for Alerte {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        self.0.pages()
    }

    fn communs(&self) -> Communs {
        self.0.communs()
    }

    fn alerte(&self, cle: &str) -> bool {
        cle == "dispositions"
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Le petit menu a les mesures de Windows 11 ([`interface_commune::mesures`]) : lignes de 32
/// points séparées de 2, pour tous les logiciels.
#[test]
fn petit_menu_aux_mesures_communes() {
    use interface_commune::mesures::{MENU_ECART, MENU_LIGNE};
    if polices::polices().is_err() {
        return;
    }
    let demo = Demo::new();
    let textes = image(egui::vec2(800.0, 1200.0), 1.0, |ui, p| {
        surface::dessiner(ui, p, Surface::Menu, &demo);
    });
    let y = |t: &str| {
        textes
            .iter()
            .find(|(x, _)| x == t)
            .map(|(_, r)| r.center().y)
            .expect(t)
    };
    let pas = y("Rechercher une mise à jour") - y("Paramètres");
    assert!((pas - MENU_LIGNE - MENU_ECART).abs() < 0.6, "pas : {pas}");
}

/// Un logiciel qui change les pages communes : ses propres `Communs`, sa phrase pour
/// « Rétablir », et une carte propre dans Mise à jour et dans À propos.
struct Variante(Demo, Communs);

impl Reglages for Variante {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, cle: &str) {
        self.0.action(cle);
    }

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        self.1.clone()
    }

    fn changer(&self, cle: &str, valeur: Valeur) {
        self.0.changer(cle, valeur);
    }

    fn dessiner_mise_a_jour(&self, ui: &mut egui::Ui, page: &mut Contexte) {
        dessin::carte_info(ui, page.palette, '\u{E7F4}', "Carte de mise à jour", "Inventée.");
    }

    fn dessiner_a_propos(&self, ui: &mut egui::Ui, page: &mut Contexte) {
        dessin::carte_info(ui, page.palette, '\u{E7F4}', "Carte à propos", "Inventée.");
        // Un groupe de changements d'un seul geste : une seule bulle « Annuler ».
        if page.sous_page.is_none() {
            page.changer_groupe(
                "Disposition",
                vec![
                    ("gauche".into(), Valeur::Nombre(1.0), Valeur::Nombre(2.0)),
                    ("droite".into(), Valeur::Nombre(3.0), Valeur::Nombre(4.0)),
                ],
            );
            page.ouvrir_sous_page(Some("Fait"));
        }
    }

    fn retablir_explication(&self) -> Option<String> {
        Some("Les dispositions inventées sont effacées.".into())
    }
}

/// Démarrage grisé avec sa raison, et la phrase de « Rétablir » fournie par le logiciel.
#[test]
fn demarrage_grise_et_retablir_du_logiciel() {
    if polices::polices().is_err() {
        return;
    }
    let base = Demo::new().communs();
    let l = Variante(
        Demo::new(),
        Communs {
            demarrage_bloque: Some("Refusé par une règle inventée.".into()),
            ..base
        },
    );
    let mut s = Parametres::new(&l, Some("general"));
    let v = image_parametres(&mut s, &l);
    assert!(texte(&v, "Refusé par une règle inventée.").is_some());
    assert!(texte(&v, Textes::default().demarrer_explication).is_none());
    s.confirmer_retablir = true;
    let v = image_parametres(&mut s, &l);
    assert!(texte(&v, "Les dispositions inventées sont effacées.").is_some());
}

/// Journal : l'interrupteur « Journal détaillé » et « Effacer le journal » ne se montrent que
/// si le logiciel les donne ; l'effacement passe par une question de confirmation.
#[test]
fn journal_detaille_et_effacer_avec_confirmation() {
    if polices::polices().is_err() {
        return;
    }
    let t = Textes::default();
    let base = Demo::new().communs();
    let sans = Variante(
        Demo::new(),
        Communs {
            journal_detaille: None,
            journal_effacable: false,
            ..base.clone()
        },
    );
    let mut s = Parametres::new(&sans, Some("general"));
    let v = image_parametres(&mut s, &sans);
    assert!(texte(&v, t.journal_detaille).is_none());
    assert!(texte(&v, t.effacer_le_journal).is_none());
    let avec = Variante(
        Demo::new(),
        Communs {
            journal_detaille: Some(false),
            journal_effacable: true,
            ..base
        },
    );
    let mut s = Parametres::new(&avec, Some("general"));
    let v = image_parametres(&mut s, &avec);
    assert!(texte(&v, t.journal_detaille).is_some());
    assert!(texte(&v, t.effacer_le_journal).is_some());
    assert!(texte(&v, t.oui_effacer).is_none());
    s.confirmer_effacer = true;
    let v = image_parametres(&mut s, &avec);
    assert!(texte(&v, t.effacer_journal_question).is_some());
    assert!(texte(&v, t.oui_effacer).is_some());
}

/// Notifications : le bouton « Essayer » ne se montre que si le logiciel le demande, et
/// seulement avec la carte « Notifications » ; sans lui, la page ne change pas.
#[test]
fn notifications_essayer_seulement_si_demande() {
    if polices::polices().is_err() {
        return;
    }
    let t = Textes::default();
    let base = Demo::new().communs();
    let essai = |notifications: Option<bool>, notifications_essai: bool| {
        let l = Variante(
            Demo::new(),
            Communs {
                notifications,
                notifications_essai,
                ..base.clone()
            },
        );
        let mut s = Parametres::new(&l, Some("general"));
        let v = image_parametres(&mut s, &l);
        (
            texte(&v, t.notifications).is_some(),
            texte(&v, t.essayer).is_some(),
        )
    };
    assert_eq!(essai(Some(false), false), (true, false));
    assert_eq!(essai(Some(false), true), (true, true));
    assert_eq!(essai(Some(true), true), (true, true));
    assert_eq!(essai(None, true), (false, false));
}

/// Mise à jour : l'interrupteur des recherches, « Automatique », la carte propre après le
/// dossier ; recherches coupées : une information, ni recherche ni dossier.
#[test]
fn mise_a_jour_recherches_et_dossier_automatique() {
    if polices::polices().is_err() {
        return;
    }
    let base = Demo::new().communs();
    let l = Variante(
        Demo::new(),
        Communs {
            recherches: Some(true),
            dossier_automatique: Some(false),
            mise_a_jour: EtatMiseAJour::Information("Dossier introuvable.".into()),
            ..base.clone()
        },
    );
    let mut s = Parametres::new(&l, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &l);
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    assert!(y("Chercher les mises à jour") < y("Dossier des installateurs"));
    assert!(y("Dossier des installateurs") < y("Carte de mise à jour"));
    assert!(texte(&v, "Automatique").is_some());
    assert!(texte(&v, "Dossier introuvable.").is_some());
    assert!(texte(&v, "Rechercher une mise à jour").is_some());
    let coupee = Variante(
        Demo::new(),
        Communs {
            recherches: Some(false),
            mise_a_jour: EtatMiseAJour::Coupee,
            ..base
        },
    );
    let mut s = Parametres::new(&coupee, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &coupee);
    assert!(texte(&v, "Recherche des mises à jour coupée").is_some());
    assert!(texte(&v, "Rechercher une mise à jour").is_none());
    assert!(texte(&v, "Dossier des installateurs").is_none());
    // L'explication passe par les espaces insécables (« Coupé : ») : on cherche sa fin.
    assert!(v.iter().any(|d| matches!(
        d,
        Dessine::Texte { texte, .. } if texte.ends_with("ne cherche pas de nouvelle version.")
    )));
}

/// À propos : « Voir les licences », la carte propre avant les crédits, un long crédit qui
/// passe à la ligne, et un groupe de changements défait d'un seul « Annuler ».
#[test]
fn a_propos_licences_credits_et_groupe() {
    if polices::polices().is_err() {
        return;
    }
    let base = Demo::new().communs();
    let long = "Une bibliothèque inventée au nom vraiment très long, bien plus large que la page".repeat(2);
    let l = Variante(
        Demo::new(),
        Communs {
            fichier_licences: true,
            credits: vec![interface_commune::Credit {
                nom: long.clone(),
                licence: "MIT".into(),
            }],
            ..base
        },
    );
    let mut s = Parametres::new(&l, Some("a_propos"));
    let v = image_parametres(&mut s, &l);
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    assert!(y("Licence") < y("Carte à propos"));
    assert!(y("Carte à propos") < y("Crédits"));
    assert!(texte(&v, "Voir les licences").is_some());
    let (credit, _) = texte(&v, &long).expect("crédit");
    assert!(credit.right() <= 920.0, "le crédit déborde : {credit:?}");
    // Les deux changements du groupe sont partis au logiciel ; « Annuler » remet les deux.
    let faits = l.0.changes.lock().expect("verrou").clone();
    assert!(faits.contains(&("gauche".into(), Valeur::Nombre(2.0))));
    assert!(faits.contains(&("droite".into(), Valeur::Nombre(4.0))));
    let defaits = s.annulation.annuler(std::time::Instant::now());
    assert_eq!(
        defaits,
        vec![
            ("gauche".to_string(), Valeur::Nombre(1.0)),
            ("droite".to_string(), Valeur::Nombre(3.0))
        ]
    );
}

/// Un logiciel à brouillon : ses réglages se modifient d'un bloc (un `Vec`), avec un pied de
/// page et une fermeture qui peut être refusée.
struct AvecBrouillon {
    identite: Identite,
    brouillon: Mutex<Vec<i32>>,
    /// La page change le brouillon à la prochaine image (une seule fois).
    modifier: AtomicBool,
    enregistrement_ok: AtomicBool,
    fermetures: Mutex<Vec<bool>>,
}

impl Reglages for AvecBrouillon {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("p", "Page du brouillon", '\u{E7F4}')]
    }

    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        ui.label("Contenu du brouillon");
        if self.modifier.swap(false, Ordering::Relaxed) {
            self.brouillon.lock().expect("verrou").push(2);
            page.brouillon_modifie("Réglages");
        }
    }

    fn dessiner_pied(&self, ui: &mut egui::Ui, _page: &mut Contexte) {
        ui.label("Réglage pas enregistré");
    }

    fn instantane(&self) -> Option<interface_commune::Instantane> {
        Some(interface_commune::Instantane::new(
            self.brouillon.lock().expect("verrou").clone(),
        ))
    }

    fn remettre(&self, instantane: &interface_commune::Instantane) {
        if let Some(v) = instantane.get::<Vec<i32>>() {
            self.brouillon.lock().expect("verrou").clone_from(v);
        }
    }

    fn peut_fermer(&self, imposee: bool) -> bool {
        self.fermetures.lock().expect("verrou").push(imposee);
        self.enregistrement_ok.load(Ordering::Relaxed)
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Un brouillon entier : la page le modifie, la bulle « Annuler » le remet d'un coup à l'état
/// d'avant l'image ; le pied de page est sous toutes les pages, communes comprises ; la
/// fermeture passe par le logiciel, qui peut la refuser.
#[test]
fn brouillon_entier_pied_de_page_et_fermeture() {
    if polices::polices().is_err() {
        return;
    }
    let l = AvecBrouillon {
        identite: identite(),
        brouillon: Mutex::new(vec![1]),
        modifier: AtomicBool::new(true),
        enregistrement_ok: AtomicBool::new(false),
        fermetures: Mutex::default(),
    };
    let mut s = Parametres::new(&l, Some("p"));
    let v = image_parametres(&mut s, &l);
    assert!(texte(&v, "Contenu du brouillon").is_some());
    assert!(
        texte(&v, "Réglage pas enregistré").is_some(),
        "pied sous une page propre"
    );
    assert_eq!(*l.brouillon.lock().expect("verrou"), vec![1, 2]);
    // La bulle est celle du brouillon : seul `annuler_brouillon` la défait, d'un coup.
    assert!(s.annulation.annuler(std::time::Instant::now()).is_empty());
    let avant = s
        .annulation
        .annuler_brouillon(std::time::Instant::now())
        .expect("bulle du brouillon");
    l.remettre(&avant);
    assert_eq!(*l.brouillon.lock().expect("verrou"), vec![1]);
    // Le pied est aussi sous les pages communes.
    let mut g = Parametres::new(&l, Some("general"));
    let v = image_parametres(&mut g, &l);
    assert!(texte(&v, "Réglage pas enregistré").is_some(), "pied sous Général");
    // Fermeture : refusée tant que l'enregistrement échoue, acceptée ensuite.
    let mut contenu = interface_commune::parametres::fenetre::ContenuParametres::new(
        std::sync::Arc::new(AvecBrouillon {
            identite: identite(),
            brouillon: Mutex::default(),
            modifier: AtomicBool::new(false),
            enregistrement_ok: AtomicBool::new(false),
            fermetures: Mutex::default(),
        }),
        Default::default(),
        None,
    );
    use interface_commune::autres_fenetres::Contenu;
    assert!(!contenu.peut_fermer(false), "refusée");
    assert!(
        !contenu.peut_fermer(true),
        "le refus est rendu tel quel, la fenêtre l'ignore"
    );
}

/// Les captures pour la documentation : le volet, le petit menu et chaque page des Paramètres,
/// en clair et en sombre, en PNG, sans aucune fenêtre. Le dossier peut être choisi par
/// `CAPTURES_VERS` (pour les regarder à la main) ; sinon un dossier temporaire.
#[test]
fn captures_pour_la_documentation() {
    use interface_commune::capture::{self, Options};
    if polices::polices().is_err() {
        // Sans les polices de Windows, une capture serait sans texte : elle le refuse.
        assert!(capture::volet(&Demo::new(), Options::default()).is_err());
        return;
    }
    let demo = Demo::new();
    let clair = capture::volet(&demo, Options::default()).expect("volet");
    let sombre = capture::volet(
        &demo,
        Options {
            sombre: true,
            ..Options::default()
        },
    )
    .expect("volet sombre");
    assert_eq!((clair.largeur, clair.hauteur), (sombre.largeur, sombre.hauteur));
    assert!(
        (LARGEUR as u32..=LARGEUR_MAX as u32).contains(&clair.largeur),
        "{}",
        clair.largeur
    );
    assert_ne!(clair.rvba, sombre.rvba, "le thème change l'image");
    // Le texte est dessiné : plus d'une couleur dans l'image.
    let couleurs: std::collections::HashSet<[u8; 4]> =
        clair.rvba.as_chunks::<4>().0.iter().copied().collect();
    assert!(couleurs.len() > 8, "{} couleurs", couleurs.len());
    // L'échelle 2 donne deux fois plus de pixels.
    let net = capture::volet(
        &demo,
        Options {
            echelle: 2.0,
            ..Options::default()
        },
    )
    .expect("volet net");
    assert_eq!(net.largeur, clair.largeur * 2);

    let dossier = std::env::var_os("CAPTURES_VERS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("interface-commune-captures-{}", std::process::id()))
        });
    let faites = capture::toutes(&demo, &dossier, 1.0).expect("captures");
    // Volet + petit menu + Accueil, 2 pages propres, Général, Mise à jour, À propos : 8 × 2 thèmes.
    assert_eq!(faites.len(), 2 * (2 + 6), "{faites:?}");
    for f in &faites {
        let octets = std::fs::read(f).expect("fichier");
        assert_eq!(&octets[..8], b"\x89PNG\r\n\x1a\n", "{f:?}");
    }
    assert!(
        faites
            .iter()
            .any(|f| f.ends_with("parametres-mise-a-jour-sombre.png"))
    );
    if std::env::var_os("CAPTURES_VERS").is_none() {
        let _ = std::fs::remove_dir_all(&dossier);
    }
}

/// Un logiciel aux nombreux appareils, dont les images sont des textures chargées dans le
/// contexte egui du volet (comme celles d'un logiciel de batteries).
struct Appareils {
    identite: Identite,
    appels: std::sync::atomic::AtomicUsize,
    texture: Mutex<Option<egui::TextureHandle>>,
    /// Le nombre d'appareils montrés (il peut changer entre deux images).
    nombre: std::sync::atomic::AtomicUsize,
    /// Un pied fixe (« Rechercher les appareils ») sous la liste.
    pied: AtomicBool,
}

impl Appareils {
    fn avec(nombre: usize) -> Self {
        Appareils {
            identite: identite(),
            appels: Default::default(),
            texture: Mutex::default(),
            nombre: nombre.into(),
            pied: AtomicBool::new(false),
        }
    }
}

impl Reglages for Appareils {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

impl Logiciel for Appareils {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Démo".into(),
            ligne: "30 appareils connectés".into(),
            detail: None,
        }
    }

    fn briques_avec_contexte(&self, ctx: &egui::Context) -> Vec<interface_commune::briques::Brique> {
        use interface_commune::briques::{Brique, ImageAppareil, PlusAppareil};
        self.appels.fetch_add(1, Ordering::AcqRel);
        let mut t = self.texture.lock().expect("verrou");
        let texture = t
            .get_or_insert_with(|| {
                ctx.load_texture(
                    "appareil",
                    egui::ColorImage::new([16, 16], vec![egui::Color32::WHITE; 256]),
                    egui::TextureOptions::LINEAR,
                )
            })
            .id();
        (0..self.nombre.load(Ordering::Acquire))
            .map(|i| Brique::Appareil {
                cle: None,
                glyphe: '\u{E7F4}',
                nom: format!("Appareil {i}"),
                pourcentage: Some(0.5),
                sous_ligne: Some("Vu à l'instant".into()),
                plus: PlusAppareil {
                    image: Some(ImageAppareil {
                        texture,
                        cote: 16.0,
                        fond: None,
                    }),
                    ..Default::default()
                },
            })
            .collect()
    }

    fn briques_du_pied(&self) -> Vec<interface_commune::briques::Brique> {
        if self.pied.load(Ordering::Acquire) {
            vec![interface_commune::briques::Brique::Action {
                cle: "rechercher".into(),
                glyphe: '\u{E72C}',
                texte: PIED.into(),
            }]
        } else {
            Vec::new()
        }
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}

/// Le texte du pied fixe des essais.
const PIED: &str = "Rechercher les appareils";

/// Le volet reçoit le contexte egui pour ses briques (une fois par image) ; sans hauteur
/// maximale il est aussi haut que son contenu (comme avant) ; avec, il ne la dépasse pas et
/// garde en haut le titre, sa version et la roue dentée, les briques défilant dessous.
#[test]
fn volet_borne_et_briques_avec_contexte() {
    let l = Appareils::avec(30);
    let mut libre = egui::Vec2::ZERO;
    image(egui::vec2(800.0, 4000.0), 1.0, |ui, p| {
        libre = surface::dessiner(ui, p, Surface::Volet, &l).1;
    });
    assert_eq!(l.appels.load(Ordering::Acquire), 3, "une fois par image");
    assert!(
        l.texture.lock().expect("verrou").is_some(),
        "texture chargée dans le contexte"
    );
    assert!(libre.y > 1000.0, "30 appareils : {libre:?}");
    for max in [400.0, 600.0] {
        let mut borne = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 4000.0), 1.5, |ui, p| {
            borne = surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(max)).1;
        });
        assert!(borne.y <= max, "{max} : {borne:?}");
        assert!(borne.y >= max - 60.0, "la place est remplie : {borne:?}");
        assert_eq!(borne.x, libre.x, "même largeur");
        if polices::polices().is_ok() {
            let (_, version) = textes
                .iter()
                .find(|(t, _)| t == "1.2.3")
                .expect("la version reste en haut, avec le titre");
            assert!(version.max.y <= borne.y, "{version:?} {borne:?}");
            let haut = textes
                .iter()
                .filter(|(t, _)| t.starts_with("Appareil "))
                .map(|(_, r)| r.min.y)
                .fold(f32::MAX, f32::min);
            assert!(haut < max, "les premiers appareils sont visibles");
        }
    }
    // Une hauteur qui suffit : rien ne change.
    let mut large = egui::Vec2::ZERO;
    image(egui::vec2(800.0, 4000.0), 1.0, |ui, p| {
        large = surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(libre.y + 100.0)).1;
    });
    assert!((large.y - libre.y).abs() <= 1.0, "{large:?} {libre:?}");
    // Le petit menu n'est jamais borné.
    let mut menu = egui::Vec2::ZERO;
    image(egui::vec2(800.0, 4000.0), 1.0, |ui, p| {
        menu = surface::dessiner_borne(ui, p, Surface::Menu, &l, Some(10.0)).1;
    });
    assert!(menu.y > 10.0);
}

/// Une fenêtre qui suit la taille mesurée, comme celle du volet : à chaque image elle a la
/// taille rendue à l'image d'avant. Rend la taille voulue après `images` images.
fn fenetre_qui_suit(
    ctx: &egui::Context,
    p: &Palette,
    depart: egui::Vec2,
    images: usize,
    dessin: impl FnMut(&mut egui::Ui, &Palette) -> egui::Vec2,
) -> egui::Vec2 {
    fenetre_qui_suit_et_formes(ctx, p, depart, images, dessin).0
}

/// Comme [`fenetre_qui_suit`], avec aussi les formes de la dernière image.
fn fenetre_qui_suit_et_formes(
    ctx: &egui::Context,
    p: &Palette,
    depart: egui::Vec2,
    images: usize,
    mut dessin: impl FnMut(&mut egui::Ui, &Palette) -> egui::Vec2,
) -> (egui::Vec2, Vec<egui::epaint::ClippedShape>) {
    let mut fenetre = depart;
    let mut voulue = depart;
    let mut formes = Vec::new();
    for _ in 0..images {
        let mut entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, fenetre)),
            ..Default::default()
        };
        entree
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let mut s = ctx.run_ui(entree, |ui| voulue = dessin(ui, p));
        s.textures_delta.clear();
        formes = s.shapes;
        fenetre = voulue;
    }
    (voulue, formes)
}

/// Les textes (avec leur place) et les rectangles dessinés (place, remplissage) d'une image.
fn textes_et_rects(formes: &[egui::epaint::ClippedShape]) -> (Vec<(String, egui::Rect)>, Vec<egui::Rect>) {
    fn parcourir(s: &egui::Shape, textes: &mut Vec<(String, egui::Rect)>, rects: &mut Vec<egui::Rect>) {
        match s {
            egui::Shape::Vec(v) => v.iter().for_each(|s| parcourir(s, textes, rects)),
            egui::Shape::Text(t) => textes.push((
                t.galley.text().to_string(),
                egui::Rect::from_min_size(t.pos, t.galley.size()),
            )),
            egui::Shape::Rect(r) if r.fill.a() > 0 => rects.push(r.rect),
            _ => {}
        }
    }
    let (mut textes, mut rects) = (Vec::new(), Vec::new());
    for c in formes {
        parcourir(&c.shape, &mut textes, &mut rects);
    }
    (textes, rects)
}

/// Une barre de défilement : un rectangle étroit et haut, collé au bord droit du volet.
fn barres(rects: &[egui::Rect], largeur: f32) -> usize {
    rects
        .iter()
        .filter(|r| r.width() <= 12.0 && r.height() >= 20.0 && r.max.x >= largeur - 14.0)
        .count()
}

/// Le pied du volet (« Rechercher les appareils ») reste fixe, visible, en bas : seule la
/// liste défile, avec une barre visible quand elle dépasse ; la fenêtre regrandit jusqu'à la
/// hauteur maximale et le pied suit la liste quand elle est courte.
#[test]
fn volet_borne_pied_fixe_et_barre_visible() {
    if polices::polices().is_err() {
        return;
    }
    let l = Appareils::avec(2);
    l.pied.store(true, Ordering::Release);
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    let a = apparence::Apparence::de(Default::default(), Some(false));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let mesure = |depart: egui::Vec2| {
        let (t, formes) = fenetre_qui_suit_et_formes(&ctx, &p, depart, 8, |ui, p| {
            surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(600.0)).1
        });
        let (textes, rects) = textes_et_rects(&formes);
        let pied = textes
            .iter()
            .find(|(t, _)| t == PIED)
            .map(|(_, r)| *r)
            .expect("le pied est dessiné");
        let dernier = textes
            .iter()
            .filter(|(t, _)| t.starts_with("Appareil "))
            .map(|(_, r)| r.max.y)
            .fold(0.0, f32::max);
        (t, pied, dernier, barres(&rects, t.x))
    };
    // Deux appareils : le pied suit la liste, pas de barre.
    let (petit, pied, dernier, barre) = mesure(egui::vec2(400.0, 800.0));
    assert!(
        pied.min.y > dernier && pied.max.y <= petit.y,
        "{pied:?} {dernier} {petit:?}"
    );
    assert_eq!(barre, 0, "pas de barre quand tout tient {petit:?} {pied:?}");
    // Six appareils (la fenêtre regrandit), puis huit : le pied reste visible dans la fenêtre.
    l.nombre.store(6, Ordering::Release);
    let (six, pied6, dernier6, barre6) = mesure(petit);
    assert!(six.y > petit.y + 100.0 && six.y <= 600.0, "{petit:?} {six:?}");
    assert!(
        pied6.min.y > dernier6 && pied6.max.y <= six.y,
        "{pied6:?} {six:?}"
    );
    assert_eq!(barre6, 0);
    // Trente appareils : la fenêtre est à la borne, le pied est fixe tout en bas, la barre se voit.
    l.nombre.store(30, Ordering::Release);
    let (plein, pied30, _, barre30) = mesure(six);
    assert!(plein.y <= 600.0 && plein.y >= 600.0 - 60.0, "{plein:?}");
    assert!(
        pied30.max.y <= plein.y && pied30.max.y >= plein.y - 30.0,
        "{pied30:?} {plein:?}"
    );
    assert!(barre30 >= 1, "la barre de défilement doit se voir ({barre30})");
    // Huit appareils, comme sur le portable : même chose, jamais coupé.
    l.nombre.store(8, Ordering::Release);
    let (huit, pied8, _, _) = mesure(plein);
    assert!(huit.y <= 600.0 && pied8.max.y <= huit.y, "{pied8:?} {huit:?}");
    // Sans pied, rien ne change : plus de texte du pied.
    l.pied.store(false, Ordering::Release);
    let (sans, formes) = fenetre_qui_suit_et_formes(&ctx, &p, huit, 8, |ui, p| {
        surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(600.0)).1
    });
    assert!(!textes_et_rects(&formes).0.iter().any(|(t, _)| t == PIED));
    assert!(sans.y >= huit.y - 60.0 - 1.0);
}

/// Le contenu grandit après l'ouverture (des appareils déconnectés apparaissent) : la fenêtre,
/// qui avait la taille du petit contenu, doit grandir avec lui, jusqu'à la hauteur maximale
/// (elle ne reste pas bloquée à sa taille d'avant, avec une barre de défilement), et rapetisser
/// quand le contenu rapetisse. Le petit menu et le volet sans borne font de même.
#[test]
fn volet_borne_grandit_avec_son_contenu() {
    let l = Appareils::avec(2);
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    let a = apparence::Apparence::de(Default::default(), Some(false));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let mesure = |max: Option<f32>, depart: egui::Vec2| {
        fenetre_qui_suit(&ctx, &p, depart, 6, |ui, p| {
            surface::dessiner_borne(ui, p, Surface::Volet, &l, max).1
        })
    };
    // Deux appareils : le volet tient largement sous la borne.
    let petit = mesure(Some(600.0), egui::vec2(400.0, 800.0));
    assert!(petit.y < 300.0, "{petit:?}");
    // Six appareils (toujours sous la borne) : la fenêtre grandit avec le contenu.
    l.nombre.store(6, Ordering::Release);
    let grand = mesure(Some(600.0), petit);
    let libre = mesure(None, egui::vec2(petit.x, 4000.0));
    assert!(
        grand.y > petit.y + 100.0,
        "n'a pas grandi : {petit:?} -> {grand:?}"
    );
    assert!(
        grand.y <= 600.0 && (grand.y - libre.y).abs() <= 1.0,
        "doit valoir le contenu {libre:?} : {grand:?}"
    );
    // Trente appareils : elle grandit jusqu'à la borne, et pas au-delà.
    l.nombre.store(30, Ordering::Release);
    let plein = mesure(Some(600.0), grand);
    assert!(plein.y <= 600.0 && plein.y >= 600.0 - 60.0, "{plein:?}");
    // Le contenu rapetisse : la fenêtre aussi.
    l.nombre.store(2, Ordering::Release);
    let retour = mesure(Some(600.0), plein);
    assert!((retour.y - petit.y).abs() <= 1.0, "{petit:?} {retour:?}");
    // Sans borne et petit menu : la fenêtre grandit aussi (le contenu déborde, il est mesuré).
    l.nombre.store(6, Ordering::Release);
    let sans = mesure(None, petit);
    assert!((sans.y - libre.y).abs() <= 1.0, "{sans:?} {libre:?}");
    let menu = fenetre_qui_suit(&ctx, &p, egui::vec2(200.0, 40.0), 6, |ui, p| {
        surface::dessiner_borne(ui, p, Surface::Menu, &l, Some(10.0)).1
    });
    assert!(menu.y > 100.0, "{menu:?}");
}

/// Les raccourcis des briques s'affichent en toutes lettres, comme Windows en français :
/// jamais de symbole (⇧, ⌃, ⌘, →), quelle que soit l'écriture donnée par le logiciel.
#[test]
fn raccourcis_en_toutes_lettres() {
    use interface_commune::briques::{self, Brique};
    let liste = vec![
        Brique::Choix {
            cle: "a".into(),
            glyphe: None,
            texte: "Premier".into(),
            coche: false,
            raccourci: Some("ctrl+shift+k".into()),
        },
        Brique::Choix {
            cle: "b".into(),
            glyphe: None,
            texte: "Deuxième".into(),
            coche: false,
            raccourci: Some("⌃⇧→".into()),
        },
        Brique::Raccourcis(vec![
            ("Écran suivant".into(), "Win+Escape".into()),
            ("Écran précédent".into(), "Ctrl+Alt+←".into()),
        ]),
    ];
    let textes = image(egui::vec2(400.0, 400.0), 1.0, |ui, p| {
        briques::dessiner(ui, p, &liste);
    });
    if polices::polices().is_err() {
        return;
    }
    let vus: Vec<&str> = textes.iter().map(|(t, _)| t.as_str()).collect();
    for attendu in [
        "Ctrl+Maj+K",
        "Ctrl+Maj+Flèche droite",
        "Win+Échap",
        "Ctrl+Alt+Flèche gauche",
    ] {
        assert!(vus.contains(&attendu), "« {attendu} » absent de {vus:?}");
    }
    for t in vus {
        assert!(
            !t.contains(['⇧', '⌃', '⌥', '⌘', '→', '←']),
            "symbole dans « {t} »"
        );
    }
}

/// Un nom de logiciel long, avec sa version, ne passe jamais sous la roue des Paramètres : le
/// titre se coupe par « … », la version reste entière, et la roue garde sa place à droite.
#[test]
fn titre_long_ne_chevauche_pas_la_roue() {
    if polices::polices().is_err() {
        return;
    }
    let mut demo = Demo::new();
    demo.identite.version = "12.34.56".into();
    let long = "Un nom de logiciel vraiment beaucoup trop long pour tenir sur une seule ligne du volet";
    *demo.titre.lock().expect("verrou") = Some(long.into());
    let roue_texte = interface_commune::icones::REGLAGES.to_string();
    for ppp in [1.0, 1.25, 1.5, 2.0] {
        let mut taille = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 1200.0), ppp, |ui, p| {
            taille = surface::dessiner(ui, p, Surface::Volet, &demo).1;
        });
        let roue = textes
            .iter()
            .find(|(t, _)| *t == roue_texte)
            .map(|(_, r)| *r)
            .expect("la roue");
        let titre = textes
            .iter()
            .find(|(t, _)| t.starts_with("Un nom de logiciel"))
            .map(|(t, r)| (t.clone(), *r))
            .expect("le titre");
        let (_, version) = textes.iter().find(|(t, _)| t == "12.34.56").expect("la version");
        // Le bouton fait 36 de large, centré sur l'icône : son bord gauche.
        let bord = roue.center().x - 18.0;
        assert!(
            titre.1.max.x <= bord - 8.0 + 0.5,
            "{ppp} : titre sous la roue ({:?} {roue:?})",
            titre.1
        );
        assert!(
            version.max.x <= bord - 8.0 + 0.5,
            "{ppp} : version sous la roue ({version:?} {roue:?})"
        );
        assert!(
            version.min.x >= titre.1.max.x,
            "{ppp} : version à droite du titre"
        );
        assert!(
            roue.center().x > taille.x - 36.0,
            "{ppp} : la roue reste à droite"
        );
        assert!((LARGEUR..=LARGEUR_MAX).contains(&taille.x), "{ppp} : {taille:?}");
    }
}

/// Les choix illustrés : l'illustration a une largeur fixe, donc le texte et la sous-ligne
/// commencent au même endroit pour 1 ou 4 cases ; l'actif montre la coche et son étiquette ;
/// les pastilles montrent le nom et la mention.
#[test]
fn choix_illustres_alignes() {
    use interface_commune::briques::{self, Brique, Case, Pastille};
    if polices::polices().is_err() {
        return;
    }
    let c = |l| Case {
        lettre: Some(l),
        logo: None,
        couleur: Some(interface_commune::apparence::couleurs_pastilles()[0]),
        ..Default::default()
    };
    let choix = |cle: &str, texte: &str, n: usize, actif: bool| Brique::ChoixIllustre {
        cle: cle.into(),
        texte: texte.into(),
        sous_ligne: Some(format!("sous {texte}")),
        illustration: (0..n).map(|_| c('A')).collect(),
        raccourci: Some("ctrl+alt+shift+1".into()),
        actif,
        etiquette_active: Some("Affiché".into()),
    };
    let liste = vec![
        Brique::Pastilles {
            titre: Some("Ordinateurs".into()),
            pastilles: vec![Pastille {
                cle: None,
                lettre: 'F',
                logo: None,
                couleur: interface_commune::apparence::couleurs_pastilles()[1],
                nom: "Fixe".into(),
                mention: Some("ce PC".into()),
                glyphe: None,
                absent: false,
            }],
        },
        choix("un", "Un", 1, true),
        choix("quatre", "Quatre", 4, false),
    ];
    let textes = image(egui::vec2(400.0, 400.0), 1.0, |ui, p| {
        briques::dessiner(ui, p, &liste);
    });
    let x = |t: &str| {
        textes
            .iter()
            .find(|(s, _)| s == t)
            .unwrap_or_else(|| panic!("« {t} » absent de {textes:?}"))
            .1
            .min
            .x
    };
    assert!((x("Un") - x("Quatre")).abs() < 0.5, "texte décalé");
    assert!(
        (x("sous Un") - x("sous Quatre")).abs() < 0.5,
        "sous-ligne décalée"
    );
    assert!((x("Un") - x("sous Un")).abs() < 0.5);
    for attendu in ["Affiché", "Ctrl+Alt+Maj+1", "Fixe", "· ce PC", "Ordinateurs"] {
        x(attendu);
    }
}

/// Un logiciel dont les pages arrivent après l'ouverture des Paramètres (cas d'une interface
/// qui reçoit sa description d'un autre processus).
struct Tardif {
    identite: Identite,
    accueil: AtomicBool,
    pages: Mutex<Vec<Page>>,
}

impl Reglages for Tardif {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn accueil(&self) -> bool {
        self.accueil.load(Ordering::Relaxed)
    }

    fn pages(&self) -> Vec<Page> {
        self.pages.lock().expect("verrou").clone()
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Des pages qui arrivent après l'ouverture apparaissent dans la navigation à l'image
/// suivante ; la page choisie est gardée ; une page qui disparaît ramène à Général.
#[test]
fn pages_arrivees_apres_l_ouverture() {
    let l = Tardif {
        identite: identite(),
        accueil: AtomicBool::new(false),
        pages: Mutex::new(Vec::new()),
    };
    let cles = |s: &Parametres| s.pages.iter().map(|p| p.cle.clone()).collect::<Vec<_>>();
    let mut s = Parametres::new(&l, None);
    assert_eq!(cles(&s), ["general", "mise_a_jour", "a_propos"]);
    s.aller_a("a_propos");

    // Les pages arrivent : la navigation les montre, « À propos » reste choisie.
    l.accueil.store(true, Ordering::Relaxed);
    *l.pages.lock().expect("verrou") = vec![
        Page::new("voix", "Voix", '\u{E720}'),
        Page::new("lecture", "Lecture", '\u{E7BE}'),
    ];
    image_parametres(&mut s, &l);
    assert_eq!(
        cles(&s),
        ["accueil", "voix", "lecture", "general", "mise_a_jour", "a_propos"]
    );
    assert_eq!(s.cle(), "a_propos");

    // Une page propre choisie reste choisie quand une autre s'ajoute devant elle.
    s.aller_a("lecture");
    l.pages
        .lock()
        .expect("verrou")
        .insert(0, Page::new("dictee", "Dictée", '\u{E720}'));
    s.suivre(&l);
    assert_eq!(s.cle(), "lecture");
    assert_eq!(s.pages.len(), 7);

    // La page montrée disparaît : retour à Général.
    l.pages.lock().expect("verrou").retain(|p| p.cle != "lecture");
    s.suivre(&l);
    assert_eq!(s.cle(), "general");
    assert!(!cles(&s).contains(&"lecture".to_string()));
}
