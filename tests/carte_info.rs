//! La carte d'information reste dans la largeur donnée (mesurée sans fenêtre). Défaut
//! corrigé : le bloc de contrôle vide ajoutait un espacement de 8 pt après le texte, et la
//! carte débordait d'autant.
// Dans un essai, un échec doit arrêter l'essai : `expect` y sont voulus.
#![allow(clippy::expect_used)]

use interface_commune::capture::{self, Options};
use interface_commune::dessin;

const EXPLICATIONS: [&str; 3] = [
    "",
    "Une explication courte.",
    "Une explication nettement plus longue qui passe à la ligne sur la carte d'information.",
];

/// (largeur donnée, largeur de la carte, hauteur de la carte).
fn mesure(largeur: f32, explication: &str, info: bool) -> (f32, f32, f32) {
    let mut m = (0.0, 0.0, 0.0);
    capture::dessin(egui::vec2(largeur, 300.0), Options::default(), |ui, p| {
        let dispo = ui.available_width();
        let r = ui
            .scope(|ui| {
                if info {
                    dessin::carte_info(ui, p, 'i', "Titre", explication);
                } else {
                    dessin::carte(ui, p, 'i', "Titre", explication, |_| {});
                }
            })
            .response
            .rect;
        m = (dispo, r.width(), r.height());
    })
    .expect("capture");
    m
}

#[test]
fn la_carte_info_reste_dans_la_largeur_donnee() {
    for largeur in [300.0, 400.0, 560.0] {
        for e in EXPLICATIONS {
            let (dispo, w, _) = mesure(largeur, e, true);
            assert!(
                w <= dispo + 0.01,
                "largeur {largeur}, {e:?} : carte {w} > {dispo}"
            );
            // Comme une carte ordinaire (même cadre) : pas plus large.
            let (_, wc, _) = mesure(largeur, e, false);
            assert!((w - wc).abs() < 0.01, "largeur {largeur} : info {w}, carte {wc}");
        }
    }
}

#[test]
fn la_carte_info_n_est_pas_plus_haute_qu_il_faut() {
    // Une ligne d'explication courte : la hauteur minimale de la carte (48 + marges).
    let (_, _, h) = mesure(560.0, "Une explication courte.", true);
    let (_, _, hc) = mesure(560.0, "Une explication courte.", false);
    assert!((h - hc).abs() < 0.01, "info {h}, carte {hc}");
}

/// Les captures de la documentation : `CARTE_INFO_CAPTURES=<dossier> cargo test …`.
#[test]
fn captures_si_demandees() {
    let Some(dossier) = std::env::var_os("CARTE_INFO_CAPTURES") else {
        return;
    };
    for (nom, sombre) in [("clair", false), ("sombre", true)] {
        let o = Options { sombre, echelle: 2.0 };
        let image = capture::dessin(egui::vec2(420.0, 190.0), o, |ui, p| {
            ui.add_space(8.0);
            dessin::carte_info(
                ui,
                p,
                'i',
                "Titre",
                "Une explication nettement plus longue qui passe à la ligne.",
            );
            ui.add_space(8.0);
            dessin::carte(ui, p, 'i', "Titre", "Même largeur.", |_| {});
        })
        .expect("capture");
        image
            .enregistrer(&std::path::Path::new(&dossier).join(format!("{nom}.png")))
            .expect("png");
    }
}
