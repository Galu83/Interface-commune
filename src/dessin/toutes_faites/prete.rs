//! La fenêtre « `<Logiciel>` x.y.z est prête », toute faite avec la fenêtre simple : le logo,
//! le titre, les nouveautés, « Plus tard » et « Installer maintenant ».

use super::simple::{Bouton, Simple};
use crate::Textes;

/// La clé du bouton « Plus tard » de [`Simple::version_prete`] (« Installer maintenant » rend
/// [`crate::cles::INSTALLER`]).
pub const PLUS_TARD: &str = "plus_tard";

impl Simple {
    /// La fenêtre « `<Logiciel>` x.y.z est prête » : le logo, le titre, les nouveautés (une
    /// par ligne), « Plus tard » et « Installer maintenant » (bouton d'accent, Entrée).
    pub fn version_prete(
        identite: &crate::Identite,
        t: &Textes,
        version: &str,
        nouveautes: &[String],
    ) -> Simple {
        Simple {
            titre: crate::textes::remplir(t.prete_titre, &[&identite.nom, version]),
            sous_titre: t.prete_explication.into(),
            logo: identite.logo,
            nouveautes: nouveautes
                .iter()
                .map(|n| format!("• {n}"))
                .collect::<Vec<_>>()
                .join("\n"),
            boutons: vec![
                Bouton::new(PLUS_TARD, t.plus_tard),
                Bouton {
                    glyphe: Some(crate::icones::INSTALLER),
                    ..Bouton::new(crate::cles::INSTALLER, t.installer_maintenant).accent()
                },
            ],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use egui::Ui;

    use super::*;
    use crate::dessin::toutes_faites::essais::{entree, palette, une_image};
    use crate::dessin::toutes_faites::{EtatSimple, dessiner_simple};
    use crate::textes::FRANCAIS;

    /// « `<Logiciel>` x.y.z est prête » : titre, nouveautés, « Plus tard » puis « Installer
    /// maintenant », qu'Entrée choisit.
    #[test]
    fn version_prete_toute_faite() {
        let id = crate::Identite {
            nom: "Exemple".into(),
            version: "1.0.0".into(),
            licence: "MIT".into(),
            description: String::new(),
            icone: |_, _| Vec::new(),
            logo: None,
        };
        let s = Simple::version_prete(&id, &FRANCAIS, "2.0.0", &["Un ajout inventé.".into()]);
        assert_eq!(s.titre, "Exemple 2.0.0 est prête");
        assert_eq!(s.nouveautes, "• Un ajout inventé.");
        let cles: Vec<&str> = s.boutons.iter().map(|b| b.cle.as_str()).collect();
        assert_eq!(cles, [PLUS_TARD, crate::cles::INSTALLER]);
        let en = Simple::version_prete(&id, &crate::textes::ANGLAIS, "2.0.0", &[]);
        assert_eq!(en.titre, "Exemple 2.0.0 is ready");
        let ctx = egui::Context::default();
        let p = palette();
        let mut e = EtatSimple::default();
        let mut rendu = Vec::new();
        let mut f = |ui: &mut Ui| {
            if let Some(c) = dessiner_simple(ui, &p, &FRANCAIS, &s, &mut e) {
                rendu.push(c.cle);
            }
        };
        une_image(&ctx, vec![], &mut f);
        une_image(&ctx, vec![entree()], &mut f);
        drop(f);
        assert_eq!(rendu, [crate::cles::INSTALLER]);
    }
}
