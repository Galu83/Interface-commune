//! La brique « appareil » du volet : tuile ou glyphe, nom, niveau, barre, sous-ligne et bouton.

use egui::{CornerRadius, FontId, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use super::{Geste, PlusAppareil, fond_survol, pourcent};
use crate::dessin::couleur::couleur;
use crate::dessin::{self, icone};
use crate::icones;
use crate::mesures::{
    LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_ICONE_X, LIGNE_MARGE_DROITE, LIGNE_TEXTE_X, VOLET_BARRE,
    VOLET_BOUTON, VOLET_LIGNE, VOLET_SOUS_LIGNE, VOLET_TUILE,
};
use crate::theme::{Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE};

/// Où commence le texte d'un appareil : après le glyphe, ou après la tuile de son image.
pub(super) fn texte_x(plus: &PlusAppareil) -> f32 {
    if plus.image.is_some() {
        8.0 + VOLET_TUILE + 12.0
    } else {
        LIGNE_TEXTE_X
    }
}

/// La hauteur d'un appareil : sa ligne, sa sous-ligne (plus haute avec un bouton), au moins
/// sa tuile.
pub(super) fn hauteur_appareil(sous_ligne: bool, plus: &PlusAppareil) -> f32 {
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

/// Ce que les morceaux d'un appareil partagent : ses bords de texte, le milieu de sa première
/// ligne et ses couleurs.
struct Gabarit {
    x0: f32,
    x1: f32,
    y: f32,
    texte: Rgb,
    secondaire: Rgb,
    barre: Rgb,
}

// Chaque argument est une donnée distincte du dessin (egui, couleurs, textes) ; les regrouper
// dans une structure n'ajouterait qu'une enveloppe.
#[allow(clippy::too_many_arguments)]
pub(super) fn appareil(
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
    let g = Gabarit {
        x0: rect.left() + texte_x(plus),
        x1: rect.right() - LIGNE_MARGE_DROITE,
        y: rect.top() + VOLET_LIGNE / 2.0 - 3.0,
        texte,
        secondaire,
        barre: plus.teinte.unwrap_or(p.accent),
    };
    image_ou_glyphe(ui, rect, glyphe, plus, &g);
    // Le niveau à droite, le nom à gauche (coupé avant le niveau, jamais par-dessus), la barre
    // dessous.
    let droite = niveau(ui, nom, pourcentage, plus, &g);
    let galley = sur_une_ligne(ui, nom, TAILLE_TEXTE, texte, droite - LIGNE_ECART / 2.0 - g.x0);
    ui.painter()
        .galley(pos2(g.x0, g.y - galley.size().y / 2.0), galley, couleur(texte));
    if let Some(v) = pourcentage {
        barre(ui, p, rect, v, &g);
    }
    // La sous-ligne, et le bouton à sa droite.
    let y2 = rect.top() + VOLET_LIGNE + VOLET_SOUS_LIGNE / 2.0 - 4.0
        + if plus.bouton.is_some() { 4.0 } else { 0.0 };
    let (geste, fin) = match &plus.bouton {
        Some((cle_b, texte_b)) => bouton(ui, p, nom, cle_b, texte_b, y2, &g),
        None => (None, g.x1),
    };
    if let Some(s) = sous_ligne {
        let galley = sur_une_ligne(ui, s, TAILLE_PETIT, secondaire, fin - g.x0);
        ui.painter().galley(
            pos2(g.x0, y2 - galley.size().y / 2.0),
            galley,
            couleur(secondaire),
        );
    }
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, cle.is_some(), nom));
    geste.or_else(|| cle.filter(|_| r.clicked()).map(|c| Geste::Action(c.to_string())))
}

/// L'image de l'appareil dans sa tuile, calée sur la grille des pixels (nette à toute échelle),
/// ou son glyphe.
fn image_ou_glyphe(ui: &Ui, rect: egui::Rect, glyphe: char, plus: &PlusAppareil, g: &Gabarit) {
    let Some(im) = plus.image else {
        icone(
            ui,
            pos2(rect.left() + LIGNE_ICONE_X, g.y),
            glyphe,
            LIGNE_ICONE,
            g.texte,
        );
        return;
    };
    let tuile = egui::Rect::from_min_size(
        pos2(rect.left() + 8.0, rect.center().y - VOLET_TUILE / 2.0),
        vec2(VOLET_TUILE, VOLET_TUILE),
    );
    if let Some(f) = im.fond {
        ui.painter().rect_filled(tuile, CornerRadius::same(6), couleur(f));
    }
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

/// Le niveau à droite (avec l'éclair pendant la charge) ; rend où il commence, ou le bord
/// droit sans niveau.
fn niveau(ui: &Ui, nom: &str, pourcentage: Option<f32>, plus: &PlusAppareil, g: &Gabarit) -> f32 {
    let mut droite = g.x1;
    if pourcentage.is_none() && plus.niveau.is_none() {
        return droite;
    }
    let t = plus
        .niveau
        .clone()
        .unwrap_or_else(|| pourcentage.map(pourcent).unwrap_or_default());
    let galley = ui
        .painter()
        .layout_no_wrap(t, FontId::proportional(TAILLE_PETIT), couleur(g.secondaire));
    droite -= galley.size().x;
    let zone = egui::Rect::from_min_size(pos2(droite, g.y - galley.size().y / 2.0), galley.size());
    ui.painter().galley(zone.min, galley, couleur(g.secondaire));
    if let Some(aide) = &plus.aide_niveau {
        ui.interact(zone, ui.scope_id().with(("niveau", nom)), Sense::hover())
            .on_hover_text(aide);
    }
    if plus.en_charge {
        droite -= 4.0;
        icone(
            ui,
            pos2(droite - TAILLE_PETIT / 2.0, g.y),
            icones::ECLAIR,
            TAILLE_PETIT,
            g.barre,
        );
        droite -= TAILLE_PETIT;
    }
    droite
}

/// La barre du niveau `v` (de 0 à 1), sous le nom.
fn barre(ui: &Ui, p: &Palette, rect: egui::Rect, v: f32, g: &Gabarit) {
    let rail = egui::Rect::from_min_max(
        pos2(g.x0, rect.top() + VOLET_LIGNE - 8.0),
        pos2(g.x1, rect.top() + VOLET_LIGNE - 8.0 + VOLET_BARRE),
    );
    let arrondi = CornerRadius::same((VOLET_BARRE / 2.0) as u8);
    ui.painter().rect_filled(rail, arrondi, couleur(p.bord_controle));
    let mut plein = rail;
    plein.set_width((rail.width() * v.clamp(0.0, 1.0)).max(VOLET_BARRE / 2.0));
    ui.painter().rect_filled(plein, arrondi, couleur(g.barre));
}

/// Le petit bouton à droite de la sous-ligne ; rend son geste (s'il est cliqué) et où la
/// sous-ligne doit s'arrêter.
fn bouton(
    ui: &Ui,
    p: &Palette,
    nom: &str,
    cle_b: &str,
    texte_b: &str,
    y2: f32,
    g: &Gabarit,
) -> (Option<Geste>, f32) {
    let galley = ui.painter().layout_no_wrap(
        texte_b.to_string(),
        FontId::proportional(TAILLE_PETIT),
        couleur(p.texte),
    );
    let taille = vec2(galley.size().x + 20.0, VOLET_BOUTON);
    let zone = egui::Rect::from_min_size(pos2(g.x1 - taille.x, y2 - taille.y / 2.0), taille);
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
        .galley(zone.center() - galley.size() / 2.0, galley, couleur(p.texte));
    rb.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte_b));
    dessin::focus(ui, p, &rb, LIGNE_ARRONDI);
    let geste = rb
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
        .then(|| Geste::Action(cle_b.to_string()));
    (geste, zone.left() - LIGNE_ECART / 2.0)
}
