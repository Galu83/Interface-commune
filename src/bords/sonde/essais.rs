//! Les essais du bord de la sonde : les vrais fils, le journal branché (C-110), la vraie boucle
//! de surveillance avec des délais courts (200 ms, jamais 15 s), le chien des créations, les
//! paniques. La décision elle-même est essayée dans le cœur, à fausse horloge.
#![allow(clippy::unwrap_used)]

use std::time::Duration;

use super::chien::regler_le_seuil_de_creation;
use super::*;

/// Les essais qui touchent à l'état global (journal, fils inscrits, délais) se suivent, avec
/// des délais courts : la surveillance réelle n'est lancée qu'une fois pour le programme.
static GLOBAL: Mutex<()> = Mutex::new(());

fn courts() {
    regler(Delais {
        pas: Duration::from_millis(50),
        blocage: Duration::from_millis(200),
    });
}

/// Un journal qui garde ses lignes, branché.
fn journal() -> Arc<Mutex<Vec<String>>> {
    let lignes: Arc<Mutex<Vec<String>>> = Arc::default();
    let l2 = lignes.clone();
    brancher_le_journal(move |l| l2.lock().unwrap().push(l));
    lignes
}

fn debrancher() {
    *SORTIE.write().unwrap() = None;
}

fn tour(s: &mut Surveillance, t: Instant, fils: &[Arc<Fil>]) -> Vec<String> {
    let mut v = Vec::new();
    s.tour(t, fils, &mut |l| v.push(l));
    v
}

/// Le relevé des vrais fils arrive au cœur : compteur, étape, boucle imbriquée. L'heure est
/// donnée à la main (`Instant` + écart) : aucune attente.
#[test]
fn la_surveillance_lit_les_vrais_fils() {
    let t0 = Instant::now();
    let sec = Duration::from_secs;
    let f = Fil::nouveau("icône", None);
    f.battre();
    f.poser("Shell_NotifyIcon");
    let fils = [f.clone()];
    let mut s = Surveillance::new(sec(15));
    assert!(tour(&mut s, t0, &fils).is_empty());
    let avant = f.entrer("TrackPopupMenu", t0 + sec(2));
    assert_eq!(avant, None);
    assert_eq!(
        tour(&mut s, t0 + sec(20), &fils),
        [
            "Le fil « icône » ne répond plus depuis 20 s : dans « TrackPopupMenu » depuis 18 s, \
             dernière étape : Shell_NotifyIcon."
        ]
    );
    f.sortir(avant);
    assert_eq!(f.etat().imbriquee, None);
    f.battre();
    assert_eq!(
        tour(&mut s, t0 + sec(25), &fils),
        ["Le fil « icône » repart après 25 s de blocage."]
    );
}

/// Deux vrais fils : deux clés distinctes (leurs adresses), un seul est dit bloqué.
#[test]
fn deux_vrais_fils_un_seul_bloque() {
    let t0 = Instant::now();
    let a = Fil::nouveau("icône", None);
    let b = Fil::nouveau("fenêtres", None);
    let fils = [a.clone(), b.clone()];
    let mut s = Surveillance::new(Duration::from_secs(15));
    let mut lignes = Vec::new();
    for i in 0..5u64 {
        a.battre();
        lignes.extend(tour(&mut s, t0 + Duration::from_secs(5 * i), &fils));
    }
    assert_eq!(
        lignes,
        ["Le fil « fenêtres » ne répond plus depuis 15 s : dernière étape : aucune."]
    );
}

/// C-110 : le journal branché reçoit les lignes de la sonde ; rebranché, seul le dernier les
/// reçoit ; débranché, les lignes sont perdues sans rien casser.
#[test]
fn le_journal_branche_recoit_les_lignes() {
    let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
    let premier = journal();
    clic("essai");
    let second = journal();
    noter("une ligne d'essai".into());
    debrancher();
    noter("perdue".into());
    let p = premier.lock().unwrap().clone();
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(
        p[0].starts_with("Icône : clic essai reçu à ") && p[0].ends_with(" UTC."),
        "{p:?}"
    );
    assert_eq!(*second.lock().unwrap(), ["une ligne d'essai"]);
}

/// La garde d'une boucle imbriquée écrit ses deux lignes et rend l'état ; un fil qui sort de
/// la surveillance ne laisse rien derrière lui.
#[test]
fn la_garde_ecrit_debut_et_fin_et_rend_l_etat() {
    let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
    courts();
    let lignes = journal();
    let inscription = inscrire("essai", None);
    etape("avant le menu");
    assert_eq!(etape_du_fil(), Some("avant le menu"));
    {
        let _g = imbriquee("menu d'essai");
        assert_eq!(
            inscription.fil.etat().imbriquee.map(|(n, _)| n),
            Some("menu d'essai")
        );
    }
    assert_eq!(inscription.fil.etat().imbriquee, None);
    assert_eq!(inscription.fil.etat().etape, "avant le menu");
    battement();
    assert_eq!(inscription.fil.battements.load(Ordering::Acquire), 1);
    let v = lignes.lock().unwrap().clone();
    assert!(
        v.iter()
            .any(|l| l == "Boucle imbriquée « menu d'essai » : début."),
        "{v:?}"
    );
    assert!(
        v.iter()
            .any(|l| l.starts_with("Boucle imbriquée « menu d'essai » : fin après ") && l.ends_with(" ms.")),
        "{v:?}"
    );
    let fil = inscription.fil.clone();
    drop(inscription);
    assert!(!FILS.lock().unwrap().iter().any(|f| Arc::ptr_eq(f, &fil)));
    assert_eq!(etape_du_fil(), None, "le fil n'est plus surveillé");
    debrancher();
}

/// Un réveil qui reste pris pour toujours (un verrou jamais rendu) n'empêche pas la
/// surveillance de dire le blocage d'un autre fil, et ne la fait jamais attendre.
#[test]
fn un_reveil_bloque_ne_bloque_pas_la_surveillance() {
    let t0 = Instant::now();
    let verrou: Arc<Mutex<()>> = Arc::default();
    let garde = verrou.lock().unwrap();
    let v2 = verrou.clone();
    let a = Fil::nouveau("fenêtres", Some(Box::new(move || drop(v2.lock()))));
    let b = Fil::nouveau("icône", None);
    b.battre();
    b.poser("attente d'un message");
    let fils = [a.clone(), b.clone()];
    let mut s = Surveillance::new(Duration::from_secs(15));
    let mut lignes = Vec::new();
    let debut = Instant::now();
    for i in 0..5u64 {
        a.battre();
        s.tour_et_reveils(t0 + Duration::from_secs(5 * i), &fils, &mut |l| lignes.push(l));
    }
    assert!(
        debut.elapsed() < Duration::from_secs(5),
        "la surveillance a attendu"
    );
    assert_eq!(lignes.len(), 1, "{lignes:?}");
    assert!(lignes[0].contains("« icône »"), "{lignes:?}");
    drop(garde);
}

/// Attend (au plus 5 s, par pas de 20 ms) qu'une ligne du journal vérifie `f`.
fn attendre(lignes: &Mutex<Vec<String>>, f: &dyn Fn(&str) -> bool) -> bool {
    let fin = Instant::now() + Duration::from_secs(5);
    while Instant::now() < fin {
        if lignes.lock().unwrap().iter().any(|l| f(l)) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

/// La vraie boucle de surveillance, avec des délais courts : un fil inscrit qui ne bat plus
/// est dit une fois, puis dit reparti. (Seul essai qui lance la boucle réelle.)
#[test]
fn la_boucle_reelle_dit_le_blocage_et_la_reprise() {
    let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
    courts();
    let lignes = journal();
    let (tx, rx) = std::sync::mpsc::channel::<()>();
    let fil = std::thread::spawn(move || {
        let _i = inscrire("fil d'essai", None);
        battement();
        etape("étape d'essai");
        // Bloqué : plus de battement, jusqu'au signal.
        let _ = rx.recv();
        // Reparti : il bat de nouveau, jusqu'à ce que l'essai ferme le canal.
        while !matches!(rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Disconnected)) {
            battement();
            std::thread::sleep(Duration::from_millis(20));
        }
    });
    assert!(attendre(&lignes, &|l| l
        .contains("« fil d'essai » ne répond plus")
        && l.contains("étape d'essai")));
    tx.send(()).unwrap();
    assert!(attendre(&lignes, &|l| l.contains("« fil d'essai » repart")));
    drop(tx);
    fil.join().unwrap();
    let v = lignes.lock().unwrap().clone();
    let n = v
        .iter()
        .filter(|l| l.contains("« fil d'essai » ne répond plus"))
        .count();
    assert_eq!(n, 1, "{v:?}");
    debrancher();
}

/// C-108, le vrai fil : une création qui ne finit pas est écrite au journal, puis sa fin aussi.
#[test]
fn le_chien_ecrit_la_creation_qui_dure() {
    let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
    let lignes = journal();
    regler_le_seuil_de_creation(Duration::from_millis(100));
    let c = creation("Fenêtre d'essai du chien");
    let dure = |l: &str| l.starts_with("Fenêtre « Fenêtre d'essai du chien » : sa création dure depuis");
    assert!(attendre(&lignes, &dure), "{:?}", lignes.lock().unwrap());
    drop(c);
    assert!(
        attendre(&lignes, &|l| l.contains("s'est enfin terminée")),
        "{:?}",
        lignes.lock().unwrap()
    );
    regler_le_seuil_de_creation(SEUIL_DE_CREATION);
    debrancher();
}

/// C-109 : une panique attrapée ; son lieu est gardé pour ce fil, court (sans le chemin du
/// PC), et lu une seule fois.
#[test]
fn lieu_de_la_panique_garde_pour_le_fil() {
    retenir_les_paniques();
    let r = std::thread::spawn(|| {
        let _ = std::panic::catch_unwind(|| panic!("essai"));
        (lieu_de_la_derniere_panique(), lieu_de_la_derniere_panique())
    })
    .join()
    .unwrap();
    assert!(
        r.0.as_deref().is_some_and(|l| l.starts_with("sonde/essais.rs:")),
        "{r:?}"
    );
    assert_eq!(r.1, None);
}
