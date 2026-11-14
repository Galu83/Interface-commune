//! Un petit logiciel d'exemple : l'icône près de l'horloge, le volet, le petit menu et les
//! Paramètres de l'interface commune, avec des données inventées. Compilé par les
//! vérifications, jamais lancé par les tests (il ouvre une icône et des fenêtres).
//!
//! `cargo run --example demo` (Windows) ; `--parametres <page>` ouvre les Paramètres.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod demo {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    use interface_commune::apparence::couleur;
    use interface_commune::autres_fenetres::{Cadre, Contenu, Ouverture};
    use interface_commune::horloge::{Horloge, Poste, instance_unique};
    use interface_commune::parametres::Contexte;
    use interface_commune::theme::couverture;
    use interface_commune::{
        Communs, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Palette, Valeur, dessin, menu,
        parametres,
    };

    /// L'icône de l'exemple : un carré arrondi, de la couleur du texte de la barre des tâches.
    fn icone(taille: u32, barre_sombre: bool) -> Vec<u8> {
        let t = taille.clamp(16, 256) as usize;
        let c: u8 = if barre_sombre { 255 } else { 0x1A };
        let s = t as f32 / 32.0;
        let mut px = vec![0u8; t * t * 4];
        for y in 0..t {
            for x in 0..t {
                let a = couverture(x, y, s, |fx, fy| {
                    (6.0..26.0).contains(&fx) && (6.0..26.0).contains(&fy)
                });
                let i = (y * t + x) * 4;
                px[i..i + 4].copy_from_slice(&[c, c, c, a]);
            }
        }
        px
    }

    /// Une autre fenêtre : un mot et un bouton qui la ferme.
    struct Bonjour;

    impl Contenu for Bonjour {
        fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
            let p = cadre.palette;
            let mut fermer = false;
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(couleur(p.fond)).inner_margin(16))
                .show(ui, |ui| {
                    ui.label("Une autre fenêtre, dans la même boucle.");
                    fermer = dessin::bouton(ui, p, None, "Fermer", dessin::Genre::Accent).clicked();
                });
            if fermer {
                cadre.fermer();
            }
        }
    }

    struct Exemple {
        identite: Identite,
        rapide: AtomicBool,
        demarrage: AtomicBool,
        poste: Mutex<Option<Poste>>,
    }

    impl Logiciel for Exemple {
        fn identite(&self) -> &Identite {
            &self.identite
        }

        fn etat(&self) -> Etat {
            Etat {
                genre: GenreEtat::Pret,
                titre: "Exemple".into(),
                ligne: "Prêt".into(),
                detail: None,
            }
        }

        fn dessiner_rapides(&self, ui: &mut egui::Ui, p: &Palette) -> Option<String> {
            let mut actif = self.rapide.load(Ordering::Relaxed);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.label("Mode essai");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    dessin::interrupteur(ui, p, &mut actif);
                });
            });
            self.rapide.store(actif, Ordering::Relaxed);
            None
        }

        fn actions_menu(&self) -> Vec<menu::Action> {
            vec![
                menu::Action::new("bonjour", '\u{E8BD}', "Dire bonjour"),
                menu::Action::new("fenetre", '\u{E737}', "Ouvrir une fenêtre"),
            ]
        }

        fn action(&self, cle: &str) {
            let Ok(p) = self.poste.lock() else { return };
            let Some(p) = p.as_ref() else { return };
            match cle {
                "bonjour" => p.notifier("Bonjour !", false, None),
                // Une autre fenêtre, dans la boucle de la bibliothèque.
                "fenetre" => {
                    p.fenetres()
                        .ouvrir("bonjour", Ouverture::new("Bonjour", [360.0, 160.0]), Bonjour);
                }
                _ => {}
            }
        }

        fn rechercher_mise_a_jour(&self) {}

        fn quitter(&self) {
            if let Ok(p) = self.poste.lock()
                && let Some(p) = p.as_ref()
            {
                p.quitter();
            }
        }

        fn pages(&self) -> Vec<Page> {
            vec![Page::new("exemple", "Exemple", '\u{E8BD}')]
        }

        fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
            dessin::carte_info(
                ui,
                page.palette,
                '\u{E946}',
                "Une page propre",
                "Rien à régler ici.",
            );
        }

        fn communs(&self) -> Communs {
            Communs {
                demarrage: Some(self.demarrage.load(Ordering::Relaxed)),
                mise_a_jour: EtatMiseAJour::AJour,
                dossier_installateurs: "Installateurs".into(),
                ..Default::default()
            }
        }

        fn changer(&self, cle: &str, valeur: Valeur) {
            if let (interface_commune::cles::DEMARRAGE, Valeur::Booleen(b)) = (cle, valeur) {
                self.demarrage.store(b, Ordering::Relaxed);
            }
        }

        fn noter(&self, _ligne: String) {}
    }

    pub fn principal() {
        let Ok(_verrou) = instance_unique("Exemple-interface-commune") else {
            return;
        };
        let args: Vec<String> = std::env::args().collect();
        let horloge = Horloge::new("Exemple-interface-commune");
        let exemple = std::sync::Arc::new(Exemple {
            identite: Identite {
                nom: "Exemple".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                licence: "MIT".into(),
                description: "Un exemple de l'interface commune.".into(),
                icone,
                glyphe: '\u{E8BD}',
            },
            rapide: AtomicBool::new(false),
            demarrage: AtomicBool::new(false),
            poste: Mutex::new(Some(horloge.poste())),
        });
        let _ = horloge.lancer(exemple, parametres::modele::page_demandee(&args));
    }
}

fn main() {
    #[cfg(windows)]
    demo::principal();
}
