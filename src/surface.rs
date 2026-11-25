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

use egui::RichText;

use crate::apparence::couleur;
use crate::briques::{self, Brique, Geste};
use crate::dessin::{self, bouton_icone, ligne_menu};
use crate::menu::{self, Choix};
use crate::mesures::{
    MENU_ECART, MENU_MARGE, SEPARATEUR, VOLET_ENTETE_DROITE, VOLET_ENTETE_ECART, VOLET_MARGE,
    VOLET_MARGE_ETAT, VOLET_ROUE,
};
use crate::polices::{semibold, titre};
use crate::theme::{Palette, Rgb, Surface, TAILLE_PETIT, TAILLE_TEXTE, TAILLE_TITRE_VOLET};
use crate::{GenreEtat, Logiciel, icones};

/// Largeur du volet et du petit menu : au moins, au plus (points ; [`crate::mesures`]).
pub use crate::mesures::{LARGEUR_MAX, MENU_LARGEUR as LARGEUR_MENU, VOLET_LARGEUR as LARGEUR};
const MARGE_ETAT: f32 = VOLET_MARGE_ETAT;
/// L'écart entre le gros titre et sa version (points).
const ECART_VERSION: f32 = 6.0;
const ROUE: f32 = VOLET_ROUE;
/// La marge à droite de l'en-tête, et l'écart minimal entre le titre (ou la version) et la
/// roue (points).
const ENTETE_DROITE: f32 = VOLET_ENTETE_DROITE;
const ENTETE_ECART: f32 = VOLET_ENTETE_ECART;

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

/// La version du logiciel (« x.y.z »), montrée à droite du gros titre.
fn version(l: &dyn Logiciel) -> String {
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

/// La largeur de la surface (points) : ce qu'il faut pour que ses textes tiennent sur une
/// ligne, mesurés avec les polices du moment, entre la largeur minimale et [`LARGEUR_MAX`].
pub fn largeur(ui: &egui::Ui, s: Surface, l: &dyn Logiciel) -> f32 {
    let (briques, pied) = briques_de(ui, s, l);
    largeur_avec(ui, s, l, &briques, &pied)
}

/// [`largeur`], avec les briques déjà demandées au logiciel pour cette image.
fn largeur_avec(ui: &egui::Ui, s: Surface, l: &dyn Logiciel, briques: &[Brique], pied: &[Brique]) -> f32 {
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
                    let police = if li.quoi == Choix::Ouvrir {
                        semibold(TAILLE_TEXTE)
                    } else {
                        egui::FontId::proportional(TAILLE_TEXTE)
                    };
                    dessin::largeur_ligne(mesure(&li.texte, police), None)
                })
                .fold(0.0, f32::max),
        ),
        Surface::Volet => {
            let e = l.etat();
            let cote = 2.0 * MARGE_ETAT;
            let v = version(l);
            // L'en-tête : le titre, sa version, un écart, puis la roue à l'extrême droite.
            let mut titre_v = mesure(&e.titre, titre(TAILLE_TITRE_VOLET));
            if !v.is_empty() {
                titre_v += ECART_VERSION + mesure(&v, egui::FontId::proportional(TAILLE_PETIT));
            }
            let mut b = MARGE_ETAT + titre_v + ENTETE_ECART + ROUE + ENTETE_DROITE;
            b = b.max(
                cote + dessin::ETAT_ICONE
                    + ui.spacing().item_spacing.x
                    + mesure(&e.ligne, semibold(TAILLE_TEXTE)),
            );
            b = b.max(briques::largeur(ui, briques));
            b = b.max(briques::largeur(ui, pied));
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
    let m = marge(s);
    let hauteur_max = match s {
        Surface::Volet => hauteur_max.filter(|h| h.is_finite()).map(|h| h.max(1.0)),
        Surface::Menu => None,
    };
    let dessin = |ui: &mut egui::Ui| {
        egui::Frame::new()
            .inner_margin(egui::Margin::same(m as i8))
            .show(ui, |ui| {
                ui.set_width(largeur - 2.0 * m);
                match s {
                    Surface::Volet => volet(ui, p, l, (&briques, &pied), hauteur_max.map(|h| h - 2.0 * m)),
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
            ui.scope_builder(egui::UiBuilder::new().max_rect(zone), dessin)
                .inner
        }
        None => dessin(ui),
    };
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

/// Le volet : le titre, sa version et la roue dentée en haut, l'état, les réglages rapides.
pub fn dessiner_volet(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) -> Option<Choisi> {
    let briques = l.briques_avec_contexte(ui.ctx());
    let pied = l.briques_du_pied();
    volet(ui, p, l, (&briques, &pied), None)
}

/// La distance du haut d'un texte à sa ligne de base (points).
fn ligne_de_base(g: &egui::Galley) -> f32 {
    g.rows
        .first()
        .and_then(|r| r.glyphs.first().map(|c| r.pos.y + c.pos.y))
        .unwrap_or_else(|| g.size().y)
}

/// Le gros titre, et à sa droite la version, petite et grise, sur la même ligne de base. Le
/// titre se coupe par « … » si la place manque ; la version reste entière.
fn titre_et_version(ui: &mut egui::Ui, p: &Palette, titre_texte: &str, version: &str) {
    let police = titre(TAILLE_TITRE_VOLET);
    let petite = egui::FontId::proportional(TAILLE_PETIT);
    let gris = couleur(p.texte_secondaire);
    let v = (!version.is_empty()).then(|| ui.painter().layout_no_wrap(version.to_string(), petite, gris));
    let reserve = v.as_ref().map_or(0.0, |v| ECART_VERSION + v.size().x);
    let place = ui.available_width();
    let haut = ui
        .allocate_ui(egui::vec2((place - reserve).max(1.0), 0.0), |ui| {
            ui.add(egui::Label::new(RichText::new(titre_texte).font(police.clone())).truncate())
        })
        .inner;
    let Some(v) = v else { return };
    let g = ui.painter().layout_no_wrap(titre_texte.to_string(), police, gris);
    let x = (haut.rect.left() + g.size().x.min(haut.rect.width()) + ECART_VERSION)
        .min(haut.rect.left() + place - v.size().x);
    let y = haut.rect.top() + ligne_de_base(&g) - ligne_de_base(&v);
    ui.painter().galley(egui::pos2(x, y), v, gris);
}

/// L'en-tête du volet : à gauche le gros titre et sa version ([`titre_et_version`]), à droite
/// la roue dentée des Paramètres, carrée (36 × 36), centrée sur la hauteur de la ligne. Le
/// titre ne passe jamais sous la roue : il garde au moins [`ENTETE_ECART`] avec elle et se
/// coupe par « … ». Rend `true` si la roue est cliquée.
fn entete(ui: &mut egui::Ui, p: &Palette, titre_texte: &str, version: &str, nom_roue: &str) -> bool {
    let largeur = ui.available_width();
    let (ligne, _) = ui.allocate_exact_size(egui::vec2(largeur, ROUE), egui::Sense::hover());
    // La roue : collée à droite, avant le titre (sa place dans l'ordre du clavier).
    let rect_roue = egui::Rect::from_min_size(
        egui::pos2(ligne.right() - ROUE, ligne.top()),
        egui::vec2(ROUE, ROUE),
    );
    let clic = ui
        .scope_builder(egui::UiBuilder::new().max_rect(rect_roue), |ui| {
            bouton_icone(ui, p, icones::REGLAGES, nom_roue).clicked()
        })
        .inner;
    // Le titre : à gauche, centré sur la hauteur de la roue, sur le reste de la place.
    let police = titre(TAILLE_TITRE_VOLET);
    let hauteur_titre = ui
        .painter()
        .layout_no_wrap(titre_texte.to_string(), police, egui::Color32::WHITE)
        .size()
        .y;
    let haut = ligne.top() + ((ROUE - hauteur_titre) / 2.0).max(0.0);
    let place = (largeur - ROUE - ENTETE_ECART).max(1.0);
    let rect_titre =
        egui::Rect::from_min_size(egui::pos2(ligne.left(), haut), egui::vec2(place, hauteur_titre));
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect_titre), |ui| {
        titre_et_version(ui, p, titre_texte, version);
    });
    clic
}

/// Le volet, avec ses briques et son pied fixe (`briques`, `pied`) ; `hauteur_max` : la hauteur
/// de son contenu à ne pas dépasser (points, sans les marges). Au-delà, seule la liste des
/// briques défile, dans la place qui reste entre l'état et le pied ; le pied reste en bas.
fn volet(
    ui: &mut egui::Ui,
    p: &Palette,
    l: &dyn Logiciel,
    (briques, pied): (&[Brique], &[Brique]),
    hauteur_max: Option<f32>,
) -> Option<Choisi> {
    let mut choisi = None;
    let haut = ui.min_rect().top();
    let e = l.etat();
    // L'en-tête : le titre et sa version à gauche, la roue des Paramètres à l'extrême droite,
    // sur la même ligne (la roue d'abord pour le clavier : elle est la première en haut).
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: MARGE_ETAT as i8,
            right: ENTETE_DROITE as i8,
            top: 4,
            bottom: 0,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if entete(ui, p, &e.titre, &version(l), l.textes().parametres) {
                choisi = Some(Choisi::Parametres);
            }
        });
    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: MARGE_ETAT as i8,
            right: MARGE_ETAT as i8,
            top: 0,
            bottom: 6,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
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
        choisi
    };
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
            let reste = (max - deja - place_pied).max(TAILLE_TEXTE * 2.0);
            ui.scope(|ui| {
                // La poignée se voit même au repos (sans survol ni défilement) : sinon, rien
                // n'indique que la liste continue.
                ui.spacing_mut().scroll.dormant_handle_opacity = 0.55;
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
        separateur(ui, p);
        match briques::dessiner(ui, p, pied) {
            Some(Geste::Action(cle)) => choisi = Some(Choisi::Rapide(cle)),
            Some(Geste::Changer(cle, v)) => l.changer(&cle, v),
            None => {}
        }
        let hauteur = ui.min_rect().bottom() - avant;
        ui.ctx().data_mut(|d| d.insert_temp(id_pied, hauteur));
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
                let r = if li.quoi == Choix::Ouvrir {
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
