//! Les raccourcis clavier tels qu'on les lit dans le volet, les menus et les Paramètres : **en
//! toutes lettres, comme Windows en français** (« Ctrl+Maj+K », « Alt+Flèche droite »,
//! « Win+Échap »), jamais en symboles (⇧, ⌃, ⌥, ⌘, →).
//!
//! Refonte, bloc B : [`lisible`] vit dans le cœur (`ic_coeur::textes::lisible`), reprise ici à
//! son chemin d'aujourd'hui ; ses essais sont dans `crates/ic-coeur/src/textes/essais.rs`. Reste
//! ici l'essai qui passe par egui : ce que saisit le champ de raccourci
//! ([`crate::dessin::raccourci_de`]) se lit bien.

pub use ic_coeur::textes::lisible;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ce_que_saisit_le_champ_de_raccourci_se_lit_bien() {
        use egui::{Key, Modifiers};
        let m = Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(lisible(&crate::dessin::raccourci_de(m, Key::K)), "Ctrl+Maj+K");
        assert_eq!(
            lisible(&crate::dessin::raccourci_de(Modifiers::ALT, Key::ArrowRight)),
            "Alt+Flèche droite"
        );
        assert_eq!(
            lisible(&crate::dessin::raccourci_de(Modifiers::NONE, Key::F5)),
            "F5"
        );
        assert_eq!(
            lisible(&crate::dessin::raccourci_de(Modifiers::CTRL, Key::Backspace)),
            "Ctrl+Retour arrière"
        );
    }
}
