//! Les flèches haut et bas du volet passent d'une ligne à l'autre, comme Tab et Maj+Tab
//! (C-33) : elles sont changées en Tab avant qu'egui lise les touches. (Demander le changement
//! de focus après le dessin d'une image ne fait rien : egui l'efface au début de l'image
//! suivante, et il écrase aussi la recherche « vers le bas » qu'il faisait lui-même : les
//! flèches ne servaient à rien. Un essai avec de vraies touches fabriquées l'a montré.)
//!
//! La règle se décide dans le cœur (`ic_coeur::gestes::clavier`) ; ici, la traduction des
//! touches d'egui. La fenêtre du volet l'appelle à chaque image (`crate::volet`, bloc J).

use egui::Key;
use ic_coeur::gestes::clavier::{Fleche, fleche_en_tab};

/// Change les flèches haut et bas sans modificateur en Maj+Tab et Tab.
pub fn fleches_en_tab(events: &mut [egui::Event]) {
    for e in events {
        if let egui::Event::Key { key, modifiers, .. } = e {
            let fleche = match *key {
                Key::ArrowDown => Fleche::Bas,
                Key::ArrowUp => Fleche::Haut,
                _ => continue,
            };
            if let Some(tab) = fleche_en_tab(fleche, modifiers.any()) {
                *key = Key::Tab;
                modifiers.shift = tab.arriere;
            }
        }
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Flèche bas = Tab, flèche haut = Maj+Tab ; les autres touches et les touches avec un
    /// modificateur ne changent pas (C-33).
    #[test]
    fn les_fleches_deviennent_tab() {
        let touche = |key, shift| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                shift,
                ..Default::default()
            },
        };
        let mut e = vec![
            touche(Key::ArrowDown, false),
            touche(Key::ArrowUp, false),
            touche(Key::ArrowDown, true),
            touche(Key::Escape, false),
        ];
        fleches_en_tab(&mut e);
        let lire = |e: &egui::Event| match e {
            egui::Event::Key { key, modifiers, .. } => (*key, modifiers.shift),
            _ => unreachable!(),
        };
        assert_eq!(lire(&e[0]), (Key::Tab, false));
        assert_eq!(lire(&e[1]), (Key::Tab, true));
        assert_eq!(lire(&e[2]), (Key::ArrowDown, true));
        assert_eq!(lire(&e[3]), (Key::Escape, false));
    }
}
