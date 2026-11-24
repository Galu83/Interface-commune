//! Le logiciel inventé des images de référence : « Fictif », des données inventées (règle n° 1),
//! qui montre chaque sorte de brique du volet, une action du petit menu et des pages propres
//! dans les Paramètres (une avec le bandeau « redémarrer », une avec un brouillon).

use std::sync::Mutex;

use interface_commune::briques::{Brique, Case, Pastille, PlusAppareil};
use interface_commune::parametres::Contexte;
use interface_commune::{
    Communs, Credit, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Reglages, Rgb, Valeur,
    couleurs, dessin, menu,
};

fn icone(t: u32, _: bool) -> Vec<u8> {
    [0x30, 0x80, 0xE0, 0xFF].repeat((t * t) as usize)
}

/// Un logo inventé : un carré vert, à toutes les tailles.
fn logo(t: u32) -> Option<Vec<u8>> {
    Some([0x20, 0xA0, 0x60, 0xFF].repeat((t * t) as usize))
}

/// L'identité du logiciel inventé.
pub fn identite() -> Identite {
    Identite {
        nom: "Fictif".into(),
        version: "2.4.0".into(),
        licence: "MIT".into(),
        description: "Un logiciel inventé pour les images de référence.".into(),
        icone,
        logo: Some(logo),
    }
}

/// Le logiciel inventé ; l'état de la mise à jour se change pour chaque image.
pub struct Fictif {
    identite: Identite,
    mise_a_jour: Mutex<EtatMiseAJour>,
}

impl Fictif {
    pub fn new() -> Fictif {
        Fictif {
            identite: identite(),
            mise_a_jour: Mutex::new(EtatMiseAJour::AJour),
        }
    }

    pub fn avec_mise_a_jour(e: EtatMiseAJour) -> Fictif {
        Fictif {
            identite: identite(),
            mise_a_jour: Mutex::new(e),
        }
    }
}

impl Reglages for Fictif {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _cle: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn accueil(&self) -> bool {
        true
    }

    fn pages(&self) -> Vec<Page> {
        vec![
            Page::new("ecrans", "Écrans", '\u{E7F4}'),
            Page::new("dispositions", "Dispositions", '\u{E8A9}'),
        ]
    }

    fn dessiner_page(&self, cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        let p = page.palette;
        if cle == "ecrans" {
            dessin::titre_section(ui, "Écran principal");
            let mut actif = true;
            dessin::carte(
                ui,
                p,
                '\u{E7F4}',
                "Garder l'écran allumé",
                "Un réglage inventé.",
                |ui| {
                    dessin::interrupteur_ecrit(ui, p, page.textes, &mut actif);
                },
            );
            dessin::carte_chevron(ui, p, '\u{E713}', "Réglages avancés", "Une sous-page inventée.");
            page.redemarrage_necessaire("Le changement d'écran demande un redémarrage.");
        } else {
            dessin::carte_info(ui, p, '\u{E946}', "Trois dispositions", "Des données inventées.");
            page.brouillon_modifie("Disposition du salon");
        }
    }

    fn dessiner_general(&self, ui: &mut egui::Ui, page: &mut Contexte) {
        dessin::carte(
            ui,
            page.palette,
            '\u{E7F4}',
            "Carte propre",
            "Un réglage inventé du logiciel.",
            |_| {},
        );
    }

    fn communs(&self) -> Communs {
        Communs {
            demarrage: Some(true),
            langue: Some(("Français".into(), vec!["Français".into(), "English".into()])),
            notifications: Some(false),
            notifications_essai: true,
            journal_detaille: Some(false),
            journal_effacable: true,
            mise_a_jour: self.mise_a_jour.lock().map(|e| e.clone()).unwrap_or_default(),
            derniere_recherche: Some("12 mars 2026 à 09:15".into()),
            dossier_installateurs: "C:\\Users\\exemple\\Installateurs".into(),
            dossier_fixe: false,
            dossier_automatique: Some(true),
            recherches: Some(true),
            fichier_licences: true,
            nouveautes: vec![
                "Une nouveauté inventée.".into(),
                "Une autre, plus longue, qui passe à la ligne dans la fenêtre.".into(),
            ],
            credits: vec![Credit {
                nom: "egui".into(),
                licence: "MIT OR Apache-2.0".into(),
            }],
            ..Default::default()
        }
    }

    fn changer(&self, _cle: &str, _valeur: Valeur) {}
}

/// Une pastille inventée.
fn pastille(lettre: char, rang: usize, nom: &str, absent: bool) -> Pastille {
    Pastille {
        cle: Some(format!("pastille_{rang}")),
        lettre,
        logo: None,
        couleur: couleurs::palette()[rang % 10],
        nom: nom.into(),
        mention: None,
        glyphe: None,
        absent,
    }
}

impl Logiciel for Fictif {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Attention,
            titre: "Bureau du salon".into(),
            ligne: "Un écran manque".into(),
            detail: Some("Une explication assez longue pour passer à la ligne dans le volet, sans jamais être coupée au bord droit.".into()),
        }
    }

    fn briques(&self) -> Vec<Brique> {
        let case = |lettre, rang: usize| Case {
            lettre: Some(lettre),
            couleur: Some(couleurs::palette()[rang]),
            ..Default::default()
        };
        vec![
            Brique::Section("Réglages rapides".into()),
            Brique::Interrupteur {
                cle: "veille".into(),
                glyphe: Some('\u{E708}'),
                texte: "Empêcher la veille".into(),
                actif: true,
            },
            Brique::Choix {
                cle: "seul".into(),
                glyphe: Some('\u{E7F4}'),
                texte: "Écran seul".into(),
                coche: true,
                raccourci: Some("Ctrl+Alt+F1".into()),
            },
            Brique::Segments {
                cle: "mode".into(),
                options: vec![("auto".into(), "Auto".into()), ("manuel".into(), "Manuel".into())],
                choisie: "auto".into(),
            },
            Brique::Section("Appareils".into()),
            Brique::Appareil {
                cle: Some("souris".into()),
                glyphe: '\u{E962}',
                nom: "Souris inventée".into(),
                pourcentage: Some(0.42),
                sous_ligne: Some("Branchée il y a 2 h".into()),
                plus: PlusAppareil {
                    teinte: Some(Rgb(0x2E, 0xA0, 0x43)),
                    en_charge: true,
                    ..Default::default()
                },
            },
            Brique::Appareil {
                cle: None,
                glyphe: '\u{E765}',
                nom: "Clavier inventé".into(),
                pourcentage: None,
                sous_ligne: Some("Éteint".into()),
                plus: PlusAppareil {
                    eteint: true,
                    bouton: Some(("oublier".into(), "Oublier".into())),
                    ..Default::default()
                },
            },
            Brique::Pastilles {
                titre: Some("Ordinateurs".into()),
                pastilles: vec![
                    pastille('A', 0, "Atelier", false),
                    pastille('B', 3, "Bureau", false),
                    pastille('C', 6, "Cuisine", true),
                ],
            },
            Brique::ChoixIllustre {
                cle: "disposition".into(),
                texte: "Disposition du salon".into(),
                sous_ligne: Some("Deux écrans".into()),
                illustration: vec![case('A', 0), case('B', 3)],
                raccourci: Some("Ctrl+Alt+F2".into()),
                actif: true,
                etiquette_active: Some("Active".into()),
            },
            Brique::Raccourcis(vec![("Ctrl+Alt+F1".into(), "Écran seul".into())]),
            Brique::Action {
                cle: "relire".into(),
                glyphe: '\u{E72C}',
                texte: "Relire les écrans".into(),
            },
        ]
    }

    fn actions_menu(&self) -> Vec<menu::Action> {
        vec![menu::Action::new("relire", '\u{E72C}', "Relire les écrans")]
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}
