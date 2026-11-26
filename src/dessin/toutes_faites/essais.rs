//! Ce que partagent les essais des fenêtres toutes faites : une image sans fenêtre, un clic,
//! une fenêtre simple inventée.

// Dans un essai, un échec doit arrêter l'essai : `unwrap` et `expect` y sont voulus.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use egui::{Ui, vec2};

use super::{Bouton, Champ, Choix, Simple, Ton};
use crate::theme::Palette;

pub(crate) fn palette() -> Palette {
    Palette::de(&crate::theme::ChoixWindows::default())
}

/// Une image d'un dessin, sans fenêtre ; `evts` : ce que l'utilisateur fait.
pub(crate) fn une_image(ctx: &egui::Context, evts: Vec<egui::Event>, f: impl FnMut(&mut Ui)) {
    let entree = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(600.0, 900.0))),
        events: evts,
        // Le temps avance : les défilements animés vont au bout.
        time: Some(ctx.input(|i| i.time) + 0.1),
        ..Default::default()
    };
    let mut f = f;
    let mut sortie = ctx.run_ui(entree, |ui| f(ui));
    sortie.textures_delta.clear();
}

pub(crate) fn cliquer(ctx: &egui::Context, pos: egui::Pos2, f: &mut impl FnMut(&mut Ui)) {
    une_image(ctx, vec![egui::Event::PointerMoved(pos)], &mut *f);
    let bouton = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Default::default(),
    };
    une_image(ctx, vec![bouton(true)], &mut *f);
    une_image(ctx, vec![bouton(false)], &mut *f);
}

/// Entrée, appuyée.
pub(crate) fn entree() -> egui::Event {
    egui::Event::Key {
        key: egui::Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Default::default(),
    }
}

/// Une fenêtre simple inventée : titre, état, barre, champ, choix, deux boutons.
pub(crate) fn simple() -> Simple {
    Simple {
        titre: "Garder comme action".into(),
        statut: "Prêt.".into(),
        ton: Ton::Bon,
        barre: Some(0.5),
        champ: Some(Champ {
            cle: "nom".into(),
            libelle: "Nom".into(),
            valeur: "Lecture du matin".into(),
        }),
        choix: Some(Choix {
            cle: "ou".into(),
            libelle: String::new(),
            valeur: "b".into(),
            options: vec![("a".into(), "Ici".into()), ("b".into(), "Là".into())],
        }),
        boutons: vec![
            Bouton::new("effacer", "Effacer")
                .danger()
                .confirmer("Effacer pour de bon ?"),
            Bouton::new("garder", "Garder").accent(),
        ],
        ..Default::default()
    }
}
