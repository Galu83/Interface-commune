//! Les éléments dessinés à la façon de Windows 11 : ligne de menu, interrupteur, carte de
//! réglage, boutons, titres. Zones cliquables d'au moins 32 points, contour visible sur
//! l'élément qui a le focus du clavier (règles : interface Windows 11, points 5 et 8).

use egui::{Align2, CornerRadius, FontId, Response, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::apparence::couleur;
use crate::icones;
use crate::polices::{icones as police_icones, semibold, titre};
use crate::textes::Textes;
use crate::theme::{Palette, Rgb};

/// Contour de focus (clavier) : 2 px de la couleur du texte, comme Windows.
pub fn focus(ui: &Ui, p: &Palette, r: &Response, arrondi: u8) {
    if r.has_focus() {
        ui.painter().rect_stroke(
            r.rect.expand(1.0),
            CornerRadius::same(arrondi + 1),
            Stroke::new(2.0, couleur(p.texte)),
            StrokeKind::Outside,
        );
    }
}

/// Une icône de Segoe Fluent Icons, centrée en `centre`.
pub fn icone(ui: &Ui, centre: egui::Pos2, glyphe: char, taille: f32, c: Rgb) {
    ui.painter().text(
        centre,
        Align2::CENTER_CENTER,
        glyphe,
        police_icones(taille),
        couleur(c),
    );
}

use crate::mesures::{
    LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_ICONE_X, LIGNE_MARGE_DROITE, LIGNE_TEXTE_X, MENU_LIGNE,
    VOLET_LIGNE,
};

/// La largeur qu'il faut à une ligne de menu pour tout montrer sans couper : `texte` et
/// `droite`, les largeurs mesurées de ses deux textes.
pub fn largeur_ligne(texte: f32, droite: Option<f32>) -> f32 {
    LIGNE_TEXTE_X + texte + droite.map_or(0.0, |d| LIGNE_ECART + d) + LIGNE_MARGE_DROITE
}

/// Une ligne du volet sur toute la largeur ([`VOLET_LIGNE`]) : icône, texte, et à droite une
/// aide discrète. L'aide de droite est toujours entière ; un texte trop long pour la place qui
/// reste finit par « … » (le nom entier reste dans l'info-bulle).
pub fn ligne(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    couleur_icone: Rgb,
    texte: &str,
    droite: Option<&str>,
) -> Response {
    ligne_de(ui, p, glyphe, couleur_icone, texte, droite, VOLET_LIGNE)
}

/// Une ligne du petit menu ([`MENU_LIGNE`], comme les menus de Windows 11).
pub fn ligne_menu(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str) -> Response {
    ligne_de(ui, p, glyphe, p.texte, texte, None, MENU_LIGNE)
}

/// Une ligne de cette hauteur ; `glyphe` espace : sans icône.
pub fn ligne_de(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    couleur_icone: Rgb,
    texte: &str,
    droite: Option<&str>,
    hauteur: f32,
) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), hauteur), Sense::click());
    let droite = droite.map(|d| {
        ui.painter().layout_no_wrap(
            d.to_string(),
            FontId::proportional(12.0),
            couleur(p.texte_secondaire),
        )
    });
    let place = rect.width()
        - LIGNE_TEXTE_X
        - LIGNE_MARGE_DROITE
        - droite.as_ref().map_or(0.0, |d| LIGNE_ECART + d.size().x);
    let mut job = egui::text::LayoutJob::simple_singleline(
        texte.to_string(),
        FontId::proportional(14.0),
        couleur(p.texte),
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(place.max(1.0));
    let galley = ui.painter().layout_job(job);
    let r = if galley.elided { r.on_hover_text(texte) } else { r };
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    if ui.is_rect_visible(rect) {
        let fond = if r.is_pointer_button_down_on() {
            Some(p.appui)
        } else if r.hovered() || r.has_focus() {
            Some(p.survol)
        } else {
            None
        };
        if let Some(f) = fond {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(LIGNE_ARRONDI), couleur(f));
        }
        if glyphe != ' ' {
            icone(
                ui,
                pos2(rect.left() + LIGNE_ICONE_X, rect.center().y),
                glyphe,
                LIGNE_ICONE,
                couleur_icone,
            );
        }
        let haut = rect.center().y - galley.size().y / 2.0;
        ui.painter()
            .galley(pos2(rect.left() + LIGNE_TEXTE_X, haut), galley, couleur(p.texte));
        if let Some(d) = droite {
            let pos = pos2(
                rect.right() - LIGNE_MARGE_DROITE - d.size().x,
                rect.center().y - d.size().y / 2.0,
            );
            ui.painter().galley(pos, d, couleur(p.texte_secondaire));
        }
        focus(ui, p, &r, 4);
    }
    r
}

/// Un bouton carré qui n'est qu'une icône (la roue dentée du volet), 36 × 36 : survol
/// arrondi, contour de focus, et son nom pour le lecteur d'écran et l'info-bulle.
pub fn bouton_icone(ui: &mut Ui, p: &Palette, glyphe: char, nom: &str) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::click());
    let r = r.on_hover_text(nom);
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, nom));
    if ui.is_rect_visible(rect) {
        let fond = if r.is_pointer_button_down_on() {
            Some(p.appui)
        } else if r.hovered() || r.has_focus() {
            Some(p.survol)
        } else {
            None
        };
        if let Some(f) = fond {
            ui.painter().rect_filled(rect, CornerRadius::same(4), couleur(f));
        }
        icone(ui, rect.center(), glyphe, 16.0, p.texte);
        focus(ui, p, &r, 4);
    }
    r
}

/// Un interrupteur Windows 11 (40 × 20).
pub fn interrupteur(ui: &mut Ui, p: &Palette, actif: &mut bool) -> Response {
    let (rect, mut r) = ui.allocate_exact_size(vec2(40.0, 32.0), Sense::click());
    if r.clicked() {
        *actif = !*actif;
        r.mark_changed();
    }
    let piste = egui::Rect::from_center_size(rect.center(), vec2(40.0, 20.0));
    let t = ui.ctx().animate_bool_responsive(r.id, *actif);
    let survol = r.hovered();
    if *actif {
        let fond = if survol {
            p.accent.vers(p.texte, 0.1)
        } else {
            p.accent
        };
        ui.painter()
            .rect_filled(piste, CornerRadius::same(10), couleur(fond));
    } else {
        ui.painter().rect_filled(
            piste,
            CornerRadius::same(10),
            couleur(if survol { p.survol } else { p.controle }),
        );
        ui.painter().rect_stroke(
            piste,
            CornerRadius::same(10),
            Stroke::new(1.0, couleur(p.texte_secondaire)),
            StrokeKind::Inside,
        );
    }
    let x = egui::lerp(piste.left() + 10.0..=piste.right() - 10.0, t);
    let pastille = if *actif { p.sur_accent } else { p.texte_secondaire };
    let rayon = if survol { 7.0 } else { 6.0 };
    ui.painter()
        .circle_filled(pos2(x, piste.center().y), rayon, couleur(pastille));
    focus(ui, p, &r, 10);
    r.widget_info(|| egui::WidgetInfo::selected(egui::Role::CheckBox, true, *actif, ""));
    r
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Normal,
    Accent,
    /// Action qui détruit : texte rouge.
    Danger,
}

/// Un bouton de 32 px de haut, avec une icône facultative.
pub fn bouton(ui: &mut Ui, p: &Palette, glyphe: Option<char>, texte: &str, genre: Genre) -> Response {
    let police = FontId::proportional(14.0);
    let largeur_texte = ui
        .painter()
        .layout_no_wrap(texte.to_string(), police.clone(), couleur(p.texte))
        .size()
        .x;
    let avec_icone = if glyphe.is_some() { 24.0 } else { 0.0 };
    let largeur = (largeur_texte + avec_icone + 28.0).max(if texte.is_empty() { 32.0 } else { 80.0 });
    let (rect, r) = ui.allocate_exact_size(vec2(largeur, 32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let (fond, bord, texte_c) = match genre {
            Genre::Accent => (
                if r.hovered() {
                    p.accent.vers(p.fond, 0.1)
                } else {
                    p.accent
                },
                p.accent,
                p.sur_accent,
            ),
            Genre::Normal | Genre::Danger => (
                if r.is_pointer_button_down_on() {
                    p.appui
                } else if r.hovered() {
                    p.survol
                } else {
                    p.controle
                },
                p.bord_controle,
                if genre == Genre::Danger { p.rouge } else { p.texte },
            ),
        };
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), couleur(fond));
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(4),
            Stroke::new(1.0, couleur(bord)),
            StrokeKind::Inside,
        );
        let mut x = rect.center().x - (largeur_texte + avec_icone) / 2.0;
        if let Some(g) = glyphe {
            icone(ui, pos2(x + 8.0, rect.center().y), g, 16.0, texte_c);
            x += avec_icone;
        }
        ui.painter().text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            texte,
            police,
            couleur(texte_c),
        );
        focus(ui, p, &r, 4);
    }
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    r
}

/// Un interrupteur avec « Activé » ou « Désactivé » écrit à sa gauche (règles : point 7).
pub fn interrupteur_ecrit(ui: &mut Ui, p: &Palette, t: &Textes, actif: &mut bool) -> Response {
    ui.horizontal(|ui| {
        let r = interrupteur(ui, p, actif);
        ui.label(if *actif { t.active } else { t.desactive });
        r
    })
    .inner
}

/// Ce que fait un champ de raccourci clavier à cette image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saisie {
    Rien,
    /// La saisie commence : le logiciel suspend ses propres raccourcis jusqu'à la fin.
    Commencee,
    /// Échap : la valeur n'a pas changé.
    Annulee,
    /// Un nouveau raccourci est écrit dans la valeur.
    Changee,
}

/// Un raccourci saisi, au format « ctrl+alt+shift+k » (repris de Vocalis).
pub fn raccourci_de(m: egui::Modifiers, touche: egui::Key) -> String {
    let mut parties = Vec::new();
    if m.ctrl || m.command {
        parties.push("ctrl".to_string());
    }
    if m.alt {
        parties.push("alt".into());
    }
    if m.shift {
        parties.push("shift".into());
    }
    parties.push(touche.name().to_lowercase());
    parties.join("+")
}

/// Un champ de raccourci clavier : la valeur et « Changer… » ; un clic lance la saisie, la
/// touche suivante (avec Ctrl, Alt, Maj) devient la valeur, Échap annule. `id` : unique dans
/// la fenêtre (la saisie en cours est retenue par egui).
pub fn champ_raccourci(ui: &mut Ui, p: &Palette, t: &Textes, id: &str, valeur: &mut String) -> Saisie {
    let id = ui.scope_id().with(("raccourci", id));
    let en_cours = ui.memory(|m| m.data.get_temp::<bool>(id).unwrap_or(false));
    if en_cours {
        ui.label(egui::RichText::new(t.appuyer_touches).color(couleur(p.accent)));
        let touche = ui.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => Some((*key, *modifiers)),
                _ => None,
            })
        });
        let Some((key, m)) = touche else {
            return Saisie::Rien;
        };
        ui.memory_mut(|mem| mem.data.remove::<bool>(id));
        if key == egui::Key::Escape {
            return Saisie::Annulee;
        }
        *valeur = raccourci_de(m, key);
        return Saisie::Changee;
    }
    let mut saisie = Saisie::Rien;
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(valeur.as_str()).strong());
        if bouton(ui, p, Some(icones::CLAVIER), t.changer_raccourci, Genre::Normal).clicked() {
            ui.memory_mut(|mem| mem.data.insert_temp(id, true));
            saisie = Saisie::Commencee;
        }
    });
    saisie
}

/// Titre de page (28 points) et une ligne d'explication.
pub fn titre_page(ui: &mut Ui, p: &Palette, texte: &str, explication: &str) {
    ui.label(egui::RichText::new(texte).font(titre(crate::theme::TAILLE_TITRE)));
    if !explication.is_empty() {
        ui.label(egui::RichText::new(explication).color(couleur(p.texte_secondaire)));
    }
    ui.add_space(8.0);
}

/// Titre de section (15 points semibold).
pub fn titre_section(ui: &mut Ui, texte: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(texte).font(semibold(crate::theme::TAILLE_SECTION)));
}

/// Le cadre d'une carte (8 points d'arrondi).
pub fn cadre_carte(p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(couleur(p.carte))
        .stroke(Stroke::new(1.0, couleur(p.bord_carte)))
        .corner_radius(CornerRadius::same(crate::theme::ARRONDI_CARTE))
        .inner_margin(egui::Margin::symmetric(16, 10))
}

/// Une carte de réglage : icône, titre, une ligne d'explication, contrôle à droite.
pub fn carte(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    texte: &str,
    explication: &str,
    controle: impl FnOnce(&mut Ui),
) {
    carte_avec(ui, p, glyphe, texte, explication, 200.0, controle);
}

/// Une carte dont les contrôles prennent `place` points (plusieurs boutons).
pub fn carte_large(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    texte: &str,
    explication: &str,
    place: f32,
    controle: impl FnOnce(&mut Ui),
) {
    carte_avec(ui, p, glyphe, texte, explication, place, controle);
}

/// Une carte d'information, sans contrôle : le texte prend toute la largeur.
pub fn carte_info(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str, explication: &str) {
    carte_avec(ui, p, glyphe, texte, explication, 0.0, |_| {});
}

/// Une carte qui ouvre une sous-page : toute la carte se clique, un chevron « › » à droite.
pub fn carte_chevron(ui: &mut Ui, p: &Palette, glyphe: char, texte: &str, explication: &str) -> bool {
    let r = cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.set_min_height(48.0);
            let (r, _) = ui.allocate_exact_size(Vec2::new(28.0, 32.0), Sense::hover());
            icone(ui, r.center(), glyphe, 20.0, p.texte);
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.add_space(4.0);
                ui.label(texte);
                if !explication.is_empty() {
                    ui.label(
                        egui::RichText::new(explication)
                            .size(crate::theme::TAILLE_PETIT)
                            .color(couleur(p.texte_secondaire)),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(vec2(24.0, 32.0), Sense::hover());
                icone(ui, r.center(), icones::CHEVRON, 12.0, p.texte_secondaire);
            });
        });
    });
    let r = ui.interact(
        r.response.rect,
        ui.scope_id().with(("chevron", texte)),
        Sense::click(),
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, texte));
    focus(ui, p, &r, crate::theme::ARRONDI_CARTE);
    r.clicked()
}

fn carte_avec(
    ui: &mut Ui,
    p: &Palette,
    glyphe: char,
    texte: &str,
    explication: &str,
    place_controle: f32,
    controle: impl FnOnce(&mut Ui),
) {
    cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.set_min_height(48.0);
            let (r, _) = ui.allocate_exact_size(Vec2::new(28.0, 32.0), Sense::hover());
            icone(ui, r.center(), glyphe, 20.0, p.texte);
            ui.add_space(8.0);
            let largeur_texte = (ui.available_width() - place_controle).max(160.0);
            ui.vertical(|ui| {
                ui.set_width(largeur_texte);
                ui.add_space(4.0);
                ui.label(texte);
                if !explication.is_empty() {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(insecables(explication))
                                .size(crate::theme::TAILLE_PETIT)
                                .color(couleur(p.texte_secondaire)),
                        )
                        .wrap(),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), controle);
        });
    });
}

/// Un bandeau en haut d'une page (redémarrage nécessaire, avertissement) : fond teinté de la
/// couleur `c`, icône, texte, et un bouton à droite. Rend `true` quand le bouton est cliqué.
pub fn bandeau(ui: &mut Ui, p: &Palette, c: Rgb, glyphe: char, texte: &str, bouton_texte: &str) -> bool {
    let mut clic = false;
    egui::Frame::new()
        .fill(couleur(p.carte.vers(c, 0.15)))
        .stroke(Stroke::new(1.0, couleur(p.bord_carte.vers(c, 0.4))))
        .corner_radius(CornerRadius::same(crate::theme::ARRONDI_CARTE))
        .inner_margin(egui::Margin::symmetric(16, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(20.0, 32.0), Sense::hover());
                icone(ui, r.center(), glyphe, 16.0, c);
                ui.label(texte);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    clic = bouton(ui, p, None, bouton_texte, Genre::Accent).clicked();
                });
            });
        });
    clic
}

/// Pour un texte qui passe à la ligne : les espaces collées aux guillemets et devant « : ; ! ? »
/// deviennent insécables, comme en typographie française (jamais un « » » seul en début de
/// ligne).
pub fn insecables(texte: &str) -> String {
    const INSECABLE: char = '\u{a0}';
    let mut s = String::with_capacity(texte.len());
    let mut car = texte.chars().peekable();
    let mut avant: Option<char> = None;
    while let Some(c) = car.next() {
        let colle = c == ' '
            && (avant == Some('«')
                || car
                    .peek()
                    .is_some_and(|n| matches!(n, '»' | ':' | ';' | '!' | '?')));
        s.push(if colle { INSECABLE } else { c });
        avant = Some(c);
    }
    s
}

/// La place de l'icône d'un état, devant son texte (points).
pub const ETAT_ICONE: f32 = 18.0;

/// Un état (icône de couleur et texte de la même couleur) qui passe à la ligne s'il est trop
/// long : l'icône reste en face de la première ligne (le volet de l'icône).
pub fn etat_long(ui: &mut Ui, c: Rgb, glyphe: char, texte: &str) {
    ui.horizontal_top(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(ETAT_ICONE, 20.0), Sense::hover());
        icone(ui, r.center(), glyphe, 14.0, c);
        ui.add(
            egui::Label::new(
                egui::RichText::new(insecables(texte))
                    .color(couleur(c))
                    .font(semibold(14.0)),
            )
            .wrap(),
        );
    });
}

/// Une pastille d'état (rond de couleur) suivie d'un texte de la même couleur.
pub fn etat(ui: &mut Ui, c: Rgb, glyphe: char, texte: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(vec2(18.0, 20.0), Sense::hover());
        icone(ui, r.center(), glyphe, 14.0, c);
        ui.label(egui::RichText::new(texte).color(couleur(c)).font(semibold(14.0)));
    });
}

#[cfg(test)]
mod essais_raccourci {
    use super::*;

    fn image(ctx: &egui::Context, evts: Vec<egui::Event>, valeur: &mut String) -> (Saisie, egui::Rect) {
        let p = Palette::de(&crate::theme::ChoixWindows::default());
        let entree = egui::RawInput {
            events: evts,
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let mut rendu = (Saisie::Rien, egui::Rect::NOTHING);
        let mut sortie = ctx.run_ui(entree, |ui| {
            rendu.0 = champ_raccourci(ui, &p, &crate::textes::FRANCAIS, "dictee", valeur);
            rendu.1 = ui.min_rect();
        });
        sortie.textures_delta.clear();
        rendu
    }

    fn touche(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn cliquer(ctx: &egui::Context, valeur: &mut String) -> Saisie {
        // Le bouton « Changer… » est le dernier élément, à droite de la valeur.
        let (_, r) = image(ctx, vec![], valeur);
        let pos = egui::pos2(r.right() - 10.0, r.center().y);
        let bouton = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        image(ctx, vec![egui::Event::PointerMoved(pos)], valeur);
        image(ctx, vec![bouton(true)], valeur);
        image(ctx, vec![bouton(false)], valeur).0
    }

    /// « Changer… » lance la saisie ; la touche suivante, avec Ctrl et Maj, devient la valeur ;
    /// Échap annule sans rien changer.
    #[test]
    fn saisie_d_un_raccourci() {
        let ctx = egui::Context::default();
        let mut v = "ctrl+alt+d".to_string();
        assert_eq!(cliquer(&ctx, &mut v), Saisie::Commencee);
        let m = egui::Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        assert_eq!(image(&ctx, vec![], &mut v).0, Saisie::Rien);
        assert_eq!(
            image(&ctx, vec![touche(egui::Key::K, m)], &mut v).0,
            Saisie::Changee
        );
        assert_eq!(v, "ctrl+shift+k");
        assert_eq!(cliquer(&ctx, &mut v), Saisie::Commencee);
        assert_eq!(
            image(&ctx, vec![touche(egui::Key::Escape, Default::default())], &mut v).0,
            Saisie::Annulee
        );
        assert_eq!(v, "ctrl+shift+k");
        // La saisie est finie : une touche ne change plus rien.
        assert_eq!(image(&ctx, vec![touche(egui::Key::A, m)], &mut v).0, Saisie::Rien);
        assert_eq!(v, "ctrl+shift+k");
    }
}
