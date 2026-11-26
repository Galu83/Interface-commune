//! La colonne de gauche des Paramètres : le logo, le nom et la version, « Rechercher un
//! paramètre », puis une ligne par page (icône, nom, point d'alerte, barre d'accent sur la
//! page choisie). Aussi le chemin « Page › Sous-page » en haut de la page.

use egui::{Align2, CornerRadius, FontId, RichText, Sense, pos2, vec2};

use super::{Logo, Parametres};
use crate::apparence::couleur;
use crate::dessin::{self, titre_page};
use crate::mesures::{
    ESPACE_COMMUNES, NAVIGATION_LIGNE as LIGNE_NAVIGATION, NAVIGATION_TEXTE_X, RAYON_ALERTE, TAILLE_LOGO,
    TAILLE_NOM,
};
#[cfg(doc)]
use crate::mesures::{
    NAVIGATION_LARGEUR_MAX as LARGEUR_NAVIGATION_MAX, NAVIGATION_LARGEUR_MIN as LARGEUR_NAVIGATION_MIN,
};
use crate::polices::semibold;
use crate::textes::remplir;
use crate::theme::{Palette, TAILLE_PETIT};
use crate::{Reglages, icones};
use ic_coeur::parametres as coeur;

impl Parametres {
    /// La largeur de la navigation (points) : ce qu'il faut à l'en-tête et aux noms des
    /// pages pour tenir sur une ligne, entre [`LARGEUR_NAVIGATION_MIN`] et
    /// [`LARGEUR_NAVIGATION_MAX`].
    pub fn largeur_navigation(&self, ui: &egui::Ui, l: &dyn Reglages) -> f32 {
        let mesure = |t: &str, f: FontId| {
            ui.painter()
                .layout_no_wrap(t.to_string(), f, egui::Color32::WHITE)
                .size()
                .x
        };
        let id = l.identite();
        let nom = mesure(&id.nom, semibold(TAILLE_NOM));
        let version = mesure(
            &remplir(self.textes.version, &[&id.version]),
            FontId::proportional(TAILLE_PETIT),
        );
        let titres = self
            .pages
            .iter()
            .map(|p| mesure(&p.titre, FontId::proportional(14.0)));
        coeur::largeur_navigation(nom, version, ui.spacing().item_spacing.x, titres)
    }

    /// Le logo à sa taille réelle (pixels de l'écran), sans flou.
    pub(super) fn logo(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let ppp = ui.ctx().pixels_per_point();
        let px = (TAILLE_LOGO * ppp).round().clamp(16.0, 256.0) as u32;
        let a_refaire = self
            .logo
            .as_ref()
            .is_none_or(|g| g.pixels != px || (g.sombre != p.sombre && l.identite().logo.is_none()));
        if a_refaire {
            let texture = l.identite().logo_rvba(px, p.sombre).map(|rvba| {
                let image = egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &rvba);
                ui.ctx()
                    .load_texture("logo du logiciel", image, egui::TextureOptions::NEAREST)
            });
            self.logo = Some(Logo {
                pixels: px,
                sombre: p.sombre,
                texture,
            });
        }
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(px as f32 / ppp), Sense::hover());
        let coin = pos2((rect.min.x * ppp).round() / ppp, (rect.min.y * ppp).round() / ppp);
        if let Some(t) = self.logo.as_ref().and_then(|g| g.texture.as_ref()) {
            let uv = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(
                t.id(),
                egui::Rect::from_min_size(coin, rect.size()),
                uv,
                egui::Color32::WHITE,
            );
        }
    }

    /// Toute la colonne : l'en-tête, la recherche (dès sept pages), puis les pages.
    pub(super) fn navigation(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        self.entete_navigation(ui, p, l);
        if coeur::recherche_montree(&self.pages) {
            ui.add(
                egui::TextEdit::singleline(&mut self.recherche)
                    .hint_text(self.textes.rechercher_un_parametre)
                    .min_size(vec2(0.0, 32.0))
                    .desired_width(f32::INFINITY)
                    .char_limit(60),
            );
            ui.add_space(6.0);
        }
        for ligne in coeur::lignes_de_navigation(&self.pages, &self.recherche) {
            if ligne.espace_avant {
                // Les trois pages communes, juste après les pages propres, séparées d'elles.
                ui.add_space(ESPACE_COMMUNES);
            }
            self.ligne_navigation(ui, p, l, ligne.rang);
            ui.add_space(2.0);
        }
    }

    /// L'en-tête : le logo, puis le nom et « Version x.y.z » l'un sous l'autre.
    fn entete_navigation(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let id = l.identite();
        ui.add_space(4.0);
        let version = remplir(self.textes.version, &[&id.version]);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            self.logo(ui, p, l);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(&id.nom).font(semibold(TAILLE_NOM)));
                ui.label(
                    RichText::new(version)
                        .size(TAILLE_PETIT)
                        .color(couleur(p.texte_secondaire)),
                );
            });
        });
        ui.add_space(18.0);
    }

    /// La ligne de la page `i` : fond au survol ou si choisie, barre d'accent, icône, nom,
    /// point d'alerte ; un clic y va.
    fn ligne_navigation(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, i: usize) {
        let (g, nom) = (self.pages[i].icone, self.pages[i].titre.clone());
        let choisie = self.page == i;
        let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), LIGNE_NAVIGATION), Sense::click());
        if choisie || r.hovered() || r.has_focus() {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(5), couleur(p.survol));
        }
        if choisie {
            let barre =
                egui::Rect::from_center_size(pos2(rect.left() + 1.5, rect.center().y), vec2(3.0, 16.0));
            ui.painter()
                .rect_filled(barre, CornerRadius::same(2), couleur(p.accent));
        }
        dessin::icone(
            ui,
            pos2(rect.left() + 16.0 + 8.0, rect.center().y),
            g,
            16.0,
            p.texte,
        );
        ui.painter().text(
            pos2(rect.left() + NAVIGATION_TEXTE_X, rect.center().y),
            Align2::LEFT_CENTER,
            &nom,
            FontId::proportional(14.0),
            couleur(p.texte),
        );
        if l.alerte(&self.pages[i].cle) {
            // Le point d'alerte de PowerPeek : à droite de la ligne, couleur d'accent.
            ui.painter().circle_filled(
                pos2(rect.right() - 16.0, rect.center().y),
                RAYON_ALERTE,
                couleur(p.accent),
            );
        }
        r.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, choisie, &nom));
        dessin::focus(ui, p, &r, 5);
        if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            self.page = i;
            self.sous_page = None;
            self.recherche_page = self.recherche.trim().to_string();
            self.confirmer_retablir = false;
            self.confirmer_effacer = false;
        }
    }

    /// Le chemin en haut de la page (« Page › Sous-page ») et le titre.
    pub(super) fn entete(&mut self, ui: &mut egui::Ui, p: &Palette, titre: &str) {
        match self.sous_page.clone() {
            None => titre_page(ui, p, titre, ""),
            Some(s) => {
                ui.horizontal(|ui| {
                    if dessin::bouton_icone(ui, p, icones::RETOUR, self.textes.retour).clicked() {
                        self.sous_page = None;
                    }
                    ui.label(
                        RichText::new(format!("{titre} › {s}"))
                            .font(crate::polices::titre(crate::theme::TAILLE_TITRE)),
                    );
                });
                ui.add_space(8.0);
            }
        }
    }
}
