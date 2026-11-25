//! Le dessin du volet (bloc G de la refonte) : l'en-tête (le titre, sa version, la roue des
//! Paramètres, dans [`entete`]), l'état en couleur et son détail qui passe à la ligne (C-25),
//! les briques, qui défilent au-delà de la hauteur maximale, et le pied fixe. Les flèches du
//! clavier y passent d'une brique à l'autre comme Tab (C-33, [`clavier`]).
//!
//! La mise en page se décide dans le cœur (`ic_coeur::placement::surface`) ; la fenêtre du
//! volet (sa boucle, son voile, son veilleur) reste dans `crate::volet` (bloc J).

pub(crate) mod clavier;
mod entete;

use egui::RichText;
use ic_coeur::placement::surface as mise_en_page;

use crate::Logiciel;
use crate::apparence::couleur;
use crate::briques::{self, Brique, Geste};
use crate::dessin::{self, surface::Choisi};
use crate::mesures::{VOLET_ENTETE_DROITE, VOLET_MARGE_ETAT};
use crate::theme::{Palette, TAILLE_PETIT};

/// Le volet : le titre, sa version et la roue dentée en haut, l'état, les réglages rapides.
pub fn dessiner_volet(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let briques = l.briques_avec_contexte(ui.ctx());
    let pied = l.briques_du_pied();
    volet(ui, p, l, (&briques, &pied), None)
}

/// Un geste dans une brique : une action devient le choix rendu, un réglage changé part tout
/// de suite au logiciel.
fn geste(l: &dyn Logiciel, g: Option<Geste>) -> Option<Choisi> {
    match g {
        Some(Geste::Action(cle)) => Some(Choisi::Rapide(cle)),
        Some(Geste::Changer(cle, v)) => {
            l.changer(&cle, v);
            None
        }
        None => None,
    }
}

/// L'en-tête et l'état du volet, en haut, qui ne défilent jamais. Rend `true` si la roue des
/// Paramètres est cliquée.
fn haut_du_volet(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> bool {
    let mut roue = false;
    let e = l.etat();
    // L'en-tête : le titre et sa version à gauche, la roue des Paramètres à l'extrême droite,
    // sur la même ligne (la roue d'abord pour le clavier : elle est la première en haut).
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: VOLET_MARGE_ETAT as i8,
            right: VOLET_ENTETE_DROITE as i8,
            top: mise_en_page::ENTETE_HAUT as i8,
            bottom: 0,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let v = dessin::surface::version(l);
            roue = entete::entete(
                ui,
                p,
                &e.titre,
                mise_en_page::version_montree(&v),
                l.textes().parametres,
            );
        });
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: VOLET_MARGE_ETAT as i8,
            right: VOLET_MARGE_ETAT as i8,
            top: 0,
            bottom: mise_en_page::ETAT_BAS as i8,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let (g, c) = e.genre.icone_et_couleur(p);
            dessin::etat_long(ui, c, g, &e.ligne);
            if let Some(m) = &e.detail {
                ui.add(
                    egui::Label::new(
                        RichText::new(dessin::insecables(m))
                            .color(couleur(p.texte_secondaire))
                            .size(TAILLE_PETIT),
                    )
                    .wrap(),
                );
            }
        });
    roue
}

/// Le volet, avec ses briques et son pied fixe (`briques`, `pied`) ; `hauteur_max` : la hauteur
/// de son contenu à ne pas dépasser (points, sans les marges). Au-delà, seule la liste des
/// briques défile, dans la place qui reste entre l'état et le pied ; le pied reste en bas.
pub(crate) fn volet(
    ui: &mut egui::Ui,
    p: &Palette,
    l: &dyn Logiciel,
    (briques, pied): (&[Brique], &[Brique]),
    hauteur_max: Option<f32>,
) -> Option<Choisi> {
    let mut choisi = None;
    let haut = ui.min_rect().top();
    if haut_du_volet(ui, p, l) {
        choisi = Some(Choisi::Parametres);
    }
    ui.add_space(mise_en_page::AVANT_BRIQUES);
    let milieu = |ui: &mut egui::Ui| geste(l, briques::dessiner(ui, p, briques));
    // La place du pied, mesurée à l'image d'avant (il ne change presque jamais de hauteur).
    let id_pied = ui.scope_id().with("hauteur du pied du volet");
    let place_pied = if pied.is_empty() {
        0.0
    } else {
        ui.ctx().data(|d| d.get_temp::<f32>(id_pied)).unwrap_or(0.0)
    };
    // Avec une hauteur maximale, les briques défilent dans ce qui reste sous l'état, au-dessus
    // du pied ; l'en-tête (avec la roue) et le pied restent en place. La barre se voit quand la
    // liste dépasse (celle d'egui n'apparaît qu'au survol, et ne se voit pas sur une capture).
    let rapide = match hauteur_max {
        None => milieu(ui),
        Some(max) => {
            let deja = ui.cursor().top() - haut;
            let reste = mise_en_page::place_des_briques(max, deja, place_pied);
            ui.scope(|ui| {
                // La poignée se voit même au repos (sans survol ni défilement).
                ui.spacing_mut().scroll.dormant_handle_opacity = mise_en_page::OPACITE_POIGNEE;
                egui::ScrollArea::vertical()
                    .id_salt("briques du volet")
                    .max_height(reste)
                    .auto_shrink([false, true])
                    .show(ui, milieu)
                    .inner
            })
            .inner
        }
    };
    if rapide.is_some() {
        choisi = rapide;
    }
    if !pied.is_empty() {
        let avant = ui.cursor().top() - ui.spacing().item_spacing.y;
        dessin::surface::separateur(ui, p);
        if let Some(c) = geste(l, briques::dessiner(ui, p, pied)) {
            choisi = Some(c);
        }
        let hauteur = ui.min_rect().bottom() - avant;
        ui.ctx().data_mut(|d| d.insert_temp(id_pied, hauteur));
    }
    choisi
}
