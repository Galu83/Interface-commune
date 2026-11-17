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

use crate::apparence::couleur;
use crate::polices::semibold;
use crate::theme::{Palette, Rgb, contraste};

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

/// Les bornes entre lesquelles la limite `i` peut aller, ses voisines étant à leur place.
fn bornes(seuils: &[u8], i: usize, ecart: u8, max: u8) -> (u8, u8) {
    let bas = if i == 0 {
        0
    } else {
        seuils[i - 1].saturating_add(ecart).min(max)
    };
    let haut = if i + 1 == seuils.len() {
        max
    } else {
        seuils[i + 1].saturating_sub(ecart)
    };
    (bas, haut.max(bas))
}

/// Met la limite `i` à `voulu`, ramené entre ses voisines. Rend vrai si elle a bougé.
pub fn deplacer(seuils: &mut [u8], i: usize, voulu: i32, ecart: u8, max: u8) -> bool {
    let (bas, haut) = bornes(seuils, i, ecart, max);
    let v = voulu.clamp(i32::from(bas), i32::from(haut)) as u8;
    let change = seuils[i] != v;
    seuils[i] = v;
    change
}

/// Le nombre écrit au-dessus d'une poignée, avec son unité (espace insécable avant l'unité).
fn texte_valeur(v: u8, unite: &str) -> String {
    if unite.is_empty() {
        v.to_string()
    } else {
        format!("{v}\u{a0}{unite}")
    }
}

/// Le pas demandé par les touches de cette image (en négatif vers la gauche), et si on veut
/// une borne (`Some(false)` : le début, `Some(true)` : la fin).
fn touches(ui: &Ui) -> (i32, Option<bool>) {
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
            borne = Some(false);
        }
        if i.consume_key(Modifiers::NONE, Key::End) {
            borne = Some(true);
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
    let max = options.max.max(1);
    let mut reponse = ReponseSeuils::default();
    let hauteur = BULLE + ECART_BULLE + DEPASSE + BARRE + DEPASSE + REPERES;
    let (zone, _) = ui.allocate_exact_size(vec2(ui.available_width(), hauteur), Sense::hover());
    // La barre laisse la place de la moitié d'une poignée à chaque bout.
    let marge = ZONE_POIGNEE / 2.0;
    let (x0, x1) = (zone.left() + marge, zone.right() - marge);
    let barre = Rect::from_min_max(
        pos2(x0, zone.top() + BULLE + ECART_BULLE + DEPASSE),
        pos2(x1, zone.top() + BULLE + ECART_BULLE + DEPASSE + BARRE),
    );
    let x_de = |v: u8| x0 + (x1 - x0) * f32::from(v) / f32::from(max);
    let valeur_de = |x: f32| (((x - x0) / (x1 - x0).max(1.0)) * f32::from(max)).round() as i32;

    // Les zones colorées : arrondies aux deux bouts seulement.
    if ui.is_rect_visible(zone) {
        let limites: Vec<f32> = std::iter::once(x0)
            .chain(seuils.iter().map(|&s| x_de(s)))
            .chain(std::iter::once(x1))
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
            let r = Rect::from_min_max(pos2(a, barre.top()), pos2(b, barre.bottom()));
            ui.painter().rect_filled(r, arrondi, couleur(z.couleur));
        }
        ui.painter().rect_stroke(
            barre,
            CornerRadius::same(6),
            Stroke::new(1.0, couleur(p.bord_controle)),
            StrokeKind::Inside,
        );
        for (v, ancre, x) in [(0, Align2::LEFT_TOP, x0), (max, Align2::RIGHT_TOP, x1)] {
            ui.painter().text(
                pos2(x, barre.bottom() + DEPASSE + 4.0),
                ancre,
                texte_valeur(v, options.unite),
                FontId::proportional(crate::theme::TAILLE_PETIT),
                couleur(p.texte_secondaire),
            );
        }
    }

    // Les poignées, de la dernière à la première : celle de gauche reste prise si deux se
    // touchent (la seconde se dégage en la poussant à droite).
    for i in (0..seuils.len()).rev() {
        let centre = pos2(x_de(seuils[i]), barre.center().y);
        let cliquable = Rect::from_center_size(centre, vec2(ZONE_POIGNEE, BARRE + 2.0 * DEPASSE + 4.0));
        let r = ui.interact(cliquable, ui.make_persistent_id((id, i)), Sense::click_and_drag());
        let nom = format!("{} / {}", zones[i].nom, zones[i + 1].nom);
        let r = r.on_hover_cursor(egui::CursorIcon::ResizeHorizontal);
        let mut voulu = i32::from(seuils[i]);
        if r.dragged()
            && let Some(pos) = r.interact_pointer_pos()
        {
            voulu = valeur_de(pos.x);
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
            let (bas, haut) = bornes(seuils, i, options.ecart_min, max);
            match borne {
                Some(false) => voulu = i32::from(bas),
                Some(true) => voulu = i32::from(haut),
                None => voulu += pas,
            }
        }
        // Rien n'est demandé : une valeur écrite à la main (deux limites au même endroit, une limite au
        // bout) reste telle quelle, elle n'est jamais bornée du seul fait d'être montrée.
        if voulu != i32::from(seuils[i]) && deplacer(seuils, i, voulu, options.ecart_min, max) {
            reponse.change = true;
        }
        reponse.glisse |= r.dragged();
        reponse.termine |= r.drag_stopped();
        let valeur = seuils[i];
        r.widget_info(|| egui::WidgetInfo::slider(true, f64::from(valeur), &nom));
        if r.clicked() || r.drag_started() {
            r.request_focus();
        }

        // Le dessin : un nombre dans une bulle, puis la poignée en travers de la barre.
        if ui.is_rect_visible(cliquable) {
            let x = x_de(valeur);
            let actif = r.hovered() || r.dragged();
            let police = semibold(crate::theme::TAILLE_PETIT + 1.0);
            let galerie =
                ui.painter()
                    .layout_no_wrap(texte_valeur(valeur, options.unite), police, couleur(p.texte));
            let taille = galerie.size() + vec2(16.0, 0.0);
            let bulle =
                Rect::from_center_size(pos2(x, zone.top() + BULLE / 2.0), vec2(taille.x.max(44.0), BULLE));
            // La bulle reste dans la zone : jamais coupée au bord de la page.
            let decalage = (zone.left() - bulle.left()).max(0.0) - (bulle.right() - zone.right()).max(0.0);
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
                pos2(x, barre.center().y),
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
    }
    reponse
}

/// Un texte lisible (noir ou blanc) sur la couleur d'une zone : pour écrire dans une zone ou
/// sur une pastille de sa couleur.
pub fn texte_sur(zone: Rgb) -> Rgb {
    let (noir, blanc) = (Rgb(0, 0, 0), Rgb(255, 255, 255));
    if contraste(zone, noir) >= contraste(zone, blanc) {
        noir
    } else {
        blanc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, PointerButton, RawInput};

    fn palette() -> Palette {
        Palette::de(&crate::theme::ChoixWindows::default())
    }

    #[test]
    fn a_limit_stays_between_its_neighbours() {
        let mut s = [20u8, 50];
        assert!(deplacer(&mut s, 0, 90, 1, 100));
        assert_eq!(s, [49, 50]);
        assert!(
            !deplacer(&mut s, 1, 0, 1, 100),
            "la deuxième ne passe pas la première"
        );
        assert_eq!(s, [49, 50]);
        assert!(deplacer(&mut s, 0, -30, 1, 100));
        assert_eq!(s, [0, 50]);
        assert!(deplacer(&mut s, 1, 400, 1, 100));
        assert_eq!(s, [0, 100]);
        assert!(!deplacer(&mut s, 1, 100, 1, 100), "rien n'a bougé");
    }

    #[test]
    fn two_limits_may_touch_without_a_gap() {
        let mut s = [20u8, 50];
        assert!(deplacer(&mut s, 0, 80, 0, 100));
        assert_eq!(s, [50, 50]);
    }

    #[test]
    fn values_are_written_with_a_non_breaking_space() {
        assert_eq!(texte_valeur(30, "%"), "30\u{a0}%");
        assert_eq!(texte_valeur(30, ""), "30");
    }

    #[test]
    fn text_on_a_zone_is_readable() {
        assert_eq!(texte_sur(Rgb(255, 255, 0)), Rgb(0, 0, 0));
        assert_eq!(texte_sur(Rgb(0, 40, 160)), Rgb(255, 255, 255));
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
    fn dragging_a_handle_moves_its_limit_and_tells_when_it_is_released() {
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
    fn a_handle_dragged_past_the_other_stops_next_to_it() {
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
    fn arrows_page_keys_and_home_end_move_the_focused_handle() {
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
    fn values_that_are_already_there_are_never_corrected_by_being_drawn() {
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
    fn nothing_moves_without_focus_or_pointer() {
        let ctx = Context::default();
        let mut s = [20u8, 50];
        let _ = image(&ctx, vec![], &mut s);
        let r = image(&ctx, vec![touche(Key::ArrowRight, Modifiers::NONE)], &mut s);
        assert_eq!(r, ReponseSeuils::default());
        assert_eq!(s, [20, 50]);
    }

    #[test]
    #[should_panic(expected = "une limite entre deux zones")]
    fn the_limits_must_match_the_zones() {
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
