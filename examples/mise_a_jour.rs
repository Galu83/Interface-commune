//! La page « Mise à jour » à la façon de Windows Update : une icône selon l'état, un titre qui
//! le dit, une ligne grise (version, dernière recherche) et un seul bouton principal. Un
//! logiciel inventé, des valeurs inventées.
//!
//! Aucune fenêtre : l'exemple écrit trois captures PNG en clair (à jour, version disponible,
//! erreur) : `cargo run --example mise_a_jour [-- <dossier>]` (sans dossier : le dossier
//! temporaire).

use std::path::PathBuf;
use std::sync::Mutex;

use interface_commune::capture::{self, Options};
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::{Communs, EtatMiseAJour, Identite, Page, Reglages};

fn icone(taille: u32, _barre_sombre: bool) -> Vec<u8> {
    vec![255; (taille * taille * 4) as usize]
}

struct Exemple {
    identite: Identite,
    etat: Mutex<EtatMiseAJour>,
}

impl Reglages for Exemple {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _cle: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        Vec::new()
    }

    fn dessiner_page(&self, _cle: &str, _ui: &mut egui::Ui, _page: &mut Contexte) {}

    fn communs(&self) -> Communs {
        Communs {
            mise_a_jour: self.etat.lock().map(|e| e.clone()).unwrap_or_default(),
            dossier_installateurs: "Installateurs".into(),
            dossier_automatique: Some(true),
            recherches: Some(true),
            nouveautes: vec!["Une couleur de plus.".into()],
            derniere_recherche: Some("20 nov. 2026, 14:05".into()),
            ..Default::default()
        }
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
        etat: Mutex::new(EtatMiseAJour::AJour),
    };
    let etats = [
        ("a-jour", EtatMiseAJour::AJour),
        ("disponible", EtatMiseAJour::Prete("1.1.0".into())),
        (
            "erreur",
            EtatMiseAJour::Erreur("Le dossier des installateurs est introuvable.".into()),
        ),
    ];
    for (nom, etat) in etats {
        if let Ok(mut e) = exemple.etat.lock() {
            *e = etat;
        }
        let mut parametres = Parametres::new(&exemple, Some("mise_a_jour"));
        let fichier = dossier.join(format!("mise-a-jour-{nom}.png"));
        capture::parametres_de(&mut parametres, &exemple, Options::default())?.enregistrer(&fichier)?;
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
