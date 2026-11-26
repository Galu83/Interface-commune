//! Le thème suivi en direct (règles : interface Windows 11, point 1) : le mode clair ou
//! sombre et l'accent de Windows, relus toutes les 2 secondes (quelques valeurs du registre,
//! rien de lent), appliqués à egui dès qu'ils changent. Quand relire, et ce qui change,
//! se décide dans le cœur (`ic_coeur::theme`, bloc C) ; les couleurs données à egui, dans le
//! dessin (`src/dessin/apparence.rs`) ; ici, lire Windows et l'heure.

use std::time::Instant;

use ic_coeur::theme;

use crate::theme::{ChoixWindows, Palette};

// La conversion d'une couleur du cœur vers egui vit avec le dessin (bloc F), reprise ici à
// son chemin d'aujourd'hui.
pub use crate::dessin::couleur::couleur;

/// Les 10 couleurs des pastilles d'appareils du volet (voir [`crate::pastilles`]).
pub use ic_coeur::theme::couleurs_pastilles;

/// Le thème est relu dans Windows à cet intervalle.
pub use ic_coeur::theme::RELECTURE;

/// Ce que Windows a choisi (sous Windows) ; ailleurs, le clair et l'accent par défaut.
pub fn choix_windows() -> ChoixWindows {
    #[cfg(windows)]
    {
        super::registre::choix_windows()
    }
    #[cfg(not(windows))]
    {
        ChoixWindows::default()
    }
}

/// Le thème suivi.
pub struct Apparence {
    pub choix: ChoixWindows,
    pub palette: Palette,
    /// Pour les captures et les essais : clair (`Some(false)`) ou sombre (`Some(true)`)
    /// imposé.
    pub force: Option<bool>,
    lu_a: Instant,
}

impl Apparence {
    pub fn new(force: Option<bool>) -> Apparence {
        Apparence::de(choix_windows(), force)
    }

    /// Avec un choix donné (essais, captures).
    pub fn de(choix: ChoixWindows, force: Option<bool>) -> Apparence {
        let choix = theme::impose(choix, force);
        Apparence {
            palette: Palette::de(&choix),
            choix,
            force,
            lu_a: Instant::now(),
        }
    }

    /// Relit Windows si 2 s ont passé ; `true` si le thème a changé (déjà appliqué).
    pub fn suivre(&mut self, ctx: &egui::Context) -> bool {
        ctx.request_repaint_after(RELECTURE);
        if !theme::faut_relire(self.lu_a.elapsed()) {
            return false;
        }
        self.relire(ctx)
    }

    /// Relit Windows tout de suite ; `true` si le thème a changé (déjà appliqué).
    pub fn relire(&mut self, ctx: &egui::Context) -> bool {
        self.lu_a = Instant::now();
        let Some((choix, palette)) = theme::relu(&self.choix, choix_windows(), self.force) else {
            return false;
        };
        self.choix = choix;
        self.palette = palette;
        self.appliquer(ctx);
        true
    }

    /// Les couleurs, tailles et espacements de Windows 11, donnés à egui.
    pub fn appliquer(&self, ctx: &egui::Context) {
        crate::dessin::apparence::appliquer(ctx, &self.palette);
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Le mode imposé l'emporte sur celui de Windows, et la palette le suit.
    #[test]
    fn le_mode_impose_l_emporte() {
        for sombre in [false, true] {
            let a = Apparence::de(
                ChoixWindows {
                    sombre: !sombre,
                    ..ChoixWindows::default()
                },
                Some(sombre),
            );
            assert_eq!(a.choix.sombre, sombre);
            assert_eq!(a.palette.sombre, sombre);
            assert_eq!(a.force, Some(sombre));
        }
        // Rien d'imposé : le choix de Windows.
        let a = Apparence::de(
            ChoixWindows {
                sombre: true,
                ..ChoixWindows::default()
            },
            None,
        );
        assert!(a.palette.sombre);
    }

    /// Juste après la lecture, rien n'est relu : `suivre` ne change rien, et demande à egui de
    /// revenir au moment de la prochaine relecture.
    #[test]
    fn rien_n_est_relu_avant_l_heure() {
        let ctx = egui::Context::default();
        let mut a = Apparence::de(ChoixWindows::default(), Some(false));
        a.appliquer(&ctx);
        assert!(!a.suivre(&ctx));
        assert!(!ctx.global_style().visuals.dark_mode);
    }

    /// Un thème imposé ne bouge pas quand Windows est relu.
    #[test]
    fn un_theme_impose_tient_a_la_relecture() {
        let ctx = egui::Context::default();
        let mut a = Apparence::de(ChoixWindows::default(), Some(true));
        a.relire(&ctx);
        assert!(a.palette.sombre);
    }
}
