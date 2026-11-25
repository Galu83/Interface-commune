//! Les briques du volet : le logiciel décrit ses réglages rapides ([`Brique`]), la
//! bibliothèque les dessine, toutes aux mêmes marges, hauteurs et tailles de texte
//! ([`crate::mesures`]). Aucun logiciel ne dessine son volet lui-même (règles : interface
//! Windows 11, point 6).

use egui::{Align2, CornerRadius, FontId, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::Valeur;
use crate::dessin::couleur::couleur;
use crate::dessin::{self, icone, interrupteur, ligne};
use crate::icones;
use crate::mesures::{
    LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_ICONE_X, LIGNE_MARGE_DROITE, LIGNE_TEXTE_X, VOLET_ECART,
    VOLET_LIGNE, VOLET_RACCOURCI, VOLET_SECTION, VOLET_SEGMENTS,
};
use crate::pastilles;
pub use crate::pastilles::{Case, Pastille};
use crate::polices::semibold;
use crate::theme::{Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE};
use crate::touches;
use ic_coeur::textes::avec_unite;

mod appareil;
use appareil::texte_x;

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
        /// Ce qui s'ajoute au besoin (image, couleur du niveau, charge, éteint, bouton) ;
        /// `Default` : l'appareil simple, un glyphe et la barre à la couleur d'accent.
        plus: PlusAppareil,
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
    /// Une rangée d'appareils (ordinateurs, claviers…) en pastilles colorées, sur 2 colonnes
    /// (retour à la ligne à partir de 3), sous un titre facultatif. Une pastille avec `cle` :
    /// un clic ferme le volet et rend la clé. Couleurs : [`crate::apparence::couleurs_pastilles`].
    Pastilles {
        titre: Option<String>,
        pastilles: Vec<Pastille>,
    },
    /// Un choix à deux lignes avec l'illustration de ce qu'il fait (une case par écran, de 1 à
    /// 4) : texte et raccourci, sous-ligne et, quand il est actif, la coche et `etiquette_active`
    /// en accent, une barre d'accent à gauche. Un clic ferme le volet et rend `cle`.
    ChoixIllustre {
        cle: String,
        texte: String,
        sous_ligne: Option<String>,
        illustration: Vec<Case>,
        raccourci: Option<String>,
        actif: bool,
        etiquette_active: Option<String>,
    },
}

/// L'image d'un appareil, dessinée par le logiciel : une texture du même contexte egui (par
/// exemple l'icône de l'appareil dans le style choisi par l'utilisateur).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageAppareil {
    pub texture: egui::TextureId,
    /// Son côté, en points : dessinée à sa taille réelle (pixels / pixels par point), jamais
    /// agrandie, au plus [`VOLET_TUILE`] moins 6.
    pub cote: f32,
    /// Le fond de sa tuile (par exemple la couleur de la barre des tâches, que l'icône
    /// suppose) ; `None` : sans tuile.
    pub fond: Option<Rgb>,
}

/// Ce qu'une brique « appareil » peut montrer de plus que son glyphe, son nom et son niveau.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlusAppareil {
    /// Une image dans une tuile de [`VOLET_TUILE`] points, à la place du glyphe ; la ligne
    /// prend alors au moins la hauteur de la tuile.
    pub image: Option<ImageAppareil>,
    /// La couleur de la barre et de l'éclair (vert, orange, rouge selon le niveau…) ; `None` :
    /// l'accent.
    pub teinte: Option<Rgb>,
    /// Le texte du niveau à la place de « 42 % » (« ≈ 50 % » pour un niveau approché).
    pub niveau: Option<String>,
    /// En charge : un éclair devant le niveau.
    pub en_charge: bool,
    /// Éteint : nom et niveau en texte secondaire, image pâlie.
    pub eteint: bool,
    /// Un petit bouton à droite de la sous-ligne : (clé rendue à `Reglages::action`, volet
    /// fermé ; texte), par exemple « Oublier » un appareil éteint.
    pub bouton: Option<(String, String)>,
    /// Une bulle d'aide sur le niveau (par exemple ce que veut dire « ≈ »).
    pub aide_niveau: Option<String>,
}

/// Ce que fait un clic sur une brique.
#[derive(Debug, Clone, PartialEq)]
pub enum Geste {
    /// Fermer le volet, puis faire cette action.
    Action(String),
    /// Changer ce réglage tout de suite, le volet reste ouvert.
    Changer(String, Valeur),
}

/// Un niveau de 0 à 1 en pourcentage, avec l'espace insécable : « 42\u{a0}% » (C-98).
fn pourcent(v: f32) -> String {
    avec_unite(format!("{:.0}", v.clamp(0.0, 1.0) * 100.0), "%")
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
                raccourci
                    .as_deref()
                    .map(|r| mesure(&touches::lisible(r), TAILLE_PETIT)),
            ),
            Brique::Segments { options, .. } => {
                options
                    .iter()
                    .map(|(_, t)| mesure(t, TAILLE_TEXTE) + 24.0)
                    .fold(0.0, f32::max)
                    * options.len() as f32
                    + 2.0 * LIGNE_MARGE_DROITE
            }
            Brique::Appareil {
                nom,
                sous_ligne,
                plus,
                ..
            } => {
                let niveau = mesure(plus.niveau.as_deref().unwrap_or("100\u{a0}%"), TAILLE_PETIT)
                    + if plus.en_charge { TAILLE_PETIT + 4.0 } else { 0.0 };
                let bouton = plus
                    .bouton
                    .as_ref()
                    .map_or(0.0, |(_, t)| mesure(t, TAILLE_PETIT) + 20.0 + LIGNE_ECART);
                let sous = sous_ligne.as_deref().map_or(0.0, |s| mesure(s, TAILLE_PETIT)) + bouton;
                dessin::largeur_ligne(mesure(nom, TAILLE_TEXTE).max(sous), Some(niveau))
                    + (texte_x(plus) - LIGNE_TEXTE_X)
            }
            Brique::Section(t) => LIGNE_MARGE_DROITE * 2.0 + mesure(t, TAILLE_PETIT),
            Brique::Raccourcis(l) => l
                .iter()
                .map(|(t, k)| {
                    2.0 * LIGNE_MARGE_DROITE
                        + mesure(t, TAILLE_PETIT)
                        + LIGNE_ECART
                        + mesure(&touches::lisible(k), TAILLE_PETIT)
                })
                .fold(0.0, f32::max),
            Brique::Action { texte, .. } => dessin::largeur_ligne(mesure(texte, TAILLE_TEXTE), None),
            Brique::Pastilles { titre, pastilles } => {
                pastilles::largeur_pastilles(ui, titre.as_deref(), pastilles)
            }
            Brique::ChoixIllustre {
                texte,
                sous_ligne,
                raccourci,
                etiquette_active,
                ..
            } => pastilles::largeur_choix(
                ui,
                texte,
                sous_ligne.as_deref(),
                raccourci.as_deref(),
                etiquette_active.as_deref(),
            ),
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
        } => brique_interrupteur(ui, p, cle, *glyphe, texte, *actif),
        Brique::Choix {
            cle,
            glyphe,
            texte,
            coche,
            raccourci,
        } => brique_choix(ui, p, cle, *glyphe, texte, *coche, raccourci.as_deref()),
        Brique::Segments {
            cle,
            options,
            choisie,
        } => brique_segments(ui, p, cle, options, choisie),
        Brique::Appareil {
            cle,
            glyphe,
            nom,
            pourcentage,
            sous_ligne,
            plus,
        } => appareil::appareil(
            ui,
            p,
            cle.as_deref(),
            *glyphe,
            nom,
            *pourcentage,
            sous_ligne.as_deref(),
            plus,
        ),
        Brique::Section(t) => {
            brique_section(ui, p, t);
            None
        }
        Brique::Raccourcis(liste) => {
            brique_raccourcis(ui, p, liste);
            None
        }
        Brique::Action { cle, glyphe, texte } => ligne(ui, p, *glyphe, p.texte, texte, None)
            .clicked()
            .then(|| Geste::Action(cle.clone())),
        Brique::Pastilles { titre, pastilles } => {
            pastilles::dessiner_pastilles(ui, p, titre.as_deref(), pastilles)
        }
        Brique::ChoixIllustre {
            cle,
            texte,
            sous_ligne,
            illustration,
            raccourci,
            actif,
            etiquette_active,
        } => pastilles::dessiner_choix(
            ui,
            p,
            cle,
            texte,
            sous_ligne.as_deref(),
            illustration,
            raccourci.as_deref(),
            *actif,
            etiquette_active.as_deref(),
        ),
    }
}

/// Un réglage oui/non : la ligne entière ou l'interrupteur le change.
fn brique_interrupteur(
    ui: &mut Ui,
    p: &Palette,
    cle: &str,
    glyphe: Option<char>,
    texte: &str,
    actif: bool,
) -> Option<Geste> {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_LIGNE), Sense::click());
    fond_survol(ui, p, rect, &r);
    if let Some(g) = glyphe {
        icone(
            ui,
            pos2(rect.left() + LIGNE_ICONE_X, rect.center().y),
            g,
            LIGNE_ICONE,
            p.texte,
        );
    }
    texte_ligne(ui, p, rect, rect.center().y, texte, TAILLE_TEXTE, false);
    let mut v = actif;
    let place = egui::Rect::from_center_size(
        pos2(rect.right() - LIGNE_MARGE_DROITE - 20.0, rect.center().y),
        vec2(40.0, 32.0),
    );
    let bascule = ui.put(place, |ui: &mut Ui| interrupteur(ui, p, &mut v));
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, actif, texte));
    dessin::focus(ui, p, &r, LIGNE_ARRONDI);
    (r.clicked() || bascule.clicked()).then(|| Geste::Changer(cle.to_string(), Valeur::Booleen(!actif)))
}

/// Un choix parmi d'autres : la coche s'il est choisi, sinon son glyphe ; le raccourci à droite.
fn brique_choix(
    ui: &mut Ui,
    p: &Palette,
    cle: &str,
    glyphe: Option<char>,
    texte: &str,
    coche: bool,
    raccourci: Option<&str>,
) -> Option<Geste> {
    let (g, c) = if coche {
        (icones::COCHE, p.accent)
    } else {
        (glyphe.unwrap_or(' '), p.texte)
    };
    let raccourci = raccourci.map(touches::lisible);
    ligne(ui, p, g, c, texte, raccourci.as_deref())
        .clicked()
        .then(|| Geste::Action(cle.to_string()))
}

/// Des segments côte à côte : un clic sur un autre segment le choisit.
fn brique_segments(
    ui: &mut Ui,
    p: &Palette,
    cle: &str,
    options: &[(String, String)],
    choisie: &str,
) -> Option<Geste> {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_SEGMENTS), Sense::hover());
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
            geste = Some(Geste::Changer(cle.to_string(), Valeur::Texte(k.clone())));
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

/// Un titre de section, en petit et en gras.
fn brique_section(ui: &mut Ui, p: &Palette, t: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_SECTION), Sense::hover());
    ui.painter().text(
        pos2(rect.left() + LIGNE_MARGE_DROITE, rect.bottom() - 6.0),
        Align2::LEFT_BOTTOM,
        t,
        semibold(TAILLE_PETIT),
        couleur(p.texte_secondaire),
    );
}

/// Des raccourcis rappelés : le texte à gauche, la touche en toutes lettres à droite.
fn brique_raccourcis(ui: &mut Ui, p: &Palette, liste: &[(String, String)]) {
    for (t, k) in liste {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_RACCOURCI), Sense::hover());
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
            touches::lisible(k),
            petit,
            couleur(p.texte_secondaire),
        );
    }
}

#[cfg(test)]
mod essais {
    use super::appareil::hauteur_appareil;
    use super::*;
    use crate::mesures::{VOLET_BOUTON, VOLET_SOUS_LIGNE, VOLET_TUILE};

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
                plus: PlusAppareil::default(),
            },
            Brique::Appareil {
                cle: None,
                glyphe: '\u{E7F4}',
                nom: "Casque éteint".into(),
                pourcentage: Some(0.22),
                sous_ligne: Some("Éteint · vu il y a 2 h".into()),
                plus: PlusAppareil {
                    teinte: Some(crate::Rgb(0x80, 0x80, 0x80)),
                    en_charge: true,
                    eteint: true,
                    bouton: Some(("oublier".into(), "Oublier".into())),
                    niveau: Some("≈ 20 %".into()),
                    ..Default::default()
                },
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

    /// Un appareil avec son image prend la hauteur de sa tuile et décale son texte ; son
    /// bouton agrandit la sous-ligne.
    #[test]
    fn appareil_avec_image_et_bouton() {
        let simple = PlusAppareil::default();
        assert_eq!(hauteur_appareil(true, &simple), VOLET_LIGNE + VOLET_SOUS_LIGNE);
        assert_eq!(texte_x(&simple), LIGNE_TEXTE_X);
        let image = PlusAppareil {
            image: Some(ImageAppareil {
                texture: egui::TextureId::Managed(1),
                cote: 38.0,
                fond: None,
            }),
            ..Default::default()
        };
        assert_eq!(hauteur_appareil(false, &image), VOLET_TUILE + 10.0);
        assert!(texte_x(&image) > 8.0 + VOLET_TUILE);
        let bouton = PlusAppareil {
            bouton: Some(("oublier".into(), "Oublier".into())),
            ..Default::default()
        };
        assert_eq!(hauteur_appareil(true, &bouton), VOLET_LIGNE + VOLET_BOUTON + 4.0);
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
            // L'appareil éteint avec son bouton « Oublier » : sa sous-ligne est plus haute.
            + VOLET_LIGNE
            + VOLET_BOUTON
            + 4.0
            + VOLET_RACCOURCI
            + VOLET_LIGNE;
        // Sept écarts entre les huit briques, plus l'espace qu'egui laisse après le groupe.
        let ecarts = h - briques;
        assert!(
            (7.0 * VOLET_ECART - 0.5..=7.0 * VOLET_ECART + 8.0).contains(&ecarts),
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

    /// R2 (C-98) : le niveau d'un appareil prend l'espace insécable avant « % », borné à 0 et 100.
    #[test]
    fn le_pourcentage_a_l_espace_insecable() {
        assert_eq!(pourcent(0.42), "42\u{a0}%");
        assert_eq!(pourcent(0.0), "0\u{a0}%");
        assert_eq!(pourcent(1.7), "100\u{a0}%");
        assert_eq!(pourcent(-0.2), "0\u{a0}%");
        assert!(!pourcent(0.5).contains(' '), "aucune espace simple");
    }
}
