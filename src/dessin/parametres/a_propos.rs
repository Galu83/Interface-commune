//! La page commune « À propos » : le logo, le nom, la version et ce que fait le logiciel,
//! la licence, les cartes propres au logiciel, puis les crédits.

use egui::RichText;

use super::Parametres;
use crate::apparence::couleur;
use crate::dessin::{self, Genre, bouton, carte, carte_info, titre_section};
use crate::polices::semibold;
use crate::textes::remplir;
use crate::theme::{Palette, TAILLE_PETIT};
use crate::{Communs, Reglages, cles, icones};

impl Parametres {
    pub(super) fn page_a_propos(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let id = l.identite();
        let t = self.textes.clone();
        // Le logo, le nom et la version, puis ce que fait le logiciel.
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.set_min_height(48.0);
                self.logo(ui, p, l);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("{} {}", id.nom, id.version)).font(semibold(14.0)));
                    if !id.description.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(dessin::insecables(&id.description))
                                    .size(TAILLE_PETIT)
                                    .color(couleur(p.texte_secondaire)),
                            )
                            .wrap(),
                        );
                    }
                });
            });
        });
        let sous_licence = remplir(t.sous_licence, &[&id.nom, &id.licence]);
        if c.fichier_licences {
            carte(ui, p, icones::LICENCE, t.licence, &sous_licence, |ui| {
                if bouton(ui, p, None, t.voir_licences, Genre::Normal).clicked() {
                    l.action(cles::LICENCES);
                }
            });
        } else {
            carte_info(ui, p, icones::LICENCE, t.licence, &sous_licence);
        }
        // Les cartes propres au logiciel, avant les crédits.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_a_propos(ui, c));
        if !c.credits.is_empty() {
            titre_section(ui, t.credits);
            dessin::cadre_carte(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                for cr in &c.credits {
                    // Un nom long passe à la ligne au lieu de déborder.
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&cr.nom);
                        ui.label(RichText::new(&cr.licence).color(couleur(p.texte_secondaire)));
                    });
                }
            });
        }
    }
}
