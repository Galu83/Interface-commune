//! Le contenu du volet (clic gauche) et du petit menu (clic droit), dessiné par egui sans
//! rien savoir de la fenêtre (règles : interface Windows 11, points 6 et 10) :
//!
//! - **le volet**, sur le modèle du volet réseau de Windows 11 : en haut, l'état en gros et
//!   en couleur ; dessous, les réglages rapides du logiciel ; en bas, la version, discrète,
//!   et à droite une roue dentée vers les Paramètres. Pas de « Quitter ».
//! - **le petit menu**, au style des menus de Windows 11, dans l'ordre de [`crate::menu`].
//!
//! **Rien n'est coupé** : la largeur suit les textes mesurés, de la largeur de base à
//! [`LARGEUR_MAX`] ; au-delà, les phrases passent à la ligne. Le volet peut avoir une hauteur
//! maximale ([`dessiner_borne`]) : l'état et le bas restent en place, les briques défilent.

use egui::RichText;

use crate::apparence::couleur;
use crate::briques::{self, Brique, Geste};
use crate::dessin::{self, bouton_icone, ligne_menu};
use crate::menu::{self, Choix};
use crate::mesures::{
    MENU_ECART, MENU_MARGE, SEPARATEUR, VOLET_MARGE, VOLET_MARGE_BAS, VOLET_MARGE_ETAT, VOLET_ROUE,
};
use crate::polices::{semibold, titre};
use crate::theme::{Palette, Rgb, Surface, TAILLE_PETIT, TAILLE_TEXTE, TAILLE_TITRE_VOLET};
use crate::{GenreEtat, Logiciel, icones};

/// Largeur du volet et du petit menu : au moins, au plus (points ; [`crate::mesures`]).
pub use crate::mesures::{LARGEUR_MAX, MENU_LARGEUR as LARGEUR_MENU, VOLET_LARGEUR as LARGEUR};
const MARGE_ETAT: f32 = VOLET_MARGE_ETAT;
const MARGE_BAS: f32 = VOLET_MARGE_BAS;
const ROUE: f32 = VOLET_ROUE;

/// La marge autour du contenu de la surface.
fn marge(s: Surface) -> f32 {
    match s {
        Surface::Volet => VOLET_MARGE,
        Surface::Menu => MENU_MARGE,
    }
}

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

/// Les briques de la surface à cette image (aucune pour le petit menu).
fn briques_de(ui: &egui::Ui, s: Surface, l: &dyn Logiciel) -> Vec<Brique> {
    match s {
        Surface::Volet => l.briques_avec_contexte(ui.ctx()),
        Surface::Menu => Vec::new(),
    }
}

/// La largeur de la surface (points) : ce qu'il faut pour que ses textes tiennent sur une
/// ligne, mesurés avec les polices du moment, entre la largeur minimale et [`LARGEUR_MAX`].
pub fn largeur(ui: &egui::Ui, s: Surface, l: &dyn Logiciel) -> f32 {
    largeur_avec(ui, s, l, &briques_de(ui, s, l))
}

/// [`largeur`], avec les briques déjà demandées au logiciel pour cette image.
fn largeur_avec(ui: &egui::Ui, s: Surface, l: &dyn Logiciel, briques: &[Brique]) -> f32 {
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
            b = b.max(briques::largeur(ui, briques));
            let bas = MARGE_BAS + mesure(&version(l), egui::FontId::proportional(TAILLE_PETIT));
            b = b.max(bas + ui.spacing().item_spacing.x + ROUE);
            (LARGEUR, b)
        }
    };
    (besoin + 2.0 * marge(s)).ceil().clamp(min, LARGEUR_MAX)
}

/// Dessine la surface à sa largeur, avec sa marge ; rend ce qui a été choisi et la taille
/// voulue pour la fenêtre (points).
pub fn dessiner(
    ui: &mut egui::Ui,
    p: &Palette,
    s: Surface,
    l: &dyn Logiciel,
) -> (Option<Choisi>, egui::Vec2) {
    dessiner_borne(ui, p, s, l, None)
}

/// Comme [`dessiner`], sans que le volet dépasse `hauteur_max` points (marges comprises) :
/// au-delà, l'état en haut et la version en bas restent en place et les briques défilent.
/// `None`, ou le petit menu : aussi haut que le contenu. Les briques ne sont demandées au
/// logiciel qu'une fois par image.
pub fn dessiner_borne(
    ui: &mut egui::Ui,
    p: &Palette,
    s: Surface,
    l: &dyn Logiciel,
    hauteur_max: Option<f32>,
) -> (Option<Choisi>, egui::Vec2) {
    let briques = briques_de(ui, s, l);
    let largeur = largeur_avec(ui, s, l, &briques);
    let m = marge(s);
    let hauteur_max = match s {
        Surface::Volet => hauteur_max.filter(|h| h.is_finite()).map(|h| h.max(1.0)),
        Surface::Menu => None,
    };
    let r = egui::Frame::new()
        .inner_margin(egui::Margin::same(m as i8))
        .show(ui, |ui| {
            ui.set_width(largeur - 2.0 * m);
            match s {
                Surface::Volet => volet(ui, p, l, &briques, hauteur_max.map(|h| h - 2.0 * m)),
                Surface::Menu => dessiner_menu(ui, p, l),
            }
        });
    let mut hauteur = r.response.rect.height().ceil();
    if let Some(max) = hauteur_max {
        hauteur = hauteur.min(max.ceil());
    }
    (r.inner, egui::vec2(largeur, hauteur))
}

/// Un trait de séparation sur toute la largeur.
fn separateur(ui: &mut egui::Ui, p: &Palette) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), SEPARATEUR), egui::Sense::hover());
    ui.painter().hline(
        r.x_range().shrink(4.0),
        r.center().y,
        egui::Stroke::new(1.0, couleur(p.separateur)),
    );
}

/// Le volet : l'état, les réglages rapides, la version et la roue dentée.
pub fn dessiner_volet(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let briques = l.briques_avec_contexte(ui.ctx());
    volet(ui, p, l, &briques, None)
}

/// La hauteur du bas du volet (séparateur, version et roue dentée) avant sa première mesure.
const PIED_ESTIME: f32 = SEPARATEUR + ROUE + 12.0;

/// Le volet, avec ses briques ; `hauteur_max` : la hauteur de son contenu à ne pas dépasser
/// (points, sans les marges), les briques défilant au-delà.
fn volet(
    ui: &mut egui::Ui,
    p: &Palette,
    l: &dyn Logiciel,
    briques: &[Brique],
    hauteur_max: Option<f32>,
) -> Option<Choisi> {
    let mut choisi = None;
    let haut = ui.min_rect().top();
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
    let milieu = |ui: &mut egui::Ui| {
        let mut choisi = None;
        match briques::dessiner(ui, p, briques) {
            Some(Geste::Action(cle)) => choisi = Some(Choisi::Rapide(cle)),
            Some(Geste::Changer(cle, v)) => l.changer(&cle, v),
            None => {}
        }
        if let Some(cle) = l.dessiner_rapides(ui, p) {
            choisi = Some(Choisi::Rapide(cle));
        }
        choisi
    };
    // Avec une hauteur maximale, les briques défilent dans ce qui reste entre l'état et le
    // bas (mesuré à l'image précédente).
    let id_pied = egui::Id::unique("hauteur du bas du volet");
    let rapide = match hauteur_max {
        None => milieu(ui),
        Some(max) => {
            let pied = ui.data(|d| d.get_temp::<f32>(id_pied)).unwrap_or(PIED_ESTIME);
            let deja = ui.cursor().top() - haut;
            let reste = (max - deja - pied).max(TAILLE_TEXTE * 2.0);
            egui::ScrollArea::vertical()
                .id_salt("briques du volet")
                .max_height(reste)
                .auto_shrink([false, true])
                .show(ui, milieu)
                .inner
        }
    };
    if rapide.is_some() {
        choisi = rapide;
    }
    let bas_du_milieu = ui.min_rect().bottom();
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
    if hauteur_max.is_some() {
        let pied = ui.min_rect().bottom() - bas_du_milieu;
        ui.data_mut(|d| d.insert_temp(id_pied, pied));
    }
    choisi
}

/// Le petit menu, dans l'ordre des règles.
pub fn dessiner_menu(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let mut choisi = None;
    ui.spacing_mut().item_spacing.y = MENU_ECART;
    for li in lignes_du_menu(l) {
        match li {
            None => separateur(ui, p),
            Some(li) => {
                if ligne_menu(ui, p, li.icone, &li.texte).clicked() {
                    choisi = Some(Choisi::Menu(li.quoi));
                }
            }
        }
    }
    choisi
}
