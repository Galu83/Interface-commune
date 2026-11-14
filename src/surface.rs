//! Le contenu du volet (clic gauche) et du petit menu (clic droit), dessiné par egui sans
//! rien savoir de la fenêtre (règles : interface Windows 11, points 6 et 10) :
//!
//! - **le volet**, sur le modèle du volet réseau de Windows 11 : en haut, l'état en gros et
//!   en couleur ; dessous, les réglages rapides du logiciel ; en bas, la version, discrète,
//!   et à droite une roue dentée vers les Paramètres. Pas de « Quitter ».
//! - **le petit menu**, au style des menus de Windows 11, dans l'ordre de [`crate::menu`].
//!
//! **Rien n'est coupé** : la largeur suit les textes mesurés, de la largeur de base à
//! [`LARGEUR_MAX`] ; au-delà, les phrases passent à la ligne.

use egui::RichText;

use crate::apparence::couleur;
use crate::dessin::{self, bouton_icone, ligne};
use crate::menu::{self, Choix};
use crate::polices::{semibold, titre};
use crate::theme::{Palette, Rgb, Surface, TAILLE_PETIT, TAILLE_TEXTE, TAILLE_TITRE_VOLET};
use crate::{GenreEtat, Logiciel, icones};

/// Largeur du volet, au moins, en points.
pub const LARGEUR: f32 = 320.0;
/// Largeur du petit menu, au moins, en points.
pub const LARGEUR_MENU: f32 = 240.0;
/// Largeur du volet et du petit menu, au plus, en points (le volet réseau de Windows 11 en
/// fait 360).
pub const LARGEUR_MAX: f32 = 400.0;
/// La marge autour du contenu, et celle, à gauche et à droite, du bloc de l'état (points).
const MARGE: f32 = 6.0;
const MARGE_ETAT: f32 = 14.0;
/// La ligne du bas : marge à gauche, et la roue dentée (points).
const MARGE_BAS: f32 = 10.0;
const ROUE: f32 = 36.0;

/// Ce que l'utilisateur a choisi dans le volet ou le petit menu (fait une fois la fenêtre
/// cachée).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choisi {
    /// Une ligne du petit menu.
    Menu(Choix),
    /// Une action des réglages rapides du volet (sa clé).
    Rapide(String),
    /// La roue dentée du volet.
    Parametres,
}

/// La couleur et l'icône d'un état.
pub fn couleur_etat(genre: GenreEtat, p: &Palette) -> (char, Rgb) {
    match genre {
        GenreEtat::Pret => (icones::COCHE, p.vert),
        GenreEtat::EnCours => (icones::MISE_A_JOUR, p.orange),
        GenreEtat::Attention => (icones::ATTENTION, p.orange),
        GenreEtat::Probleme => (icones::PROBLEME, p.rouge),
    }
}

fn version(l: &dyn Logiciel) -> String {
    let i = l.identite();
    format!("{} {}", i.nom, i.version)
}

fn lignes_du_menu(l: &dyn Logiciel) -> Vec<Option<menu::Ligne>> {
    menu::lignes(l.textes(), &l.actions_menu(), &l.lignes_en_plus())
}

/// La largeur de la surface (points) : ce qu'il faut pour que ses textes tiennent sur une
/// ligne, mesurés avec les polices du moment, entre la largeur minimale et [`LARGEUR_MAX`].
pub fn largeur(ui: &egui::Ui, s: Surface, l: &dyn Logiciel) -> f32 {
    let mesure = |t: &str, f: egui::FontId| {
        ui.painter()
            .layout_no_wrap(t.to_string(), f, egui::Color32::WHITE)
            .size()
            .x
    };
    let (min, besoin) = match s {
        Surface::Menu => (
            LARGEUR_MENU,
            lignes_du_menu(l)
                .iter()
                .flatten()
                .map(|li| {
                    dessin::largeur_ligne(mesure(&li.texte, egui::FontId::proportional(TAILLE_TEXTE)), None)
                })
                .fold(0.0, f32::max),
        ),
        Surface::Volet => {
            let e = l.etat();
            let cote = 2.0 * MARGE_ETAT;
            let mut b = cote + mesure(&e.titre, titre(TAILLE_TITRE_VOLET));
            b = b.max(
                cote + dessin::ETAT_ICONE
                    + ui.spacing().item_spacing.x
                    + mesure(&e.ligne, semibold(TAILLE_TEXTE)),
            );
            b = b.max(l.largeur_rapides(ui));
            let bas = MARGE_BAS + mesure(&version(l), egui::FontId::proportional(TAILLE_PETIT));
            b = b.max(bas + ui.spacing().item_spacing.x + ROUE);
            (LARGEUR, b)
        }
    };
    (besoin + 2.0 * MARGE).ceil().clamp(min, LARGEUR_MAX)
}

/// Dessine la surface à sa largeur, avec sa marge ; rend ce qui a été choisi et la taille
/// voulue pour la fenêtre (points).
pub fn dessiner(
    ui: &mut egui::Ui,
    p: &Palette,
    s: Surface,
    l: &dyn Logiciel,
) -> (Option<Choisi>, egui::Vec2) {
    let largeur = largeur(ui, s, l);
    let r = egui::Frame::new()
        .inner_margin(egui::Margin::same(MARGE as i8))
        .show(ui, |ui| {
            ui.set_width(largeur - 2.0 * MARGE);
            match s {
                Surface::Volet => dessiner_volet(ui, p, l),
                Surface::Menu => dessiner_menu(ui, p, l),
            }
        });
    (r.inner, egui::vec2(largeur, r.response.rect.height().ceil()))
}

/// Un trait de séparation sur toute la largeur.
fn separateur(ui: &mut egui::Ui, p: &Palette) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 9.0), egui::Sense::hover());
    ui.painter().hline(
        r.x_range().shrink(4.0),
        r.center().y,
        egui::Stroke::new(1.0, couleur(p.separateur)),
    );
}

/// Le volet : l'état, les réglages rapides, la version et la roue dentée.
pub fn dessiner_volet(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let mut choisi = None;
    let e = l.etat();
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: MARGE_ETAT as i8,
            right: MARGE_ETAT as i8,
            top: 10,
            bottom: 6,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(egui::Label::new(RichText::new(&e.titre).font(titre(TAILLE_TITRE_VOLET))).truncate());
            let (g, c) = couleur_etat(e.genre, p);
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
    ui.add_space(2.0);
    if let Some(cle) = l.dessiner_rapides(ui, p) {
        choisi = Some(Choisi::Rapide(cle));
    }
    ui.add_space(2.0);
    separateur(ui, p);
    ui.horizontal(|ui| {
        ui.add_space(MARGE_BAS);
        ui.label(
            RichText::new(version(l))
                .color(couleur(p.texte_secondaire))
                .size(TAILLE_PETIT),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if bouton_icone(ui, p, icones::REGLAGES, l.textes().parametres).clicked() {
                choisi = Some(Choisi::Parametres);
            }
        });
    });
    choisi
}

/// Le petit menu, dans l'ordre des règles.
pub fn dessiner_menu(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let mut choisi = None;
    for li in lignes_du_menu(l) {
        match li {
            None => separateur(ui, p),
            Some(li) => {
                if ligne(ui, p, li.icone, p.texte, &li.texte, None).clicked() {
                    choisi = Some(Choisi::Menu(li.quoi));
                }
            }
        }
    }
    choisi
}
