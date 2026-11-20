//! Les couleurs dans les Paramètres, aux couleurs du volet : la pastille d'un appareil dans une
//! carte, et la rangée « choisir une couleur » (comme l'accent de Windows 11) avec la bulle
//! « Annuler ». Un logiciel inventé, des valeurs inventées.
//!
//! Aucune fenêtre : l'exemple écrit quatre captures PNG (clair et sombre, avant et après un
//! choix) : `cargo run --example couleurs [-- <dossier>]` (sans dossier : le dossier temporaire).
//!
//! Dans un vrai logiciel, la page tient en quelques lignes :
//!
//! ```ignore
//! if let Some(rang) = couleurs::carte_couleur(
//!     ui, page, glyphe, "Couleur du PC", "Elle colore aussi le volet.",
//!     &couleurs::palette(), rang_actuel, Some(("pc.couleur", "Couleur du PC")),
//! ) {
//!     // `Reglages::changer("pc.couleur", Valeur::Nombre(rang))` est aussi appelé : « Annuler »
//!     // y revient avec l'ancien rang. Avec `None` au lieu de `Some(..)`, c'est à toi de tout faire.
//! }
//! ```

use std::path::PathBuf;
use std::sync::Mutex;

use interface_commune::capture::{self, Options};
use interface_commune::couleurs::{carte_couleur, palette, pastille_de_carte};
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::pastilles::Pastille;
use interface_commune::{Communs, Identite, Page, Reglages, Valeur, dessin};

fn icone(taille: u32, _barre_sombre: bool) -> Vec<u8> {
    vec![255; (taille * taille * 4) as usize]
}

struct Exemple {
    identite: Identite,
    /// Les rangs des couleurs des deux ordinateurs inventés.
    rangs: Mutex<[usize; 2]>,
}

impl Exemple {
    fn rang(&self, pc: usize) -> usize {
        self.rangs.lock().map_or(0, |r| r[pc])
    }
}

impl Reglages for Exemple {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _cle: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("ordinateurs", "Ordinateurs", '\u{E7F4}')]
    }

    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        let noms = [
            ("Fixe", "F", "Cet ordinateur."),
            ("Portable", "P", "Dans la pièce d'à côté."),
        ];
        for (i, (nom, lettre, explication)) in noms.into_iter().enumerate() {
            let pastille = Pastille {
                cle: None,
                lettre: lettre.chars().next().unwrap_or('?'),
                couleur: palette()[self.rang(i)],
                nom: nom.into(),
                mention: None,
                glyphe: None,
                absent: false,
            };
            let palette_page = page.palette;
            dessin::carte(ui, palette_page, '\u{E7F4}', nom, explication, |ui| {
                pastille_de_carte(ui, palette_page, &pastille);
            });
            ui.add_space(4.0);
            let texte = format!("Couleur du {}", nom.to_lowercase());
            carte_couleur(
                ui,
                page,
                '\u{E790}',
                &texte,
                "Elle colore aussi le volet.",
                &palette(),
                self.rang(i),
                Some((&format!("couleur.{i}"), &texte)),
            );
            ui.add_space(12.0);
        }
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, cle: &str, valeur: Valeur) {
        if let (Some(i), Valeur::Nombre(n)) = (cle.strip_prefix("couleur."), valeur)
            && let Ok(i) = i.parse::<usize>()
            && let Ok(mut r) = self.rangs.lock()
            && i < 2
        {
            r[i] = n as usize;
        }
    }
}

fn principal() -> Result<(), String> {
    let dossier = std::env::args_os()
        .nth(1)
        .map_or_else(std::env::temp_dir, PathBuf::from);
    for (sombre, theme) in [(false, "clair"), (true, "sombre")] {
        let exemple = Exemple {
            identite: Identite {
                nom: "Exemple".into(),
                version: "1.0.0".into(),
                licence: "MIT".into(),
                description: "Un exemple de l'interface commune.".into(),
                icone,
                logo: None,
            },
            rangs: Mutex::new([0, 4]),
        };
        let mut parametres = Parametres::new(&exemple, Some("ordinateurs"));
        let options = Options {
            sombre,
            ..Options::default()
        };
        let fichier = dossier.join(format!("couleurs-{theme}.png"));
        capture::parametres_de(&mut parametres, &exemple, options)?.enregistrer(&fichier)?;
        println!("{}", fichier.display());
        // Le choix que ferait un clic sur la troisième couleur : appliqué, avec « Annuler ».
        parametres.changer(
            &exemple,
            "couleur.0",
            "Couleur du fixe",
            Valeur::Nombre(0.0),
            Valeur::Nombre(2.0),
        );
        let fichier = dossier.join(format!("couleurs-{theme}-choisie.png"));
        capture::parametres_de(&mut parametres, &exemple, options)?.enregistrer(&fichier)?;
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
