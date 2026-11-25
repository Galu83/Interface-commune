//! Deux briques du volet pour une rangée d'appareils et des choix illustrés : les pastilles
//! ([`crate::briques::Brique::Pastilles`]) et le choix illustré
//! ([`crate::briques::Brique::ChoixIllustre`]). Leurs types et leur dessin vivent ici ;
//! `briques.rs` les appelle comme les autres briques. Mesures : [`crate::mesures`].

use egui::{Align2, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::briques::Geste;
use crate::dessin::couleur::couleur;
use crate::dessin::{self, icone};
use crate::icones;
use crate::mesures::{
    ILLUSTRE_BARRE, ILLUSTRE_CASE_ECART, ILLUSTRE_CASE_H, ILLUSTRE_CASE_L, ILLUSTRE_CASES_MAX,
    ILLUSTRE_LARGEUR, ILLUSTRE_LIGNE, LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_MARGE_DROITE,
    PASTILLE_HAUTEUR, PASTILLE_LARGEUR, VOLET_LIGNE, VOLET_SECTION,
};
use crate::place::{vers_egui, vers_le_coeur};
use crate::polices::semibold;
use crate::theme::{Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE};
use crate::touches;

/// La lettre lisible sur un fond, et les 10 couleurs des pastilles : écrites dans le cœur
/// (`ic_coeur::theme`, bloc C), reprises ici à leurs chemins.
pub use ic_coeur::theme::{couleur_lettre, couleurs_pastilles};

/// Le fond d'une case d'écran en veille (sombre dans les deux modes : un écran éteint).
const FOND_VEILLE: Rgb = Rgb(0x1E, 0x1E, 0x1E);
const BORD_VEILLE: Rgb = Rgb(0x9A, 0x9A, 0x9A);
const LUNE_VEILLE: Rgb = Rgb(0xD0, 0xD0, 0xD0);
/// Le bord en tirets clair d'une case d'écran débranché (son opacité : [`OPACITE_DEBRANCHE`]).
const BORD_DEBRANCHE: Rgb = Rgb(0xC8, 0xC8, 0xC8);
/// Marge à gauche d'une pastille, et espace entre son carré et son nom.
const PASTILLE_MARGE: f32 = 8.0;
const PASTILLE_ESPACE: f32 = 8.0;
/// Marge à gauche de l'illustration d'un choix, et espace entre elle et le texte.
const ILLUSTRE_MARGE: f32 = 10.0;
const ILLUSTRE_ESPACE: f32 = 12.0;

/// Un appareil de la rangée : un carré coloré avec sa lettre, son nom.
#[derive(Debug, Clone, PartialEq)]
pub struct Pastille {
    /// Avec une clé, un clic ferme le volet et la rend (`Reglages::action`).
    pub cle: Option<String>,
    /// La lettre du carré : aussi le repli quand `logo` manque dans la police d'icônes.
    pub lettre: char,
    /// Un glyphe de Segoe Fluent Icons dessiné dans le carré **à la place de la lettre**
    /// (le logo de l'appareil). `None` : la lettre, comme avant. Si la police d'icônes n'a pas
    /// ce glyphe (Windows 10 : Segoe MDL2 Assets), la lettre s'affiche : jamais un carré vide.
    pub logo: Option<char>,
    pub couleur: Rgb,
    pub nom: String,
    /// Une mention en secondaire après le nom (« ce PC »).
    pub mention: Option<String>,
    /// Un petit glyphe de Segoe Fluent Icons après la mention (une souris).
    pub glyphe: Option<char>,
    /// Absent : carré hachuré, texte grisé.
    pub absent: bool,
}

/// Une case de l'illustration d'un choix : un écran.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Case {
    pub lettre: Option<char>,
    /// Un glyphe de Segoe Fluent Icons à la place de la lettre (le logo du PC). `None` par
    /// défaut : la lettre. Absent de la police d'icônes : la lettre revient.
    pub logo: Option<char>,
    pub couleur: Option<Rgb>,
    /// Écran en veille : case sombre, lune, bordure en tirets.
    pub veille: bool,
    /// Écran débranché : il garde sa place, mais en pâle (opacité 0,4), bord en tirets de
    /// 1 point, lettre ou logo gardé ; s'il est aussi en veille, débranché l'emporte.
    pub debranche: bool,
    /// Où est cet écran sur le bureau : `[x, y, largeur, hauteur]` en pixels, dans le repère du
    /// bureau (des valeurs quelconques, négatives permises). Quand toutes les cases d'un choix
    /// ont une place valide (largeur et hauteur > 0), l'illustration est le vrai plan des
    /// écrans ([`plan_des_cases`]) ; sinon, la rangée de cases égales.
    pub place: Option<[i32; 4]>,
}

/// La plus petite hauteur (en points) d'une case du plan qui porte encore sa lettre ou sa
/// lune ; en dessous, la case n'est que sa couleur.
const PLAN_CASE_H_MIN: f32 = 12.0;
/// Le vide entre le bord de la zone et le plan, en haut et en bas.
const PLAN_MARGE_V: f32 = 4.0;

/// Le plan des écrans : chaque place (`[x, y, largeur, hauteur]` en pixels du bureau, valeurs
/// quelconques) devient un rectangle dans `zone`, proportions gardées, centré, chaque case
/// réduite de 1 point de chaque côté. Calcul pur, dans le cœur
/// ([`ic_coeur::placement::plan::plan_des_cases`], bloc D) et testé là.
pub fn plan_des_cases(places: &[[i32; 4]], zone: Rect) -> Vec<Rect> {
    ic_coeur::placement::plan::plan_des_cases(places, vers_le_coeur(zone))
        .into_iter()
        .map(vers_egui)
        .collect()
}

/// Les places de toutes les cases, si TOUTES sont valides (largeur et hauteur > 0) ; sinon rien.
fn places_valides(cases: &[Case]) -> Option<Vec<[i32; 4]>> {
    if cases.is_empty() {
        return None;
    }
    cases
        .iter()
        .map(|c| c.place.filter(|p| p[2] > 0 && p[3] > 0))
        .collect()
}

// Ce qu'un carré montre, le logo ou la lettre : dans le cœur (bloc F), repris ici à son chemin
// d'aujourd'hui.
pub use ic_coeur::theme::{Marque, marque};

// L'état d'une case d'écran (débranché l'emporte sur la veille) et l'opacité d'un écran
// débranché : dans le cœur aussi.
use ic_coeur::theme::{EtatCase, OPACITE_DEBRANCHE, etat_case};

// ---- Les pastilles --------------------------------------------------------------------

fn mesure(ui: &Ui, t: &str, taille: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(t.to_string(), FontId::proportional(taille), egui::Color32::WHITE)
        .size()
        .x
}

fn texte_mention(m: &str) -> String {
    format!("· {m}")
}

/// La largeur d'une pastille (carré, nom, mention, glyphe).
fn largeur_pastille(ui: &Ui, pa: &Pastille) -> f32 {
    PASTILLE_MARGE
        + PASTILLE_LARGEUR
        + PASTILLE_ESPACE
        + mesure(ui, &pa.nom, TAILLE_TEXTE)
        + pa.mention
            .as_deref()
            .map_or(0.0, |m| 4.0 + mesure(ui, &texte_mention(m), TAILLE_PETIT))
        + pa.glyphe.map_or(0.0, |_| 6.0 + LIGNE_ICONE)
        + LIGNE_MARGE_DROITE
}

/// La largeur qu'il faut à la rangée : deux colonnes égales, de la plus large pastille.
pub fn largeur_pastilles(ui: &Ui, titre: Option<&str>, pastilles: &[Pastille]) -> f32 {
    let colonne = pastilles
        .iter()
        .map(|pa| largeur_pastille(ui, pa))
        .fold(0.0, f32::max);
    let colonnes = pastilles.len().clamp(1, 2) as f32;
    let titre = titre.map_or(0.0, |t| 2.0 * LIGNE_MARGE_DROITE + mesure(ui, t, TAILLE_PETIT));
    (colonne * colonnes).max(titre)
}

/// Le nombre de rangées de la grille (2 colonnes).
fn rangees(n: usize) -> usize {
    n.div_ceil(2)
}

/// Le titre d'une section (aussi celui des pastilles).
pub fn titre_section(ui: &mut Ui, p: &Palette, t: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), VOLET_SECTION), Sense::hover());
    ui.painter().text(
        pos2(rect.left() + LIGNE_MARGE_DROITE, rect.bottom() - 6.0),
        Align2::LEFT_BOTTOM,
        t,
        semibold(TAILLE_PETIT),
        couleur(p.texte_secondaire),
    );
}

/// Le carré coloré d'une pastille ou d'une case : un fond, sa lettre ; absent : hachuré.
#[allow(clippy::too_many_arguments)]
pub(crate) fn carre(
    ui: &Ui,
    p: &Palette,
    rect: Rect,
    lettre: Option<char>,
    logo: Option<char>,
    fond: Rgb,
    absent: bool,
    taille: f32,
) {
    let arrondi = CornerRadius::same(4);
    let peintre = ui.painter();
    if absent {
        peintre.rect_filled(rect, arrondi, couleur(p.controle));
        let hachure = Stroke::new(1.0, couleur(fond).gamma_multiply(0.55));
        let dans = peintre.with_clip_rect(rect);
        let mut x = rect.left() - rect.height();
        while x < rect.right() {
            dans.line_segment(
                [pos2(x, rect.bottom()), pos2(x + rect.height(), rect.top())],
                hachure,
            );
            x += 5.0;
        }
        peintre.rect_stroke(
            rect,
            arrondi,
            Stroke::new(1.0, couleur(p.bord_controle)),
            StrokeKind::Inside,
        );
    } else {
        peintre.rect_filled(rect, arrondi, couleur(fond));
    }
    // Le glyphe prend la couleur de la lettre : blanc ou noir selon le fond (contraste au
    // moins 3:1, comme la lettre), en clair comme en sombre.
    let c = if absent {
        p.texte_desactive
    } else {
        couleur_lettre(fond)
    };
    marque_au_centre(peintre, rect, logo, lettre, taille, c);
}

/// Le logo (s'il est dans la police d'icônes) ou la lettre, au centre de `rect`.
fn marque_au_centre(
    peintre: &egui::Painter,
    rect: Rect,
    logo: Option<char>,
    lettre: Option<char>,
    taille: f32,
    c: Rgb,
) {
    match marque(logo, lettre, crate::polices::icone_disponible) {
        Some(Marque::Logo(g)) => {
            let t = (taille + 4.0).min(rect.height() - 4.0).max(taille);
            peintre.text(
                rect.center(),
                Align2::CENTER_CENTER,
                g,
                crate::polices::icones(t),
                couleur(c),
            );
        }
        Some(Marque::Lettre(l)) => {
            peintre.text(
                rect.center(),
                Align2::CENTER_CENTER,
                l,
                semibold(taille),
                couleur(c),
            );
        }
        None => {}
    }
}

fn pastille(ui: &mut Ui, p: &Palette, rect: Rect, num: usize, pa: &Pastille) -> Option<Geste> {
    let r = ui.interact(
        rect,
        ui.scope_id().with(("pastille", num)),
        if pa.cle.is_some() {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if pa.cle.is_some() {
        let f = if r.is_pointer_button_down_on() {
            Some(p.appui)
        } else if r.hovered() || r.has_focus() {
            Some(p.survol)
        } else {
            None
        };
        if let Some(f) = f {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(LIGNE_ARRONDI), couleur(f));
        }
    }
    let cy = rect.center().y;
    let carre_rect = Rect::from_min_size(
        pos2(rect.left() + PASTILLE_MARGE, cy - PASTILLE_HAUTEUR / 2.0),
        vec2(PASTILLE_LARGEUR, PASTILLE_HAUTEUR),
    );
    carre(
        ui,
        p,
        carre_rect,
        Some(pa.lettre),
        pa.logo,
        pa.couleur,
        pa.absent,
        TAILLE_PETIT,
    );
    textes_pastille(ui, p, rect, carre_rect.right() + PASTILLE_ESPACE, pa);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, pa.cle.is_some(), &pa.nom));
    dessin::focus(ui, p, &r, LIGNE_ARRONDI);
    pa.cle
        .as_ref()
        .filter(|_| r.clicked())
        .map(|c| Geste::Action(c.clone()))
}

/// Le nom de la pastille à partir de `x0`, puis sa mention et son glyphe ; le nom se coupe
/// (« … ») avant eux, jamais par-dessus.
fn textes_pastille(ui: &Ui, p: &Palette, rect: Rect, x0: f32, pa: &Pastille) {
    let (texte, second) = if pa.absent {
        (p.texte_desactive, p.texte_desactive)
    } else {
        (p.texte, p.texte_secondaire)
    };
    let mention = pa.mention.as_deref().map(texte_mention);
    let mention_l = mention
        .as_deref()
        .map_or(0.0, |m| 4.0 + mesure(ui, m, TAILLE_PETIT));
    let glyphe_l = pa.glyphe.map_or(0.0, |_| 6.0 + LIGNE_ICONE);
    let cy = rect.center().y;
    let place = rect.right() - LIGNE_MARGE_DROITE - mention_l - glyphe_l - x0;
    let mut job = egui::text::LayoutJob::simple_singleline(
        pa.nom.clone(),
        FontId::proportional(TAILLE_TEXTE),
        couleur(texte),
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(place.max(1.0));
    let g = ui.painter().layout_job(job);
    let nom_l = g.size().x;
    ui.painter()
        .galley(pos2(x0, cy - g.size().y / 2.0), g, couleur(texte));
    let mut x = x0 + nom_l;
    if let Some(m) = mention {
        x += 4.0;
        ui.painter().text(
            pos2(x, cy),
            Align2::LEFT_CENTER,
            &m,
            FontId::proportional(TAILLE_PETIT),
            couleur(second),
        );
        x += mesure(ui, &m, TAILLE_PETIT);
    }
    if let Some(gl) = pa.glyphe {
        icone(ui, pos2(x + 6.0 + LIGNE_ICONE / 2.0, cy), gl, LIGNE_ICONE, second);
    }
}

/// Dessine la rangée : le titre, puis les pastilles sur 2 colonnes (retour à la ligne).
pub fn dessiner_pastilles(
    ui: &mut Ui,
    p: &Palette,
    titre: Option<&str>,
    pastilles: &[Pastille],
) -> Option<Geste> {
    if let Some(t) = titre {
        titre_section(ui, p, t);
    }
    if pastilles.is_empty() {
        return None;
    }
    let (rect, _) = ui.allocate_exact_size(
        vec2(
            ui.available_width(),
            rangees(pastilles.len()) as f32 * VOLET_LIGNE,
        ),
        Sense::hover(),
    );
    let w = rect.width() / 2.0;
    let mut geste = None;
    for (i, pa) in pastilles.iter().enumerate() {
        let cellule = Rect::from_min_size(
            pos2(
                rect.left() + w * (i % 2) as f32,
                rect.top() + VOLET_LIGNE * (i / 2) as f32,
            ),
            vec2(w, VOLET_LIGNE),
        );
        if let Some(g) = pastille(ui, p, cellule, i, pa) {
            geste = Some(g);
        }
    }
    geste
}

// ---- Le choix illustré ----------------------------------------------------------------

/// La largeur qu'il faut à un choix illustré : l'illustration, puis la plus large de ses deux
/// lignes (texte et raccourci ; sous-ligne et étiquette).
pub fn largeur_choix(
    ui: &Ui,
    texte: &str,
    sous_ligne: Option<&str>,
    raccourci: Option<&str>,
    etiquette: Option<&str>,
) -> f32 {
    let haut = mesure(ui, texte, TAILLE_TEXTE)
        + raccourci.map_or(0.0, |r| {
            LIGNE_ECART + mesure(ui, &touches::lisible(r), TAILLE_PETIT)
        });
    let bas = sous_ligne.map_or(0.0, |s| mesure(ui, s, TAILLE_PETIT))
        + etiquette.map_or(0.0, |e| {
            LIGNE_ECART + TAILLE_PETIT + 4.0 + mesure(ui, e, TAILLE_PETIT)
        });
    ILLUSTRE_MARGE + ILLUSTRE_LARGEUR + ILLUSTRE_ESPACE + haut.max(bas) + LIGNE_MARGE_DROITE
}

/// Comment dessiner une case : la rangée d'écrans égaux (lettre, lune, pied), ou une case du
/// plan (sans pied, qui toucherait l'écran du dessous ; seulement la couleur si elle est trop
/// petite pour porter une lettre).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Genre {
    Rangee,
    Plan,
    PlanCouleur,
}

/// Le genre des cases d'un plan : la couleur seule si la plus petite fait moins de
/// [`PLAN_CASE_H_MIN`] de haut (une lettre y serait écrasée, et il ne faut pas fausser les
/// proportions en agrandissant la case).
fn genre_du_plan(rects: &[Rect]) -> Genre {
    let plus_petite = rects.iter().map(|r| r.height()).fold(f32::MAX, f32::min);
    if plus_petite < PLAN_CASE_H_MIN {
        Genre::PlanCouleur
    } else {
        Genre::Plan
    }
}

/// Le bord en tirets de 1 point d'une case (écran en veille ou débranché).
fn bord_en_tirets(peintre: &egui::Painter, rect: Rect, c: Rgb) {
    let (a, b, d, e) = (
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    );
    peintre.extend(egui::Shape::dashed_line(
        &[a, b, d, e, a],
        Stroke::new(1.0, couleur(c)),
        2.0,
        2.0,
    ));
}

/// Le pied d'un écran de la rangée (pas dans le plan : il tomberait sur l'écran du dessous).
fn pied(peintre: &egui::Painter, p: &Palette, rect: Rect) {
    let y = rect.bottom() + 3.0;
    peintre.line_segment(
        [pos2(rect.center().x - 5.0, y), pos2(rect.center().x + 5.0, y)],
        Stroke::new(2.0, couleur(p.texte_desactive)),
    );
}

/// Un écran débranché : sa case garde sa place, sa couleur et sa marque, mais toute en pâle
/// ([`OPACITE_DEBRANCHE`]), avec un bord en tirets clair. Dessiné à la place de la veille.
fn case_debranchee(ui: &Ui, p: &Palette, rect: Rect, c: &Case, genre: Genre, arrondi: CornerRadius) {
    let mut pale = ui.painter().clone();
    pale.multiply_opacity(OPACITE_DEBRANCHE);
    let fond = c.couleur.unwrap_or(p.controle);
    pale.rect_filled(rect, arrondi, couleur(fond));
    if genre != Genre::PlanCouleur {
        let teinte = if c.couleur.is_some() {
            couleur_lettre(fond)
        } else {
            p.texte_secondaire
        };
        marque_au_centre(&pale, rect, c.logo, c.lettre, TAILLE_PETIT, teinte);
    }
    bord_en_tirets(&pale, rect, BORD_DEBRANCHE);
    if genre == Genre::Rangee {
        pied(&pale, p, rect);
    }
}

fn case(ui: &Ui, p: &Palette, rect: Rect, c: &Case, genre: Genre) {
    let arrondi = CornerRadius::same(if genre == Genre::Rangee { 3 } else { 2 });
    let etat = etat_case(c.veille, c.debranche);
    if etat == EtatCase::Debranche {
        case_debranchee(ui, p, rect, c, genre, arrondi);
        return;
    }
    let veille = etat == EtatCase::Veille;
    if genre == Genre::PlanCouleur {
        // Seulement la couleur : le fond de la case, sans lettre ni lune.
        if veille {
            ui.painter().rect_filled(rect, arrondi, couleur(FOND_VEILLE));
            ui.painter().rect_stroke(
                rect,
                arrondi,
                Stroke::new(1.0, couleur(BORD_VEILLE)),
                StrokeKind::Inside,
            );
        } else if let Some(f) = c.couleur {
            ui.painter().rect_filled(rect, arrondi, couleur(f));
        } else {
            ui.painter().rect_filled(rect, arrondi, couleur(p.controle));
            ui.painter().rect_stroke(
                rect,
                arrondi,
                Stroke::new(1.0, couleur(p.bord_controle)),
                StrokeKind::Inside,
            );
        }
        return;
    }
    if veille {
        ui.painter().rect_filled(rect, arrondi, couleur(FOND_VEILLE));
        bord_en_tirets(ui.painter(), rect, BORD_VEILLE);
        icone(ui, rect.center(), icones::LUNE, 12.0, LUNE_VEILLE);
    } else if let Some(f) = c.couleur {
        carre(ui, p, rect, c.lettre, c.logo, f, false, TAILLE_PETIT);
    } else {
        ui.painter().rect_filled(rect, arrondi, couleur(p.controle));
        ui.painter().rect_stroke(
            rect,
            arrondi,
            Stroke::new(1.0, couleur(p.bord_controle)),
            StrokeKind::Inside,
        );
        match marque(c.logo, c.lettre, crate::polices::icone_disponible) {
            Some(Marque::Logo(g)) => icone(ui, rect.center(), g, TAILLE_PETIT + 2.0, p.texte_secondaire),
            Some(Marque::Lettre(l)) => {
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    l,
                    semibold(TAILLE_PETIT),
                    couleur(p.texte_secondaire),
                );
            }
            None => {}
        }
    }
    if genre == Genre::Rangee {
        pied(ui.painter(), p, rect);
    }
}

/// Où commence le texte d'un choix illustré, depuis son bord gauche : le même quel que soit
/// le nombre de cases.
pub fn texte_x() -> f32 {
    ILLUSTRE_MARGE + ILLUSTRE_LARGEUR + ILLUSTRE_ESPACE
}

/// Un texte sur une ligne, coupé (« … ») au-delà de `largeur`.
fn coupe(ui: &Ui, texte: &str, taille: f32, c: Rgb, largeur: f32) -> std::sync::Arc<egui::Galley> {
    let mut job =
        egui::text::LayoutJob::simple_singleline(texte.to_string(), FontId::proportional(taille), couleur(c));
    job.wrap = egui::text::TextWrapping::truncate_at_width(largeur.max(1.0));
    ui.painter().layout_job(job)
}

/// Dessine un choix illustré ; rend l'action de son clic.
#[allow(clippy::too_many_arguments)]
pub fn dessiner_choix(
    ui: &mut Ui,
    p: &Palette,
    cle: &str,
    texte: &str,
    sous_ligne: Option<&str>,
    illustration: &[Case],
    raccourci: Option<&str>,
    actif: bool,
    etiquette: Option<&str>,
) -> Option<Geste> {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), ILLUSTRE_LIGNE), Sense::click());
    fond_choix(ui, p, rect, &r, actif);
    dessiner_illustration(ui, p, rect, illustration);
    premiere_ligne(ui, p, rect, texte, raccourci);
    seconde_ligne(ui, p, rect, sous_ligne, actif, etiquette);
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, actif, texte));
    dessin::focus(ui, p, &r, LIGNE_ARRONDI);
    r.clicked().then(|| Geste::Action(cle.to_string()))
}

/// Le fond d'un choix illustré : teinté d'accent avec sa barre s'il est actif, sinon au survol.
fn fond_choix(ui: &Ui, p: &Palette, rect: Rect, r: &egui::Response, actif: bool) {
    let arrondi = CornerRadius::same(LIGNE_ARRONDI);
    let accent = p.accent;
    if actif {
        let alpha = if r.is_pointer_button_down_on() {
            52
        } else if r.hovered() || r.has_focus() {
            44
        } else {
            30
        };
        ui.painter().rect_filled(
            rect,
            arrondi,
            egui::Color32::from_rgba_unmultiplied(accent.0, accent.1, accent.2, alpha),
        );
        let barre = Rect::from_center_size(
            pos2(rect.left() + ILLUSTRE_BARRE[0] / 2.0, rect.center().y),
            vec2(ILLUSTRE_BARRE[0], ILLUSTRE_BARRE[1]),
        );
        ui.painter()
            .rect_filled(barre, CornerRadius::same(2), couleur(accent));
    } else if r.is_pointer_button_down_on() {
        ui.painter().rect_filled(rect, arrondi, couleur(p.appui));
    } else if r.hovered() || r.has_focus() {
        ui.painter().rect_filled(rect, arrondi, couleur(p.survol));
    }
}

/// L'illustration, à gauche, de largeur fixe : de 1 à 4 cases, au vrai plan des écrans s'il
/// est donné, sinon en rangée.
fn dessiner_illustration(ui: &Ui, p: &Palette, rect: Rect, illustration: &[Case]) {
    let n = illustration.len().min(ILLUSTRE_CASES_MAX);
    let cases = &illustration[..n];
    if let Some(places) = places_valides(cases) {
        // Le vrai plan des écrans : la largeur d'aujourd'hui, toute la hauteur utile.
        let zone = Rect::from_min_max(
            pos2(rect.left() + ILLUSTRE_MARGE, rect.top() + PLAN_MARGE_V),
            pos2(
                rect.left() + ILLUSTRE_MARGE + ILLUSTRE_LARGEUR,
                rect.bottom() - PLAN_MARGE_V,
            ),
        );
        let rects = plan_des_cases(&places, zone);
        let genre = genre_du_plan(&rects);
        for (c, r) in cases.iter().zip(&rects) {
            case(ui, p, *r, c, genre);
        }
    } else {
        let haut = rect.center().y - (ILLUSTRE_CASE_H + 5.0) / 2.0;
        for (i, c) in cases.iter().enumerate() {
            let x = rect.left() + ILLUSTRE_MARGE + i as f32 * (ILLUSTRE_CASE_L + ILLUSTRE_CASE_ECART);
            case(
                ui,
                p,
                Rect::from_min_size(pos2(x, haut), vec2(ILLUSTRE_CASE_L, ILLUSTRE_CASE_H)),
                c,
                Genre::Rangee,
            );
        }
    }
}

/// La première ligne : le texte, coupé avant le raccourci à droite.
fn premiere_ligne(ui: &Ui, p: &Palette, rect: Rect, texte: &str, raccourci: Option<&str>) {
    let x0 = rect.left() + texte_x();
    let x1 = rect.right() - LIGNE_MARGE_DROITE;
    let y1 = rect.top() + 17.0;
    let petit = FontId::proportional(TAILLE_PETIT);
    let mut fin1 = x1;
    if let Some(rac) = raccourci {
        let g =
            ui.painter()
                .layout_no_wrap(touches::lisible(rac), petit.clone(), couleur(p.texte_secondaire));
        fin1 -= g.size().x;
        ui.painter()
            .galley(pos2(fin1, y1 - g.size().y / 2.0), g, couleur(p.texte_secondaire));
    }
    let g = coupe(ui, texte, TAILLE_TEXTE, p.texte, fin1 - LIGNE_ECART / 2.0 - x0);
    ui.painter()
        .galley(pos2(x0, y1 - g.size().y / 2.0), g, couleur(p.texte));
}

/// La seconde ligne : la sous-ligne, coupée avant la coche (et son étiquette) d'un choix actif.
fn seconde_ligne(
    ui: &Ui,
    p: &Palette,
    rect: Rect,
    sous_ligne: Option<&str>,
    actif: bool,
    etiquette: Option<&str>,
) {
    let x0 = rect.left() + texte_x();
    let x1 = rect.right() - LIGNE_MARGE_DROITE;
    let y2 = rect.top() + 37.0;
    let accent = p.accent;
    let mut fin2 = x1;
    if actif {
        if let Some(e) = etiquette {
            let g = ui.painter().layout_no_wrap(
                e.to_string(),
                FontId::proportional(TAILLE_PETIT),
                couleur(accent),
            );
            fin2 -= g.size().x;
            ui.painter()
                .galley(pos2(fin2, y2 - g.size().y / 2.0), g, couleur(accent));
            fin2 -= 4.0 + TAILLE_PETIT;
        } else {
            fin2 -= TAILLE_PETIT;
        }
        icone(
            ui,
            pos2(fin2 + TAILLE_PETIT / 2.0, y2),
            icones::COCHE,
            TAILLE_PETIT,
            accent,
        );
    }
    if let Some(s) = sous_ligne {
        let g = coupe(
            ui,
            s,
            TAILLE_PETIT,
            p.texte_secondaire,
            fin2 - LIGNE_ECART / 2.0 - x0,
        );
        ui.painter()
            .galley(pos2(x0, y2 - g.size().y / 2.0), g, couleur(p.texte_secondaire));
    }
}

#[cfg(test)]
mod essais;
