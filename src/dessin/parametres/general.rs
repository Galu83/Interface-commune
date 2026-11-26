//! La page commune « Général » : démarrage avec Windows, langue, notifications, les cartes
//! propres au logiciel, le journal (et « Effacer » avec confirmation), puis « Rétablir les
//! réglages par défaut ».

use super::Parametres;
use crate::dessin::{
    Genre, bouton, carte, carte_large, interrupteur_ecrit, place_des_boutons, titre_section,
};
use crate::parametres::modele::Valeur;
use crate::theme::Palette;
use crate::{Communs, Reglages, cles, icones};

impl Parametres {
    /// Toute la page, de haut en bas.
    pub(super) fn page_general(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        self.demarrage(ui, p, l, c);
        self.langue(ui, p, l, c);
        self.notifications(ui, p, l, c);
        // Les cartes propres au logiciel, avant le journal.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_general(ui, c));
        self.journal(ui, p, l, c);
        self.journal_detaille(ui, p, l, c);
        self.remise_a_zero(ui, p, l);
    }

    /// « Démarrer avec Windows » : grisé, avec la raison, quand c'est interdit.
    fn demarrage(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let Some(actif) = c.demarrage else {
            return;
        };
        let t = self.textes.clone();
        let mut v = actif;
        let bloque = c.demarrage_bloque.as_deref();
        carte(
            ui,
            p,
            icones::DEMARRAGE,
            t.demarrer_avec_windows,
            bloque.unwrap_or(t.demarrer_explication),
            |ui| {
                ui.add_enabled_ui(bloque.is_none(), |ui| {
                    interrupteur_ecrit(ui, p, &t, &mut v);
                });
            },
        );
        if v != actif {
            self.changer(
                l,
                cles::DEMARRAGE,
                t.demarrer_avec_windows,
                Valeur::Booleen(actif),
                Valeur::Booleen(v),
            );
        }
    }

    /// La langue du logiciel, dans une liste.
    fn langue(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let Some((langue, langues)) = &c.langue else {
            return;
        };
        let t = self.textes.clone();
        let mut choix = langue.clone();
        carte(ui, p, icones::LANGUE, t.langue, t.langue_explication, |ui| {
            egui::ComboBox::from_id_salt("langue")
                .selected_text(choix.clone())
                .width(170.0)
                .show_ui(ui, |ui| {
                    for x in langues {
                        ui.selectable_value(&mut choix, x.clone(), x);
                    }
                });
        });
        if choix != *langue {
            self.changer(
                l,
                cles::LANGUE,
                t.langue,
                Valeur::Texte(langue.clone()),
                Valeur::Texte(choix),
            );
        }
    }

    /// Les notifications, et « Essayer » si le logiciel le propose.
    fn notifications(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let Some(actif) = c.notifications else {
            return;
        };
        let t = self.textes.clone();
        let mut v = actif;
        carte(
            ui,
            p,
            icones::NOTIFICATIONS,
            t.notifications,
            t.notifications_explication,
            |ui| {
                interrupteur_ecrit(ui, p, &t, &mut v);
                if c.notifications_essai && bouton(ui, p, None, t.essayer, Genre::Normal).clicked() {
                    l.action(cles::NOTIFICATIONS_ESSAYER);
                }
            },
        );
        if v != actif {
            self.changer(
                l,
                cles::NOTIFICATIONS,
                t.notifications,
                Valeur::Booleen(actif),
                Valeur::Booleen(v),
            );
        }
    }

    /// « Ouvrir le journal », et « Effacer le journal » avec sa confirmation.
    fn journal(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let t = self.textes.clone();
        if self.confirmer_effacer {
            carte_large(
                ui,
                p,
                icones::JOURNAL,
                t.effacer_journal_question,
                t.effacer_journal_explication,
                260.0,
                |ui| {
                    if bouton(ui, p, None, t.non, Genre::Normal).clicked() {
                        self.confirmer_effacer = false;
                    }
                    if bouton(ui, p, None, t.oui_effacer, Genre::Danger).clicked() {
                        self.confirmer_effacer = false;
                        l.action(cles::JOURNAL_EFFACER);
                    }
                },
            );
        } else {
            // La place se mesure sur les boutons montrés : « Effacer le journal » en plus ne tient
            // pas dans celle d'un contrôle simple, et recouvrirait la fin de l'explication.
            let mut boutons = vec![(None, t.ouvrir_le_journal)];
            if c.journal_effacable {
                boutons.push((None, t.effacer_le_journal));
            }
            let place = place_des_boutons(ui, &boutons);
            carte_large(
                ui,
                p,
                icones::JOURNAL,
                t.journal,
                t.journal_explication,
                place,
                |ui| {
                    if bouton(ui, p, None, t.ouvrir_le_journal, Genre::Normal).clicked() {
                        l.action(cles::JOURNAL);
                    }
                    if c.journal_effacable
                        && bouton(ui, p, None, t.effacer_le_journal, Genre::Danger).clicked()
                    {
                        self.confirmer_effacer = true;
                    }
                },
            );
        }
    }

    /// Le journal détaillé, si le logiciel le propose.
    fn journal_detaille(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let Some(actif) = c.journal_detaille else {
            return;
        };
        let t = self.textes.clone();
        let mut v = actif;
        carte(
            ui,
            p,
            icones::JOURNAL,
            t.journal_detaille,
            t.journal_detaille_explication,
            |ui| {
                interrupteur_ecrit(ui, p, &t, &mut v);
            },
        );
        if v != actif {
            self.changer(
                l,
                cles::JOURNAL_DETAILLE,
                t.journal_detaille,
                Valeur::Booleen(actif),
                Valeur::Booleen(v),
            );
        }
    }

    /// « Remise à zéro » : « Rétablir les réglages par défaut », avec sa confirmation.
    fn remise_a_zero(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let t = self.textes.clone();
        titre_section(ui, t.remise_a_zero);
        if self.confirmer_retablir {
            let explication = l.retablir_explication();
            carte_large(
                ui,
                p,
                icones::RETABLIR,
                t.retablir_question,
                explication.as_deref().unwrap_or(t.retablir_question_explication),
                260.0,
                |ui| {
                    if bouton(ui, p, None, t.non, Genre::Normal).clicked() {
                        self.confirmer_retablir = false;
                    }
                    if bouton(ui, p, None, t.oui_retablir, Genre::Danger).clicked() {
                        self.confirmer_retablir = false;
                        l.action(cles::RETABLIR);
                    }
                },
            );
        } else {
            carte(
                ui,
                p,
                icones::RETABLIR,
                t.retablir,
                t.retablir_explication,
                |ui| {
                    if bouton(ui, p, None, t.retablir_bouton, Genre::Danger).clicked() {
                        self.confirmer_retablir = true;
                    }
                },
            );
        }
    }
}
