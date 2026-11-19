//! Le thème suivi en direct (règles : interface Windows 11, point 1) : le mode clair ou
//! sombre et l'accent de Windows, relus toutes les 2 secondes (quelques valeurs du registre,
//! rien de lent), appliqués à egui dès qu'ils changent.

use std::time::{Duration, Instant};

use egui::{Color32, FontId, TextStyle};

use crate::polices::{semibold, titre};
use crate::theme::{
    ARRONDI_CARTE, ARRONDI_CONTROLE, ChoixWindows, Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE, TAILLE_TITRE,
};

/// Une couleur de la palette pour egui.
pub fn couleur(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

/// Les 6 couleurs des pastilles d'appareils du volet (voir [`crate::pastilles`]).
pub fn couleurs_pastilles() -> [Rgb; 6] {
    crate::pastilles::couleurs_pastilles()
}

/// Ce que Windows a choisi (sous Windows) ; ailleurs, le clair et l'accent par défaut.
pub fn choix_windows() -> ChoixWindows {
    #[cfg(windows)]
    {
        crate::systeme::choix_windows()
    }
    #[cfg(not(windows))]
    {
        ChoixWindows::default()
    }
}

/// Le thème suivi.
pub struct Apparence {
    pub choix: ChoixWindows,
    pub palette: Palette,
    /// Pour les captures et les essais : clair (`Some(false)`) ou sombre (`Some(true)`)
    /// imposé.
    pub force: Option<bool>,
    lu_a: Instant,
}

/// Le thème est relu dans Windows à cet intervalle.
pub const RELECTURE: Duration = Duration::from_secs(2);

impl Apparence {
    pub fn new(force: Option<bool>) -> Apparence {
        Apparence::de(choix_windows(), force)
    }

    /// Avec un choix donné (essais, captures).
    pub fn de(mut choix: ChoixWindows, force: Option<bool>) -> Apparence {
        if let Some(f) = force {
            choix.sombre = f;
        }
        Apparence {
            palette: Palette::de(&choix),
            choix,
            force,
            lu_a: Instant::now(),
        }
    }

    /// Relit Windows si 2 s ont passé ; `true` si le thème a changé (déjà appliqué).
    pub fn suivre(&mut self, ctx: &egui::Context) -> bool {
        ctx.request_repaint_after(RELECTURE);
        if self.lu_a.elapsed() < RELECTURE {
            return false;
        }
        self.relire(ctx)
    }

    /// Relit Windows tout de suite ; `true` si le thème a changé (déjà appliqué).
    pub fn relire(&mut self, ctx: &egui::Context) -> bool {
        self.lu_a = Instant::now();
        let mut choix = choix_windows();
        if let Some(f) = self.force {
            choix.sombre = f;
        }
        if choix == self.choix {
            return false;
        }
        self.choix = choix;
        self.palette = Palette::de(&self.choix);
        self.appliquer(ctx);
        true
    }

    /// Les couleurs, tailles et espacements de Windows 11, donnés à egui.
    pub fn appliquer(&self, ctx: &egui::Context) {
        let p = &self.palette;
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
                (TextStyle::Name("section".into()), semibold(15.0)),
            ]
            .into();
            s.spacing.item_spacing = egui::vec2(8.0, 8.0);
            s.spacing.button_padding = egui::vec2(12.0, 6.0);
            s.spacing.interact_size = egui::vec2(40.0, 32.0);
            s.spacing.combo_height = 320.0;
            s.interaction.selectable_labels = false;
        });
    }
}
