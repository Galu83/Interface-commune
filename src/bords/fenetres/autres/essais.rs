//! Les essais des autres fenêtres, sur le faux moteur egui
//! ([`crate::bords::fenetres::faux_moteur`]) : egui croit avoir de vraies fenêtres, rien ne
//! s'ouvre.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use crate::bords::fenetres::faux_moteur::{CREATION_RATEE, CROIX, contexte, image};

use super::delais::VERIFICATION;
use super::*;

#[derive(Clone, Default)]
struct Trace(Arc<Mutex<Vec<String>>>);

struct Essai {
    nom: &'static str,
    trace: Trace,
    fermer_a: Option<u32>,
}

impl Contenu for Essai {
    fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
        ui.label(self.nom);
        self.trace.0.lock().unwrap().push(format!(
            "{} image {} montrée {}",
            self.nom, cadre.image, cadre.montree
        ));
        if Some(cadre.image) == self.fermer_a {
            cadre.fermer();
        }
    }

    fn fermee(&mut self, _place: Option<Place>) {
        self.trace.0.lock().unwrap().push(format!("{} fermée", self.nom));
    }
}

/// Le défaut corrigé : eframe ne peut pas créer la fenêtre (Windows refait l'affichage) et
/// ne dessine rien ; egui paniquait (« the user callback was never called ») et la panique
/// emportait la boucle des fenêtres. Maintenant : aucune panique, la fenêtre reste et
/// réessaie ; dessinée dès que la création réussit ; après trois échecs de suite, fermée.
#[test]
fn creation_ratee_sans_panique_puis_nouvel_essai() {
    let ctx = contexte();
    let f = AutresFenetres::default();
    let trace = Trace::default();
    f.ouvrir(
        "p",
        Ouverture::new("P", [200.0, 100.0]),
        Essai {
            nom: "p",
            trace: trace.clone(),
            fermer_a: None,
        },
    );
    CREATION_RATEE.with(|c| c.set(true));
    let (dessinees, _) = image(&ctx, &f);
    assert!(dessinees.is_empty());
    assert!(
        f.ouverte("p"),
        "une création ratée : la fenêtre attend un nouvel essai"
    );
    // Elle attend ~500 ms : les images qui suivent ne refont pas l'essai (ni ne le
    // comptent comme un échec de plus), et la fenêtre principale est réveillée après
    // l'attente, pas tout de suite.
    let attente = f.prochaine_image().expect("un nouvel essai est à faire");
    assert!(attente > std::time::Duration::from_millis(100), "{attente:?}");
    assert!(attente <= NOUVEL_ESSAI, "{attente:?}");
    for _ in 0..5 {
        let _ = image(&ctx, &f);
    }
    assert!(f.ouverte("p"), "pas d'essai avant la fin de l'attente");
    // La création réussit à l'essai suivant : dessinée, le compte repart.
    f.sans_attente();
    CREATION_RATEE.with(|c| c.set(false));
    let (dessinees, _) = image(&ctx, &f);
    assert_eq!(dessinees, vec![id_de("p")]);
    assert!(trace.0.lock().unwrap().iter().any(|l| l.starts_with("p image")));
    // Trois échecs de suite : fermée (son contenu l'apprend), la boucle continue.
    CREATION_RATEE.with(|c| c.set(true));
    for _ in 0..CREATIONS_MAX {
        f.sans_attente();
        let _ = image(&ctx, &f);
    }
    CREATION_RATEE.with(|c| c.set(false));
    assert!(!f.ouverte("p"), "abandonnée après {CREATIONS_MAX} échecs");
    assert!(trace.0.lock().unwrap().iter().any(|l| l == "p fermée"));
}

/// L'attente avant la création d'une fenêtre : après un changement d'affichage, jusqu'à la
/// fin de la reprise ; après une création ratée, jusqu'au nouvel essai ; le plus long des
/// deux ; rien sinon (dessiner tout de suite).
#[test]
fn attendre_ou_dessiner() {
    use std::time::Duration as D;
    let ms = D::from_millis;
    // Aucun changement d'affichage, aucun échec : dessiner.
    assert_eq!(attente_avant_creation(None, None, None), None);
    // Un changement d'affichage tout juste arrivé : attendre la reprise entière.
    assert_eq!(attente_avant_creation(Some(D::ZERO), None, None), Some(REPRISE));
    // À mi-chemin : attendre le reste.
    assert_eq!(attente_avant_creation(Some(ms(400)), None, None), Some(ms(600)));
    // Reprise finie (pile, ou bien après) : dessiner.
    assert_eq!(attente_avant_creation(Some(REPRISE), None, None), None);
    assert_eq!(attente_avant_creation(Some(D::from_secs(30)), None, None), None);
    // Une création ratée attend son nouvel essai ; avec un changement d'affichage : le plus
    // long des deux.
    assert_eq!(attente_avant_creation(None, Some(ms(500)), None), Some(ms(500)));
    assert_eq!(
        attente_avant_creation(Some(ms(900)), Some(ms(500)), None),
        Some(ms(500))
    );
    assert_eq!(
        attente_avant_creation(Some(ms(100)), Some(ms(500)), None),
        Some(ms(900))
    );
    // Un reste nul n'est pas une attente.
    assert_eq!(attente_avant_creation(None, Some(D::ZERO), None), None);
    // Jamais plus d'une seconde d'attente pour un changement d'affichage.
    assert!(REPRISE <= D::from_secs(1));
}

/// La création après destruction : rien ne se crée tant qu'une fenêtre retirée existe,
/// ni pendant 100 ms après la dernière destruction ; une fenêtre qui ne disparaît pas
/// n'attend pas plus de 2 s ; une destruction constatée tard date de son retrait.
#[test]
fn creation_apres_destruction() {
    use std::time::{Duration as D, Instant};
    let t0 = Instant::now();
    let ms = D::from_millis;
    // La décision pure.
    assert_eq!(attente_apres_destruction(false, None), None);
    assert_eq!(attente_apres_destruction(true, None), Some(VERIFICATION));
    assert_eq!(attente_apres_destruction(false, Some(ms(30))), Some(ms(70)));
    assert_eq!(attente_apres_destruction(false, Some(APRES_DESTRUCTION)), None);
    assert_eq!(attente_apres_destruction(false, Some(D::from_secs(9))), None);

    // Rien de retiré : créer.
    let mut d = Destructions::default();
    assert_eq!(d.attente(t0, |_| true), None);

    // Deux fenêtres retirées, encore là : attendre la vérification suivante.
    d.retiree(10, t0);
    d.retiree(11, t0);
    assert_eq!(d.attente(t0 + ms(5), |_| true), Some(VERIFICATION));
    // L'une détruite, l'autre encore là : toujours attendre.
    assert_eq!(d.attente(t0 + ms(20), |h| h == 11), Some(VERIFICATION));
    // Les deux détruites à t0 + 30 ms : encore 100 ms de calme.
    assert_eq!(d.attente(t0 + ms(30), |_| false), Some(APRES_DESTRUCTION));
    assert_eq!(d.attente(t0 + ms(80), |_| false), Some(ms(50)));
    // Le calme passé : créer.
    assert_eq!(d.attente(t0 + ms(130), |_| false), None);

    // Une fenêtre qui ne disparaît jamais n'arrête pas la création pour toujours.
    let mut d = Destructions::default();
    d.retiree(20, t0);
    assert_eq!(d.attente(t0 + ms(500), |_| true), Some(VERIFICATION));
    assert_eq!(d.attente(t0 + DESTRUCTION_MAX + ms(1), |_| true), None);

    // Détruite il y a longtemps (personne ne regardait) : pas de calme de plus.
    let mut d = Destructions::default();
    d.retiree(30, t0);
    assert_eq!(d.attente(t0 + D::from_secs(1), |_| false), None);
}

/// Une création différée (une fenêtre vient d'être retirée) ne montre rien à egui :
/// `show_viewport_immediate` n'est pas appelée tant qu'on attend (sinon, hors du contexte de
/// winit, eframe rend la main sans dessiner et egui paniquait). La reprise vient d'un réveil
/// par message (`reveiller_dans`), pas d'une minuterie de repeinture ; puis la fenêtre est
/// créée et dessinée.
#[test]
fn creation_differee_jamais_montree_puis_reveillee() {
    use std::time::{Duration as D, Instant};
    let ctx = contexte();
    let f = AutresFenetres::default();
    f.ouvrir("d", Ouverture::new("D", [100.0, 100.0]), Vide);
    // Une fenêtre vient d'être retirée : 100 ms de calme avant toute création.
    verrou(&f.destructions).retiree(1, Instant::now());
    for _ in 0..3 {
        let (dessinees, _) = image(&ctx, &f);
        assert!(
            dessinees.is_empty(),
            "création différée : rien à egui, {dessinees:?}"
        );
        assert!(f.ouverte("d"));
    }
    let attente = f.prochaine_image().expect("la création attend");
    assert!(attente > D::ZERO && attente <= APRES_DESTRUCTION, "{attente:?}");
    // Le réveil programmé sonne (par `reveiller`, pas par une repeinture d'egui).
    let fin = Instant::now() + D::from_secs(2);
    while f.reveils.load(std::sync::atomic::Ordering::Acquire) == 0 && Instant::now() < fin {
        std::thread::sleep(D::from_millis(10));
    }
    assert!(
        f.reveils.load(std::sync::atomic::Ordering::Acquire) >= 1,
        "réveil programmé"
    );
    // Le calme passé : la fenêtre est créée et dessinée.
    std::thread::sleep(APRES_DESTRUCTION + D::from_millis(50));
    let (dessinees, _) = image(&ctx, &f);
    assert_eq!(dessinees, vec![id_de("d")]);
}

/// Un contenu sans rien dedans.
struct Vide;
impl Contenu for Vide {
    fn dessiner(&mut self, _: &mut egui::Ui, _: &mut Cadre<'_>) {}
}

/// Une panique du contenu lui-même (son dessin a commencé) n'est pas prise pour une
/// création ratée : elle suit son cours (la boucle la rattrape plus haut).
#[test]
fn panique_du_contenu_suit_son_cours() {
    struct Panique;
    impl Contenu for Panique {
        fn dessiner(&mut self, _: &mut egui::Ui, _: &mut Cadre<'_>) {
            panic!("dessin (essai)");
        }
    }
    let ctx = contexte();
    let f = AutresFenetres::default();
    f.ouvrir("q", Ouverture::new("Q", [200.0, 100.0]), Panique);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| image(&ctx, &f)));
    assert!(r.is_err());
}

/// Un contenu qui refuse de se fermer tant que son enregistrement échoue.
struct Refuse {
    trace: Trace,
    enregistre: Arc<std::sync::atomic::AtomicBool>,
}

impl Contenu for Refuse {
    fn dessiner(&mut self, ui: &mut egui::Ui, _: &mut Cadre<'_>) {
        ui.label("refuse");
    }

    fn peut_fermer(&mut self, imposee: bool) -> bool {
        let ok = self.enregistre.load(std::sync::atomic::Ordering::Relaxed);
        self.trace
            .0
            .lock()
            .unwrap()
            .push(format!("peut_fermer imposée {imposee} → {ok}"));
        ok
    }

    fn fermee(&mut self, _place: Option<Place>) {
        self.trace.0.lock().unwrap().push("fermée".into());
    }
}

/// La croix passe par le crochet : refusée, la fenêtre reste ouverte (et l'ordre
/// d'annuler la fermeture part) ; acceptée, elle se ferme, sans second appel. Fermée par
/// le logiciel, le crochet est appelé quand même (imposée), et son refus est ignoré.
#[test]
fn crochet_de_fermeture_avec_refus() {
    let ctx = contexte();
    let f = AutresFenetres::default();
    let trace = Trace::default();
    let enregistre = Arc::new(std::sync::atomic::AtomicBool::new(false));
    f.ouvrir(
        "p",
        Ouverture::new("P", [200.0, 100.0]),
        Refuse {
            trace: trace.clone(),
            enregistre: enregistre.clone(),
        },
    );
    let _ = image(&ctx, &f);
    let _ = image(&ctx, &f);
    CROIX.with(|c| c.borrow_mut().push(id_de("p")));
    let _ = image(&ctx, &f);
    assert!(f.ouverte("p"), "refusée : reste ouverte");
    assert!(
        crate::bords::fenetres::faux_moteur::ORDRES.with(|o| o
            .borrow()
            .iter()
            .any(|(i, c)| *i == id_de("p") && matches!(c, ViewportCommand::CancelClose))),
        "l'ordre d'annuler la fermeture part"
    );
    enregistre.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = image(&ctx, &f);
    assert!(f.cles().is_empty(), "acceptée : fermée");
    CROIX.with(|c| c.borrow_mut().clear());
    assert_eq!(
        trace.0.lock().unwrap().clone(),
        vec![
            "peut_fermer imposée false → false",
            "peut_fermer imposée false → true",
            "fermée"
        ]
    );
    // Imposée : le contenu est consulté, sa réponse ne retient rien.
    enregistre.store(false, std::sync::atomic::Ordering::Relaxed);
    let trace = Trace::default();
    f.ouvrir(
        "q",
        Ouverture::new("Q", [200.0, 100.0]),
        Refuse {
            trace: trace.clone(),
            enregistre,
        },
    );
    let _ = image(&ctx, &f);
    f.tout_fermer();
    let _ = image(&ctx, &f);
    assert!(f.cles().is_empty());
    assert_eq!(
        trace.0.lock().unwrap().clone(),
        vec!["peut_fermer imposée true → false", "fermée"]
    );
}

/// Un contenu pas prêt garde sa fenêtre cachée, même après ses deux images ; elle se montre
/// à l'image qui suit le moment où il l'est.
#[test]
fn un_contenu_pas_pret_garde_sa_fenetre_cachee() {
    struct Attend(Arc<std::sync::atomic::AtomicBool>, Trace);
    impl Contenu for Attend {
        fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
            ui.label("attend");
            self.1
                .0
                .lock()
                .unwrap()
                .push(format!("image {} montrée {}", cadre.image, cadre.montree));
        }
        fn prete(&self) -> bool {
            self.0.load(std::sync::atomic::Ordering::Acquire)
        }
    }
    let ctx = contexte();
    let f = AutresFenetres::default();
    f.lier(Some(ctx.clone()));
    let (pret, trace) = (
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
        Trace::default(),
    );
    f.ouvrir(
        "carte",
        Ouverture::new("Carte", [300.0, 200.0]).sans_focus(),
        Attend(pret.clone(), trace.clone()),
    );
    for _ in 0..4 {
        let _ = image(&ctx, &f);
    }
    assert!(f.a_faire(), "pas prête : encore cachée");
    pret.store(true, std::sync::atomic::Ordering::Release);
    let _ = image(&ctx, &f);
    assert!(!f.a_faire(), "prête : montrée");
    let t = trace.0.lock().unwrap().clone();
    assert_eq!(t[3], "image 4 montrée false", "{t:?}");
    assert_eq!(t[4], "image 5 montrée true", "{t:?}");
}

/// Une deuxième fenêtre s'ouvre puis se ferme dans la même boucle, sans en créer une
/// autre ; elle est dessinée cachée deux fois avant d'être montrée.
#[test]
fn deuxieme_fenetre_ouverte_puis_fermee_dans_la_meme_boucle() {
    let ctx = contexte();
    let f = AutresFenetres::default();
    f.lier(Some(ctx.clone()));
    let trace = Trace::default();
    let id = id_de("lecture");
    assert!(f.ouvrir(
        "lecture",
        Ouverture::new("Lecture", [300.0, 200.0]),
        Essai {
            nom: "lecture",
            trace: trace.clone(),
            fermer_a: None,
        },
    ));
    assert!(
        f.a_faire(),
        "à ouvrir : une image même la fenêtre principale cachée"
    );
    let (d, g) = image(&ctx, &f);
    assert_eq!(d, vec![id]);
    assert_eq!(g, vec![id]);
    let _ = image(&ctx, &f);
    assert!(!f.a_faire(), "montrée : plus rien à faire");
    assert!(f.ouverte("lecture"));
    // Une deuxième ouverture de la même clé ne la recrée pas : elle passe devant.
    assert!(!f.ouvrir(
        "lecture",
        Ouverture::new("Lecture", [300.0, 200.0]),
        Essai {
            nom: "autre",
            trace: trace.clone(),
            fermer_a: None,
        },
    ));
    f.fermer("lecture");
    assert!(!f.ouverte("lecture"));
    assert!(f.a_faire());
    let (d, g) = image(&ctx, &f);
    assert!(d.is_empty(), "{d:?}");
    assert!(g.is_empty(), "egui détruit la fenêtre : {g:?}");
    assert!(f.cles().is_empty());
    let t = trace.0.lock().unwrap().clone();
    assert_eq!(
        t,
        vec![
            "lecture image 1 montrée false",
            "lecture image 2 montrée true",
            "lecture fermée",
        ]
    );
}

/// Plusieurs fenêtres à la fois ; la croix ou le contenu en ferment une, les autres
/// restent.
#[test]
fn plusieurs_fenetres_et_croix() {
    let ctx = contexte();
    let f = AutresFenetres::default();
    let trace = Trace::default();
    for (cle, fermer_a) in [("a", None), ("b", Some(3)), ("c", None)] {
        f.ouvrir(
            cle,
            Ouverture::new(cle, [200.0, 100.0]),
            Essai {
                nom: cle,
                trace: trace.clone(),
                fermer_a,
            },
        );
    }
    let (d, _) = image(&ctx, &f);
    assert_eq!(d.len(), 3);
    let _ = image(&ctx, &f);
    CROIX.with(|c| c.borrow_mut().push(id_de("a")));
    let _ = image(&ctx, &f);
    CROIX.with(|c| c.borrow_mut().clear());
    let (d, g) = image(&ctx, &f);
    assert_eq!(f.cles(), vec!["c".to_string()]);
    assert_eq!(d, vec![id_de("c")]);
    assert_eq!(g, vec![id_de("c")]);
    let t = trace.0.lock().unwrap().clone();
    assert!(t.contains(&"a fermée".to_string()), "{t:?}");
    assert!(t.contains(&"b fermée".to_string()), "{t:?}");
    f.tout_fermer();
    let (_, g) = image(&ctx, &f);
    assert!(g.is_empty());
}

/// Un contenu peut ouvrir une autre fenêtre pendant son dessin (aucun verrou tenu).
#[test]
fn un_contenu_ouvre_une_autre_fenetre() {
    struct Ouvre(AutresFenetres);
    impl Contenu for Ouvre {
        fn dessiner(&mut self, _: &mut egui::Ui, _: &mut Cadre<'_>) {
            self.0.ouvrir(
                "fille",
                Ouverture::new("Fille", [100.0, 100.0]),
                Essai {
                    nom: "fille",
                    trace: Trace::default(),
                    fermer_a: None,
                },
            );
        }
    }
    let ctx = contexte();
    let f = AutresFenetres::default();
    f.ouvrir("mere", Ouverture::new("Mère", [100.0, 100.0]), Ouvre(f.clone()));
    let _ = image(&ctx, &f);
    let (d, _) = image(&ctx, &f);
    assert_eq!(d.len(), 2);
}
