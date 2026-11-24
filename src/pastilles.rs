//! Deux briques du volet pour une rangée d'appareils et des choix illustrés : les pastilles
//! ([`crate::briques::Brique::Pastilles`]) et le choix illustré
//! ([`crate::briques::Brique::ChoixIllustre`]). Leurs types et leur dessin vivent ici ;
//! `briques.rs` les appelle comme les autres briques. Mesures : [`crate::mesures`].

use egui::{Align2, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::apparence::couleur;
use crate::briques::Geste;
use crate::dessin::{self, icone};
use crate::icones;
use crate::mesures::{
    ILLUSTRE_BARRE, ILLUSTRE_CASE_ECART, ILLUSTRE_CASE_H, ILLUSTRE_CASE_L, ILLUSTRE_CASES_MAX,
    ILLUSTRE_LARGEUR, ILLUSTRE_LIGNE, LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_MARGE_DROITE,
    PASTILLE_HAUTEUR, PASTILLE_LARGEUR, VOLET_LIGNE, VOLET_SECTION,
};
use crate::polices::semibold;
use crate::theme::{Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE, contraste};
use crate::touches;

const BLANC: Rgb = Rgb(255, 255, 255);
const NOIR: Rgb = Rgb(0, 0, 0);
/// Le fond d'une case d'écran en veille (sombre dans les deux modes : un écran éteint).
const FOND_VEILLE: Rgb = Rgb(0x1E, 0x1E, 0x1E);
const BORD_VEILLE: Rgb = Rgb(0x9A, 0x9A, 0x9A);
const LUNE_VEILLE: Rgb = Rgb(0xD0, 0xD0, 0xD0);
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
/// quelconques) devient un rectangle dans `zone`. L'ensemble est mis à l'échelle (le plus petit
/// des deux rapports : les proportions sont gardées) et centré dans `zone` ; chaque case est
/// réduite de 1 point de chaque côté, pour que deux écrans collés ne se touchent pas.
/// Une place sans surface (largeur ou hauteur ≤ 0), une liste vide ou une zone sans surface :
/// une liste vide.
pub fn plan_des_cases(places: &[[i32; 4]], zone: Rect) -> Vec<Rect> {
    if places.is_empty()
        || places.iter().any(|p| p[2] <= 0 || p[3] <= 0)
        || zone.width() <= 0.0
        || zone.height() <= 0.0
    {
        return Vec::new();
    }
    // L'enveloppe de tous les écrans (en f64 : des pixels de bureau, sans dépassement).
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in places {
        let (x, y) = (f64::from(p[0]), f64::from(p[1]));
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x + f64::from(p[2]));
        y1 = y1.max(y + f64::from(p[3]));
    }
    let (largeur, hauteur) = (x1 - x0, y1 - y0);
    let echelle = (f64::from(zone.width()) / largeur).min(f64::from(zone.height()) / hauteur);
    let decalage_x = f64::from(zone.center().x) - largeur * echelle / 2.0;
    let decalage_y = f64::from(zone.center().y) - hauteur * echelle / 2.0;
    places
        .iter()
        .map(|p| {
            let gauche = decalage_x + (f64::from(p[0]) - x0) * echelle;
            let haut = decalage_y + (f64::from(p[1]) - y0) * echelle;
            let droite = gauche + f64::from(p[2]) * echelle;
            let bas = haut + f64::from(p[3]) * echelle;
            // Réduit de 1 point de chaque côté, sans jamais inverser la case.
            let (cx, cy) = ((gauche + droite) / 2.0, (haut + bas) / 2.0);
            let demi_l = ((droite - gauche) / 2.0 - 1.0).max(0.0);
            let demi_h = ((bas - haut) / 2.0 - 1.0).max(0.0);
            Rect::from_min_max(
                pos2((cx - demi_l) as f32, (cy - demi_h) as f32),
                pos2((cx + demi_l) as f32, (cy + demi_h) as f32),
            )
        })
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

/// Ce qu'un carré montre : le logo (glyphe), ou la lettre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marque {
    Logo(char),
    Lettre(char),
}

/// Le logo si la police d'icônes l'a (`disponible`), sinon la lettre, sinon rien : jamais un
/// carré vide à la place d'un glyphe absent. Pure : la police est passée en paramètre.
pub fn marque(logo: Option<char>, lettre: Option<char>, disponible: impl Fn(char) -> bool) -> Option<Marque> {
    match logo {
        Some(g) if disponible(g) => Some(Marque::Logo(g)),
        _ => lettre.map(Marque::Lettre),
    }
}

/// La lettre lisible sur ce fond : blanche, ou noire si le fond est clair (contraste
/// calculé, le plus fort des deux).
pub fn couleur_lettre(fond: Rgb) -> Rgb {
    if contraste(fond, BLANC) >= contraste(fond, NOIR) {
        BLANC
    } else {
        NOIR
    }
}

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
    match marque(logo, lettre, crate::polices::icone_disponible) {
        Some(Marque::Logo(g)) => {
            let t = (taille + 4.0).min(rect.height() - 4.0).max(taille);
            icone(ui, rect.center(), g, t, c);
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
    let x0 = carre_rect.right() + PASTILLE_ESPACE;
    // Le nom se coupe (« … ») avant la mention et le glyphe, jamais par-dessus.
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
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, pa.cle.is_some(), &pa.nom));
    dessin::focus(ui, p, &r, LIGNE_ARRONDI);
    pa.cle
        .as_ref()
        .filter(|_| r.clicked())
        .map(|c| Geste::Action(c.clone()))
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

/// Les 10 couleurs des pastilles et des cases : lisibles en clair comme en sombre (3 contre
/// 1 au moins avec le fond des deux modes, lettre à 4,5 contre 1), sans vert, orange, rouge
/// ni bleu d'accent, qui disent un état ou l'accent de Windows, et à un écart ΔE2000 d'au
/// moins 12 l'une de l'autre (contrôlé par `tests/palette_pastilles.rs`). Un logiciel prend la
/// suivante pour chaque appareil. Les 6 premières ne bougent jamais : un rang déjà enregistré
/// garde sa couleur ; les suivantes s'ajoutent derrière.
pub fn couleurs_pastilles() -> [Rgb; 10] {
    [
        Rgb::hex(0x7B57C9), // violet
        Rgb::hex(0xB03A9C), // magenta
        Rgb::hex(0x1D7F86), // sarcelle foncé
        Rgb::hex(0x94604F), // brun
        Rgb::hex(0x667A94), // ardoise
        Rgb::hex(0x857A2A), // olive
        Rgb::hex(0x3865F3), // indigo
        Rgb::hex(0xC0416B), // framboise
        Rgb::hex(0x8E679E), // prune
        Rgb::hex(0x7C6D61), // taupe
    ]
}

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

fn case(ui: &Ui, p: &Palette, rect: Rect, c: &Case, genre: Genre) {
    let arrondi = CornerRadius::same(if genre == Genre::Rangee { 3 } else { 2 });
    if genre == Genre::PlanCouleur {
        // Seulement la couleur : le fond de la case, sans lettre ni lune.
        if c.veille {
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
    if c.veille {
        ui.painter().rect_filled(rect, arrondi, couleur(FOND_VEILLE));
        let (a, b, d, e) = (
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        );
        ui.painter().extend(egui::Shape::dashed_line(
            &[a, b, d, e, a],
            Stroke::new(1.0, couleur(BORD_VEILLE)),
            2.0,
            2.0,
        ));
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
    // Le pied de l'écran (pas dans le plan : il tomberait sur l'écran du dessous).
    if genre == Genre::Rangee {
        let y = rect.bottom() + 3.0;
        ui.painter().line_segment(
            [pos2(rect.center().x - 5.0, y), pos2(rect.center().x + 5.0, y)],
            Stroke::new(2.0, couleur(p.texte_desactive)),
        );
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
    // L'illustration : à gauche, largeur fixe, de 1 à 4 cases.
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
    // Les deux lignes de texte.
    let x0 = rect.left() + texte_x();
    let x1 = rect.right() - LIGNE_MARGE_DROITE;
    let (y1, y2) = (rect.top() + 17.0, rect.top() + 37.0);
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
    let mut fin2 = x1;
    if actif {
        if let Some(e) = etiquette {
            let g = ui
                .painter()
                .layout_no_wrap(e.to_string(), petit.clone(), couleur(accent));
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
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::RadioButton, true, actif, texte));
    dessin::focus(ui, p, &r, LIGNE_ARRONDI);
    r.clicked().then(|| Geste::Action(cle.to_string()))
}

#[cfg(test)]
mod essais {
    use super::*;
    use crate::briques::{self, Brique};
    use crate::theme::ChoixWindows;

    fn pa(cle: Option<&str>, lettre: char, nom: &str) -> Pastille {
        Pastille {
            cle: cle.map(Into::into),
            lettre,
            logo: None,
            couleur: couleurs_pastilles()[0],
            nom: nom.into(),
            mention: None,
            glyphe: None,
            absent: false,
        }
    }

    fn case_ecran(l: char) -> Case {
        Case {
            lettre: Some(l),
            couleur: Some(couleurs_pastilles()[0]),
            ..Default::default()
        }
    }

    fn avec_place(place: Option<[i32; 4]>) -> Case {
        Case {
            place,
            ..case_ecran('A')
        }
    }

    fn zone() -> Rect {
        Rect::from_min_max(pos2(10.0, 4.0), pos2(115.0, 48.0))
    }

    /// Le cas du propriétaire (valeurs inventées) : un fixe 2560×1440 à gauche, un portable
    /// 2560×1440 à droite, et sous lui l'écran intégré, plus petit, 1920×1200.
    fn cas_proprietaire() -> Vec<[i32; 4]> {
        vec![
            [0, 0, 2560, 1440],
            [2560, 0, 2560, 1440],
            [2560, 1440, 1920, 1200],
        ]
    }

    fn proche(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    /// Le plan garde les proportions : un seul facteur d'échelle pour la largeur et la hauteur.
    #[test]
    fn plan_garde_les_proportions() {
        let r = plan_des_cases(&cas_proprietaire(), zone());
        assert_eq!(r.len(), 3);
        // Les deux grands écrans : même taille, 16/9 (à 2 points de réduction près).
        assert!(proche(r[0].width(), r[1].width()));
        assert!(proche(r[0].height(), r[1].height()));
        assert!(proche((r[0].width() + 2.0) / (r[0].height() + 2.0), 16.0 / 9.0));
        // L'écran intégré : 1920×1200 = 16/10, plus petit, aligné à gauche de l'écran du dessus.
        assert!(proche((r[2].width() + 2.0) / (r[2].height() + 2.0), 16.0 / 10.0));
        assert!(r[2].width() < r[1].width() && r[2].height() < r[1].height());
        assert!(proche(r[2].left(), r[1].left()));
        // Il est sous l'écran du dessus, avec 2 points de vide.
        assert!(proche(r[2].top() - r[1].bottom(), 2.0));
        // Le portable est collé à droite du fixe : 2 points de vide.
        assert!(proche(r[1].left() - r[0].right(), 2.0));
    }

    /// L'ensemble est centré dans la zone, et tient dedans (hauteur limitante ici).
    #[test]
    fn plan_centre_dans_la_zone() {
        let r = plan_des_cases(&cas_proprietaire(), zone());
        let tout = r.iter().fold(r[0], |a, b| a.union(*b));
        // Le vide de 1 point de chaque côté est symétrique : le centre est celui de la zone.
        assert!(proche(tout.center().x, zone().center().x), "{tout:?}");
        assert!(proche(tout.center().y, zone().center().y), "{tout:?}");
        assert!(zone().contains_rect(tout));
        // Ici la hauteur limite : l'ensemble la remplit (au vide de 1 point près).
        assert!(proche(tout.height() + 2.0, zone().height()), "{tout:?}");
        assert!(tout.width() + 2.0 < zone().width());
    }

    /// La largeur peut limiter aussi : deux écrans côte à côte, dans une zone basse et large.
    #[test]
    fn plan_limite_par_la_largeur() {
        let r = plan_des_cases(&[[0, 0, 2000, 500], [2000, 0, 2000, 500]], zone());
        let tout = r[0].union(r[1]);
        assert!(proche(tout.width() + 2.0, zone().width()), "{tout:?}");
        assert!(tout.height() + 2.0 < zone().height());
        assert!(proche(tout.center().y, zone().center().y));
    }

    /// Des coordonnées négatives et un décalage quelconque donnent le même plan.
    #[test]
    fn plan_independant_de_l_origine() {
        let a = plan_des_cases(&cas_proprietaire(), zone());
        let decale: Vec<[i32; 4]> = cas_proprietaire()
            .iter()
            .map(|p| [p[0] - 7000, p[1] - 3000, p[2], p[3]])
            .collect();
        let b = plan_des_cases(&decale, zone());
        for (x, y) in a.iter().zip(&b) {
            assert!(
                proche(x.left(), y.left()) && proche(x.top(), y.top()),
                "{x:?} {y:?}"
            );
            assert!(proche(x.width(), y.width()) && proche(x.height(), y.height()));
        }
    }

    /// Un seul écran remplit la zone en hauteur ; deux écrans collés ne se touchent pas.
    #[test]
    fn plan_un_ecran_et_ecrans_colles() {
        let r = plan_des_cases(&[[-1920, 0, 1920, 1080]], zone());
        assert_eq!(r.len(), 1);
        assert!(proche(r[0].center().x, zone().center().x));
        assert!(proche(r[0].height() + 2.0, zone().height()));
        let d = plan_des_cases(&[[0, 0, 1920, 1080], [1920, 0, 1920, 1080]], zone());
        assert!(d[1].left() - d[0].right() >= 2.0 - 0.01);
        assert!(!d[0].intersects(d[1]));
    }

    /// Une place sans surface, une liste vide ou une zone vide : rien à dessiner.
    #[test]
    fn plan_refuse_le_vide() {
        assert!(plan_des_cases(&[], zone()).is_empty());
        assert!(plan_des_cases(&[[0, 0, 0, 100]], zone()).is_empty());
        assert!(plan_des_cases(&[[0, 0, 100, 100], [100, 0, 100, -5]], zone()).is_empty());
        assert!(plan_des_cases(&[[0, 0, 100, 100]], Rect::NOTHING).is_empty());
    }

    /// Le plan ne sert que si TOUTES les cases ont une place valide.
    #[test]
    fn plan_ou_rangee() {
        let ok = [Some([0, 0, 100, 100]), Some([100, 0, 100, 100])];
        let c: Vec<Case> = ok.iter().map(|p| avec_place(*p)).collect();
        assert_eq!(places_valides(&c).map(|v| v.len()), Some(2));
        for faux in [
            None,
            Some([0, 0, 0, 100]),
            Some([0, 0, 100, 0]),
            Some([0, 0, -1, 5]),
        ] {
            let c = vec![avec_place(ok[0]), avec_place(faux)];
            assert_eq!(places_valides(&c), None, "{faux:?}");
        }
        assert_eq!(places_valides(&[]), None);
    }

    /// La plus petite case fait-elle moins de 12 points ? Alors seulement la couleur.
    #[test]
    fn plan_petite_case_en_couleur_seule() {
        let r = plan_des_cases(&cas_proprietaire(), zone());
        assert_eq!(
            genre_du_plan(&r),
            Genre::Plan,
            "le cas du propriétaire garde ses lettres"
        );
        // Un petit écran à côté d'un très grand : sa case tombe sous 12 points.
        let r = plan_des_cases(&[[0, 0, 3840, 2160], [3840, 0, 800, 480]], zone());
        assert!(r[1].height() < PLAN_CASE_H_MIN);
        assert_eq!(genre_du_plan(&r), Genre::PlanCouleur);
        // Les proportions ne sont pas faussées : la case garde son rapport 5/3.
        assert!(proche(
            (r[1].width() + 2.0) / (r[1].height() + 2.0),
            800.0 / 480.0
        ));
    }

    /// Le plan se dessine dans le volet (3 cases, puis une place manquante : la rangée, puis
    /// 5 cases : 4 au plus), sans changer la hauteur de la ligne.
    #[test]
    fn plan_dans_le_volet() {
        let p = Palette::de(&ChoixWindows::default());
        let ctx = egui::Context::default();
        let cas = |cases: Vec<Case>| {
            let mut h = 0.0;
            let mut sortie = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(320.0, 300.0))),
                    ..Default::default()
                },
                |ui| {
                    let h0 = ui.cursor().top();
                    dessiner_choix(ui, &p, "c", "Choix", Some("sous"), &cases, None, true, None);
                    h = ui.cursor().top() - h0;
                },
            );
            sortie.textures_delta.clear();
            h
        };
        let places = cas_proprietaire();
        let trois: Vec<Case> = places.iter().map(|q| avec_place(Some(*q))).collect();
        // La référence : la rangée d'aujourd'hui (sans place).
        let rangee = cas(vec![case_ecran('A'); 3]);
        assert!(
            (ILLUSTRE_LIGNE..=ILLUSTRE_LIGNE + 8.0).contains(&rangee),
            "{rangee}"
        );
        assert_eq!(cas(trois.clone()), rangee);
        let mut manque = trois.clone();
        manque[2].place = None;
        assert_eq!(cas(manque), rangee);
        let cinq: Vec<Case> = (0..5).map(|i| avec_place(Some([i * 100, 0, 100, 100]))).collect();
        assert_eq!(cas(cinq), rangee);
    }

    /// Le texte ne bouge pas, avec ou sans plan : la zone du plan garde la largeur d'aujourd'hui.
    #[test]
    fn plan_ne_deplace_pas_le_texte() {
        assert!(ILLUSTRE_MARGE + ILLUSTRE_LARGEUR <= texte_x());
    }

    fn liste() -> Vec<Brique> {
        vec![
            Brique::Pastilles {
                titre: Some("Ordinateurs".into()),
                pastilles: vec![
                    pa(Some("fixe"), 'F', "Fixe"),
                    pa(None, 'P', "Portable"),
                    pa(Some("tour"), 'T', "Tour"),
                ],
            },
            Brique::ChoixIllustre {
                cle: "bureau".into(),
                texte: "Bureau".into(),
                sous_ligne: Some("Fixe à gauche".into()),
                illustration: vec![case_ecran('F'), case_ecran('P')],
                raccourci: Some("Ctrl+Alt+Maj+F1".into()),
                actif: true,
                etiquette_active: Some("Affiché".into()),
            },
        ]
    }

    /// Une image de la liste ; rend le geste et les hauteurs des deux briques.
    fn image(ctx: &egui::Context, evts: Vec<egui::Event>) -> (Option<Geste>, [f32; 2]) {
        let p = Palette::de(&ChoixWindows::default());
        let entree = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(320.0, 600.0))),
            events: evts,
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let mut rendu = (None, [0.0; 2]);
        let mut sortie = ctx.run_ui(entree, |ui| {
            let l = liste();
            let h0 = ui.cursor().top();
            let mut g = briques::dessiner(ui, &p, &l[..1]);
            let h1 = ui.cursor().top();
            g = g.or(briques::dessiner(ui, &p, &l[1..]));
            rendu = (g, [h1 - h0, ui.cursor().top() - h1]);
        });
        sortie.textures_delta.clear();
        rendu
    }

    fn clic(ctx: &egui::Context, pos: egui::Pos2) -> Option<Geste> {
        let bouton = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        image(ctx, vec![egui::Event::PointerMoved(pos)]);
        image(ctx, vec![bouton(true)]);
        image(ctx, vec![bouton(false)]).0
    }

    /// La lettre est blanche sur un fond foncé, noire sur un fond clair.
    #[test]
    fn lettre_selon_le_fond() {
        assert_eq!(couleur_lettre(Rgb(0x30, 0x20, 0x70)), BLANC);
        assert_eq!(couleur_lettre(Rgb(0xF0, 0xE0, 0x90)), NOIR);
        assert_eq!(couleur_lettre(NOIR), BLANC);
        assert_eq!(couleur_lettre(BLANC), NOIR);
    }

    /// 3 pastilles : 2 colonnes, donc 2 rangées ; le choix illustré a sa hauteur propre.
    #[test]
    fn hauteurs() {
        let ctx = egui::Context::default();
        let (_, h) = image(&ctx, vec![]);
        assert!(
            (h[0] - (VOLET_SECTION + 2.0 * VOLET_LIGNE + 2.0 * 2.0)).abs() <= 8.0,
            "{h:?}"
        );
        assert!((h[1] - ILLUSTRE_LIGNE).abs() <= 8.0, "{h:?}");
        assert_eq!(rangees(1), 1);
        assert_eq!(rangees(2), 1);
        assert_eq!(rangees(3), 2);
        assert_eq!(rangees(4), 2);
    }

    /// Une pastille avec clé rend sa clé ; sans clé, un clic ne fait rien ; le choix illustré
    /// rend la sienne.
    #[test]
    fn gestes() {
        let ctx = egui::Context::default();
        image(&ctx, vec![]);
        let y = VOLET_SECTION + 2.0 + VOLET_LIGNE / 2.0;
        assert_eq!(clic(&ctx, pos2(40.0, y)), Some(Geste::Action("fixe".into())));
        assert_eq!(clic(&ctx, pos2(200.0, y)), None, "pastille sans clé");
        // Rangée 2, colonne 1 : « Tour ».
        assert_eq!(
            clic(&ctx, pos2(40.0, y + VOLET_LIGNE)),
            Some(Geste::Action("tour".into()))
        );
        let (_, h) = image(&ctx, vec![]);
        let y_choix = VOLET_SECTION + 2.0 + 2.0 * VOLET_LIGNE + 2.0 + 20.0;
        assert!(h[0] > 0.0);
        assert_eq!(
            clic(&ctx, pos2(200.0, y_choix)),
            Some(Geste::Action("bureau".into()))
        );
    }

    /// La largeur de l'illustration est fixe : 1 ou 4 cases, le texte est au même endroit.
    #[test]
    fn texte_au_meme_endroit() {
        assert!(texte_x() >= ILLUSTRE_LARGEUR);
        let ctx = egui::Context::default();
        let mut largeur = 0.0;
        let mut sortie = ctx.run_ui(Default::default(), |ui| {
            largeur = briques::largeur(ui, &liste());
        });
        sortie.textures_delta.clear();
        assert!(largeur >= texte_x() + LIGNE_MARGE_DROITE, "{largeur}");
    }
}
