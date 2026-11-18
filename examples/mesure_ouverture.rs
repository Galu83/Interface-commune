//! Mesure l'ouverture du volet et du petit menu, image par image, avec la vraie fenêtre
//! (egui, OpenGL), mais voilée : rien ne s'affiche à l'écran et le clavier reste à
//! l'application en cours. Un logiciel inventé ; aucune icône près de l'horloge.
//!
//! `cargo run --release --example mesure_ouverture` (Windows). Pour chaque ouverture :
//! le délai du clic à la première image, le nombre d'images de mise en place, le temps
//! jusqu'à la première image visible (opacité > 0), jusqu'à « lisible » (opacité ≥ 80 %),
//! jusqu'à la fin de l'animation, et le plus long écart entre deux images.

#[cfg(windows)]
mod mesure {
    use std::sync::atomic::Ordering;
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::{Duration, Instant};

    use interface_commune::fenetres::{Demande, Liens, canal};
    use interface_commune::theme::Surface;
    use interface_commune::volet::{ImageVue, observer, veiller};
    use interface_commune::{
        Communs, Etat, GenreEtat, Identite, Logiciel, Reglages, Valeur, polices, systeme,
    };

    fn icone(t: u32, _: bool) -> Vec<u8> {
        vec![255; (t * t * 4) as usize]
    }

    struct Essai {
        identite: Identite,
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
                titre: "Bureau d'essai".into(),
                ligne: "Prêt".into(),
                detail: None,
            }
        }
        fn quitter(&self) {}
        fn noter(&self, ligne: String) {
            println!("  journal : {ligne}");
        }
    }

    /// Le résumé d'une ouverture.
    fn resumer(surface: Surface, images: &[ImageVue]) {
        let ms = |d: Duration| d.as_secs_f64() * 1000.0;
        let Some(premiere) = images.first() else {
            println!("{surface:?} : aucune image");
            return;
        };
        let mise_en_place = images.iter().filter(|i| i.mise_en_place).count();
        let montree = images.iter().find(|i| !i.mise_en_place);
        let visible = images.iter().find(|i| !i.mise_en_place && i.opacite > 0);
        let lisible = images.iter().find(|i| !i.mise_en_place && i.opacite >= 204);
        let finie = images.iter().find(|i| i.finie);
        let ecarts: Vec<f64> = images
            .windows(2)
            .map(|w| ms(w[1].depuis_clic.saturating_sub(w[0].depuis_clic)))
            .collect();
        let pire = ecarts.iter().copied().fold(0.0, f64::max);
        let animees = images.iter().filter(|i| !i.mise_en_place).count();
        let f =
            |i: Option<&ImageVue>| i.map_or("—".to_string(), |i| format!("{:.0} ms", ms(i.depuis_clic)));
        println!(
            "{surface:?} : 1re image {:.0} ms ; {mise_en_place} image(s) de mise en place ; voile levé à {} (opacité {}) ; 1re image visible {} ; lisible {} ; animation finie {} ({animees} images, pire écart {pire:.0} ms)",
            ms(premiere.depuis_clic),
            f(montree),
            montree.map_or(0, |i| i.opacite),
            f(visible),
            f(lisible),
            f(finie),
        );
    }

    pub fn lancer() {
        let images: Arc<Mutex<Vec<ImageVue>>> = Arc::default();
        let i2 = images.clone();
        observer(Some(Box::new(move |i| {
            if let Ok(mut v) = i2.lock() {
                v.push(i);
            }
        })));
        let liens = Arc::new(Liens::default());
        let (demandes, reception) = canal();
        let logiciel: Arc<dyn Logiciel> = Arc::new(Essai {
            identite: Identite {
                nom: "Essai".into(),
                version: "0.0.0".into(),
                licence: "MIT".into(),
                description: "Mesure de l'ouverture.".into(),
                icone,
                logo: None,
            },
        });
        let polices = match polices::polices() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Polices : {e}");
                return;
            }
        };
        let (fini, fin) = mpsc::channel();
        let (l2, li2) = (logiciel.clone(), liens.clone());
        let fil = std::thread::spawn(move || {
            let r = veiller(l2, li2, polices, &reception, None);
            let _ = fini.send(());
            r.map(|_| ()).map_err(|e| e.0)
        });
        let debut = Instant::now();
        while liens.surface_prete.load(Ordering::Acquire) == 0 {
            if debut.elapsed() > Duration::from_secs(10) || fin.try_recv().is_ok() {
                eprintln!("La fenêtre n'a pas été préparée.");
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        std::thread::sleep(Duration::from_millis(300));
        // Une icône inventée, en bas à droite de l'écran principal (barre des tâches en bas).
        let Some((e, _)) = systeme::ecran_autour(0, 0) else {
            eprintln!("Aucun écran.");
            return;
        };
        let icone = [e[2] - 80, e[3] - 40, e[2] - 56, e[3] - 16];
        let ancre = [e[2] - 68, e[3] - 28];
        let ordre = [
            Surface::Volet,
            Surface::Volet,
            Surface::Menu,
            Surface::Menu,
            Surface::Volet,
            Surface::Menu,
            Surface::Volet,
            Surface::Volet,
        ];
        for s in ordre {
            if let Ok(mut v) = images.lock() {
                v.clear();
            }
            let clic = Instant::now();
            let d = match s {
                Surface::Volet => Demande::Volet { icone, clic },
                Surface::Menu => Demande::Menu { icone, ancre, clic },
            };
            if demandes.send(d).is_err() {
                break;
            }
            // Comme l'icône : la fenêtre prête est montrée tout de suite, encore invisible.
            systeme::reveiller(liens.surface_prete.load(Ordering::Acquire));
            std::thread::sleep(Duration::from_millis(700));
            liens.fermer.store(true, Ordering::Release);
            liens.redessiner();
            std::thread::sleep(Duration::from_millis(400));
            let v = images.lock().map(|v| v.clone()).unwrap_or_default();
            resumer(s, &v);
        }
        let _ = demandes.send(Demande::Quitter);
        let _ = fin.recv_timeout(Duration::from_secs(5));
        observer(None);
        drop(fil);
    }
}

fn main() {
    #[cfg(windows)]
    mesure::lancer();
}
