//! Ce que partagent `tests/sans_fenetre.rs` (le volet et le petit menu) et
//! `tests/sans_fenetre_parametres.rs` (les Paramètres) : le logiciel inventé « Démo », le
//! logiciel « Seul » qui ne prend que les Paramètres, et les images egui sans fenêtre.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::{
    Communs, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Palette, Reglages, Textes, Valeur,
    apparence, dessin, menu, polices, textes,
};

pub fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

/// Un logo inventé : un carré bleu, à toutes les tailles.
pub fn logo(t: u32) -> Option<Vec<u8>> {
    Some([0x20, 0x60, 0xD0, 0xFF].repeat((t * t) as usize))
}

pub fn identite() -> Identite {
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
pub struct Demo {
    pub identite: Identite,
    pub anglais: AtomicBool,
    pub changes: Mutex<Vec<(String, Valeur)>>,
    pub actions: Mutex<Vec<String>>,
    /// Un gros titre à la place du nom « Bureau du salon » (essai des noms longs).
    pub titre: Mutex<Option<String>>,
}

impl Demo {
    pub fn new() -> Demo {
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

    // Le réglage rapide d'avant (`dessiner_rapides`, retiré au bloc F) devient une brique.
    fn briques(&self) -> Vec<interface_commune::briques::Brique> {
        vec![interface_commune::briques::Brique::Choix {
            cle: "ecran".into(),
            glyphe: Some('\u{E7F4}'),
            texte: "Écran seul".into(),
            coche: false,
            raccourci: Some("Ctrl+Alt+F1".into()),
        }]
    }

    fn actions_menu(&self) -> Vec<menu::Action> {
        vec![menu::Action::new("relire", '\u{E72C}', "Relire les écrans")]
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}

/// Un logiciel qui ne prend que les Paramètres : ni volet, ni petit menu, ni icône.
pub struct Seul {
    pub identite: Identite,
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
pub fn image(
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

pub fn formes(s: &egui::Shape, textes: &mut Vec<(String, egui::Rect)>) {
    match s {
        egui::Shape::Vec(v) => v.iter().for_each(|s| formes(s, textes)),
        egui::Shape::Text(t) => textes.push((
            t.galley.text().to_string(),
            egui::Rect::from_min_size(t.pos, t.galley.size()),
        )),
        _ => {}
    }
}

/// Ce qu'une image dessine : les textes (avec leur taille de police) et les images.
#[derive(Debug, Clone)]
pub enum Dessine {
    Texte {
        texte: String,
        rect: egui::Rect,
        taille: f32,
    },
    Image(egui::Rect),
    /// Un rond plein (centre, rayon).
    Rond(egui::Pos2, f32),
}

pub fn dessins(s: &egui::Shape, v: &mut Vec<Dessine>) {
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
pub fn image_parametres(s: &mut Parametres, l: &dyn Reglages) -> Vec<Dessine> {
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

pub fn texte<'a>(v: &'a [Dessine], cherche: &str) -> Option<(&'a egui::Rect, f32)> {
    v.iter().find_map(|d| match d {
        Dessine::Texte {
            texte, rect, taille, ..
        } if texte == cherche => Some((rect, *taille)),
        _ => None,
    })
}

/// Un texte dessiné, retrouvé sans tenir compte des espaces insécables posées au dessin.
pub fn texte_souple<'a>(v: &'a [Dessine], cherche: &str) -> Option<&'a egui::Rect> {
    let simple = |s: &str| s.replace(['\u{a0}', '\u{202f}'], " ");
    v.iter().find_map(|d| match d {
        Dessine::Texte { texte, rect, .. } if simple(texte) == cherche => Some(rect),
        _ => None,
    })
}
