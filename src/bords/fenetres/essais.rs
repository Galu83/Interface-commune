//! Les essais des boucles des fenêtres, sur le faux moteur ([`super::faux_moteur`]) : la
//! surface préparée une fois, la fenêtre perdue recréée et le clic refait, les pertes en
//! rafale, la supervision d'une boucle du logiciel, le dernier filet du fil.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use ic_coeur::gestes::ouverture::{Clic, Ouvert, decider_le_clic};
use ic_coeur::gestes::secours::{Fin, Perte, RECREATIONS_EN_RAFALE, Reprise};

use super::boucle::{GardeDuFil, Suite, boucle_des_fenetres, suite};
use super::canal::canal;
use super::faux_moteur::{Faux, Supervisee, Tour, menu, volet};
use super::liens::{Demande, Liens};
use crate::theme::Surface;

/// Règle : la première ouverture aussi rapide que les suivantes. La surface (volet et
/// petit menu) est préparée une fois au repos ; deux ouvertures (et plus) ne la
/// reconstruisent pas, ni les Paramètres, ouverts dans la même boucle. Une demande
/// arrivée avant la préparation n'est pas perdue.
#[test]
fn deux_ouvertures_ne_reconstruisent_pas_le_volet() {
    let liens = Liens::default();
    let (envoi, reception) = canal();
    for d in [
        Demande::Parametres(None),
        volet(),
        volet(),
        menu(),
        volet(),
        Demande::Parametres(Some("general".into())),
        menu(),
        volet(),
        Demande::Quitter,
    ] {
        envoi.send(d).expect("demande");
    }
    let mut faux = Faux::default();
    boucle_des_fenetres(&liens, &mut faux, &reception);
    assert_eq!(faux.preparations, 1, "{:?}", faux.ouvertures);
    assert_eq!(
        faux.ouvertures,
        vec![
            (Surface::Volet, 1),
            (Surface::Volet, 1),
            (Surface::Menu, 1),
            (Surface::Volet, 1),
            (Surface::Menu, 1),
            (Surface::Volet, 1),
        ]
    );
    assert_eq!(faux.parametres, vec![None, Some("general".to_string())]);
}

/// Le défaut corrigé : la fenêtre du volet perdue (une panique dans la boucle des
/// fenêtres, après une nuit d'écrans en veille) tuait le fil des fenêtres en silence, et
/// plus aucun clic n'ouvrait rien. Maintenant elle est recréée tout de suite, le clic en
/// cours est refait, les clics suivants marchent, et le journal dit la perte et sa cause.
#[test]
fn fenetre_perdue_recreee_et_le_clic_ouvre_quand_meme() {
    let liens = Arc::new(Liens::default());
    let (envoi, reception) = canal();
    for d in [volet(), menu(), volet(), menu(), Demande::Quitter] {
        envoi.send(d).expect("demande");
    }
    let mut faux = Faux {
        liens: Some(liens.clone()),
        pertes_au_menu: 1,
        ..Faux::default()
    };
    boucle_des_fenetres(&liens, &mut faux, &reception);
    assert_eq!(faux.preparations, 2, "recréée une fois");
    assert_eq!(
        faux.ouvertures,
        vec![
            (Surface::Volet, 1),
            (Surface::Menu, 2),
            (Surface::Volet, 2),
            (Surface::Menu, 2),
        ]
    );
    assert_eq!(faux.secours, 0);
    assert!(!liens.egui_impossible.load(Ordering::Acquire));
    assert_eq!(faux.notes.len(), 1, "{:?}", faux.notes);
    let note = &faux.notes[0];
    assert!(note.contains("perdue"), "{note}");
    assert!(note.contains("contexte OpenGL perdu (essai)"), "{note}");
    assert!(note.contains("recréée"), "{note}");
    assert!(note.contains("le petit menu demandé est rouvert"), "{note}");
}

/// Des pertes en rafale ne font jamais tourner la boucle à vide : au-delà de trois en une
/// minute, le clic reçoit le menu de secours, la fenêtre attend le clic suivant pour être
/// recréée, et ce clic-là ouvre le volet. Chaque perte est notée.
#[test]
fn pertes_en_rafale_menu_de_secours_puis_recreee_au_clic_suivant() {
    let liens = Arc::new(Liens::default());
    let (envoi, reception) = canal();
    for d in [menu(), volet(), Demande::Quitter] {
        envoi.send(d).expect("demande");
    }
    let mut faux = Faux {
        liens: Some(liens.clone()),
        pertes_au_menu: usize::MAX,
        ..Faux::default()
    };
    boucle_des_fenetres(&liens, &mut faux, &reception);
    // Le menu : perdu, puis refait trois fois (trois recréations), puis le secours.
    assert_eq!(faux.secours, 1);
    assert_eq!(faux.notes.len(), RECREATIONS_EN_RAFALE + 1, "{:?}", faux.notes);
    assert!(faux.notes[RECREATIONS_EN_RAFALE].contains("menu de secours pour ce clic"));
    // Le volet, au clic suivant : la fenêtre est recréée et l'ouvre.
    assert_eq!(faux.preparations, RECREATIONS_EN_RAFALE + 2);
    assert_eq!(faux.ouvertures, vec![(Surface::Volet, RECREATIONS_EN_RAFALE + 2)]);
    assert!(!liens.egui_impossible.load(Ordering::Acquire));
}

/// La boucle d'un logiciel perdue (panique) est recréée tout de suite, trois fois en une
/// minute ; la quatrième perte fait une rafale : le menu de Windows pour ce clic, puis la
/// supervision attend le clic suivant et la recrée (jamais bloquée jusqu'au prochain
/// lancement). La cause de la panique est dans le journal.
#[test]
fn supervision_rafale_puis_reprise_au_clic_suivant() {
    let mut s = Supervisee::new(&[
        (0, Tour::Panique),
        (1, Tour::Panique),
        (2, Tour::Panique),
        (3, Tour::Panique),
        // Recréée au clic suivant : elle tient, puis le logiciel se ferme.
        (10, Tour::Quitter),
    ]);
    assert_eq!(s.superviser(), Fin::Quitter);
    assert_eq!(s.tours, 5);
    assert_eq!(
        s.reprises(),
        vec![
            Reprise::Maintenant,
            Reprise::Maintenant,
            Reprise::Maintenant,
            Reprise::AuClicSuivant
        ]
    );
    assert_eq!(s.attentes, 1);
    let e = &s.evenements[3];
    assert_eq!(e.nombre, RECREATIONS_EN_RAFALE + 1);
    assert!(
        matches!(&e.perte, Perte::Panique(c) if c.contains("contexte OpenGL perdu (essai)")),
        "{e:?}"
    );
    let phrase = e.phrase();
    assert!(phrase.contains("contexte OpenGL perdu (essai)"), "{phrase}");
    assert!(phrase.contains("4 fois en 60 s"), "{phrase}");
    assert!(phrase.contains("recréées au clic suivant"), "{phrase}");
    assert!(s.evenements[0].phrase().ends_with(": recréées."));
}

/// Après une rafale, une nouvelle perte dans la même minute redonne le menu de Windows et
/// attend encore le clic suivant (jamais de boucle folle) ; une minute plus tard, les
/// vieilles pertes sont oubliées et la boucle est de nouveau recréée tout de suite.
#[test]
fn supervision_reprise_apres_rafale() {
    let mut s = Supervisee::new(&[
        (0, Tour::Panique),
        (0, Tour::Panique),
        (0, Tour::Panique),
        (0, Tour::Panique),
        (5, Tour::Panique),
        (120, Tour::Panique),
        (121, Tour::Quitter),
    ]);
    assert_eq!(s.superviser(), Fin::Quitter);
    assert_eq!(
        s.reprises(),
        vec![
            Reprise::Maintenant,
            Reprise::Maintenant,
            Reprise::Maintenant,
            Reprise::AuClicSuivant,
            Reprise::AuClicSuivant,
            Reprise::Maintenant,
        ]
    );
    assert_eq!(s.attentes, 2);
    assert_eq!(s.evenements[5].nombre, 1);
}

/// Des pertes espacées de plus d'une minute ne font jamais de rafale : toutes recréées
/// tout de suite. Une fermeture par Windows compte comme une perte.
#[test]
fn supervision_pertes_espacees_toujours_recreees() {
    let mut script: Vec<(u64, Tour)> = (0..10)
        .map(|i| {
            let tour = if i % 2 == 0 { Tour::Panique } else { Tour::Fermee };
            (i * 61, tour)
        })
        .collect();
    script.push((1_000, Tour::Quitter));
    let mut s = Supervisee::new(&script);
    assert_eq!(s.superviser(), Fin::Quitter);
    assert_eq!(s.tours, 11);
    assert_eq!(s.reprises(), vec![Reprise::Maintenant; 10]);
    assert_eq!(s.attentes, 0);
    assert_eq!(s.evenements[1].perte, Perte::Fermee);
    assert!(s.evenements[1].phrase().contains("fermées par Windows"));
}

/// Un échec au tout premier tour (ni OpenGL ni fenêtre) : le menu de Windows pour de bon.
/// Un échec après un premier tour est passager : le menu de Windows pour ce clic, un
/// nouvel essai au clic suivant, sans compter comme une perte.
#[test]
fn supervision_echec_premier_tour_ou_passager() {
    let mut s = Supervisee::new(&[(0, Tour::Erreur)]);
    assert_eq!(
        s.superviser(),
        Fin::Impossible("OpenGL indisponible (essai)".into())
    );
    assert!(s.evenements.is_empty());

    let mut s = Supervisee::new(&[(0, Tour::Panique), (1, Tour::Erreur), (2, Tour::Quitter)]);
    assert_eq!(s.superviser(), Fin::Quitter);
    assert_eq!(s.reprises(), vec![Reprise::Maintenant, Reprise::AuClicSuivant]);
    assert_eq!(s.evenements[1].nombre, 1, "l'échec ne compte pas");
    assert_eq!(s.attentes, 1);
    let phrase = s.evenements[1].phrase();
    assert!(phrase.contains("OpenGL indisponible (essai)"), "{phrase}");
    assert!(phrase.contains("nouvel essai au clic suivant"), "{phrase}");
}

/// Le logiciel se ferme : pendant la pause d'une rafale, ou à la fin d'un tour, rien
/// n'est recréé.
#[test]
fn supervision_quitter() {
    let mut s = Supervisee::new(&[(0, Tour::Panique); 4]);
    s.ferme_en_pause = true;
    assert_eq!(s.superviser(), Fin::Quitter);
    assert_eq!((s.tours, s.attentes), (4, 1));

    let mut s = Supervisee::new(&[(0, Tour::Quitter)]);
    assert_eq!(s.superviser(), Fin::Quitter);
    assert!(s.evenements.is_empty());
}

/// Une panique hors de la boucle arrête le fil des fenêtres : la garde fait passer l'icône
/// au menu de secours (un clic ouvre toujours quelque chose) et le note. À la fermeture
/// normale du logiciel, rien de tel.
#[test]
fn fil_des_fenetres_arrete_menu_de_secours() {
    let liens = Arc::new(Liens::default());
    liens
        .surface_prete
        .store(1234, std::sync::atomic::Ordering::Release);
    liens.mettre(Ouvert::Surface(Surface::Volet));
    let notes = Arc::new(Mutex::new(Vec::<String>::new()));
    let (l2, n2) = (liens.clone(), notes.clone());
    let fil = std::thread::spawn(move || {
        let _garde = GardeDuFil::new(l2, move |t| n2.lock().expect("notes").push(t));
        panic!("panique hors de la boucle (essai)");
    });
    assert!(fil.join().is_err());
    assert!(liens.egui_impossible.load(Ordering::Acquire));
    assert_eq!(liens.surface_prete.load(Ordering::Acquire), 0);
    assert_eq!(liens.ouvert(), Ouvert::Rien);
    let notes = notes.lock().expect("notes");
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("panique"), "{}", notes[0]);

    let liens = Arc::new(Liens::default());
    liens.tout_fermer();
    drop(GardeDuFil::new(liens.clone(), |_| panic!("rien à noter")));
    assert!(!liens.egui_impossible.load(Ordering::Acquire));
}

/// Le logiciel se ferme : plus rien n'est préparé.
#[test]
fn rien_n_est_prepare_apres_quitter() {
    let liens = Liens::default();
    liens.tout_fermer();
    let (_envoi, reception) = canal();
    let mut faux = Faux::default();
    boucle_des_fenetres(&liens, &mut faux, &reception);
    assert_eq!(faux.preparations, 0);
}

/// Sans OpenGL ni polices : jamais de surface, le menu de secours à la place.
#[test]
fn sans_opengl_le_menu_de_secours() {
    let liens = Liens::default();
    liens.egui_impossible.store(true, Ordering::Release);
    let (envoi, reception) = canal();
    for d in [volet(), menu(), Demande::Parametres(None), Demande::Quitter] {
        envoi.send(d).expect("demande");
    }
    let mut faux = Faux::default();
    boucle_des_fenetres(&liens, &mut faux, &reception);
    assert_eq!(faux.preparations, 0);
    assert_eq!(faux.secours, 2);
    assert_eq!(faux.parametres, vec![None]);
}

/// Le volet ouvert devant les Paramètres : `ouvert()` rend la surface ; une fois la surface
/// fermée, il redonne les Paramètres (restés ouverts derrière).
#[test]
fn surface_devant_les_parametres_puis_retour() {
    struct Rien;
    impl crate::autres_fenetres::Contenu for Rien {
        fn dessiner(&mut self, _: &mut egui::Ui, _: &mut crate::autres_fenetres::Cadre<'_>) {}
    }
    let liens = Liens::default();
    liens.autres.ouvrir(
        crate::parametres::CLE_FENETRE,
        crate::autres_fenetres::Ouverture::new("Paramètres", [100.0, 100.0]),
        Rien,
    );
    assert_eq!(liens.ouvert(), Ouvert::Parametres);
    let clic = decider_le_clic(liens.ouvert(), Surface::Menu, liens.juste_fermee(Surface::Menu));
    assert_eq!(clic, Clic::Ouvrir);
    liens.mettre(Ouvert::Surface(Surface::Menu));
    assert_eq!(liens.ouvert(), Ouvert::Surface(Surface::Menu));
    assert!(liens.autres.ouverte(crate::parametres::CLE_FENETRE));
    // Fermeture de la surface (perte du focus comprise) : les Paramètres sont toujours là.
    liens.mettre(Ouvert::Rien);
    assert_eq!(liens.ouvert(), Ouvert::Parametres);
    assert!(liens.autres.ouverte(crate::parametres::CLE_FENETRE));
}

/// Les Paramètres ouverts dans la boucle comptent comme « ouverts » pour l'icône.
#[test]
fn parametres_ouverts_dans_la_boucle() {
    struct Rien;
    impl crate::autres_fenetres::Contenu for Rien {
        fn dessiner(&mut self, _: &mut egui::Ui, _: &mut crate::autres_fenetres::Cadre<'_>) {}
    }
    let liens = Liens::default();
    assert_eq!(liens.ouvert(), Ouvert::Rien);
    liens.autres.ouvrir(
        crate::parametres::CLE_FENETRE,
        crate::autres_fenetres::Ouverture::new("Paramètres", [100.0, 100.0]),
        Rien,
    );
    assert_eq!(liens.ouvert(), Ouvert::Parametres);
    liens.mettre(Ouvert::Surface(Surface::Volet));
    assert_eq!(liens.ouvert(), Ouvert::Surface(Surface::Volet));
}

/// Clic gauche : le volet ; clic droit : le petit menu ; une autre fenêtre : on rend la
/// main. Un deuxième clic pendant l'ouverture ne rouvre rien.
#[test]
fn suite_des_demandes() {
    assert_eq!(suite(&volet(), None), Suite::Montrer(Surface::Volet));
    assert_eq!(suite(&menu(), None), Suite::Montrer(Surface::Menu));
    assert_eq!(suite(&volet(), Some(Surface::Volet)), Suite::Ignorer);
    assert_eq!(
        suite(&menu(), Some(Surface::Volet)),
        Suite::Montrer(Surface::Menu)
    );
    assert_eq!(suite(&Demande::Parametres(None), None), Suite::Parametres);

    assert_eq!(suite(&Demande::Quitter, Some(Surface::Menu)), Suite::Sortir);
}
