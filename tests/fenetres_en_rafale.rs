//! Des fenêtres ouvertes puis fermées tour après tour, sans pause entre deux tours : une
//! fenêtre créée quelques millisecondes après la destruction des précédentes ne doit jamais
//! bloquer la boucle des fenêtres (un blocage sans fin dans `show_viewport_immediate`, tous les
//! fils à 0 % de processeur, a été vu avec un processeur très chargé).
//!
//! Les fenêtres ne sont jamais vues : la fenêtre principale est voilée et hors de l'écran, les
//! autres sont posées hors de l'écran par leur contenu, sans focus ni bouton dans la barre des
//! tâches. Un seul essai dans ce fichier : winit n'accepte qu'une boucle de fenêtres par
//! programme.
//!
//! Réglages (variables d'environnement, pour chercher le défaut sous charge ; la charge reste
//! sous le plafond de 75 % du processeur) :
//! - `ESSAI_RAFALE_TOURS` : le nombre de tours (6 par défaut) ;
//! - `ESSAI_RAFALE_CHARGE` : le nombre de fils qui saturent chacun un cœur (0 par défaut ;
//!   borné aux trois quarts des cœurs, moins un).

#![cfg(windows)]
// Dans un essai, un échec doit arrêter l'essai : `expect` et `panic!` y sont voulus.
#![allow(clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use eframe::egui;
use interface_commune::autres_fenetres::{AutresFenetres, Cadre, Contenu, Ouverture};
use interface_commune::theme::ChoixWindows;
use interface_commune::{Palette, systeme, volet};

/// Le nombre de fenêtres ouvertes à chaque tour.
const PAR_TOUR: usize = 3;

/// Un contenu vide, posé hors de l'écran dès sa première image.
struct Vide {
    images: Arc<AtomicU32>,
}

impl Contenu for Vide {
    fn dessiner(&mut self, _ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
        self.images.fetch_add(1, Ordering::AcqRel);
        if let Some(h) = cadre.hwnd {
            systeme::poser(h, [-32_000, -32_000, 60, 40]);
        }
    }
}

#[derive(PartialEq, Eq)]
enum Phase {
    /// Les fenêtres du tour sont ouvertes ; on attend qu'elles soient dessinées.
    Ouvertes(Instant),
    /// Tous les tours sont finis : la fenêtre principale se ferme.
    Fin,
}

struct Rafale {
    autres: AutresFenetres,
    palette: Palette,
    tour: u32,
    tours: u32,
    phase: Phase,
    images: Arc<AtomicU32>,
    /// Le nombre de tours finis (lu par la surveillance du test).
    avance: Arc<AtomicU32>,
}

impl Rafale {
    /// Les noms des fenêtres du tour : deux séries qui alternent, pour qu'un tour s'ouvre
    /// pendant que les fenêtres du tour précédent (d'autres noms) se ferment.
    fn cle(&self, tour: u32, i: usize) -> String {
        format!("rafale-{}-{i}", tour % 2)
    }

    fn ouvrir_le_tour(&mut self) {
        self.images.store(0, Ordering::Release);
        for i in 0..PAR_TOUR {
            self.autres.ouvrir(
                &self.cle(self.tour, i),
                Ouverture::new(&format!("Rafale {} {i}", self.tour % 2), [60.0, 40.0])
                    .sans_focus()
                    .pose_par_le_contenu(),
                Vide {
                    images: self.images.clone(),
                },
            );
        }
        self.phase = Phase::Ouvertes(Instant::now());
    }
}

impl eframe::App for Rafale {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Voilée avant sa première image : eframe la montre d'office après celle-ci.
        if let Some(h) = systeme::hwnd_de(frame) {
            systeme::voiler(h, true);
        }
        self.autres.montrer(&ctx, &self.palette);
        match self.phase {
            Phase::Ouvertes(depuis) => {
                // Chaque fenêtre a été dessinée au moins quatre fois (cachée, puis montrée).
                let dessinees = self.images.load(Ordering::Acquire) >= 4 * PAR_TOUR as u32;
                if dessinees || depuis.elapsed() > Duration::from_secs(20) {
                    // Les fenêtres du tour se ferment, et celles du suivant s'ouvrent aussitôt,
                    // dans la même image : aucune pause.
                    for i in 0..PAR_TOUR {
                        self.autres.fermer(&self.cle(self.tour, i));
                    }
                    self.tour += 1;
                    self.avance.store(self.tour, Ordering::Release);
                    if self.tour >= self.tours {
                        self.phase = Phase::Fin;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                    self.ouvrir_le_tour();
                }
            }
            Phase::Fin => return,
        }
        ctx.request_repaint();
    }
}

/// Des fils qui saturent chacun un cœur, tant que `fin` n'est pas levé.
fn charger(fils: usize, fin: &Arc<AtomicBool>) -> Vec<std::thread::JoinHandle<()>> {
    (0..fils)
        .map(|_| {
            let fin = fin.clone();
            std::thread::spawn(move || {
                let mut x = 1u64;
                while !fin.load(Ordering::Relaxed) {
                    for _ in 0..10_000 {
                        x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    }
                    std::hint::black_box(x);
                }
            })
        })
        .collect()
}

fn reglage(nom: &str, defaut: u32) -> u32 {
    std::env::var(nom)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaut)
}

#[test]
fn fenetres_ouvertes_et_fermees_sans_pause_ne_bloquent_jamais() {
    let tours = reglage("ESSAI_RAFALE_TOURS", 6);
    let coeurs = std::thread::available_parallelism().map_or(1, usize::from);
    // La charge ne dépasse jamais les trois quarts des cœurs, moins un (le plafond de 75 %).
    let plafond = (coeurs * 3 / 4).saturating_sub(1);
    let charge = (reglage("ESSAI_RAFALE_CHARGE", 0) as usize).min(plafond);
    let fin = Arc::new(AtomicBool::new(false));
    let fils = charger(charge, &fin);

    // La surveillance du test : sans avancée pendant 90 s, la boucle est bloquée pour de bon.
    let avance = Arc::new(AtomicU32::new(0));
    let termine = Arc::new(AtomicBool::new(false));
    let garde = {
        let (avance, termine) = (avance.clone(), termine.clone());
        std::thread::spawn(move || {
            let (mut dernier, mut depuis) = (0, Instant::now());
            while !termine.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_millis(200));
                let a = avance.load(Ordering::Acquire);
                if a != dernier {
                    (dernier, depuis) = (a, Instant::now());
                } else if depuis.elapsed() > Duration::from_secs(90) {
                    eprintln!("BLOQUÉ : aucun tour fini depuis 90 s (tour {a}).");
                    std::process::exit(3);
                }
            }
        })
    };

    // Le journal de la bibliothèque : aucune création de fenêtre ne doit rater.
    let journal = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let j2 = journal.clone();
    interface_commune::sonde::brancher_le_journal(move |l| {
        if let Ok(mut j) = j2.lock() {
            j.push(l);
        }
    });

    let viewport = egui::ViewportBuilder::default()
        .with_title("Essai en rafale")
        .with_position([-32_000.0, -32_000.0])
        .with_inner_size([40.0, 40.0])
        .with_decorations(false)
        .with_taskbar(false)
        .with_visible(false);
    let a2 = avance.clone();
    let resultat = eframe::run_native(
        "Essai en rafale",
        volet::options(viewport),
        Box::new(move |cc| {
            let autres = AutresFenetres::default();
            autres.lier(Some(cc.egui_ctx.clone()));
            let mut app = Rafale {
                autres,
                palette: Palette::de(&ChoixWindows::default()),
                tour: 0,
                tours,
                phase: Phase::Fin,
                images: Arc::new(AtomicU32::new(0)),
                avance: a2,
            };
            app.ouvrir_le_tour();
            Ok(Box::new(app))
        }),
    );

    termine.store(true, Ordering::Release);
    fin.store(true, Ordering::Release);
    for f in fils {
        let _ = f.join();
    }
    let _ = garde.join();
    resultat.expect("la boucle des fenêtres");
    assert_eq!(avance.load(Ordering::Acquire), tours, "tous les tours finis");
    let lignes = journal.lock().expect("journal").clone();
    assert!(
        lignes.iter().all(|l| !l.contains("pas pu la créer")),
        "une fenêtre n'a pas pu être créée : {lignes:#?}"
    );
}
