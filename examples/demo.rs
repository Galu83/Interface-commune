//! Un petit logiciel d'exemple : l'icône près de l'horloge, le volet, le petit menu et les
//! Paramètres de l'interface commune, avec des données inventées. Compilé par les
//! vérifications, jamais lancé par les tests (il ouvre une icône et des fenêtres).
//!
//! `cargo run --example demo` (Windows) ; `--parametres <page>` ouvre les Paramètres.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod demo {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    use interface_commune::apparence::couleur;
    use interface_commune::autres_fenetres::{Cadre, Contenu, Ouverture};
    use interface_commune::briques::Brique;
    use interface_commune::horloge::{Horloge, Poste, Reveil, instance_unique_avec};
    use interface_commune::parametres::Contexte;
    use interface_commune::theme::couverture;
    use interface_commune::toutes_faites::{self, Bouton, Champ, Lecture, ReponseLecture, Simple, Ton};
    use interface_commune::{
        Communs, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Reglages, Valeur, dessin, menu,
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
        /// La place des Paramètres, retenue le temps du lancement (essai de la réouverture
        /// sur un autre écran).
        place: Mutex<Option<interface_commune::place::Place>>,
    }

    impl Logiciel for Exemple {
        fn etat(&self) -> Etat {
            Etat {
                genre: GenreEtat::Pret,
                titre: "Exemple".into(),
                ligne: "Prêt".into(),
                detail: None,
            }
        }

        fn briques(&self) -> Vec<Brique> {
            vec![
                Brique::Interrupteur {
                    cle: "essai".into(),
                    glyphe: Some('\u{E7F4}'),
                    texte: "Mode essai".into(),
                    actif: self.rapide.load(Ordering::Relaxed),
                },
                Brique::Choix {
                    cle: "bonjour".into(),
                    glyphe: None,
                    texte: "Dire bonjour".into(),
                    coche: false,
                    raccourci: Some("Ctrl+Alt+B".into()),
                },
            ]
        }

        fn actions_menu(&self) -> Vec<menu::Action> {
            vec![
                menu::Action::new("bonjour", '\u{E8BD}', "Dire bonjour"),
                menu::Action::new("fenetre", '\u{E737}', "Ouvrir une fenêtre"),
                menu::Action::new("simple", '\u{E8A5}', "Fenêtre simple"),
                menu::Action::new("lecture", '\u{E8D6}', "Fenêtre de lecture"),
            ]
        }

        fn quitter(&self) {
            if let Ok(p) = self.poste.lock()
                && let Some(p) = p.as_ref()
            {
                p.quitter();
            }
        }

        fn noter(&self, _ligne: String) {}
    }

    impl Reglages for Exemple {
        fn identite(&self) -> &Identite {
            &self.identite
        }

        fn place_parametres(&self) -> Option<interface_commune::place::Place> {
            self.place.lock().ok().and_then(|p| *p)
        }

        fn retenir_place(&self, place: interface_commune::place::Place) {
            if let Ok(mut p) = self.place.lock() {
                *p = Some(place);
            }
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
                // Les fenêtres toutes faites.
                "simple" => {
                    let simple = Simple {
                        titre: "Garder comme action".into(),
                        sous_titre: "Un exemple de fenêtre simple.".into(),
                        statut: "Prêt.".into(),
                        ton: Ton::Bon,
                        barre: Some(0.6),
                        champ: Some(Champ {
                            cle: "nom".into(),
                            libelle: "Nom".into(),
                            valeur: "Mon action".into(),
                        }),
                        boutons: vec![
                            Bouton::new("annuler", "Annuler"),
                            Bouton::new("garder", "Garder").accent(),
                        ],
                        ..Default::default()
                    };
                    let _ = toutes_faites::ouvrir_simple(
                        &p.fenetres(),
                        "simple",
                        Ouverture::new("Exemple", [0.0, 0.0]),
                        self.textes(),
                        simple,
                        |_| {},
                    );
                }
                "lecture" => {
                    let texte = "Un texte inventé, lu à voix haute.\nCliquer sur un mot le surligne.";
                    let mut l = Lecture::de(texte);
                    l.mot = Some(0);
                    let poignee: Arc<Mutex<Option<toutes_faites::FenetreLecture>>> = Arc::default();
                    let p2 = poignee.clone();
                    let ouverte = toutes_faites::ouvrir_lecture(
                        &p.fenetres(),
                        "lecture",
                        Ouverture::new("Lecture", [520.0, 360.0]),
                        l,
                        None,
                        move |r| {
                            if let ReponseLecture::Mot(m) = r
                                && let Ok(g) = p2.lock()
                                && let Some(f) = g.as_ref()
                            {
                                f.changer(|l| l.mot = Some(m));
                            }
                        },
                    );
                    if let Ok(mut g) = poignee.lock() {
                        *g = ouverte;
                    }
                }
                _ => {}
            }
        }

        fn rechercher_mise_a_jour(&self) {}

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
            match (cle, valeur) {
                (interface_commune::cles::DEMARRAGE, Valeur::Booleen(b)) => {
                    self.demarrage.store(b, Ordering::Relaxed)
                }
                ("essai", Valeur::Booleen(b)) => self.rapide.store(b, Ordering::Relaxed),
                _ => {}
            }
        }
    }

    pub fn principal() {
        let args: Vec<String> = std::env::args().collect();
        // Un second lancement transmet ses arguments (`--parametres a_propos`) à l'instance en
        // cours, qui ouvre la page ; sans argument, il ouvre simplement les Paramètres.
        let reveil = match args.get(1..) {
            Some(a) if !a.is_empty() => Reveil::Arguments(a.to_vec()),
            _ => Reveil::Parametres,
        };
        let Ok(_verrou) = instance_unique_avec("Exemple-interface-commune", &reveil) else {
            return;
        };
        let horloge = Horloge::new("Exemple-interface-commune").sans_fantome();
        let pour_les_arguments = horloge.poste();
        let horloge = horloge.recevoir_arguments(move |recus| {
            let mut tout = vec![String::new()];
            tout.extend(recus);
            if let Some(page) = parametres::modele::page_demandee(&tout) {
                pour_les_arguments.ouvrir_parametres(page.as_deref());
            }
        });
        let exemple = std::sync::Arc::new(Exemple {
            identite: Identite {
                nom: "Exemple".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                licence: "MIT".into(),
                description: "Un exemple de l'interface commune.".into(),
                icone,
                logo: None,
            },
            rapide: AtomicBool::new(false),
            demarrage: AtomicBool::new(false),
            poste: Mutex::new(Some(horloge.poste())),
            place: Mutex::new(None),
        });
        let _ = horloge.lancer(exemple, parametres::modele::page_demandee(&args));
    }
}

fn main() {
    #[cfg(windows)]
    demo::principal();
}
