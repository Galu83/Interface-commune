//! La conversion d'une couleur du cœur ([`Rgb`]) vers egui : le seul passage de l'une à l'autre.

use egui::Color32;

use crate::theme::Rgb;

/// Une couleur de la palette pour egui.
pub fn couleur(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}
