//! Une section repliable dans les Paramètres, comme l'« Expander » de Windows 11 : une carte
//! (titre, explication) et un chevron qui tourne. Un logiciel inventé, des valeurs inventées.
//!
//! Aucune fenêtre : l'exemple écrit deux captures PNG en clair, la section repliée puis
//! ouverte (comme si on était arrivé par « Rechercher un paramètre ») :
//! `cargo run --example section_repliable [-- <dossier>]` (sans dossier : le dossier temporaire).
//!
//! Dans un vrai logiciel :
//!
//! ```ignore
//! page.section_repliable(ui, "Réglages avancés", "Pour les cas rares.", &["Délai"], |ui, page| {
//!     // Les cartes de la section ; `page` signale les changements comme d'habitude.
//! });
//! // Et les mêmes mots dans `Page::new(..).avec_mots(&["Réglages avancés", "Délai"])`,
//! // sinon « Rechercher un paramètre » ne trouve pas la page.
//! ```

use std::path::PathBuf;

use interface_commune::capture::{self, Options};
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::{Communs, Identite, Page, Reglages, dessin};

fn icone(taille: u32, _barre_sombre: bool) -> Vec<u8> {
    vec![255; (taille * taille * 4) as usize]
}

struct Exemple {
    identite: Identite,
}

impl Reglages for Exemple {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _cle: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("ecrans", "Écrans", '\u{E7F4}').avec_mots(&["Réglages avancés", "Délai de veille"])]
    }

    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        let p = page.palette;
        dessin::carte(ui, p, '\u{E7F4}', "Écran du bureau", "Branché en HDMI.", |ui| {
            dessin::bouton(ui, p, None, "Identifier", dessin::Genre::Normal);
        });
        ui.add_space(8.0);
        page.section_repliable_avec_icone(
            ui,
            '\u{E713}',
            "Réglages avancés",
            "Pour les cas rares.",
            &["Délai de veille"],
            |ui, page| {
                let p = page.palette;
                dessin::carte(ui, p, '\u{E823}', "Délai de veille", "Après 10 minutes.", |ui| {
                    dessin::bouton(ui, p, None, "Changer", dessin::Genre::Normal);
                });
                ui.add_space(4.0);
                dessin::carte(ui, p, '\u{E9D9}', "Détection", "Automatique.", |ui| {
                    dessin::bouton(ui, p, None, "Relancer", dessin::Genre::Normal);
                });
            },
        );
        ui.add_space(8.0);
        page.section_repliable(ui, "Une autre section", "Sans icône.", &[], |ui, page| {
            dessin::carte(ui, page.palette, '\u{E946}', "Rien à régler", "", |_| {});
        });
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _cle: &str, _valeur: interface_commune::Valeur) {}
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
    };
    let options = Options::default();
    let mut parametres = Parametres::new(&exemple, Some("ecrans"));
    let fichier = dossier.join("section-repliable-repliee.png");
    capture::parametres_de(&mut parametres, &exemple, options)?.enregistrer(&fichier)?;
    println!("{}", fichier.display());
    parametres.aller_a_par_recherche("ecrans", "delai veille");
    let fichier = dossier.join("section-repliable-ouverte.png");
    capture::parametres_de(&mut parametres, &exemple, options)?.enregistrer(&fichier)?;
    println!("{}", fichier.display());
    Ok(())
}

fn main() {
    if let Err(e) = principal() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
