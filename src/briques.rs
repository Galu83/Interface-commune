//! Les briques du volet : le logiciel décrit ses réglages rapides ([`Brique`]), la
//! bibliothèque les dessine, toutes aux mêmes marges, hauteurs et tailles de texte
//! ([`crate::mesures`]). Aucun logiciel ne dessine son volet lui-même (règles : interface
//! Windows 11, point 6).

use egui::{Align2, CornerRadius, FontId, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::Valeur;
use crate::apparence::couleur;
use crate::dessin::{self, icone, interrupteur, ligne};
use crate::icones;
use crate::mesures::{
    LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_ICONE_X, LIGNE_MARGE_DROITE, LIGNE_TEXTE_X, VOLET_BARRE,
    VOLET_ECART, VOLET_LIGNE, VOLET_RACCOURCI, VOLET_SECTION, VOLET_SEGMENTS, VOLET_SOUS_LIGNE,
};
use crate::polices::semibold;
use crate::theme::{Palette, TAILLE_PETIT, TAILLE_TEXTE};

/// Une brique du volet.
#[derive(Debug, Clone, PartialEq)]
pub enum Brique {
    /// Un réglage oui/non : un clic le change (`Reglages::changer`, `Valeur::Booleen`) sans
    /// fermer le volet.
    Interrupteur {
        cle: String,
        glyphe: Option<char>,
        texte: String,
        actif: bool,
    },
    /// Un choix parmi d'autres : coché ou non, un raccourci à droite ; un clic ferme le volet
    /// et rend `cle` (`Reglages::action`).
    Choix {
        cle: String,
        glyphe: Option<char>,
        texte: String,
        coche: bool,
        raccourci: Option<String>,
    },
    /// Un choix segmenté (2 à 4 options) : un clic change `cle` (`Valeur::Texte`, la clé de
    /// l'option) sans fermer le volet.
    Segments {
        cle: String,
        /// (clé, texte) de chaque option.
        options: Vec<(String, String)>,
        choisie: String,
    },
    /// Un appareil : icône, nom, pourcentage et sa barre, une sous-ligne. Avec une clé, un
    /// clic ferme le volet et la rend.
    Appareil {
        cle: Option<String>,
        glyphe: char,
        nom: String,
        pourcentage: Option<f32>,
        sous_ligne: Option<String>,
    },
    /// Un titre de section.
    Section(String),
    /// Une liste discrète de raccourcis : (ce qu'il fait, ses touches).
    Raccourcis(Vec<(String, String)>),
    /// Une action : un clic ferme le volet et rend `cle`.
    Action {
        cle: String,
        glyphe: char,
        texte: String,
    },
}

/// Ce que fait un clic sur une brique.
#[derive(Debug, Clone, PartialEq)]
pub enum Geste {
    /// Fermer le volet, puis faire cette action.
    Action(String),
    /// Changer ce réglage tout de suite, le volet reste ouvert.
    Changer(String, Valeur),
}

fn pourcent(v: f32) -> String {
    format!("{:.0} %", (v.clamp(0.0, 1.0) * 100.0))
}

/// La largeur qu'il faut aux briques pour tout montrer sur une ligne (points).
pub fn largeur(ui: &Ui, briques: &[Brique]) -> f32 {
    let mesure = |t: &str, taille: f32| {
        ui.painter()
            .layout_no_wrap(t.to_string(), FontId::proportional(taille), egui::Color32::WHITE)
            .size()
            .x
    };
    briques
        .iter()
        .map(|b| match b {
            Brique::Interrupteur { texte, .. } => {
                dessin::largeur_ligne(mesure(texte, TAILLE_TEXTE), Some(40.0))
            }
            Brique::Choix { texte, raccourci, .. } => dessin::largeur_ligne(
                mesure(texte, TAILLE_TEXTE),
                raccourci.as_deref().map(|r| mesure(r, TAILLE_PETIT)),
            ),
            Brique::Segments { options, .. } => {
                options
                    .iter()
                    .map(|(_, t)| mesure(t, TAILLE_TEXTE) + 24.0)
                    .fold(0.0, f32::max)
                    * options.len() as f32
                    + 2.0 * LIGNE_MARGE_DROITE
            }
            Brique::Appareil { nom, sous_ligne, .. } => dessin::largeur_ligne(
                mesure(nom, TAILLE_TEXTE).max(sous_ligne.as_deref().map_or(0.0, |s| mesure(s, TAILLE_PETIT))),
                Some(mesure("100 %", TAILLE_PETIT)),
            ),
            Brique::Section(t) => LIGNE_MARGE_DROITE * 2.0 + mesure(t, TAILLE_PETIT),
            Brique::Raccourcis(l) => l
                .iter()
                .map(|(t, k)| {
                    2.0 * LIGNE_MARGE_DROITE + mesure(t, TAILLE_PETIT) + LIGNE_ECART + mesure(k, TAILLE_PETIT)
                })
                .fold(0.0, f32::max),
            Brique::Action { texte, .. } => dessin::largeur_ligne(mesure(texte, TAILLE_TEXTE), None),
        })
        .fold(0.0, f32::max)
}

/// Dessine les briques, de haut en bas ; rend le geste de l'utilisateur.
pub fn dessiner(ui: &mut Ui, p: &Palette, briques: &[Brique]) -> Option<Geste> {
    let mut geste = None;
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = VOLET_ECART;
        for b in briques {
            if let Some(g) = brique(ui, p, b) {
                geste = Some(g);
            }
        }
    });
    geste
}

fn texte_ligne(ui: &Ui, p: &Palette, rect: egui::Rect, y: f32, texte: &str, taille: f32, secondaire: bool) {
    ui.painter().text(
        pos2(rect.left() + LIGNE_TEXTE_X, y),
        Align2::LEFT_CENTER,
        texte,
        FontId::proportional(taille),
        couleur(if secondaire { p.texte_secondaire } else { p.texte }),
    );
}

fn fond_survol(ui: &Ui, p: &Palette, rect: egui::Rect, r: &egui::Response) {
    let f = if r.is_pointer_button_down_on() {
        Some(p.appui)
    } else if r.hovered() || r.has_focus() {
        Some(p.survol)
    } else {
        None
    };
    if let Some(f) = f {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(LIGNE_ARRONDI), couleur(f));
    }
}

fn brique(ui: &mut Ui, p: &Palette, b: &Brique) -> Option<Geste> {
    match b {
        Brique::Interrupteur {
            cle,
            glyphe,
            texte,
            actif,
        } => {
            let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_LIGNE), Sense::click());
            fond_survol(ui, p, rect, &r);
            if let Some(g) = glyphe {
                icone(
                    ui,
                    pos2(rect.left() + LIGNE_ICONE_X, rect.center().y),
                    *g,
                    LIGNE_ICONE,
                    p.texte,
                );
            }
            texte_ligne(ui, p, rect, rect.center().y, texte, TAILLE_TEXTE, false);
            let mut v = *actif;
            let place = egui::Rect::from_center_size(
                pos2(rect.right() - LIGNE_MARGE_DROITE - 20.0, rect.center().y),
                vec2(40.0, 32.0),
            );
            let bascule = ui.put(place, |ui: &mut Ui| interrupteur(ui, p, &mut v));
            r.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, *actif, texte));
            dessin::focus(ui, p, &r, LIGNE_ARRONDI);
            (r.clicked() || bascule.clicked()).then(|| Geste::Changer(cle.clone(), Valeur::Booleen(!*actif)))
        }
        Brique::Choix {
            cle,
            glyphe,
            texte,
            coche,
            raccourci,
        } => {
            let (g, c) = if *coche {
                (icones::COCHE, p.accent)
            } else {
                (glyphe.unwrap_or(' '), p.texte)
            };
            ligne(ui, p, g, c, texte, raccourci.as_deref())
                .clicked()
                .then(|| Geste::Action(cle.clone()))
        }
        Brique::Segments {
            cle,
            options,
            choisie,
        } => {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), VOLET_SEGMENTS), Sense::hover());
            let rect = rect.shrink2(vec2(LIGNE_MARGE_DROITE, 0.0));
            let n = options.len().max(1) as f32;
            let w = rect.width() / n;
            let mut geste = None;
            for (i, (k, t)) in options.iter().enumerate() {
                let case = egui::Rect::from_min_size(
                    pos2(rect.left() + w * i as f32, rect.top()),
                    vec2(w, rect.height()),
                );
                let r = ui.interact(case, ui.scope_id().with(("segment", cle, k)), Sense::click());
                let oui = k == choisie;
                let fond = if oui {
                    p.accent
                } else if r.hovered() {
                    p.survol
                } else {
                    p.controle
                };
                ui.painter()
                    .rect_filled(case.shrink(1.0), CornerRadius::same(LIGNE_ARRONDI), couleur(fond));
                ui.painter().text(
                    case.center(),
                    Align2::CENTER_CENTER,
                    t,
                    FontId::proportional(TAILLE_TEXTE),
                    couleur(if oui { p.sur_accent } else { p.texte }),
                );
                r.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, oui, t));
                dessin::focus(ui, p, &r, LIGNE_ARRONDI);
                if r.clicked() && !oui {
                    geste = Some(Geste::Changer(cle.clone(), Valeur::Texte(k.clone())));
                }
            }
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(LIGNE_ARRONDI),
                Stroke::new(1.0, couleur(p.bord_controle)),
                StrokeKind::Inside,
            );
            geste
        }
        Brique::Appareil {
            cle,
            glyphe,
            nom,
            pourcentage,
            sous_ligne,
        } => {
            let h = VOLET_LIGNE + sous_ligne.as_ref().map_or(0.0, |_| VOLET_SOUS_LIGNE);
            let sens = if cle.is_some() {
                Sense::click()
            } else {
                Sense::hover()
            };
            let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), h), sens);
            if cle.is_some() {
                fond_survol(ui, p, rect, &r);
            }
            let y = rect.top() + VOLET_LIGNE / 2.0 - 3.0;
            icone(
                ui,
                pos2(rect.left() + LIGNE_ICONE_X, y),
                *glyphe,
                LIGNE_ICONE,
                p.texte,
            );
            texte_ligne(ui, p, rect, y, nom, TAILLE_TEXTE, false);
            if let Some(v) = pourcentage {
                ui.painter().text(
                    pos2(rect.right() - LIGNE_MARGE_DROITE, y),
                    Align2::RIGHT_CENTER,
                    pourcent(*v),
                    FontId::proportional(TAILLE_PETIT),
                    couleur(p.texte_secondaire),
                );
                let rail = egui::Rect::from_min_max(
                    pos2(rect.left() + LIGNE_TEXTE_X, rect.top() + VOLET_LIGNE - 8.0),
                    pos2(
                        rect.right() - LIGNE_MARGE_DROITE,
                        rect.top() + VOLET_LIGNE - 8.0 + VOLET_BARRE,
                    ),
                );
                let arrondi = CornerRadius::same((VOLET_BARRE / 2.0) as u8);
                ui.painter().rect_filled(rail, arrondi, couleur(p.bord_controle));
                let mut plein = rail;
                plein.set_width(rail.width() * v.clamp(0.0, 1.0));
                ui.painter().rect_filled(plein, arrondi, couleur(p.accent));
            }
            if let Some(s) = sous_ligne {
                texte_ligne(
                    ui,
                    p,
                    rect,
                    rect.top() + VOLET_LIGNE + VOLET_SOUS_LIGNE / 2.0 - 4.0,
                    s,
                    TAILLE_PETIT,
                    true,
                );
            }
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, cle.is_some(), nom));
            cle.as_ref()
                .filter(|_| r.clicked())
                .map(|c| Geste::Action(c.clone()))
        }
        Brique::Section(t) => {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_SECTION), Sense::hover());
            ui.painter().text(
                pos2(rect.left() + LIGNE_MARGE_DROITE, rect.bottom() - 6.0),
                Align2::LEFT_BOTTOM,
                t,
                semibold(TAILLE_PETIT),
                couleur(p.texte_secondaire),
            );
            None
        }
        Brique::Raccourcis(liste) => {
            for (t, k) in liste {
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), VOLET_RACCOURCI), Sense::hover());
                let y = rect.center().y;
                let petit = FontId::proportional(TAILLE_PETIT);
                ui.painter().text(
                    pos2(rect.left() + LIGNE_MARGE_DROITE, y),
                    Align2::LEFT_CENTER,
                    t,
                    petit.clone(),
                    couleur(p.texte_secondaire),
                );
                ui.painter().text(
                    pos2(rect.right() - LIGNE_MARGE_DROITE, y),
                    Align2::RIGHT_CENTER,
                    k,
                    petit,
                    couleur(p.texte_secondaire),
                );
            }
            None
        }
        Brique::Action { cle, glyphe, texte } => ligne(ui, p, *glyphe, p.texte, texte, None)
            .clicked()
            .then(|| Geste::Action(cle.clone())),
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    fn toutes() -> Vec<Brique> {
        vec![
            Brique::Section("Écrans".into()),
            Brique::Interrupteur {
                cle: "auto".into(),
                glyphe: Some('\u{E7F4}'),
                texte: "Changer tout seul".into(),
                actif: false,
            },
            Brique::Choix {
                cle: "salon".into(),
                glyphe: None,
                texte: "Bureau du salon".into(),
                coche: true,
                raccourci: Some("Ctrl+Alt+1".into()),
            },
            Brique::Segments {
                cle: "mode".into(),
                options: vec![("a".into(), "Un".into()), ("b".into(), "Deux".into())],
                choisie: "a".into(),
            },
            Brique::Appareil {
                cle: None,
                glyphe: '\u{E7F4}',
                nom: "Souris".into(),
                pourcentage: Some(0.42),
                sous_ligne: Some("Chargée il y a 2 jours".into()),
            },
            Brique::Raccourcis(vec![("Écran suivant".into(), "Ctrl+Alt+→".into())]),
            Brique::Action {
                cle: "relire".into(),
                glyphe: '\u{E72C}',
                texte: "Relire les écrans".into(),
            },
        ]
    }

    /// Une image des briques ; rend le geste et la hauteur dessinée.
    fn image(ctx: &egui::Context, evts: Vec<egui::Event>) -> (Option<Geste>, f32) {
        let p = Palette::de(&crate::theme::ChoixWindows::default());
        let entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(320.0, 800.0))),
            events: evts,
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let mut rendu = (None, 0.0);
        let mut sortie = ctx.run_ui(entree, |ui| {
            let haut = ui.cursor().top();
            rendu.0 = dessiner(ui, &p, &toutes());
            rendu.1 = ui.cursor().top() - haut;
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

    /// Chaque brique a sa hauteur commune, les briques sont séparées du même écart.
    #[test]
    fn hauteurs_communes() {
        let ctx = egui::Context::default();
        let (_, h) = image(&ctx, vec![]);
        let briques = VOLET_SECTION
            + VOLET_LIGNE
            + VOLET_LIGNE
            + VOLET_SEGMENTS
            + VOLET_LIGNE
            + VOLET_SOUS_LIGNE
            + VOLET_RACCOURCI
            + VOLET_LIGNE;
        // Six écarts entre les sept briques, plus l'espace qu'egui laisse après le groupe.
        let ecarts = h - briques;
        assert!(
            (6.0 * VOLET_ECART - 0.5..=6.0 * VOLET_ECART + 8.0).contains(&ecarts),
            "{h} : {ecarts}"
        );
    }

    /// L'interrupteur et le choix segmenté changent le réglage ; le choix et l'action
    /// rendent leur clé.
    #[test]
    fn gestes() {
        let ctx = egui::Context::default();
        image(&ctx, vec![]);
        let y_inter = VOLET_SECTION + VOLET_ECART + VOLET_LIGNE / 2.0;
        assert_eq!(
            clic(&ctx, pos2(100.0, y_inter)),
            Some(Geste::Changer("auto".into(), Valeur::Booleen(true)))
        );
        let y_choix = y_inter + VOLET_LIGNE + VOLET_ECART;
        assert_eq!(
            clic(&ctx, pos2(100.0, y_choix)),
            Some(Geste::Action("salon".into()))
        );
        let y_seg = y_choix + VOLET_LIGNE / 2.0 + VOLET_ECART + VOLET_SEGMENTS / 2.0;
        assert_eq!(
            clic(&ctx, pos2(250.0, y_seg)),
            Some(Geste::Changer("mode".into(), Valeur::Texte("b".into())))
        );
        assert_eq!(clic(&ctx, pos2(80.0, y_seg)), None, "l'option déjà choisie");
    }
}
