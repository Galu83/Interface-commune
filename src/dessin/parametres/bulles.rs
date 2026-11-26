//! Ce qui s'ajoute à la page : le bandeau « Redémarrer maintenant » en haut, la bulle
//! « Annuler » quelques secondes après un changement, et la carte « Conserver ces
//! modifications ? », devant tout le reste, avec son compte à rebours.

use std::time::Instant;

use egui::{CornerRadius, RichText, Sense, vec2};

use super::Parametres;
use crate::apparence::couleur;
use crate::dessin::{self, Genre, bouton};
use crate::parametres::modele;
use crate::polices::semibold;
use crate::textes::remplir;
use crate::theme::Palette;
use crate::{Reglages, cles, icones};
use ic_coeur::parametres::envois;

impl Parametres {
    /// Le bandeau « Redémarrer maintenant », en haut de la page, quand un réglage le demande.
    pub(super) fn bandeau_redemarrer(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let Some(r) = self.redemarrage.clone() else {
            return;
        };
        if dessin::bandeau(
            ui,
            p,
            p.orange,
            icones::RETABLIR,
            &r,
            self.textes.redemarrer_maintenant,
        ) {
            l.action(cles::REDEMARRER);
        }
        ui.add_space(8.0);
    }

    /// La bulle « Annuler », quelques secondes après un changement.
    pub(super) fn bulle_annuler(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let maintenant = Instant::now();
        let Some(a) = self.annulation.a_annuler(maintenant).cloned() else {
            return;
        };
        if let Some(reste) = self.annulation.reste(maintenant) {
            ui.ctx().request_repaint_after(reste);
        }
        let mut annuler = false;
        let mut fermer = false;
        let t = &self.textes;
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(remplir(t.modifie, &[&a.titre]));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    fermer = dessin::bouton_icone(ui, p, icones::FERMER, t.fermer).clicked();
                    annuler = bouton(ui, p, Some(icones::ANNULER), t.annuler, Genre::Normal).clicked();
                });
            });
        });
        ui.add_space(8.0);
        if annuler {
            Self::envoyer(l, envois::annuler(&mut self.annulation, Instant::now()));
        } else if fermer {
            self.annulation.oublier();
        }
    }

    /// La carte « Conserver ces modifications ? », devant les Paramètres (le reste ne répond
    /// plus tant qu'elle est là), avec le compte à rebours. « Revenir » est le bouton par
    /// défaut (Entrée) ; Échap = « Revenir ».
    pub(super) fn carte_conserver(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let maintenant = Instant::now();
        let Some(titre) = self.confirmation.en_attente(maintenant).map(|a| a.titre.clone()) else {
            self.focus_conserver = false;
            return;
        };
        let secondes = self.confirmation.secondes(maintenant).unwrap_or(0);
        let ctx = ui.ctx().clone();
        if let Some(d) = self.confirmation.prochain_changement(maintenant) {
            ctx.request_repaint_after(d);
        }
        let t = self.textes.clone();
        let (mut garder, mut revenir) = (false, false);
        let reste = self
            .confirmation
            .reste(maintenant)
            .map_or(0.0, |r| r.as_secs_f32() / modele::DUREE_CONSERVER.as_secs_f32());
        let donner_le_focus = !self.focus_conserver;
        let cadre = dessin::cadre_carte(p)
            .fill(couleur(p.fond_menu))
            .inner_margin(egui::Margin::same(20))
            .shadow(egui::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: egui::Color32::from_black_alpha(if p.sombre { 130 } else { 60 }),
            });
        let id = egui::Id::unique("interface_commune_conserver");
        // Sans fondu : une boîte de dialogue de Windows apparaît d'un coup, tout de suite lisible.
        let modale = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .backdrop_color(egui::Color32::from_black_alpha(if p.sombre { 130 } else { 90 }))
            .frame(cadre)
            .show(&ctx, |ui| {
                ui.set_width(380.0);
                ui.label(RichText::new(t.conserver_question).font(semibold(18.0)));
                ui.add_space(8.0);
                ui.label(remplir(t.modifie, &[&titre]));
                ui.add_space(2.0);
                ui.label(
                    RichText::new(remplir(t.conserver_retour, &[&secondes.to_string()]))
                        .color(couleur(p.texte_secondaire)),
                );
                ui.add_space(10.0);
                // Le temps qui reste, en barre fine : elle se vide jusqu'au retour automatique.
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(2), couleur(p.survol));
                let plein = egui::Rect::from_min_size(rect.min, vec2(rect.width() * reste, rect.height()));
                ui.painter()
                    .rect_filled(plein, CornerRadius::same(2), couleur(p.accent));
                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // De droite à gauche : « Revenir » (par défaut), puis « Conserver ».
                    let r = bouton(ui, p, None, t.revenir, Genre::Accent);
                    if donner_le_focus {
                        r.request_focus();
                    }
                    revenir = r.clicked();
                    garder = bouton(ui, p, None, t.conserver, Genre::Normal).clicked();
                });
            });
        self.focus_conserver = true;
        // Échap = « Revenir » (la carte est devant : rien d'autre ne le prend).
        let echap = modale.is_top_modal
            && !modale.any_popup_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if garder {
            self.conserver();
        } else if revenir || echap {
            self.revenir(l);
        }
    }
}
