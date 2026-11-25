//! Le contenu du volet (clic gauche) et du petit menu (clic droit), dessiné par egui sans
//! rien savoir de la fenêtre (règles : interface Windows 11, points 6 et 10) :
//!
//! - **le volet**, sur le modèle du volet de OneDrive : en haut, le gros titre (le nom du
//!   logiciel) avec sa version, petite et grise, à sa droite, et à l'extrême droite de la même
//!   ligne la roue dentée des Paramètres ; puis l'état en couleur ; dessous, les réglages
//!   rapides du logiciel. Pas de ligne du bas, pas de « Quitter ».
//! - **le petit menu**, au style des menus de Windows 11, dans l'ordre de [`crate::menu`].
//!
//! **Rien n'est coupé** : la largeur suit les textes mesurés, de la largeur de base à
//! [`LARGEUR_MAX`] ; au-delà, les phrases passent à la ligne. Le volet peut avoir une hauteur
//! maximale ([`dessiner_borne`]) : l'en-tête (titre, roue) et l'état restent en place, les
//! briques défilent.
//!
//! Bloc G de la refonte : écrit ici (`src/dessin/surface.rs`, le volet dans
//! `src/dessin/volet.rs`), repris au chemin d'aujourd'hui `interface_commune::surface`. La
//! mise en page (largeur, hauteur permise, en-tête, place des briques) se décide dans le
//! cœur (`ic_coeur::placement::surface`), l'ordre du menu aussi (`ic_coeur::gestes::menu`) ;
//! ce module mesure les textes et dessine.

use ic_coeur::placement::surface as mise_en_page;

use crate::apparence::couleur;
use crate::briques::{self, Brique};
use crate::dessin::{self, ligne_menu};
use crate::menu::{self, Choix};
use crate::mesures::{MENU_ECART, SEPARATEUR};
use crate::polices::{semibold, titre};
use crate::theme::{Palette, Rgb, Surface, TAILLE_PETIT, TAILLE_TEXTE, TAILLE_TITRE_VOLET};
use crate::{GenreEtat, Logiciel};

pub use crate::dessin::volet::dessiner_volet;
/// Largeur du volet et du petit menu : au moins, au plus (points ; [`crate::mesures`]).
pub use crate::mesures::{LARGEUR_MAX, MENU_LARGEUR as LARGEUR_MENU, VOLET_LARGEUR as LARGEUR};

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

/// La couleur et l'icône d'un état (décidées dans le cœur : `GenreEtat::icone_et_couleur`).
pub fn couleur_etat(genre: GenreEtat, p: &Palette) -> (char, Rgb) {
    genre.icone_et_couleur(p)
}

/// La version du logiciel (« x.y.z »), montrée à droite du gros titre.
pub(crate) fn version(l: &dyn Logiciel) -> String {
    l.identite().version.clone()
}

fn lignes_du_menu(l: &dyn Logiciel) -> Vec<Option<menu::Ligne>> {
    // « Ouvrir <logiciel> » en tête, seulement s'il a une fenêtre principale.
    let nom = l.fenetre_principale().map(|_| l.identite().nom.as_str());
    menu::lignes_avec(l.textes(), nom, &l.actions_menu(), &l.lignes_en_plus())
}

/// Les briques de la surface à cette image : la liste, puis le pied fixe (aucune pour le
/// petit menu).
fn briques_de(ui: &egui::Ui, s: Surface, l: &dyn Logiciel) -> (Vec<Brique>, Vec<Brique>) {
    match s {
        Surface::Volet => (l.briques_avec_contexte(ui.ctx()), l.briques_du_pied()),
        Surface::Menu => (Vec::new(), Vec::new()),
    }
}

/// La largeur d'un texte sur une ligne, avec cette police (points).
fn mesure(ui: &egui::Ui, t: &str, f: egui::FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(t.to_string(), f, egui::Color32::WHITE)
        .size()
        .x
}

/// La largeur de la surface (points) : ce qu'il faut pour que ses textes tiennent sur une
/// ligne, mesurés avec les polices du moment, entre la largeur minimale et [`LARGEUR_MAX`].
pub fn largeur(ui: &egui::Ui, s: Surface, l: &dyn Logiciel) -> f32 {
    let (briques, pied) = briques_de(ui, s, l);
    largeur_avec(ui, s, l, &briques, &pied)
}

/// [`largeur`], avec les briques déjà demandées au logiciel pour cette image.
fn largeur_avec(ui: &egui::Ui, s: Surface, l: &dyn Logiciel, briques: &[Brique], pied: &[Brique]) -> f32 {
    let besoin = match s {
        Surface::Menu => mise_en_page::besoin_du_menu(lignes_du_menu(l).iter().flatten().map(|li| {
            let police = if li.en_gras() {
                semibold(TAILLE_TEXTE)
            } else {
                egui::FontId::proportional(TAILLE_TEXTE)
            };
            dessin::largeur_ligne(mesure(ui, &li.texte, police), None)
        })),
        Surface::Volet => {
            let e = l.etat();
            let v = version(l);
            let version = mise_en_page::version_montree(&v)
                .map(|v| mesure(ui, v, egui::FontId::proportional(TAILLE_PETIT)));
            mise_en_page::besoin_du_volet(&mise_en_page::MesuresDuVolet {
                titre: mesure(ui, &e.titre, titre(TAILLE_TITRE_VOLET)),
                version,
                icone_etat: dessin::ETAT_ICONE,
                ecart_etat: ui.spacing().item_spacing.x,
                ligne_etat: mesure(ui, &e.ligne, semibold(TAILLE_TEXTE)),
                briques: briques::largeur(ui, briques),
                pied: briques::largeur(ui, pied),
            })
        }
    };
    mise_en_page::largeur(s, besoin)
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
/// au-delà, l'en-tête (titre, roue) et l'état restent en place et les briques défilent.
/// `None`, ou le petit menu : aussi haut que le contenu. Les briques ne sont demandées au
/// logiciel qu'une fois par image.
pub fn dessiner_borne(
    ui: &mut egui::Ui,
    p: &Palette,
    s: Surface,
    l: &dyn Logiciel,
    hauteur_max: Option<f32>,
) -> (Option<Choisi>, egui::Vec2) {
    let (briques, pied) = briques_de(ui, s, l);
    let largeur = largeur_avec(ui, s, l, &briques, &pied);
    let m = mise_en_page::marge(s);
    let hauteur_max = mise_en_page::hauteur_max(s, hauteur_max);
    let contenu = |ui: &mut egui::Ui| {
        egui::Frame::new()
            .inner_margin(egui::Margin::same(m as i8))
            .show(ui, |ui| {
                ui.set_width(largeur - 2.0 * m);
                match s {
                    Surface::Volet => dessin::volet::volet(
                        ui,
                        p,
                        l,
                        (&briques, &pied),
                        mise_en_page::hauteur_max_du_contenu(s, hauteur_max),
                    ),
                    Surface::Menu => dessiner_menu(ui, p, l),
                }
            })
    };
    let r = match hauteur_max {
        // Avec une borne, le défilement ne dépasse jamais la place que lui laisse son parent :
        // dessiné dans la fenêtre telle qu'elle est, le volet ne pourrait plus grandir quand
        // son contenu grandit (la hauteur mesurée resterait celle de la fenêtre). On le dessine
        // donc dans une zone qui a toute la hauteur permise, quelle que soit la fenêtre.
        Some(max) => {
            let zone = egui::Rect::from_min_size(
                ui.cursor().min,
                egui::vec2(largeur.max(ui.available_width()), max),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(zone), contenu)
                .inner
        }
        None => contenu(ui),
    };
    let hauteur = mise_en_page::hauteur_rendue(r.response.rect.height(), hauteur_max);
    (r.inner, egui::vec2(largeur, hauteur))
}

/// Un trait de séparation sur toute la largeur.
pub(crate) fn separateur(ui: &mut egui::Ui, p: &Palette) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), SEPARATEUR), egui::Sense::hover());
    ui.painter().hline(
        r.x_range().shrink(mise_en_page::RETRAIT_SEPARATEUR),
        r.center().y,
        egui::Stroke::new(1.0, couleur(p.separateur)),
    );
}

/// Le petit menu, dans l'ordre des règles.
pub fn dessiner_menu(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let mut choisi = None;
    ui.spacing_mut().item_spacing.y = MENU_ECART;
    for li in lignes_du_menu(l) {
        match li {
            None => separateur(ui, p),
            Some(li) => {
                let r = if li.en_gras() {
                    dessin::ligne_menu_grasse(ui, p, li.icone, &li.texte)
                } else {
                    ligne_menu(ui, p, li.icone, &li.texte)
                };
                if r.clicked() {
                    choisi = Some(Choisi::Menu(li.quoi));
                }
            }
        }
    }
    choisi
}
