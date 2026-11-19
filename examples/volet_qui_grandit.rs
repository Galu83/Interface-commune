//! Un volet à hauteur maximale (600 points) dont le contenu grandit : 2, puis 6, puis 30
//! appareils inventés. Sans aucune fenêtre : trois captures PNG, et leurs hauteurs sur la
//! sortie. Le volet doit grandir avec son contenu jusqu'à 600 points, puis défiler.
//!
//! `cargo run --example volet_qui_grandit [dossier]` (Windows : il faut les polices Segoe).

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use interface_commune::briques::Brique;
use interface_commune::capture::{self, Options};
use interface_commune::{Communs, Etat, GenreEtat, Identite, Logiciel, Reglages, Valeur};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

/// Un logiciel inventé dont le nombre d'appareils change.
struct Appareils {
    identite: Identite,
    nombre: AtomicUsize,
}

impl Reglages for Appareils {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

impl Logiciel for Appareils {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Démo".into(),
            ligne: format!("{} appareils", self.nombre.load(Ordering::Acquire)),
            detail: None,
        }
    }

    fn briques(&self) -> Vec<Brique> {
        (0..self.nombre.load(Ordering::Acquire))
            .map(|i| Brique::Appareil {
                cle: None,
                glyphe: '\u{E7F4}',
                nom: format!("Appareil {}", i + 1),
                pourcentage: Some(0.5),
                sous_ligne: Some("Vu à l'instant".into()),
                plus: Default::default(),
            })
            .collect()
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}

fn main() -> Result<(), String> {
    let dossier = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("volet-qui-grandit"));
    let l = Appareils {
        identite: Identite {
            nom: "Démo".into(),
            version: "1.2.3".into(),
            licence: "MIT".into(),
            description: "Un logiciel inventé pour les essais.".into(),
            icone,
            logo: None,
        },
        nombre: AtomicUsize::new(2),
    };
    for n in [2, 6, 30] {
        l.nombre.store(n, Ordering::Release);
        let image = capture::volet_borne(&l, 600.0, Options::default())?;
        let f = dossier.join(format!("volet-{n}-appareils.png"));
        image.enregistrer(&f)?;
        println!(
            "{n} appareils : {} x {} points -> {}",
            image.largeur,
            image.hauteur,
            f.display()
        );
    }
    Ok(())
}
