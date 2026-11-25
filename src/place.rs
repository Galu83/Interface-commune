//! Où ouvrir le volet et le petit menu de l'icône, et où rouvrir les Paramètres. Comme les
//! menus de Windows 11 : collé à la barre des tâches, en face de l'icône, en entier sur son
//! écran.
//!
//! Le calcul vit dans le cœur (`ic_coeur::placement`, bloc D) et y est testé sans fenêtre ;
//! ce module le reprend à ses chemins, avec l'instant de la bibliothèque
//! (`std::time::Instant`).

pub use ic_coeur::placement::{
    Cote, ECART_VOLET, Place, PlacementVolet, RETENIR_APRES, Rect, Retenue, centre_de, centrer,
    cote_de_la_barre, ecran_de, garder_visible, place_au_changement_d_echelle, place_valable, placer,
    placer_au_point, placer_contre_l_icone,
};

/// Le suivi de la place d'une fenêtre ouverte ([`ic_coeur::placement::SuiviDePlace`]),
/// compté en `Instant`.
pub type SuiviDePlace = ic_coeur::placement::SuiviDePlace<std::time::Instant>;

/// Un rectangle egui vers celui du cœur ([`ic_coeur::placement::plan::Rectangle`]).
pub(crate) fn vers_le_coeur(r: egui::Rect) -> ic_coeur::placement::plan::Rectangle {
    ic_coeur::placement::plan::Rectangle::de_min_max([r.min.x, r.min.y], [r.max.x, r.max.y])
}

/// Un rectangle du cœur vers egui.
pub(crate) fn vers_egui(r: ic_coeur::placement::plan::Rectangle) -> egui::Rect {
    egui::Rect::from_min_max(egui::pos2(r.min[0], r.min[1]), egui::pos2(r.max[0], r.max[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'aller et retour entre egui et le cœur ne change rien, pas même un rectangle vide ;
    /// le cœur calcule comme egui (largeur, centre, réduction, union).
    #[test]
    fn rectangles_egui_et_coeur() {
        let r = egui::Rect::from_min_size(egui::pos2(10.5, -3.25), egui::vec2(120.0, 44.75));
        assert_eq!(vers_egui(vers_le_coeur(r)), r);
        assert_eq!(vers_egui(vers_le_coeur(egui::Rect::NOTHING)), egui::Rect::NOTHING);
        let c = vers_le_coeur(r);
        assert_eq!(c.largeur(), r.width());
        assert_eq!(c.hauteur(), r.height());
        assert_eq!(c.centre(), [r.center().x, r.center().y]);
        assert_eq!(vers_egui(c.reduit(4.0)), r.shrink(4.0));
        let a = egui::Rect::from_min_max(egui::pos2(-7.0, 2.0), egui::pos2(3.0, 90.0));
        assert_eq!(vers_egui(c.union(vers_le_coeur(a))), r.union(a));
    }
}
