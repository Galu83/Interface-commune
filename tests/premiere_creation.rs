//! L'ouverture des Paramètres par une demande (`Demande::Parametres` : une 2e instance du
//! logiciel, l'entrée « Paramètres » du menu, un double-clic), dans la vraie boucle des fenêtres
//! au repos (la fenêtre principale cachée), doit réussir **au premier essai** : aucune ligne
//! « Windows n'a pas pu la créer » au journal.
//!
//! Rien n'est vu : la fenêtre principale est voilée (`volet::observer`) et les autres fenêtres
//! sont créées et dessinées mais jamais montrées (`autres_fenetres::garder_cachees`) : les
//! fenêtres sont vraies (egui, OpenGL), mais n'apparaissent sur aucun écran et ne prennent
//! jamais le focus. Un seul essai dans ce fichier : winit n'accepte qu'une boucle de fenêtres
//! par programme.

#![cfg(windows)]
#![allow(clippy::expect_used)]

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use interface_commune::autres_fenetres::garder_cachees;
use interface_commune::fenetres::{Demande, Liens, canal};
use interface_commune::parametres::CLE_FENETRE;
use interface_commune::volet::{observer, veiller};
use interface_commune::{Communs, Etat, GenreEtat, Identite, Logiciel, Reglages, Valeur, polices};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

struct Essai {
    identite: Identite,
    journal: Arc<Mutex<Vec<String>>>,
}

impl Reglages for Essai {
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

impl Logiciel for Essai {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Essai".into(),
            ligne: "Prêt".into(),
            detail: None,
        }
    }
    fn quitter(&self) {}
    fn noter(&self, ligne: String) {
        if let Ok(mut j) = self.journal.lock() {
            j.push(ligne);
        }
    }
}

#[test]
fn les_parametres_s_ouvrent_au_premier_essai() {
    // Mode d'essai : rien n'est montré, le clavier reste où il est.
    observer(Some(Box::new(|_| {})));
    garder_cachees(true);
    let journal = Arc::new(Mutex::new(Vec::<String>::new()));
    let j2 = journal.clone();
    interface_commune::sonde::brancher_le_journal(move |l| {
        if let Ok(mut j) = j2.lock() {
            j.push(l);
        }
    });
    let liens = Arc::new(Liens::default());
    let (demandes, reception) = canal();
    let logiciel: Arc<dyn Logiciel> = Arc::new(Essai {
        identite: Identite {
            nom: "Essai".into(),
            version: "0.0.0".into(),
            licence: "MIT".into(),
            description: "Première création.".into(),
            icone,
            logo: None,
        },
        journal: journal.clone(),
    });
    let polices = polices::polices().expect("polices");
    let (fini, fin) = mpsc::channel();
    let (l2, li2) = (logiciel.clone(), liens.clone());
    let fil = std::thread::spawn(move || {
        let r = veiller(l2, li2, polices, &reception, None);
        let _ = fini.send(format!("{:?}", r.as_ref().map(|_| ()).map_err(|e| e.0.clone())));
        r.map(|_| ()).map_err(|e| e.0)
    });
    // Au repos : la fenêtre principale est prête (cachée).
    let debut = Instant::now();
    while liens.surface_prete.load(Ordering::Acquire) == 0 {
        let fini_deja = fin.try_recv().ok();
        assert!(
            debut.elapsed() < Duration::from_secs(10) && fini_deja.is_none(),
            "la fenêtre principale n'a pas été préparée ({fini_deja:?}) : {:#?}",
            journal.lock().expect("journal")
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(300));

    // Une demande des Paramètres, comme celle d'une 2e instance ou du menu.
    demandes.send(Demande::Parametres(None)).expect("demande envoyée");
    // Les Paramètres sont créés et dessinés (cachés) : on laisse le
    // temps d'un éventuel nouvel essai (500 ms) et de l'ouverture.
    std::thread::sleep(Duration::from_millis(1_800));
    let ouverts = liens.autres.ouverte(CLE_FENETRE);
    let lignes = journal.lock().expect("journal").clone();

    liens.autres.tout_fermer();
    std::thread::sleep(Duration::from_millis(400));
    let _ = demandes.send(Demande::Quitter);
    let _ = fin.recv_timeout(Duration::from_secs(5));
    garder_cachees(false);
    observer(None);
    drop(fil);

    assert!(ouverts, "les Paramètres ne sont pas ouverts : {lignes:#?}");
    assert!(
        lignes.iter().all(|l| !l.contains("pas pu la créer")),
        "la création a raté au premier essai : {lignes:#?}"
    );
}
