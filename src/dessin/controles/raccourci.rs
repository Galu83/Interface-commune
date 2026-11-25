//! Le champ de raccourci clavier : la saisie d'une touche avec Ctrl, Alt ou Maj.

use egui::Ui;

use super::base::{Genre, bouton};
use crate::dessin::couleur::couleur;
use crate::icones;
use crate::textes::Textes;
use crate::theme::Palette;

/// Ce que fait un champ de raccourci clavier à cette image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saisie {
    Rien,
    /// La saisie commence : le logiciel suspend ses propres raccourcis jusqu'à la fin.
    Commencee,
    /// Échap : la valeur n'a pas changé.
    Annulee,
    /// Un nouveau raccourci est écrit dans la valeur.
    Changee,
}

/// Un raccourci saisi, au format « ctrl+alt+shift+k » (repris de Vocalis) ; l'affichage le met
/// en toutes lettres (« Ctrl+Alt+Maj+K », [`crate::touches::lisible`]), la valeur ne change pas.
pub fn raccourci_de(m: egui::Modifiers, touche: egui::Key) -> String {
    let mut parties = Vec::new();
    if m.ctrl || m.command {
        parties.push("ctrl".to_string());
    }
    if m.alt {
        parties.push("alt".into());
    }
    if m.shift {
        parties.push("shift".into());
    }
    parties.push(touche.name().to_lowercase());
    parties.join("+")
}

/// Un champ de raccourci clavier : la valeur et « Changer… » ; un clic lance la saisie, la
/// touche suivante (avec Ctrl, Alt, Maj) devient la valeur, Échap annule. `id` : unique dans
/// la fenêtre (la saisie en cours est retenue par egui).
pub fn champ_raccourci(ui: &mut Ui, p: &Palette, t: &Textes, id: &str, valeur: &mut String) -> Saisie {
    let id = ui.scope_id().with(("raccourci", id));
    let en_cours = ui.memory(|m| m.data.get_temp::<bool>(id).unwrap_or(false));
    if en_cours {
        ui.label(egui::RichText::new(t.appuyer_touches).color(couleur(p.accent)));
        let touche = ui.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => Some((*key, *modifiers)),
                _ => None,
            })
        });
        let Some((key, m)) = touche else {
            return Saisie::Rien;
        };
        ui.memory_mut(|mem| mem.data.remove::<bool>(id));
        if key == egui::Key::Escape {
            return Saisie::Annulee;
        }
        *valeur = raccourci_de(m, key);
        return Saisie::Changee;
    }
    let mut saisie = Saisie::Rien;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(crate::touches::lisible(valeur)).strong());
        if bouton(ui, p, Some(icones::CLAVIER), t.changer_raccourci, Genre::Normal).clicked() {
            ui.memory_mut(|mem| mem.data.insert_temp(id, true));
            saisie = Saisie::Commencee;
        }
    });
    saisie
}

#[cfg(test)]
mod essais_raccourci {
    use super::*;

    fn image(ctx: &egui::Context, evts: Vec<egui::Event>, valeur: &mut String) -> (Saisie, egui::Rect) {
        let p = Palette::de(&crate::theme::ChoixWindows::default());
        let entree = egui::RawInput {
            events: evts,
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let mut rendu = (Saisie::Rien, egui::Rect::NOTHING);
        let mut sortie = ctx.run_ui(entree, |ui| {
            rendu.0 = champ_raccourci(ui, &p, &crate::textes::FRANCAIS, "dictee", valeur);
            rendu.1 = ui.min_rect();
        });
        sortie.textures_delta.clear();
        rendu
    }

    fn touche(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn cliquer(ctx: &egui::Context, valeur: &mut String) -> Saisie {
        // Le bouton « Changer… » est le dernier élément, à droite de la valeur.
        let (_, r) = image(ctx, vec![], valeur);
        let pos = egui::pos2(r.right() - 10.0, r.center().y);
        let bouton = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        image(ctx, vec![egui::Event::PointerMoved(pos)], valeur);
        image(ctx, vec![bouton(true)], valeur);
        image(ctx, vec![bouton(false)], valeur).0
    }

    /// « Changer… » lance la saisie ; la touche suivante, avec Ctrl et Maj, devient la valeur ;
    /// Échap annule sans rien changer.
    #[test]
    fn saisie_d_un_raccourci() {
        let ctx = egui::Context::default();
        let mut v = "ctrl+alt+d".to_string();
        assert_eq!(cliquer(&ctx, &mut v), Saisie::Commencee);
        let m = egui::Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(image(&ctx, vec![], &mut v).0, Saisie::Rien);
        assert_eq!(
            image(&ctx, vec![touche(egui::Key::K, m)], &mut v).0,
            Saisie::Changee
        );
        assert_eq!(v, "ctrl+shift+k");
        assert_eq!(cliquer(&ctx, &mut v), Saisie::Commencee);
        assert_eq!(
            image(&ctx, vec![touche(egui::Key::Escape, Default::default())], &mut v).0,
            Saisie::Annulee
        );
        assert_eq!(v, "ctrl+shift+k");
        // La saisie est finie : une touche ne change plus rien.
        assert_eq!(image(&ctx, vec![touche(egui::Key::A, m)], &mut v).0, Saisie::Rien);
        assert_eq!(v, "ctrl+shift+k");
    }
}
