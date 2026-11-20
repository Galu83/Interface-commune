//! Le nuancier : choisir une couleur d'un clic dans une grille de pastilles, sans jamais taper
//! de code. Le logiciel donne les couleurs (rangées comme il veut : teintes en colonnes, tons
//! en lignes) ; la bibliothèque les dessine à la façon de Windows 11 : zones cliquables de 32
//! points, anneau sur la couleur choisie, contour de focus au clavier (règles : interface
//! Windows 11, points 5 et 8).
//!
//! - [`pastille`] : une couleur ronde, cliquable ;
//! - [`grille`] : toutes les couleurs, rend celle qu'on clique ;
//! - [`bouton_nuancier`] : une pastille qui ouvre la grille dans une bulle.

use egui::{Popup, PopupCloseBehavior, Response, Sense, Stroke, Ui, vec2};

use crate::apparence::couleur;
use crate::dessin::focus;
use crate::theme::{Palette, Rgb};

/// Côté de la zone cliquable d'une pastille, en points.
pub const ZONE: f32 = 32.0;
/// Rayon de la couleur dessinée.
const RAYON: f32 = 11.0;

/// « #RRVVBB », le nom lu par un lecteur d'écran quand la couleur n'a pas d'autre nom.
fn code(c: Rgb) -> String {
    format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2)
}

/// Une couleur ronde de 32 points, cliquable. `choisie` : un anneau de la couleur du texte
/// autour. `nom` : ce que dit un lecteur d'écran (le code de la couleur s'y ajoute).
pub fn pastille(ui: &mut Ui, p: &Palette, c: Rgb, nom: &str, choisie: bool) -> Response {
    let (_, r) = ui.allocate_exact_size(vec2(ZONE, ZONE), Sense::click());
    dessiner_pastille(ui, p, &r, c, nom, choisie);
    r
}

/// Dessine la pastille ronde dans la zone de `r` (déjà posée, avec sa propre clé : le choix
/// d'une couleur au clavier en a besoin, [`crate::couleurs`]).
pub fn dessiner_pastille(ui: &Ui, p: &Palette, r: &Response, c: Rgb, nom: &str, choisie: bool) {
    if ui.is_rect_visible(r.rect) {
        let centre = r.rect.center();
        let rayon = if r.hovered() { RAYON + 1.0 } else { RAYON };
        if choisie {
            ui.painter()
                .circle_stroke(centre, RAYON + 3.5, Stroke::new(2.0, couleur(p.texte)));
        }
        ui.painter().circle_filled(centre, rayon, couleur(c));
        // Un fin contour : une couleur proche du fond (blanc sur clair, noir sur sombre) se voit.
        ui.painter()
            .circle_stroke(centre, rayon, Stroke::new(1.0, couleur(p.bord_controle)));
        focus(ui, p, r, 16);
    }
    let libelle = format!("{nom} {}", code(c));
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &libelle));
}

/// La grille : une ligne de pastilles par ligne de `couleurs`. Rend la couleur cliquée à cette
/// image ; `actuelle` porte l'anneau.
pub fn grille(
    ui: &mut Ui,
    p: &Palette,
    couleurs: &[Vec<Rgb>],
    actuelle: Option<Rgb>,
    nom: &str,
) -> Option<Rgb> {
    let mut choix = None;
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    for ligne in couleurs {
        ui.horizontal(|ui| {
            for &c in ligne {
                if pastille(ui, p, c, nom, actuelle == Some(c)).clicked() {
                    choix = Some(c);
                }
            }
        });
    }
    choix
}

/// Une pastille de la couleur `c` qui ouvre le nuancier dans une bulle ; rend la couleur
/// choisie (la bulle se ferme alors). `nom` : ce que dit un lecteur d'écran, par exemple
/// « Couleur d'un bon niveau ».
pub fn bouton_nuancier(ui: &mut Ui, p: &Palette, c: Rgb, nom: &str, couleurs: &[Vec<Rgb>]) -> Option<Rgb> {
    let r = pastille(ui, p, c, nom, false);
    let mut choix = None;
    Popup::from_toggle_button_response(&r)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_min_width(couleurs.iter().map(Vec::len).max().unwrap_or(0) as f32 * ZONE);
            choix = grille(ui, p, couleurs, Some(c), nom);
            if choix.is_some() {
                ui.close();
            }
        });
    choix
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Palette;
    use egui::{Context, Event, Modifiers, PointerButton, RawInput, pos2};

    fn palette() -> Palette {
        Palette::de(&crate::theme::ChoixWindows::default())
    }

    fn grille_d_essai() -> Vec<Vec<Rgb>> {
        vec![
            vec![Rgb(255, 0, 0), Rgb(0, 255, 0), Rgb(0, 0, 255)],
            vec![Rgb(10, 10, 10), Rgb(128, 128, 128), Rgb(250, 250, 250)],
        ]
    }

    /// Une image de la grille, avec les événements donnés ; rend le choix et la taille.
    fn image(ctx: &Context, events: Vec<Event>, actuelle: Option<Rgb>) -> (Option<Rgb>, egui::Vec2) {
        let p = palette();
        let g = grille_d_essai();
        let mut sortie = (None, egui::Vec2::ZERO);
        let mut rendu = ctx.run_ui(
            RawInput {
                events,
                screen_rect: Some(egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0))),
                ..Default::default()
            },
            |ui| {
                let avant = ui.cursor().min;
                sortie.0 = grille(ui, &p, &g, actuelle, "Couleur");
                sortie.1 = ui.min_rect().max - avant;
            },
        );
        rendu.textures_delta.clear();
        sortie
    }

    #[test]
    fn la_grille_est_faite_de_cases_de_32_points() {
        let ctx = Context::default();
        let (choix, taille) = image(&ctx, vec![], None);
        assert_eq!(choix, None);
        assert_eq!(taille, vec2(3.0 * ZONE, 2.0 * ZONE));
    }

    #[test]
    fn un_clic_donne_la_couleur_sous_le_pointeur() {
        let ctx = Context::default();
        let _ = image(&ctx, vec![], None);
        // Le centre de la deuxième pastille de la première ligne : le vert.
        let au = pos2(ZONE * 1.5, ZONE * 0.5);
        let appui = |pressed| Event::PointerButton {
            pos: au,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let _ = image(&ctx, vec![Event::PointerMoved(au)], None);
        let _ = image(&ctx, vec![appui(true)], None);
        let (choix, _) = image(&ctx, vec![appui(false)], None);
        assert_eq!(choix, Some(Rgb(0, 255, 0)));
        // Sans clic, rien n'est choisi, même avec une couleur actuelle.
        let (rien, _) = image(&ctx, vec![], Some(Rgb(0, 255, 0)));
        assert_eq!(rien, None);
    }

    /// Une image du bouton, qui ouvre la grille dans une bulle ; rend le choix et le centre du bouton.
    fn image_bouton(ctx: &Context, events: Vec<Event>) -> Option<Rgb> {
        let p = palette();
        let g = grille_d_essai();
        let mut choix = None;
        let mut rendu = ctx.run_ui(
            RawInput {
                events,
                screen_rect: Some(egui::Rect::from_min_size(pos2(0.0, 0.0), vec2(500.0, 400.0))),
                ..Default::default()
            },
            |ui| {
                choix = bouton_nuancier(ui, &p, Rgb(9, 9, 9), "Bon niveau", &g);
            },
        );
        rendu.textures_delta.clear();
        choix
    }

    #[test]
    fn le_bouton_ouvre_la_grille_et_un_clic_dedans_la_ferme() {
        let ctx = Context::default();
        let bouton = pos2(ZONE / 2.0, ZONE / 2.0);
        let evenement = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let _ = image_bouton(&ctx, vec![]);
        let _ = image_bouton(&ctx, vec![Event::PointerMoved(bouton)]);
        let _ = image_bouton(&ctx, vec![evenement(bouton, true)]);
        assert_eq!(image_bouton(&ctx, vec![evenement(bouton, false)]), None);
        assert!(egui::Popup::is_any_open(&ctx), "la bulle devrait être ouverte");
        // Un clic sur la première pastille de la bulle donne sa couleur et ferme la bulle.
        // La bulle prend sa place à l'image suivante, une fois mesurée.
        let _ = image_bouton(&ctx, vec![]);
        let _ = image_bouton(&ctx, vec![]);
        let bulle = ctx
            .memory(|m| {
                m.areas()
                    .visible_layer_ids()
                    .into_iter()
                    .filter(|l| l.order == egui::Order::Foreground)
                    .filter_map(|l| m.area_rect(l.id))
                    .last()
            })
            .expect("la bulle a un emplacement");
        let au = bulle.min + vec2(20.0, 20.0);
        let _ = image_bouton(&ctx, vec![Event::PointerMoved(au)]);
        let _ = image_bouton(&ctx, vec![evenement(au, true)]);
        let gagne = image_bouton(&ctx, vec![evenement(au, false)]);
        assert_eq!(gagne, Some(Rgb(255, 0, 0)), "bulle en {bulle:?}");
        let _ = image_bouton(&ctx, vec![]);
        assert!(
            !egui::Popup::is_any_open(&ctx),
            "la bulle doit se fermer après le choix"
        );
    }

    #[test]
    fn les_lecteurs_d_ecran_recoivent_un_nom_avec_le_code() {
        assert_eq!(code(Rgb(1, 2, 255)), "#0102FF");
    }
}
