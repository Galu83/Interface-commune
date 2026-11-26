//! Les contrôles de base : focus, icône, lignes du volet et du petit menu, boutons.

use egui::{Align2, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::dessin::couleur::couleur;
use crate::mesures::{
    LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_ICONE_X, LIGNE_MARGE_DROITE, LIGNE_TEXTE_X, MENU_LIGNE,
    VOLET_LIGNE,
};
use crate::polices::{icones as police_icones, semibold};
use crate::theme::{Palette, Rgb};

/// Contour de focus (clavier) : 2 px de la couleur du texte, comme Windows.
pub fn focus(ui: &Ui, p: &Palette, r: &Response, arrondi: u8) {
    if r.has_focus() {
        ui.painter().rect_stroke(
            r.rect.expand(1.0),
            CornerRadius::same(arrondi + 1),
            Stroke::new(2.0, couleur(p.texte)),
            StrokeKind::Outside,
        );
    }
}

/// Une icône de Segoe Fluent Icons, centrée en `centre`.
pub fn icone(ui: &Ui, centre: egui::Pos2, glyphe: char, taille: f32, c: Rgb) {
    ui.painter().text(
        centre,
        Align2::CENTER_CENTER,
        glyphe,
        police_icones(taille),
        couleur(c),
    );
}

/// La largeur qu'il faut à une ligne de menu pour tout montrer sans couper : `texte` et
/// `droite`, les largeurs mesurées de ses deux textes.
pub fn largeur_ligne(texte: f32, droite: Option<f32>) -> f32 {
    LIGNE_TEXTE_X + texte + droite.map_or(0.0, |d| LIGNE_ECART + d) + LIGNE_MARGE_DROITE
}

/// Une ligne du volet sur toute la largeur ([`VOLET_LIGNE`]) : icône, texte, et à droite une
/// aide discrète. L'aide de droite est toujours entière ; un texte trop long pour la place qui
/// reste finit par « … » (le nom entier reste dans l'info-bulle).
pub fn ligne(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    couleur_icone: Rgb,
    texte: &str,
    droite: Option<&str>,
) -> Response {
    ligne_de(ui, p, glyphe, couleur_icone, texte, droite, VOLET_LIGNE)
}

/// Une ligne du petit menu ([`MENU_LIGNE`], comme les menus de Windows 11).
pub fn ligne_menu(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str) -> Response {
    ligne_de(ui, p, glyphe, p.texte, texte, None, MENU_LIGNE)
}

/// La ligne du petit menu par défaut (« Ouvrir `<logiciel>` »), en gras (semibold), comme
/// l'entrée par défaut des menus de Windows.
pub fn ligne_menu_grasse(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str) -> Response {
    ligne_police(ui, p, glyphe, p.texte, texte, None, MENU_LIGNE, semibold(14.0))
}

/// Une ligne de cette hauteur ; `glyphe` espace : sans icône.
pub fn ligne_de(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    couleur_icone: Rgb,
    texte: &str,
    droite: Option<&str>,
    hauteur: f32,
) -> Response {
    ligne_police(
        ui,
        p,
        glyphe,
        couleur_icone,
        texte,
        droite,
        hauteur,
        FontId::proportional(14.0),
    )
}

/// Une ligne de cette hauteur, son texte dans cette police.
#[allow(clippy::too_many_arguments)]
fn ligne_police(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    couleur_icone: Rgb,
    texte: &str,
    droite: Option<&str>,
    hauteur: f32,
    police: FontId,
) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), hauteur), Sense::click());
    let droite = droite.map(|d| {
        ui.painter().layout_no_wrap(
            d.to_string(),
            FontId::proportional(12.0),
            couleur(p.texte_secondaire),
        )
    });
    let place = rect.width()
        - LIGNE_TEXTE_X
        - LIGNE_MARGE_DROITE
        - droite.as_ref().map_or(0.0, |d| LIGNE_ECART + d.size().x);
    let mut job = egui::text::LayoutJob::simple_singleline(texte.to_string(), police, couleur(p.texte));
    job.wrap = egui::text::TextWrapping::truncate_at_width(place.max(1.0));
    let galley = ui.painter().layout_job(job);
    let r = if galley.elided { r.on_hover_text(texte) } else { r };
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    if ui.is_rect_visible(rect) {
        let fond = if r.is_pointer_button_down_on() {
            Some(p.appui)
        } else if r.hovered() || r.has_focus() {
            Some(p.survol)
        } else {
            None
        };
        if let Some(f) = fond {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(LIGNE_ARRONDI), couleur(f));
        }
        if glyphe != ' ' {
            icone(
                ui,
                pos2(rect.left() + LIGNE_ICONE_X, rect.center().y),
                glyphe,
                LIGNE_ICONE,
                couleur_icone,
            );
        }
        let haut = rect.center().y - galley.size().y / 2.0;
        ui.painter()
            .galley(pos2(rect.left() + LIGNE_TEXTE_X, haut), galley, couleur(p.texte));
        if let Some(d) = droite {
            let pos = pos2(
                rect.right() - LIGNE_MARGE_DROITE - d.size().x,
                rect.center().y - d.size().y / 2.0,
            );
            ui.painter().galley(pos, d, couleur(p.texte_secondaire));
        }
        focus(ui, p, &r, 4);
    }
    r
}

/// Un bouton carré qui n'est qu'une icône (la roue dentée, en haut à droite du volet), 36 × 36 : survol
/// arrondi, contour de focus, et son nom pour le lecteur d'écran et l'info-bulle.
pub fn bouton_icone(ui: &mut Ui, p: &Palette, glyphe: char, nom: &str) -> Response {
    let (rect, r) = ui.allocate_exact_size(
        vec2(crate::mesures::VOLET_ROUE, crate::mesures::VOLET_ROUE),
        Sense::click(),
    );
    let r = r.on_hover_text(nom);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom));
    if ui.is_rect_visible(rect) {
        let fond = if r.is_pointer_button_down_on() {
            Some(p.appui)
        } else if r.hovered() || r.has_focus() {
            Some(p.survol)
        } else {
            None
        };
        if let Some(f) = fond {
            ui.painter().rect_filled(rect, CornerRadius::same(4), couleur(f));
        }
        icone(ui, rect.center(), glyphe, 16.0, p.texte);
        focus(ui, p, &r, 4);
    }
    r
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Normal,
    Accent,
    /// Action qui détruit : texte rouge.
    Danger,
}

/// La largeur du texte d'un [`bouton`] et la place de son icône (points).
fn mesure_bouton(ui: &Ui, glyphe: Option<char>, texte: &str) -> (f32, f32) {
    let largeur_texte = ui
        .painter()
        .layout_no_wrap(
            texte.to_string(),
            FontId::proportional(14.0),
            egui::Color32::PLACEHOLDER,
        )
        .size()
        .x;
    (largeur_texte, if glyphe.is_some() { 24.0 } else { 0.0 })
}

/// La largeur qu'aura un [`bouton`] (points), sans le dessiner : pour réserver sa place.
pub(crate) fn largeur_bouton(ui: &Ui, glyphe: Option<char>, texte: &str) -> f32 {
    let (largeur_texte, avec_icone) = mesure_bouton(ui, glyphe, texte);
    (largeur_texte + avec_icone + 28.0).max(if texte.is_empty() { 32.0 } else { 80.0 })
}

/// Un bouton de 32 px de haut, avec une icône facultative.
pub fn bouton(ui: &mut Ui, p: &Palette, glyphe: Option<char>, texte: &str, genre: Genre) -> Response {
    let police = FontId::proportional(14.0);
    let (largeur_texte, avec_icone) = mesure_bouton(ui, glyphe, texte);
    let largeur = largeur_bouton(ui, glyphe, texte);
    let (rect, r) = ui.allocate_exact_size(vec2(largeur, 32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let (fond, bord, texte_c) = match genre {
            Genre::Accent => (
                if r.hovered() {
                    p.accent.vers(p.fond, 0.1)
                } else {
                    p.accent
                },
                p.accent,
                p.sur_accent,
            ),
            Genre::Normal | Genre::Danger => (
                if r.is_pointer_button_down_on() {
                    p.appui
                } else if r.hovered() {
                    p.survol
                } else {
                    p.controle
                },
                p.bord_controle,
                if genre == Genre::Danger { p.rouge } else { p.texte },
            ),
        };
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), couleur(fond));
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, couleur(bord)),
            StrokeKind::Inside,
        );
        let mut x = rect.center().x - (largeur_texte + avec_icone) / 2.0;
        if let Some(g) = glyphe {
            icone(ui, pos2(x + 8.0, rect.center().y), g, 16.0, texte_c);
            x += avec_icone;
        }
        ui.painter().text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            texte,
            police,
            couleur(texte_c),
        );
        focus(ui, p, &r, 4);
    }
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    r
}
