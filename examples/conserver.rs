//! « Conserver ces modifications ? » : un réglage qui peut couper l'écran, la souris, le
//! clavier ou la liaison est appliqué tout de suite, puis une carte demande de le garder ;
//! sans réponse en 15 s, tout revient comme avant (comme Windows après un changement de
//! résolution d'écran). Un logiciel inventé, des données inventées.
//!
//! Aucune fenêtre : l'exemple met en page les Paramètres avec la carte ouverte et écrit deux
//! captures PNG (clair et sombre) : `cargo run --example conserver [-- <dossier>]` (sans
//! dossier : le dossier temporaire).
//!
//! Dans un vrai logiciel, tout tient en une ligne dans la page du réglage :
//!
//! ```ignore
//! page.changer_a_confirmer("affichage.mode", "Mode d'affichage", avant, apres);
//! ```

use std::path::PathBuf;
use std::sync::Mutex;

use interface_commune::capture::{self, Options};
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::{Communs, Identite, Page, Reglages, Valeur, dessin};

/// Les modes d'affichage du logiciel inventé.
const MODES: [&str; 3] = ["Étendre", "Dupliquer", "Second écran seul"];

fn icone(taille: u32, _barre_sombre: bool) -> Vec<u8> {
    vec![255; (taille * taille * 4) as usize]
}

struct Exemple {
    identite: Identite,
    mode: Mutex<String>,
}

impl Reglages for Exemple {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _cle: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("affichage", "Affichage", '\u{E7F4}')]
    }

    /// La page « Affichage » : une liste de modes. Changer de mode peut couper l'écran : il
    /// est donc **à confirmer** (`changer_a_confirmer`), pas seulement changé.
    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        let actuel = self.mode.lock().map(|m| m.clone()).unwrap_or_default();
        let mut choix = actuel.clone();
        dessin::carte(
            ui,
            page.palette,
            '\u{E7F4}',
            "Mode d'affichage",
            "Comment les deux écrans sont utilisés.",
            |ui| {
                egui::ComboBox::from_id_salt("mode")
                    .selected_text(choix.clone())
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        for m in MODES {
                            ui.selectable_value(&mut choix, m.to_string(), m);
                        }
                    });
            },
        );
        if choix != actuel {
            page.changer_a_confirmer(
                "affichage.mode",
                "Mode d'affichage",
                Valeur::Texte(actuel),
                Valeur::Texte(choix),
            );
        }
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    /// Appelé pour tout changement, et aussi quand « Revenir » (ou le délai écoulé, ou la
    /// fermeture des Paramètres) remet la valeur d'avant : le même chemin qu'« Annuler ».
    fn changer(&self, cle: &str, valeur: Valeur) {
        if let ("affichage.mode", Valeur::Texte(t)) = (cle, valeur)
            && let Ok(mut m) = self.mode.lock()
        {
            *m = t;
        }
    }
}

fn principal() -> Result<(), String> {
    let dossier = std::env::args_os()
        .nth(1)
        .map_or_else(std::env::temp_dir, PathBuf::from);
    let exemple = Exemple {
        identite: Identite {
            nom: "Exemple".into(),
            version: "1.0.0".into(),
            licence: "MIT".into(),
            description: "Un exemple de l'interface commune.".into(),
            icone,
            logo: None,
        },
        mode: Mutex::new(MODES[0].to_string()),
    };
    for (sombre, theme) in [(false, "clair"), (true, "sombre")] {
        let mut parametres = Parametres::new(&exemple, Some("affichage"));
        // Le changement que ferait un clic dans la liste : appliqué, puis à confirmer.
        parametres.changer_a_confirmer(
            &exemple,
            "affichage.mode",
            "Mode d'affichage",
            Valeur::Texte(MODES[0].into()),
            Valeur::Texte(MODES[1].into()),
        );
        let image = capture::parametres_de(
            &mut parametres,
            &exemple,
            Options {
                sombre,
                ..Options::default()
            },
        )?;
        let fichier = dossier.join(format!("conserver-{theme}.png"));
        image.enregistrer(&fichier)?;
        println!("{}", fichier.display());
    }
    Ok(())
}

fn main() {
    if let Err(e) = principal() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
