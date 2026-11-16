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
    VOLET_BOUTON, VOLET_ECART, VOLET_LIGNE, VOLET_RACCOURCI, VOLET_SECTION, VOLET_SEGMENTS, VOLET_SOUS_LIGNE,
    VOLET_TUILE,
};
use crate::polices::semibold;
use crate::theme::{Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE};

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
            Brique::Appareil {
                nom,
                sous_ligne,
                plus,
                ..
            } => {
                let niveau = mesure(plus.niveau.as_deref().unwrap_or("100 %"), TAILLE_PETIT)
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

/// Où commence le texte d'un appareil : après le glyphe, ou après la tuile de son image.
fn texte_x(plus: &PlusAppareil) -> f32 {
    if plus.image.is_some() {
        8.0 + VOLET_TUILE + 12.0
    } else {
        LIGNE_TEXTE_X
    }
}

/// La hauteur d'un appareil : sa ligne, sa sous-ligne (plus haute avec un bouton), au moins
/// sa tuile.
fn hauteur_appareil(sous_ligne: bool, plus: &PlusAppareil) -> f32 {
    let mut h = VOLET_LIGNE;
    if sous_ligne || plus.bouton.is_some() {
        h += VOLET_SOUS_LIGNE;
    }
    if plus.bouton.is_some() {
        h += VOLET_BOUTON - VOLET_SOUS_LIGNE + 4.0;
    }
    if plus.image.is_some() {
        h = h.max(VOLET_TUILE + 10.0);
    }
    h
}

/// Un texte sur une ligne, coupé (« … ») au-delà de `largeur`.
fn sur_une_ligne(ui: &Ui, texte: &str, taille: f32, c: Rgb, largeur: f32) -> std::sync::Arc<egui::Galley> {
    let mut job =
        egui::text::LayoutJob::simple_singleline(texte.to_string(), FontId::proportional(taille), couleur(c));
    job.wrap = egui::text::TextWrapping::truncate_at_width(largeur.max(0.0));
    ui.painter().layout_job(job)
}

#[allow(clippy::too_many_arguments)]
fn appareil(
    ui: &mut Ui,
    p: &Palette,
    cle: Option<&str>,
    glyphe: char,
    nom: &str,
    pourcentage: Option<f32>,
    sous_ligne: Option<&str>,
    plus: &PlusAppareil,
) -> Option<Geste> {
    let h = hauteur_appareil(sous_ligne.is_some(), plus);
    let sens = if cle.is_some() {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), h), sens);
    if cle.is_some() {
        fond_survol(ui, p, rect, &r);
    }
    let (texte, secondaire) = if plus.eteint {
        (p.texte_secondaire, p.texte_desactive)
    } else {
        (p.texte, p.texte_secondaire)
    };
    let x0 = rect.left() + texte_x(plus);
    let x1 = rect.right() - LIGNE_MARGE_DROITE;
    let y = rect.top() + VOLET_LIGNE / 2.0 - 3.0;
    match plus.image {
        Some(im) => {
            let tuile = egui::Rect::from_min_size(
                pos2(rect.left() + 8.0, rect.center().y - VOLET_TUILE / 2.0),
                vec2(VOLET_TUILE, VOLET_TUILE),
            );
            if let Some(f) = im.fond {
                ui.painter().rect_filled(tuile, CornerRadius::same(6), couleur(f));
            }
            // Calée sur la grille des pixels : nette à toute échelle.
            let ppp = ui.ctx().pixels_per_point();
            let cote = im.cote.min(VOLET_TUILE - 6.0);
            let c = tuile.center();
            let coin = pos2(
                ((c.x - cote / 2.0) * ppp).round() / ppp,
                ((c.y - cote / 2.0) * ppp).round() / ppp,
            );
            let voile = if plus.eteint {
                egui::Color32::from_white_alpha(150)
            } else {
                egui::Color32::WHITE
            };
            ui.painter().image(
                im.texture,
                egui::Rect::from_min_size(coin, vec2(cote, cote)),
                egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                voile,
            );
        }
        None => icone(
            ui,
            pos2(rect.left() + LIGNE_ICONE_X, y),
            glyphe,
            LIGNE_ICONE,
            texte,
        ),
    }
    // Le niveau à droite (avec l'éclair pendant la charge), le nom à gauche, la barre dessous.
    let mut droite = x1;
    let barre = plus.teinte.unwrap_or(p.accent);
    if pourcentage.is_some() || plus.niveau.is_some() {
        let t = plus
            .niveau
            .clone()
            .unwrap_or_else(|| pourcentage.map(pourcent).unwrap_or_default());
        let g = ui
            .painter()
            .layout_no_wrap(t, FontId::proportional(TAILLE_PETIT), couleur(secondaire));
        droite -= g.size().x;
        let zone = egui::Rect::from_min_size(pos2(droite, y - g.size().y / 2.0), g.size());
        ui.painter().galley(zone.min, g, couleur(secondaire));
        if let Some(aide) = &plus.aide_niveau {
            ui.interact(zone, ui.scope_id().with(("niveau", nom)), Sense::hover())
                .on_hover_text(aide);
        }
        if plus.en_charge {
            droite -= 4.0;
            icone(
                ui,
                pos2(droite - TAILLE_PETIT / 2.0, y),
                icones::ECLAIR,
                TAILLE_PETIT,
                barre,
            );
            droite -= TAILLE_PETIT;
        }
    }
    // Un nom trop long est coupé avant le niveau, jamais par-dessus.
    let g = sur_une_ligne(ui, nom, TAILLE_TEXTE, texte, droite - LIGNE_ECART / 2.0 - x0);
    ui.painter()
        .galley(pos2(x0, y - g.size().y / 2.0), g, couleur(texte));
    if let Some(v) = pourcentage {
        let rail = egui::Rect::from_min_max(
            pos2(x0, rect.top() + VOLET_LIGNE - 8.0),
            pos2(x1, rect.top() + VOLET_LIGNE - 8.0 + VOLET_BARRE),
        );
        let arrondi = CornerRadius::same((VOLET_BARRE / 2.0) as u8);
        ui.painter().rect_filled(rail, arrondi, couleur(p.bord_controle));
        let mut plein = rail;
        plein.set_width((rail.width() * v.clamp(0.0, 1.0)).max(VOLET_BARRE / 2.0));
        ui.painter().rect_filled(plein, arrondi, couleur(barre));
    }
    // La sous-ligne, et le bouton à sa droite.
    let y2 = rect.top() + VOLET_LIGNE + VOLET_SOUS_LIGNE / 2.0 - 4.0
        + if plus.bouton.is_some() { 4.0 } else { 0.0 };
    let mut geste = None;
    let mut fin = x1;
    if let Some((cle_b, texte_b)) = &plus.bouton {
        let g = ui.painter().layout_no_wrap(
            texte_b.clone(),
            FontId::proportional(TAILLE_PETIT),
            couleur(p.texte),
        );
        let taille = vec2(g.size().x + 20.0, VOLET_BOUTON);
        let zone = egui::Rect::from_min_size(pos2(x1 - taille.x, y2 - taille.y / 2.0), taille);
        let rb = ui.interact(zone, ui.scope_id().with(("bouton", nom, cle_b)), Sense::click());
        let fond = if rb.is_pointer_button_down_on() {
            p.appui
        } else if rb.hovered() {
            p.survol
        } else {
            p.controle
        };
        let arrondi = CornerRadius::same(LIGNE_ARRONDI);
        ui.painter().rect_filled(zone, arrondi, couleur(fond));
        ui.painter().rect_stroke(
            zone,
            arrondi,
            Stroke::new(1.0, couleur(p.bord_controle)),
            StrokeKind::Inside,
        );
        ui.painter()
            .galley(zone.center() - g.size() / 2.0, g, couleur(p.texte));
        rb.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte_b));
        dessin::focus(ui, p, &rb, LIGNE_ARRONDI);
        if rb.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            geste = Some(Geste::Action(cle_b.clone()));
        }
        fin = zone.left() - LIGNE_ECART / 2.0;
    }
    if let Some(s) = sous_ligne {
        let g = sur_une_ligne(ui, s, TAILLE_PETIT, secondaire, fin - x0);
        ui.painter()
            .galley(pos2(x0, y2 - g.size().y / 2.0), g, couleur(secondaire));
    }
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, cle.is_some(), nom));
    geste.or_else(|| cle.filter(|_| r.clicked()).map(|c| Geste::Action(c.to_string())))
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
            plus,
        } => appareil(
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
}
