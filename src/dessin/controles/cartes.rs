//! Les titres, les cartes de réglage, la section repliable, le bandeau et l'état du volet.

use egui::{CornerRadius, Response, Sense, Stroke, Ui, Vec2, vec2};
use ic_coeur::textes::insecables;

use super::base::{Genre, bouton, focus, icone};
use crate::dessin::couleur::couleur;
use crate::icones;
use crate::polices::{semibold, titre};
use crate::theme::{Palette, Rgb};

/// Titre de page (28 points) et une ligne d'explication.
pub fn titre_page(ui: &mut Ui, p: &Palette, texte: &str, explication: &str) {
    ui.label(egui::RichText::new(texte).font(titre(crate::theme::TAILLE_TITRE)));
    if !explication.is_empty() {
        ui.label(egui::RichText::new(explication).color(couleur(p.texte_secondaire)));
    }
    ui.add_space(8.0);
}

/// Titre de section (15 points semibold).
pub fn titre_section(ui: &mut Ui, texte: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(texte).font(semibold(crate::theme::TAILLE_SECTION)));
}

/// Le cadre d'une carte (8 points d'arrondi).
pub fn cadre_carte(p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(couleur(p.carte))
        .stroke(Stroke::new(1.0, couleur(p.bord_carte)))
        .corner_radius(CornerRadius::same(crate::theme::ARRONDI_CARTE))
        .inner_margin(egui::Margin::symmetric(16, 10))
}

/// Une carte de réglage : icône, titre, une ligne d'explication, contrôle à droite.
pub fn carte(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    texte: &str,
    explication: &str,
    controle: impl FnOnce(&mut Ui),
) {
    carte_avec(ui, p, glyphe, texte, explication, 200.0, Some(controle));
}

/// Une carte dont les contrôles prennent `place` points (plusieurs boutons).
pub fn carte_large(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    texte: &str,
    explication: &str,
    place: f32,
    controle: impl FnOnce(&mut Ui),
) {
    carte_avec(ui, p, glyphe, texte, explication, place, Some(controle));
}

/// Une carte d'information, sans contrôle : le texte prend toute la largeur.
pub fn carte_info(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str, explication: &str) {
    carte_avec(ui, p, glyphe, texte, explication, 0.0, None::<fn(&mut Ui)>);
}

/// Une carte qui ouvre une sous-page : toute la carte se clique, un chevron « › » à droite.
pub fn carte_chevron(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str, explication: &str) -> bool {
    let r = cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.set_min_height(48.0);
            let (r, _) = ui.allocate_exact_size(Vec2::new(28.0, 32.0), Sense::hover());
            icone(ui, r.center(), glyphe, 20.0, p.texte);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.add_space(4.0);
                ui.label(texte);
                if !explication.is_empty() {
                    ui.label(
                        egui::RichText::new(explication)
                            .size(crate::theme::TAILLE_EXPLICATION)
                            .color(couleur(p.texte_secondaire)),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(vec2(24.0, 32.0), Sense::hover());
                icone(ui, r.center(), icones::CHEVRON, 12.0, p.texte_secondaire);
            });
        });
    });
    let r = ui.interact(
        r.response.rect,
        ui.scope_id().with(("chevron", texte)),
        Sense::click(),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    focus(ui, p, &r, crate::theme::ARRONDI_CARTE);
    r.clicked()
}

/// L'en-tête d'une section repliable, à la façon de l'« Expander » de Windows 11 : une carte
/// (icône facultative, titre, explication) dont toute la surface se clique, et un chevron vers le bas
/// à droite qui tourne de 180° à l'ouverture. `part` : 0 (replié) à 1 (ouvert), pour la
/// rotation ; `ouvert` : l'état voulu (annoncé aux lecteurs d'écran). Le clavier : l'en-tête
/// prend le focus, Entrée et Espace comptent comme un clic. `id` : l'identité de la section.
#[allow(clippy::too_many_arguments)]
pub fn entete_section(
    ui: &mut Ui,
    p: &Palette,
    id: egui::Id,
    glyphe: Option<char>,
    texte: &str,
    explication: &str,
    part: f32,
    ouvert: bool,
) -> Response {
    let r = cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.set_min_height(48.0);
            if let Some(g) = glyphe {
                let (r, _) = ui.allocate_exact_size(Vec2::new(28.0, 32.0), Sense::hover());
                icone(ui, r.center(), g, 20.0, p.texte);
                ui.add_space(8.0);
            }
            ui.vertical(|ui| {
                ui.add_space(4.0);
                ui.label(texte);
                if !explication.is_empty() {
                    ui.label(
                        egui::RichText::new(explication)
                            .size(crate::theme::TAILLE_EXPLICATION)
                            .color(couleur(p.texte_secondaire)),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(vec2(24.0, 32.0), Sense::hover());
                chevron_qui_tourne(ui, r.center(), part, p.texte_secondaire);
            });
        });
    });
    let r = ui.interact(r.response.rect, id.with("entete"), Sense::click());
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    ui.ctx().accesskit_node_builder(r.id, |n| n.set_expanded(ouvert));
    focus(ui, p, &r, crate::theme::ARRONDI_CARTE);
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Un chevron vers le bas de 12 points centré en `centre`, tourné de `part` demi-tour (0 : vers le
/// bas ; 1 : vers le haut).
fn chevron_qui_tourne(ui: &Ui, centre: egui::Pos2, part: f32, c: Rgb) {
    let angle = part.clamp(0.0, 1.0) * std::f32::consts::PI;
    let (s, co) = angle.sin_cos();
    let point = |x: f32, y: f32| centre + vec2(x * co - y * s, x * s + y * co);
    ui.painter().add(egui::Shape::line(
        vec![point(-5.0, -2.5), point(0.0, 2.5), point(5.0, -2.5)],
        Stroke::new(1.5, couleur(c)),
    ));
}

fn carte_avec(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    texte: &str,
    explication: &str,
    place_controle: f32,
    controle: Option<impl FnOnce(&mut Ui)>,
) {
    cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.set_min_height(48.0);
            let (r, _) = ui.allocate_exact_size(Vec2::new(28.0, 32.0), Sense::hover());
            icone(ui, r.center(), glyphe, 20.0, p.texte);
            ui.add_space(8.0);
            let largeur_texte = (ui.available_width() - place_controle).max(160.0);
            ui.vertical(|ui| {
                ui.set_width(largeur_texte);
                ui.add_space(4.0);
                ui.label(texte);
                if !explication.is_empty() {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(insecables(explication))
                                .size(crate::theme::TAILLE_EXPLICATION)
                                .color(couleur(p.texte_secondaire)),
                        )
                        .wrap(),
                    );
                }
            });
            // Sans contrôle, pas de bloc vide : il ajouterait un espacement (8 pt) après le texte.
            if let Some(controle) = controle {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), controle);
            }
        });
    });
}

/// Un bandeau en haut d'une page (redémarrage nécessaire, avertissement) : fond teinté de la
/// couleur `c`, icône, texte, et un bouton à droite. Rend `true` quand le bouton est cliqué.
pub fn bandeau(ui: &mut Ui, p: &Palette, c: Rgb, glyphe: char, texte: &str, bouton_texte: &str) -> bool {
    let mut clic = false;
    egui::Frame::new()
        .fill(couleur(p.carte.vers(c, 0.15)))
        .stroke(Stroke::new(1.0, couleur(p.bord_carte.vers(c, 0.4))))
        .corner_radius(CornerRadius::same(crate::theme::ARRONDI_CARTE))
        .inner_margin(egui::Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(20.0, 32.0), Sense::hover());
                icone(ui, r.center(), glyphe, 16.0, c);
                ui.label(texte);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    clic = bouton(ui, p, None, bouton_texte, Genre::Accent).clicked();
                });
            });
        });
    clic
}

/// La place de l'icône d'un état, devant son texte (points).
pub const ETAT_ICONE: f32 = 18.0;

/// Un état (icône de couleur et texte de la même couleur) qui passe à la ligne s'il est trop
/// long : l'icône reste en face de la première ligne (le volet de l'icône).
pub fn etat_long(ui: &mut Ui, c: Rgb, glyphe: char, texte: &str) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(ETAT_ICONE, 20.0), Sense::hover());
        icone(ui, r.center(), glyphe, 14.0, c);
        ui.add(
            egui::Label::new(
                egui::RichText::new(insecables(texte))
                    .color(couleur(c))
                    .font(semibold(14.0)),
            )
            .wrap(),
        );
    });
}

/// Une pastille d'état (rond de couleur) suivie d'un texte de la même couleur.
pub fn etat(ui: &mut Ui, c: Rgb, glyphe: char, texte: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(18.0, 20.0), Sense::hover());
        icone(ui, r.center(), glyphe, 14.0, c);
        ui.label(egui::RichText::new(texte).color(couleur(c)).font(semibold(14.0)));
    });
}
