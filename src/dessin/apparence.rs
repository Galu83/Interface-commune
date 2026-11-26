//! Les couleurs, tailles et espacements de Windows 11, donnés à egui (règles : interface
//! Windows 11, points 1 à 3) : du dessin pur, d'après une palette déjà calculée. Lire le thème
//! de Windows et le suivre reste dans les bords (`apparence::Apparence`, bloc L).

use egui::{FontId, TextStyle};

use crate::dessin::couleur::couleur;
use crate::polices::{semibold, titre};
use crate::theme::{ARRONDI_CARTE, ARRONDI_CONTROLE, Palette, TAILLE_PETIT, TAILLE_TEXTE, TAILLE_TITRE};

/// Le style de texte des titres de section (« section »), en Semibold.
pub(crate) const SECTION: &str = "section";
/// La taille des titres de section (points).
pub(crate) const TAILLE_SECTION: f32 = 15.0;

/// Les couleurs de la palette, les styles de texte et les espacements, donnés à egui.
pub(crate) fn appliquer(ctx: &egui::Context, p: &Palette) {
    let mut v = if p.sombre {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.dark_mode = p.sombre;
    v.override_text_color = Some(couleur(p.texte));
    v.panel_fill = couleur(p.fond);
    v.window_fill = couleur(p.fond_menu);
    v.window_stroke = egui::Stroke::new(1.0, couleur(p.bord_carte));
    v.extreme_bg_color = couleur(p.controle);
    v.text_edit_bg_color = Some(couleur(p.controle));
    v.faint_bg_color = couleur(p.carte);
    v.code_bg_color = couleur(p.carte);
    v.hyperlink_color = couleur(p.accent);
    v.error_fg_color = couleur(p.rouge);
    v.warn_fg_color = couleur(p.orange);
    v.selection.bg_fill = couleur(p.accent).gamma_multiply(0.45);
    v.selection.stroke = egui::Stroke::new(1.0, couleur(p.texte));
    v.text_cursor.stroke = egui::Stroke::new(1.5, couleur(p.texte));
    v.window_corner_radius = egui::CornerRadius::same(ARRONDI_CARTE);
    v.menu_corner_radius = egui::CornerRadius::same(ARRONDI_CARTE);
    v.popup_shadow = egui::Shadow::NONE;
    v.window_shadow = egui::Shadow::NONE;
    let arrondi = egui::CornerRadius::same(ARRONDI_CONTROLE);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = couleur(p.carte);
    w.noninteractive.weak_bg_fill = couleur(p.carte);
    w.noninteractive.bg_stroke = egui::Stroke::new(1.0, couleur(p.separateur));
    w.noninteractive.fg_stroke = egui::Stroke::new(1.0, couleur(p.texte));
    for (etat, fond) in [
        (&mut w.inactive, p.controle),
        (&mut w.hovered, p.survol),
        (&mut w.active, p.appui),
        (&mut w.open, p.appui),
    ] {
        etat.bg_fill = couleur(fond);
        etat.weak_bg_fill = couleur(fond);
        etat.bg_stroke = egui::Stroke::new(1.0, couleur(p.bord_controle));
        etat.fg_stroke = egui::Stroke::new(1.0, couleur(p.texte));
        etat.corner_radius = arrondi;
        etat.expansion = 0.0;
    }
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.text_styles = [
            (TextStyle::Heading, titre(TAILLE_TITRE)),
            (TextStyle::Body, FontId::proportional(TAILLE_TEXTE)),
            (TextStyle::Button, FontId::proportional(TAILLE_TEXTE)),
            (TextStyle::Small, FontId::proportional(TAILLE_PETIT)),
            (TextStyle::Monospace, FontId::monospace(TAILLE_TEXTE)),
            (TextStyle::Name(SECTION.into()), semibold(TAILLE_SECTION)),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size = egui::vec2(40.0, 32.0);
        s.spacing.combo_height = 320.0;
        s.interaction.selectable_labels = false;
    });
}

#[cfg(test)]
mod essais {
    use super::*;
    use crate::theme::{ChoixWindows, Rgb};

    fn palette(sombre: bool) -> Palette {
        Palette::de(&ChoixWindows {
            sombre,
            accent: Some(Rgb(0x00, 0x78, 0xD4)),
            ..ChoixWindows::default()
        })
    }

    /// Le mode sombre ou clair de la palette devient celui d'egui, avec ses couleurs.
    #[test]
    fn le_mode_et_les_couleurs_passent_a_egui() {
        for sombre in [false, true] {
            let ctx = egui::Context::default();
            let p = palette(sombre);
            appliquer(&ctx, &p);
            let v = ctx.global_style().visuals.clone();
            assert_eq!(v.dark_mode, sombre);
            assert_eq!(v.panel_fill, couleur(p.fond));
            assert_eq!(v.override_text_color, Some(couleur(p.texte)));
            assert_eq!(v.hyperlink_color, couleur(p.accent));
            assert_eq!(v.widgets.hovered.bg_fill, couleur(p.survol));
            assert_eq!(
                v.widgets.inactive.corner_radius,
                egui::CornerRadius::same(ARRONDI_CONTROLE)
            );
        }
    }

    /// Les styles de texte de Windows 11, dont « section » en Semibold, et des zones
    /// cliquables d'au moins 32 points (point 5).
    #[test]
    fn les_styles_de_texte_et_les_espacements() {
        let ctx = egui::Context::default();
        appliquer(&ctx, &palette(false));
        let s = ctx.global_style();
        assert_eq!(s.text_styles.get(&TextStyle::Heading), Some(&titre(TAILLE_TITRE)));
        assert_eq!(
            s.text_styles.get(&TextStyle::Body),
            Some(&FontId::proportional(TAILLE_TEXTE))
        );
        assert_eq!(
            s.text_styles.get(&TextStyle::Name(SECTION.into())),
            Some(&semibold(TAILLE_SECTION))
        );
        assert_eq!(s.text_styles.len(), 6);
        assert!(s.spacing.interact_size.y >= 32.0);
        assert!(!s.interaction.selectable_labels);
    }
}
