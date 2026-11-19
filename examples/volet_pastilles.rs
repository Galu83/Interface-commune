//! Le volet d'un partage de clavier et de souris inventé : une rangée d'ordinateurs en
//! pastilles et des choix illustrés (une case par écran). Sans aucune fenêtre : des captures
//! PNG en clair et en sombre, avec 2 puis 4 ordinateurs, dans le dossier donné.
//!
//! `cargo run --example volet_pastilles [dossier]` (Windows : il faut les polices Segoe).

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use interface_commune::apparence::couleurs_pastilles;
use interface_commune::briques::{Brique, Case, Pastille};
use interface_commune::capture::{self, Options};
use interface_commune::{Communs, Etat, GenreEtat, Identite, Logiciel, Reglages, Valeur};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

struct Partage {
    identite: Identite,
    ordinateurs: AtomicUsize,
}

impl Reglages for Partage {
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

impl Logiciel for Partage {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Démo".into(),
            ligne: "Prêt · Bureau".into(),
            detail: Some("Clavier et souris : sur ce PC".into()),
        }
    }

    fn briques(&self) -> Vec<Brique> {
        let couleurs = couleurs_pastilles();
        let noms = [("F", "Fixe"), ("P", "Portable"), ("T", "Tour"), ("B", "Bureau")];
        let n = self.ordinateurs.load(Ordering::Acquire);
        let pastilles = (0..n)
            .map(|i| Pastille {
                cle: Some(format!("pc{i}")),
                lettre: noms[i].0.chars().next().unwrap_or('?'),
                couleur: couleurs[i],
                nom: noms[i].1.into(),
                mention: (i == 0).then(|| "ce PC".into()),
                glyphe: (i == 0).then_some('\u{E962}'),
                absent: n == 4 && i == 3,
            })
            .collect();
        let case = |i: usize| Case {
            lettre: noms[i].0.chars().next(),
            couleur: Some(couleurs[i]),
            veille: false,
        };
        let veille = Case {
            lettre: None,
            couleur: None,
            veille: true,
        };
        let choix = |cle: &str, texte: &str, sous: &str, rac: &str, actif: bool, cases: Vec<Case>| {
            Brique::ChoixIllustre {
                cle: cle.into(),
                texte: texte.into(),
                sous_ligne: Some(sous.into()),
                illustration: cases,
                raccourci: Some(rac.into()),
                actif,
                etiquette_active: Some("Affiché".into()),
            }
        };
        let mut v = vec![
            Brique::Pastilles {
                titre: Some("Ordinateurs".into()),
                pastilles,
            },
            Brique::Section("Ce qu'affichent tes écrans".into()),
            choix(
                "bureau",
                "Bureau",
                "Fixe à gauche · Portable à droite",
                "Ctrl+Alt+Maj+F1",
                true,
                vec![case(0), case(1)],
            ),
            choix(
                "tout-fixe",
                "Tout sur le fixe",
                "Fixe sur les 2 écrans",
                "Ctrl+Alt+Maj+F2",
                false,
                vec![case(0), case(0)],
            ),
            choix(
                "soiree",
                "Soirée",
                "Fixe à gauche · droite en veille",
                "Ctrl+Alt+Maj+F3",
                false,
                vec![case(0), veille],
            ),
        ];
        if n == 4 {
            v.push(choix(
                "quatre",
                "Les quatre",
                "Un écran par ordinateur",
                "Ctrl+Alt+Maj+F4",
                false,
                vec![case(0), case(1), case(2), case(3)],
            ));
        }
        v
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}

fn main() -> Result<(), String> {
    let dossier = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("volet-pastilles"));
    let l = Partage {
        identite: Identite {
            nom: "Démo".into(),
            version: "1.2.3".into(),
            licence: "MIT".into(),
            description: "Un logiciel inventé pour les essais.".into(),
            icone,
            logo: None,
        },
        ordinateurs: AtomicUsize::new(2),
    };
    for n in [2, 4] {
        l.ordinateurs.store(n, Ordering::Release);
        for (sombre, theme) in [(false, "clair"), (true, "sombre")] {
            let image = capture::volet(&l, Options { sombre, echelle: 2.0 })?;
            let f = dossier.join(format!("volet-pastilles-{n}-pc-{theme}.png"));
            image.enregistrer(&f)?;
            println!(
                "{n} PC, {theme} : {} x {} -> {}",
                image.largeur,
                image.hauteur,
                f.display()
            );
        }
    }
    Ok(())
}
