//! L'interrupteur, seul ou écrit (« Activé »), et la roue dentée des Paramètres (Ctrl+virgule).

use egui::{Align2, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use super::base::{bouton_icone, focus};
use crate::dessin::couleur::couleur;
use crate::icones;
use crate::textes::Textes;
use crate::theme::Palette;

/// Un interrupteur Windows 11 (40 × 20).
pub fn interrupteur(ui: &mut Ui, p: &Palette, actif: &mut bool) -> Response {
    let (rect, mut r) = ui.allocate_exact_size(vec2(40.0, 32.0), Sense::click());
    if r.clicked() {
        *actif = !*actif;
        r.mark_changed();
    }
    let piste = egui::Rect::from_center_size(rect.center(), vec2(40.0, 20.0));
    dessiner_interrupteur(ui, p, &r, piste, *actif);
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, *actif, ""));
    r
}

/// La piste (40 × 20) et la pastille d'un interrupteur, et son contour de focus.
fn dessiner_interrupteur(ui: &Ui, p: &Palette, r: &Response, piste: egui::Rect, actif: bool) {
    let t = ui.ctx().animate_bool_responsive(r.id, actif);
    let survol = r.hovered();
    if actif {
        let fond = if survol {
            p.accent.vers(p.texte, 0.1)
        } else {
            p.accent
        };
        ui.painter()
            .rect_filled(piste, CornerRadius::same(10), couleur(fond));
    } else {
        ui.painter().rect_filled(
            piste,
            CornerRadius::same(10),
            couleur(if survol { p.survol } else { p.controle }),
        );
        ui.painter().rect_stroke(
            piste,
            CornerRadius::same(10),
            Stroke::new(1.0, couleur(p.texte_secondaire)),
            StrokeKind::Inside,
        );
    }
    let x = egui::lerp(piste.left() + 10.0..=piste.right() - 10.0, t);
    let pastille = if actif { p.sur_accent } else { p.texte_secondaire };
    let rayon = if survol { 7.0 } else { 6.0 };
    ui.painter()
        .circle_filled(pos2(x, piste.center().y), rayon, couleur(pastille));
    focus(ui, p, r, 10);
}

/// Un interrupteur avec « Activé » ou « Désactivé » écrit à sa gauche, en 14 px, comme dans
/// les Paramètres de Windows (règles : point 7). Le texte et l'interrupteur se cliquent
/// ensemble ; la place du texte ne change pas d'un état à l'autre ([`largeur_interrupteur_ecrit`]).
pub fn interrupteur_ecrit(ui: &mut Ui, p: &Palette, t: &Textes, actif: &mut bool) -> Response {
    let police = FontId::proportional(14.0);
    let mesure = |s: &str| {
        ui.painter()
            .layout_no_wrap(s.to_string(), police.clone(), couleur(p.texte))
            .size()
            .x
    };
    let texte = mesure(t.active).max(mesure(t.desactive));
    let (zone, mut r) = ui.allocate_exact_size(vec2(largeur_interrupteur_ecrit(texte), 32.0), Sense::click());
    if r.clicked() {
        *actif = !*actif;
        r.mark_changed();
    }
    let ecrit = if *actif { t.active } else { t.desactive };
    if ui.is_rect_visible(zone) {
        ui.painter().text(
            pos2(zone.left(), zone.center().y),
            Align2::LEFT_CENTER,
            ecrit,
            police,
            couleur(p.texte),
        );
        let piste = egui::Rect::from_min_size(
            pos2(zone.right() - 40.0, zone.center().y - 10.0),
            vec2(40.0, 20.0),
        );
        dessiner_interrupteur(ui, p, &r, piste, *actif);
    }
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, *actif, ecrit));
    r
}

/// La largeur d'un [`interrupteur_ecrit`] : le plus long des deux textes, 12 px, puis
/// l'interrupteur (40 px).
pub fn largeur_interrupteur_ecrit(texte_le_plus_long: f32) -> f32 {
    texte_le_plus_long + 12.0 + 40.0
}

/// La roue dentée (E713) des Paramètres, en haut à droite d'une fenêtre principale : 36 × 36,
/// info-bulle « Paramètres » ; Ctrl+virgule fait de même, où que soit le focus. Rend vrai
/// quand il faut ouvrir les Paramètres (le logiciel les ouvre, par exemple avec
/// `FenetresCommunes::ouvrir_parametres`). À poser dans une mise en page de droite à gauche.
pub fn roue_parametres(ui: &mut Ui, p: &Palette, t: &Textes) -> bool {
    let clic = bouton_icone(ui, p, icones::REGLAGES, t.parametres).clicked();
    let raccourci = ui.input_mut(|i| i.consume_shortcut(&RACCOURCI_PARAMETRES));
    clic || raccourci
}

/// Ctrl+virgule : les Paramètres, comme dans les logiciels de Windows 11.
pub const RACCOURCI_PARAMETRES: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Comma);

#[cfg(test)]
mod essais_roue_et_interrupteur {
    use super::*;

    fn image(ctx: &egui::Context, evts: Vec<egui::Event>, f: impl FnMut(&mut Ui)) {
        let entree = egui::RawInput {
            events: evts,
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let mut sortie = ctx.run_ui(entree, f);
        sortie.textures_delta.clear();
    }

    fn clic(pos: egui::Pos2) -> Vec<Vec<egui::Event>> {
        let bouton = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        vec![
            vec![egui::Event::PointerMoved(pos)],
            vec![bouton(true)],
            vec![bouton(false)],
        ]
    }

    /// La roue dentée, posée de droite à gauche, est en haut à droite ; un clic ou Ctrl+virgule
    /// rend vrai, sinon faux.
    #[test]
    fn roue_en_haut_a_droite_et_ctrl_virgule() {
        let p = Palette::de(&crate::theme::ChoixWindows::default());
        let t = &crate::textes::FRANCAIS;
        let ctx = egui::Context::default();
        // Rend : la roue a-t-elle demandé les Paramètres, sa place, la place de la page.
        let poser = |evts| {
            let mut rendu = (false, egui::Rect::NOTHING, egui::Rect::NOTHING);
            image(&ctx, evts, |ui| {
                rendu.2 = ui.max_rect();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    rendu.0 = roue_parametres(ui, &p, t);
                    rendu.1 = ui.min_rect();
                });
            });
            rendu
        };
        let (ouvrir, zone, page) = poser(vec![]);
        assert!(!ouvrir);
        assert_eq!(zone.right(), page.right());
        assert_eq!(zone.top(), page.top());
        let mut vu = false;
        for evts in clic(zone.center()) {
            vu |= poser(evts).0;
        }
        assert!(vu, "le clic sur la roue ouvre les Paramètres");
        let ctrl = egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        };
        let virgule = egui::Event::Key {
            key: egui::Key::Comma,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: ctrl,
        };
        assert!(poser(vec![virgule]).0, "Ctrl+virgule ouvre les Paramètres");
        assert!(!poser(vec![]).0);
    }

    /// « Activé » ou « Désactivé » à gauche de l'interrupteur : un clic sur le texte bascule
    /// aussi ; la largeur ne change pas d'un état à l'autre.
    #[test]
    fn interrupteur_ecrit_a_gauche() {
        let p = Palette::de(&crate::theme::ChoixWindows::default());
        let t = &crate::textes::FRANCAIS;
        let ctx = egui::Context::default();
        let mut actif = false;
        let mut zone = egui::Rect::NOTHING;
        let poser = |evts, actif: &mut bool, zone: &mut egui::Rect| {
            image(&ctx, evts, |ui| {
                *zone = interrupteur_ecrit(ui, &p, t, actif).rect;
            })
        };
        poser(vec![], &mut actif, &mut zone);
        let avant = zone;
        assert!(avant.width() > largeur_interrupteur_ecrit(0.0));
        assert_eq!(avant.height(), 32.0);
        // Un clic sur le texte, à gauche : l'interrupteur bascule.
        for evts in clic(egui::pos2(avant.left() + 5.0, avant.center().y)) {
            poser(evts, &mut actif, &mut zone);
        }
        assert!(actif);
        assert_eq!(zone, avant, "même place pour « Activé » et « Désactivé »");
    }
}
