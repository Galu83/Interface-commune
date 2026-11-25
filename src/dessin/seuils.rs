//! La barre à seuils : une barre de 0 à 100 coupée en zones colorées, dont on règle les limites
//! en faisant glisser la poignée entre deux zones (par exemple « critique », « bas », « bon » pour
//! le niveau d'une batterie). Le nombre de chaque limite est écrit au-dessus de sa poignée.
//!
//! Au clavier (règles : interface Windows 11, point 8) : chaque poignée prend le focus par Tab ;
//! flèches : un pas ; Maj + flèche ou Page haut / Page bas : dix pas ; Début et Fin : la borne
//! la plus proche. Une poignée ne passe jamais l'autre : les zones restent dans l'ordre.
//!
//! Le logiciel donne les zones (de gauche à droite) et les seuils ; la bibliothèque dessine et
//! change les seuils. Une poignée glissée rend `glisse` : le logiciel peut alors montrer
//! l'effet tout de suite et n'enregistrer qu'au relâchement (`termine`).

use egui::{
    Align2, CornerRadius, EventFilter, FontId, Key, Modifiers, Rect, Sense, Stroke, StrokeKind, Ui, pos2,
    vec2,
};

use crate::dessin::couleur::couleur;
use crate::polices::semibold;
use crate::theme::{Palette, Rgb};

/// Hauteur de la barre colorée, en points.
const BARRE: f32 = 28.0;
/// Ce que la poignée dépasse de la barre, en haut comme en bas.
const DEPASSE: f32 = 6.0;
/// Largeur dessinée de la poignée, et de sa zone cliquable (au moins 32 points, règle 8).
const POIGNEE: f32 = 8.0;
const ZONE_POIGNEE: f32 = 32.0;
/// Place du nombre au-dessus de la barre, et des repères en dessous.
const BULLE: f32 = 22.0;
const ECART_BULLE: f32 = 8.0;
const REPERES: f32 = 20.0;

/// Une zone de la barre : son nom (dit par un lecteur d'écran) et sa couleur.
#[derive(Debug, Clone, Copy)]
pub struct Zone<'a> {
    pub nom: &'a str,
    pub couleur: Rgb,
}

/// Les réglages de la barre.
#[derive(Debug, Clone, Copy)]
pub struct Options<'a> {
    /// La valeur la plus haute (la barre va de 0 à `max`).
    pub max: u8,
    /// Écart minimal entre deux limites quand on en déplace une (0 : elles peuvent se toucher).
    pub ecart_min: u8,
    /// Ce qui suit chaque nombre : « % ».
    pub unite: &'a str,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Options {
            max: 100,
            ecart_min: 1,
            unite: "%",
        }
    }
}

/// Ce qui s'est passé à cette image.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReponseSeuils {
    /// Un seuil a changé (glissé ou au clavier).
    pub change: bool,
    /// Une poignée est en train d'être glissée.
    pub glisse: bool,
    /// Une poignée vient d'être relâchée.
    pub termine: bool,
}

// Où une limite peut aller et ce que demandent les touches : du calcul pur, dans le cœur
// (bloc F), repris ici à son chemin d'aujourd'hui.
pub use ic_coeur::gestes::seuils::deplacer;
use ic_coeur::gestes::seuils::{Borne, voulu_au_clavier};

/// Le nombre écrit au-dessus d'une poignée, avec son unité (espace insécable avant l'unité).
fn texte_valeur(v: u8, unite: &str) -> String {
    ic_coeur::textes::avec_unite(v, unite)
}

/// Le pas demandé par les touches de cette image (en négatif vers la gauche), et la borne
/// demandée (Début ou Fin).
fn touches(ui: &Ui) -> (i32, Option<Borne>) {
    let mut pas = 0;
    let mut borne = None;
    ui.input_mut(|i| {
        for (touche, p) in [
            (Key::ArrowLeft, -1),
            (Key::ArrowDown, -1),
            (Key::ArrowRight, 1),
            (Key::ArrowUp, 1),
        ] {
            if i.consume_key(Modifiers::SHIFT, touche) {
                pas += p * 10;
            }
            if i.consume_key(Modifiers::NONE, touche) {
                pas += p;
            }
        }
        if i.consume_key(Modifiers::NONE, Key::PageUp) {
            pas += 10;
        }
        if i.consume_key(Modifiers::NONE, Key::PageDown) {
            pas -= 10;
        }
        if i.consume_key(Modifiers::NONE, Key::Home) {
            borne = Some(Borne::Debut);
        }
        if i.consume_key(Modifiers::NONE, Key::End) {
            borne = Some(Borne::Fin);
        }
    });
    (pas, borne)
}

/// La barre à seuils : `zones` de gauche à droite, `seuils` les limites entre deux zones
/// (`zones.len() - 1` nombres qui montent). `id` la distingue d'une autre barre de la page.
///
/// # Panics
/// Si `seuils.len() + 1 != zones.len()`.
pub fn barre_seuils(
    ui: &mut Ui,
    p: &Palette,
    id: &str,
    zones: &[Zone<'_>],
    seuils: &mut [u8],
    options: &Options<'_>,
) -> ReponseSeuils {
    assert_eq!(seuils.len() + 1, zones.len(), "une limite entre deux zones");
    let mut reponse = ReponseSeuils::default();
    let hauteur = BULLE + ECART_BULLE + DEPASSE + BARRE + DEPASSE + REPERES;
    let (zone, _) = ui.allocate_exact_size(vec2(ui.available_width(), hauteur), Sense::hover());
    let g = Gabarit::de(zone, options.max.max(1));
    if ui.is_rect_visible(zone) {
        dessiner_zones(ui, p, &g, zones, seuils, options.unite);
    }

    // Les poignées, de la dernière à la première : celle de gauche reste prise si deux se
    // touchent (la seconde se dégage en la poussant à droite).
    for i in (0..seuils.len()).rev() {
        let centre = pos2(g.x_de(seuils[i]), g.barre.center().y);
        let cliquable = Rect::from_center_size(centre, vec2(ZONE_POIGNEE, BARRE + 2.0 * DEPASSE + 4.0));
        let r = ui
            .interact(cliquable, ui.make_persistent_id((id, i)), Sense::click_and_drag())
            .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
        let nom = format!("{} / {}", zones[i].nom, zones[i + 1].nom);
        if suivre_poignee(ui, &g, &r, i, seuils, options) {
            reponse.change = true;
        }
        reponse.glisse |= r.dragged();
        reponse.termine |= r.drag_stopped();
        let valeur = seuils[i];
        r.widget_info(|| egui::WidgetInfo::slider(true, f64::from(valeur), &nom));
        if r.clicked() || r.drag_started() {
            r.request_focus();
        }
        if ui.is_rect_visible(cliquable) {
            dessiner_poignee(ui, p, &g, &r, valeur, options.unite);
        }
    }
    reponse
}

/// Où se trouve la barre dans sa zone, et comment passer d'une valeur à sa place.
struct Gabarit {
    zone: Rect,
    barre: Rect,
    x0: f32,
    x1: f32,
    max: u8,
}

impl Gabarit {
    /// La barre laisse la place de la moitié d'une poignée à chaque bout.
    fn de(zone: Rect, max: u8) -> Self {
        let marge = ZONE_POIGNEE / 2.0;
        let (x0, x1) = (zone.left() + marge, zone.right() - marge);
        let barre = Rect::from_min_max(
            pos2(x0, zone.top() + BULLE + ECART_BULLE + DEPASSE),
            pos2(x1, zone.top() + BULLE + ECART_BULLE + DEPASSE + BARRE),
        );
        Self {
            zone,
            barre,
            x0,
            x1,
            max,
        }
    }

    fn x_de(&self, v: u8) -> f32 {
        self.x0 + (self.x1 - self.x0) * f32::from(v) / f32::from(self.max)
    }

    fn valeur_de(&self, x: f32) -> i32 {
        (((x - self.x0) / (self.x1 - self.x0).max(1.0)) * f32::from(self.max)).round() as i32
    }
}

/// Les zones colorées (arrondies aux deux bouts seulement), le cadre et les repères des bouts.
fn dessiner_zones(ui: &Ui, p: &Palette, g: &Gabarit, zones: &[Zone<'_>], seuils: &[u8], unite: &str) {
    let limites: Vec<f32> = std::iter::once(g.x0)
        .chain(seuils.iter().map(|&s| g.x_de(s)))
        .chain(std::iter::once(g.x1))
        .collect();
    for (n, z) in zones.iter().enumerate() {
        let (a, b) = (limites[n], limites[n + 1]);
        if b - a < 0.5 {
            continue;
        }
        let rond = |haut: bool| if haut { 6 } else { 0 };
        let arrondi = CornerRadius {
            nw: rond(n == 0),
            sw: rond(n == 0),
            ne: rond(n + 1 == zones.len()),
            se: rond(n + 1 == zones.len()),
        };
        let r = Rect::from_min_max(pos2(a, g.barre.top()), pos2(b, g.barre.bottom()));
        ui.painter().rect_filled(r, arrondi, couleur(z.couleur));
    }
    ui.painter().rect_stroke(
        g.barre,
        CornerRadius::same(6),
        Stroke::new(1.0, couleur(p.bord_controle)),
        StrokeKind::Inside,
    );
    for (v, ancre, x) in [(0, Align2::LEFT_TOP, g.x0), (g.max, Align2::RIGHT_TOP, g.x1)] {
        ui.painter().text(
            pos2(x, g.barre.bottom() + DEPASSE + 4.0),
            ancre,
            texte_valeur(v, unite),
            FontId::proportional(crate::theme::TAILLE_PETIT),
            couleur(p.texte_secondaire),
        );
    }
}

/// La poignée `i` suit la souris ou le clavier ; rend vrai si sa limite a bougé.
fn suivre_poignee(
    ui: &Ui,
    g: &Gabarit,
    r: &egui::Response,
    i: usize,
    seuils: &mut [u8],
    options: &Options<'_>,
) -> bool {
    let mut voulu = i32::from(seuils[i]);
    if r.dragged()
        && let Some(pos) = r.interact_pointer_pos()
    {
        voulu = g.valeur_de(pos.x);
    }
    if r.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                r.id,
                EventFilter {
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    ..Default::default()
                },
            );
        });
        let (pas, borne) = touches(ui);
        voulu = voulu_au_clavier(seuils, i, voulu, pas, borne, options.ecart_min, g.max);
    }
    // Rien n'est demandé : une valeur écrite à la main (deux limites au même endroit, une limite au
    // bout) reste telle quelle, elle n'est jamais bornée du seul fait d'être montrée.
    voulu != i32::from(seuils[i]) && deplacer(seuils, i, voulu, options.ecart_min, g.max)
}

/// Le dessin d'une poignée : un nombre dans une bulle, puis la poignée en travers de la barre.
fn dessiner_poignee(ui: &Ui, p: &Palette, g: &Gabarit, r: &egui::Response, valeur: u8, unite: &str) {
    let x = g.x_de(valeur);
    let actif = r.hovered() || r.dragged();
    let police = semibold(crate::theme::TAILLE_PETIT + 1.0);
    let galerie = ui
        .painter()
        .layout_no_wrap(texte_valeur(valeur, unite), police, couleur(p.texte));
    let taille = galerie.size() + vec2(16.0, 0.0);
    let bulle = Rect::from_center_size(
        pos2(x, g.zone.top() + BULLE / 2.0),
        vec2(taille.x.max(44.0), BULLE),
    );
    // La bulle reste dans la zone : jamais coupée au bord de la page.
    let decalage = (g.zone.left() - bulle.left()).max(0.0) - (bulle.right() - g.zone.right()).max(0.0);
    let bulle = bulle.translate(vec2(decalage, 0.0));
    ui.painter().rect_filled(
        bulle,
        CornerRadius::same(4),
        couleur(if actif { p.survol } else { p.controle }),
    );
    ui.painter().rect_stroke(
        bulle,
        CornerRadius::same(4),
        Stroke::new(1.0, couleur(p.bord_controle)),
        StrokeKind::Inside,
    );
    ui.painter()
        .galley(bulle.center() - galerie.size() / 2.0, galerie, couleur(p.texte));
    let pastille = Rect::from_center_size(
        pos2(x, g.barre.center().y),
        vec2(if actif { POIGNEE + 2.0 } else { POIGNEE }, BARRE + 2.0 * DEPASSE),
    );
    ui.painter()
        .rect_filled(pastille, CornerRadius::same(4), couleur(p.texte));
    ui.painter().rect_stroke(
        pastille,
        CornerRadius::same(4),
        Stroke::new(1.5, couleur(p.fond)),
        StrokeKind::Outside,
    );
    if r.has_focus() {
        ui.painter().rect_stroke(
            pastille.expand(4.0),
            CornerRadius::same(7),
            Stroke::new(2.0, couleur(p.texte)),
            StrokeKind::Outside,
        );
    }
}

// Un texte lisible (noir ou blanc) sur la couleur d'une zone : dans le cœur (bloc F), repris
// ici à son chemin d'aujourd'hui.
pub use ic_coeur::theme::texte_sur;

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, RawInput};

    fn palette() -> Palette {
        Palette::de(&crate::theme::ChoixWindows::default())
    }

    // Le déplacement d'une limite et ses bornes sont essayés dans le cœur
    // (`ic_coeur::gestes::seuils`).
    #[test]
    fn les_nombres_ont_une_espace_insecable() {
        assert_eq!(texte_valeur(30, "%"), "30\u{a0}%");
        assert_eq!(texte_valeur(30, ""), "30");
    }

    const LARGEUR: f32 = 432.0;

    /// Une image de la barre (trois zones) dans une fenêtre de `LARGEUR` points de large.
    fn image(ctx: &Context, events: Vec<Event>, seuils: &mut [u8; 2]) -> ReponseSeuils {
        let p = palette();
        let zones = [
            Zone {
                nom: "Critique",
                couleur: Rgb(200, 0, 0),
            },
            Zone {
                nom: "Bas",
                couleur: Rgb(220, 200, 0),
            },
            Zone {
                nom: "Bon",
                couleur: Rgb(0, 160, 0),
            },
        ];
        let mut sortie = ReponseSeuils::default();
        let mut rendu = ctx.run_ui(
            RawInput {
                events,
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(LARGEUR, 300.0))),
                ..Default::default()
            },
            |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                ui.set_width(LARGEUR);
                sortie = barre_seuils(ui, &p, "essai", &zones, seuils, &Options::default());
            },
        );
        rendu.textures_delta.clear();
        sortie
    }

    /// Où est la poignée `v` (la barre commence à 16 points et s'étend sur `LARGEUR - 32`).
    fn x_de(v: u8) -> f32 {
        16.0 + (LARGEUR - 32.0) * f32::from(v) / 100.0
    }

    fn y_barre() -> f32 {
        BULLE + ECART_BULLE + DEPASSE + BARRE / 2.0
    }

    fn bouton(pos: egui::Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn glisser_une_poignee_deplace_sa_limite_et_dit_le_relachement() {
        let ctx = Context::default();
        let mut s = [20u8, 50];
        let _ = image(&ctx, vec![], &mut s);
        let debut = pos2(x_de(20), y_barre());
        let _ = image(&ctx, vec![Event::PointerMoved(debut)], &mut s);
        let _ = image(&ctx, vec![bouton(debut, true)], &mut s);
        let loin = pos2(x_de(35), y_barre());
        let r = image(&ctx, vec![Event::PointerMoved(loin)], &mut s);
        assert!(r.change && r.glisse, "{r:?}");
        assert_eq!(s, [35, 50]);
        let r = image(&ctx, vec![bouton(loin, false)], &mut s);
        assert!(r.termine && !r.glisse, "{r:?}");
        assert_eq!(s, [35, 50]);
    }

    #[test]
    fn une_poignee_poussee_contre_l_autre_s_arrete_a_cote() {
        let ctx = Context::default();
        let mut s = [20u8, 50];
        let _ = image(&ctx, vec![], &mut s);
        let debut = pos2(x_de(20), y_barre());
        let _ = image(&ctx, vec![Event::PointerMoved(debut)], &mut s);
        let _ = image(&ctx, vec![bouton(debut, true)], &mut s);
        let _ = image(&ctx, vec![Event::PointerMoved(pos2(x_de(90), y_barre()))], &mut s);
        assert_eq!(s, [49, 50]);
    }

    fn touche(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    /// Donne le focus à la première poignée en cliquant dessus, sans la déplacer.
    fn focaliser(ctx: &Context, s: &mut [u8; 2]) {
        let _ = image(ctx, vec![], s);
        let au = pos2(x_de(s[0]), y_barre());
        let _ = image(ctx, vec![Event::PointerMoved(au)], s);
        let _ = image(ctx, vec![bouton(au, true)], s);
        let _ = image(ctx, vec![bouton(au, false)], s);
        // Une image de plus : la poignée a le focus et verrouille les flèches pour elle.
        let _ = image(ctx, vec![], s);
    }

    #[test]
    fn fleches_pages_debut_et_fin_deplacent_la_poignee() {
        let ctx = Context::default();
        let mut s = [20u8, 50];
        focaliser(&ctx, &mut s);
        assert_eq!(s, [20, 50]);
        let r = image(&ctx, vec![touche(Key::ArrowRight, Modifiers::NONE)], &mut s);
        assert!(r.change && !r.glisse);
        assert_eq!(s, [21, 50]);
        let _ = image(&ctx, vec![touche(Key::ArrowLeft, Modifiers::SHIFT)], &mut s);
        assert_eq!(s, [11, 50]);
        let _ = image(&ctx, vec![touche(Key::PageUp, Modifiers::NONE)], &mut s);
        assert_eq!(s, [21, 50]);
        let _ = image(&ctx, vec![touche(Key::End, Modifiers::NONE)], &mut s);
        assert_eq!(s, [49, 50], "la fin, c'est contre l'autre limite");
        let _ = image(&ctx, vec![touche(Key::Home, Modifiers::NONE)], &mut s);
        assert_eq!(s, [0, 50]);
    }

    /// Des seuils écrits à la main (collés l'un à l'autre, contre un bout) sont dessinés tels quels :
    /// la barre ne les change pas toute seule.
    #[test]
    fn des_valeurs_dessinees_ne_sont_jamais_corrigees() {
        let ctx = Context::default();
        for mut s in [[0u8, 0], [100, 100], [50, 50], [0, 100]] {
            let avant = s;
            let r = image(&ctx, vec![], &mut s);
            let r2 = image(&ctx, vec![], &mut s);
            assert_eq!(
                (r, r2),
                (ReponseSeuils::default(), ReponseSeuils::default()),
                "{avant:?}"
            );
            assert_eq!(s, avant);
        }
    }

    #[test]
    fn rien_ne_bouge_sans_focus_ni_souris() {
        let ctx = Context::default();
        let mut s = [20u8, 50];
        let _ = image(&ctx, vec![], &mut s);
        let r = image(&ctx, vec![touche(Key::ArrowRight, Modifiers::NONE)], &mut s);
        assert_eq!(r, ReponseSeuils::default());
        assert_eq!(s, [20, 50]);
    }

    #[test]
    #[should_panic(expected = "une limite entre deux zones")]
    fn les_limites_vont_avec_les_zones() {
        let ctx = Context::default();
        let p = palette();
        let _ = ctx.run_ui(RawInput::default(), |ui| {
            let mut s = [10u8];
            let zones = [Zone {
                nom: "a",
                couleur: Rgb(0, 0, 0),
            }; 3];
            barre_seuils(ui, &p, "x", &zones, &mut s, &Options::default());
        });
    }
}
