//! Une vraie fenêtre (eframe, OpenGL) perdue par une panique peut être recréée sur le même
//! fil : la boucle de winit, gardée par eframe pour le fil, resservira. C'est ce que fait la
//! boucle des fenêtres quand la fenêtre du volet est perdue (`fenetres::boucle_des_fenetres`).
//!
//! Les fenêtres ne sont jamais montrées : voilées (`DWMWA_CLOAK`) avant leur première image,
//! hors de l'écran, sans bouton dans la barre des tâches. Un seul essai dans ce fichier : winit
//! n'accepte qu'une boucle de fenêtres par programme.

#![cfg(windows)]
// Dans un essai, un échec doit arrêter l'essai : `expect` et `panic!` y sont voulus.
#![allow(clippy::expect_used, clippy::panic)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use eframe::egui;
use interface_commune::{systeme, volet};

struct Essai {
    paniquer: bool,
    images: u32,
}

impl eframe::App for Essai {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Voilée avant sa première image : eframe la montre d'office après celle-ci.
        if let Some(h) = systeme::hwnd_de(frame) {
            systeme::voiler(h, true);
        }
        self.images += 1;
        if self.paniquer {
            panic!("contexte OpenGL perdu (essai)");
        }
        if self.images >= 2 {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        } else {
            ui.ctx().request_repaint();
        }
    }
}

fn fenetre(paniquer: bool) -> Result<u32, String> {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Essai de recréation")
        .with_position([-32_000.0, -32_000.0])
        .with_inner_size([40.0, 40.0])
        .with_decorations(false)
        .with_taskbar(false)
        .with_visible(false);
    let images = std::rc::Rc::new(std::cell::Cell::new(0));
    let i2 = images.clone();
    struct Compte(Essai, std::rc::Rc<std::cell::Cell<u32>>);
    impl eframe::App for Compte {
        fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
            self.0.ui(ui, frame);
            self.1.set(self.0.images);
        }
    }
    eframe::run_native(
        "Essai de recréation",
        volet::options(viewport),
        Box::new(move |_| Ok(Box::new(Compte(Essai { paniquer, images: 0 }, i2)))),
    )
    .map_err(|e| e.to_string())?;
    Ok(images.get())
}

#[test]
fn fenetre_perdue_par_une_panique_puis_recreee_sur_le_meme_fil() {
    let perdue = catch_unwind(AssertUnwindSafe(|| fenetre(true)));
    assert!(perdue.is_err(), "la panique remonte de la boucle de winit");
    for _ in 0..2 {
        let images = fenetre(false).expect("fenêtre recréée après la panique");
        assert!(images >= 2, "{images} image(s)");
    }
}
