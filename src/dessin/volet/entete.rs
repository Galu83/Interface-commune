//! L'en-tête du volet : à gauche le gros titre et sa version, petite et grise, sur la même
//! ligne de base ; à droite la roue dentée des Paramètres, carrée (36 × 36), centrée sur la
//! hauteur de la ligne. Le titre ne passe jamais sous la roue : il se coupe par « … ». Les
//! places se décident dans le cœur (`ic_coeur::placement::surface::entete`).

use egui::RichText;
use ic_coeur::placement::surface as mise_en_page;

use crate::apparence::couleur;
use crate::dessin::bouton_icone;
use crate::icones;
use crate::mesures::VOLET_ROUE;
use crate::place::{vers_egui, vers_le_coeur};
use crate::polices::titre;
use crate::theme::{Palette, TAILLE_PETIT, TAILLE_TITRE_VOLET};

/// La distance du haut d'un texte à sa ligne de base (points).
fn ligne_de_base(g: &egui::Galley) -> f32 {
    g.rows
        .first()
        .and_then(|r| r.glyphs.first().map(|c| r.pos.y + c.pos.y))
        .unwrap_or_else(|| g.size().y)
}

/// Le gros titre, et à sa droite la version, petite et grise, sur la même ligne de base. Le
/// titre se coupe par « … » si la place manque ; la version reste entière.
fn titre_et_version(ui: &mut egui::Ui, p: &Palette, titre_texte: &str, version: Option<&str>) {
    let police = titre(TAILLE_TITRE_VOLET);
    let petite = egui::FontId::proportional(TAILLE_PETIT);
    let gris = couleur(p.texte_secondaire);
    let v = version.map(|v| ui.painter().layout_no_wrap(v.to_string(), petite, gris));
    let place = ui.available_width();
    let largeur_titre = mise_en_page::place_du_titre(place, v.as_ref().map(|v| v.size().x));
    let haut = ui
        .allocate_ui(egui::vec2(largeur_titre, 0.0), |ui| {
            ui.add(egui::Label::new(RichText::new(titre_texte).font(police.clone())).truncate())
        })
        .inner;
    let Some(v) = v else { return };
    let g = ui.painter().layout_no_wrap(titre_texte.to_string(), police, gris);
    let [x, y] = mise_en_page::coin_de_la_version(
        vers_le_coeur(haut.rect),
        g.size().x,
        place,
        v.size().x,
        (ligne_de_base(&g), ligne_de_base(&v)),
    );
    ui.painter().galley(egui::pos2(x, y), v, gris);
}

/// L'en-tête du volet ; rend `true` si la roue est cliquée.
pub(super) fn entete(
    ui: &mut egui::Ui,
    p: &Palette,
    titre_texte: &str,
    version: Option<&str>,
    nom_roue: &str,
) -> bool {
    let largeur = ui.available_width();
    let (ligne, _) = ui.allocate_exact_size(egui::vec2(largeur, VOLET_ROUE), egui::Sense::hover());
    let hauteur_titre = ui
        .painter()
        .layout_no_wrap(
            titre_texte.to_string(),
            titre(TAILLE_TITRE_VOLET),
            egui::Color32::WHITE,
        )
        .size()
        .y;
    let places = mise_en_page::entete([ligne.left(), ligne.top()], largeur, hauteur_titre);
    // La roue : collée à droite, avant le titre (sa place dans l'ordre du clavier).
    let clic = ui
        .scope_builder(egui::UiBuilder::new().max_rect(vers_egui(places.roue)), |ui| {
            bouton_icone(ui, p, icones::REGLAGES, nom_roue).clicked()
        })
        .inner;
    // Le titre : à gauche, centré sur la hauteur de la roue, sur le reste de la place.
    ui.scope_builder(egui::UiBuilder::new().max_rect(vers_egui(places.titre)), |ui| {
        titre_et_version(ui, p, titre_texte, version);
    });
    clic
}
