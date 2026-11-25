//! Le menu flottant de choix, au style des menus de Windows 11 : des lignes avec un glyphe sur
//! sa couleur (comme la pastille d'un appareil), un texte, un sous-texte et une coche.
//!
//! Il s'ouvre **dans la fenêtre** (une `egui::Area`), jamais dans une nouvelle fenêtre de
//! Windows, sous la case qui l'ouvre ou au-dessus si la place manque, sans sortir de la fenêtre.
//! Échap ou un clic à côté le ferme ; les flèches, Début, Fin et Entrée le conduisent au
//! clavier ; un contour de focus montre la ligne active.
//!
//! Deux parties, séparées :
//! - la **logique**, des fonctions pures testées sans fenêtre : [`suivre`] (ligne active,
//!   flèches, Début, Fin, Entrée, Échap, lignes grisées sautées), [`ligne_initiale`],
//!   [`placer`] (sous la case ou au-dessus), [`nom_accessible`] ;
//! - le **dessin** : [`afficher`], qui appelle ces fonctions.
//!
//! ```ignore
//! // Dans l'état du logiciel : `menu: MenuChoix`. Dans la page :
//! if r_case.clicked() {
//!     menu.basculer(&lignes);
//! }
//! match menu_choix::afficher(ui, p, &mut menu, id, r_case.rect, &lignes, "coché") {
//!     Some(Resultat::Choisi(rang)) => { /* lignes[rang].cle */ }
//!     Some(Resultat::Ferme) | None => {}
//! }
//! ```
//!
//! Le sens ne repose jamais sur la couleur seule : chaque ligne a son texte, et une coche
//! dessinée pour la ligne choisie. Chaque ligne a un nom pour le lecteur d'écran
//! ([`nom_accessible`] : texte, sous-texte, « coché »).

use egui::{
    Align2, CornerRadius, Event, Id, Key, Modifiers, Order, Rect, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    pos2, vec2,
};

use crate::dessin::couleur::couleur;
use crate::dessin::icone;
use crate::icones;
use crate::mesures::{
    LARGEUR_MAX, LIGNE_ARRONDI, LIGNE_ECART, LIGNE_ICONE, LIGNE_MARGE_DROITE, MENU_ECART, MENU_LARGEUR,
    MENU_LIGNE, MENU_MARGE, PASTILLE_HAUTEUR, PASTILLE_LARGEUR,
};
use crate::pastilles::{Marque, couleur_lettre, marque};
use crate::place::{vers_egui, vers_le_coeur};
use crate::polices::{icone_disponible, semibold};
use crate::theme::{Palette, Rgb, TAILLE_PETIT, TAILLE_TEXTE};

/// Hauteur d'une ligne qui a un sous-texte.
pub const LIGNE_DOUBLE: f32 = 48.0;
// Le vide entre la case et le menu, et entre le menu et le bord de la fenêtre : avec la
// place du menu, du calcul pur, dans le cœur (bloc D).
pub use ic_coeur::placement::plan::{ECART_ANCRE, MARGE_FENETRE};
/// Marge à gauche d'une ligne, et espace entre le carré coloré et le texte.
const MARGE_GAUCHE: f32 = 8.0;
const ESPACE_CARRE: f32 = 10.0;
/// Arrondi du menu (les menus de Windows 11).
const ARRONDI_MENU: u8 = 8;

// ---- Les lignes ------------------------------------------------------------------------

/// Une ligne du menu.
#[derive(Debug, Clone, PartialEq)]
pub struct Ligne {
    /// Ce que le logiciel reçoit quand la ligne est choisie.
    pub cle: String,
    /// Un glyphe de Segoe Fluent Icons dans le carré coloré, à la place de la lettre. Absent
    /// de la police d'icônes : la lettre (jamais un carré vide).
    pub logo: Option<char>,
    /// La lettre du carré (aussi le repli du logo).
    pub lettre: Option<char>,
    /// La couleur du carré. `None` : un carré neutre (le fond des contrôles).
    pub couleur: Option<Rgb>,
    pub texte: String,
    pub sous_texte: Option<String>,
    /// La ligne choisie : une coche dessinée à droite.
    pub coche: bool,
    /// Une ligne grisée : on la voit, on ne la choisit pas, les flèches la sautent.
    pub grise: bool,
}

impl Ligne {
    /// Une ligne simple : une clé et un texte, sans carré, sans coche.
    pub fn new(cle: impl Into<String>, texte: impl Into<String>) -> Ligne {
        Ligne {
            cle: cle.into(),
            logo: None,
            lettre: None,
            couleur: None,
            texte: texte.into(),
            sous_texte: None,
            coche: false,
            grise: false,
        }
    }

    fn a_un_carre(&self) -> bool {
        self.logo.is_some() || self.lettre.is_some() || self.couleur.is_some()
    }

    fn hauteur(&self) -> f32 {
        if self.sous_texte.is_some() {
            LIGNE_DOUBLE
        } else {
            MENU_LIGNE
        }
    }
}

/// Le nom d'une ligne pour le lecteur d'écran : le texte, le sous-texte, puis `mot_coche`
/// (« coché ») si elle est choisie. La couleur n'y est pour rien. Calcul pur, dans le cœur
/// ([`ic_coeur::gestes::choix::nom_accessible`], bloc F).
pub fn nom_accessible(l: &Ligne, mot_coche: &str) -> String {
    ic_coeur::gestes::choix::nom_accessible(&l.texte, l.sous_texte.as_deref(), l.coche, mot_coche)
}

// ---- La logique (pure, dans le cœur) ---------------------------------------------------

// Les touches du menu et ce qu'elles font : du calcul pur, dans le cœur (bloc F), repris ici à
// leurs chemins d'aujourd'hui.
pub use ic_coeur::gestes::choix::{Effet, Touche, suivre};

/// La ligne active à l'ouverture : la ligne cochée si elle est libre, sinon la première
/// ligne libre, sinon aucune ([`ic_coeur::gestes::choix::initiale`]).
pub fn ligne_initiale(lignes: &[Ligne]) -> Option<usize> {
    let cochees: Vec<bool> = lignes.iter().map(|l| l.coche).collect();
    let grisees: Vec<bool> = lignes.iter().map(|l| l.grise).collect();
    ic_coeur::gestes::choix::initiale(&cochees, &grisees)
}

/// Où poser un menu de `taille` pour la case `ancre`, dans `fenetre` : **sous la case** s'il y
/// a la place, sinon **au-dessus**, sinon du côté qui en a le plus (la hauteur est alors
/// réduite, le menu défile) ; jamais hors de la fenêtre. Calcul pur, dans le cœur
/// ([`ic_coeur::placement::plan::placer_le_menu`], bloc D) et testé là.
pub fn placer(ancre: Rect, taille: Vec2, fenetre: Rect) -> Rect {
    let place = ic_coeur::placement::plan::placer_le_menu(
        vers_le_coeur(ancre),
        [taille.x, taille.y],
        vers_le_coeur(fenetre),
    );
    vers_egui(place)
}

/// La hauteur du menu : ses lignes, leurs écarts et ses marges.
pub fn hauteur_totale(lignes: &[Ligne]) -> f32 {
    let somme: f32 = lignes.iter().map(Ligne::hauteur).sum();
    let ecarts = MENU_ECART * lignes.len().saturating_sub(1) as f32;
    somme + ecarts + 2.0 * MENU_MARGE
}

// ---- L'état et le dessin ---------------------------------------------------------------

/// Ce que le menu rend quand il se passe quelque chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resultat {
    /// La ligne de ce rang est choisie (souris, Entrée) ; le menu est fermé.
    Choisi(usize),
    /// Fermé sans choix (Échap, clic à côté) ; le focus revient à la case.
    Ferme,
}

/// L'état d'un menu : ouvert ou non, la ligne active. Le logiciel en garde un par menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MenuChoix {
    ouvert: bool,
    active: Option<usize>,
    /// Le contour de focus ne se voit qu'au clavier.
    clavier: bool,
    /// Vrai une fois que le menu a été dessiné : la touche ou le clic qui l'a ouvert ne le
    /// refermera pas dans la même image.
    pret: bool,
}

impl MenuChoix {
    pub fn est_ouvert(&self) -> bool {
        self.ouvert
    }

    /// La ligne active (le rang), s'il y en a une.
    pub fn active(&self) -> Option<usize> {
        self.active
    }

    /// Ouvre le menu : la ligne cochée est active.
    pub fn ouvrir(&mut self, lignes: &[Ligne]) {
        *self = MenuChoix {
            ouvert: true,
            active: ligne_initiale(lignes),
            clavier: true,
            pret: false,
        };
    }

    pub fn fermer(&mut self) {
        *self = MenuChoix::default();
    }

    /// Ouvre s'il est fermé, ferme s'il est ouvert : le clic sur la case.
    pub fn basculer(&mut self, lignes: &[Ligne]) {
        if self.ouvert {
            self.fermer();
        } else {
            self.ouvrir(lignes);
        }
    }
}

fn largeur_texte(ui: &Ui, t: &str, taille: f32, gras: bool) -> f32 {
    let police = if gras {
        semibold(taille)
    } else {
        egui::FontId::proportional(taille)
    };
    ui.painter()
        .layout_no_wrap(t.to_string(), police, egui::Color32::WHITE)
        .size()
        .x
}

/// La largeur du menu : la plus large ligne (carré, texte, sous-texte, coche), entre
/// [`MENU_LARGEUR`] et [`LARGEUR_MAX`].
pub fn largeur_menu(ui: &Ui, lignes: &[Ligne]) -> f32 {
    let carre = lignes.iter().any(Ligne::a_un_carre);
    let coche = lignes.iter().any(|l| l.coche);
    let gauche = MARGE_GAUCHE
        + if carre {
            PASTILLE_LARGEUR + ESPACE_CARRE
        } else {
            0.0
        };
    let droite = if coche {
        LIGNE_ECART + LIGNE_ICONE + LIGNE_MARGE_DROITE
    } else {
        LIGNE_MARGE_DROITE
    };
    let texte = lignes
        .iter()
        .map(|l| {
            largeur_texte(ui, &l.texte, TAILLE_TEXTE, false).max(
                l.sous_texte
                    .as_deref()
                    .map_or(0.0, |s| largeur_texte(ui, s, TAILLE_PETIT, false)),
            )
        })
        .fold(0.0, f32::max);
    (gauche + texte + droite + 2.0 * MENU_MARGE).clamp(MENU_LARGEUR, LARGEUR_MAX)
}

/// Les touches du clavier de cette image, consommées : le menu les prend, rien d'autre ne
/// les voit tant qu'il est ouvert.
fn touches_de_l_image(ui: &Ui) -> Vec<Touche> {
    let table = [
        (Key::ArrowUp, Touche::Haut),
        (Key::ArrowDown, Touche::Bas),
        (Key::Home, Touche::Debut),
        (Key::End, Touche::Fin),
        (Key::Enter, Touche::Entree),
        (Key::Escape, Touche::Echap),
    ];
    // Dans l'ordre où elles sont arrivées (deux flèches dans une image avancent de deux).
    let mut vues: Vec<(usize, Touche)> = Vec::new();
    ui.input_mut(|e| {
        for (rang, ev) in e.events.iter().enumerate() {
            if let Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = ev
                && modifiers.is_none()
                && let Some((_, t)) = table.iter().find(|(k, _)| k == key)
            {
                vues.push((rang, *t));
            }
        }
        for (k, _) in table {
            e.consume_key(Modifiers::NONE, k);
        }
    });
    vues.into_iter().map(|(_, t)| t).collect()
}

/// Le carré coloré d'une ligne (même dessin que la pastille d'un appareil).
fn dessiner_carre(ui: &Ui, p: &Palette, rect: Rect, l: &Ligne, grise: bool) {
    let arrondi = CornerRadius::same(4);
    if let Some(fond) = l.couleur.filter(|_| !grise) {
        ui.painter().rect_filled(rect, arrondi, couleur(fond));
    } else {
        ui.painter().rect_filled(rect, arrondi, couleur(p.controle));
        ui.painter().rect_stroke(
            rect,
            arrondi,
            Stroke::new(1.0, couleur(p.bord_controle)),
            StrokeKind::Inside,
        );
    }
    // Blanc ou noir selon la couleur, comme la lettre d'une pastille (contraste ≥ 3:1) ;
    // sur un carré neutre, le texte secondaire de la palette.
    let c = match (l.couleur, grise) {
        (Some(f), false) => couleur_lettre(f),
        (_, true) => p.texte_desactive,
        (None, false) => p.texte_secondaire,
    };
    match marque(l.logo, l.lettre, icone_disponible) {
        Some(Marque::Logo(g)) => icone(ui, rect.center(), g, TAILLE_PETIT + 4.0, c),
        Some(Marque::Lettre(t)) => {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                t,
                semibold(TAILLE_PETIT),
                couleur(c),
            );
        }
        None => {}
    }
}

/// Dessine le menu s'il est ouvert, et le conduit (souris et clavier). `ancre` : la case qui
/// l'ouvre (le menu se place par rapport à elle) ; `id` : une clé propre à ce menu ;
/// `mot_coche` : « coché », pour le lecteur d'écran.
///
/// Rend `Some(Choisi(rang))` ou `Some(Ferme)` l'image où cela arrive (le menu est alors déjà
/// fermé), sinon `None`.
pub fn afficher(
    ui: &mut Ui,
    p: &Palette,
    menu: &mut MenuChoix,
    id: Id,
    ancre: Rect,
    lignes: &[Ligne],
    mot_coche: &str,
) -> Option<Resultat> {
    if !menu.ouvert {
        return None;
    }
    if lignes.is_empty() {
        menu.fermer();
        return Some(Resultat::Ferme);
    }
    let dessinee_avant = menu.pret;
    menu.pret = true;

    // Le clavier : pas dans l'image d'ouverture (la touche qui ouvre ne choisit pas).
    if dessinee_avant && let Some(fin) = suivre_le_clavier(ui, menu, lignes) {
        return Some(fin);
    }

    let fenetre = ui.ctx().content_rect();
    let largeur = largeur_menu(ui, lignes);
    let complet = vec2(largeur, hauteur_totale(lignes));
    let place = placer(ancre, complet, fenetre);
    let defile = place.height() < complet.y - 0.5;
    let forme = Forme {
        carre: lignes.iter().any(Ligne::a_un_carre),
        choix_unique: lignes.iter().any(|l| l.coche),
        mot_coche,
    };

    let mut choisi = None;
    let reponse = egui::Area::new(id)
        .order(Order::Foreground)
        .fixed_pos(place.min)
        .constrain(false)
        .fade_in(false)
        .show(ui.ctx(), |ui| {
            choisi = cadre(ui, p, menu, place, defile, lignes, &forme);
        });

    if let Some(i) = choisi {
        menu.fermer();
        return Some(Resultat::Choisi(i));
    }
    // Un clic à côté (ni dans le menu, ni sur la case) le ferme.
    if dessinee_avant && clic_dehors(ui, reponse.response.rect, ancre) {
        menu.fermer();
        return Some(Resultat::Ferme);
    }
    None
}

/// Les touches de l'image ; rend la fin du menu (choisi ou fermé) si l'une d'elles le ferme.
fn suivre_le_clavier(ui: &Ui, menu: &mut MenuChoix, lignes: &[Ligne]) -> Option<Resultat> {
    let grisees: Vec<bool> = lignes.iter().map(|l| l.grise).collect();
    for t in touches_de_l_image(ui) {
        menu.clavier = true;
        match suivre(&grisees, menu.active, t) {
            Effet::Rien => {}
            Effet::Active(i) => menu.active = Some(i),
            Effet::Choisir(i) => {
                menu.fermer();
                return Some(Resultat::Choisi(i));
            }
            Effet::Fermer => {
                menu.fermer();
                return Some(Resultat::Ferme);
            }
        }
    }
    None
}

/// Vrai si un bouton de la souris a été pressé hors du menu et hors de la case qui l'ouvre.
fn clic_dehors(ui: &Ui, menu: Rect, ancre: Rect) -> bool {
    ui.input(|e| {
        e.events.iter().any(|ev| {
            matches!(ev, Event::PointerButton { pressed: true, pos, .. }
                if !menu.contains(*pos) && !ancre.contains(*pos))
        })
    })
}

/// Ce que toutes les lignes d'un menu partagent : un carré devant, une coche (choix unique),
/// le mot « coché » pour le lecteur d'écran.
struct Forme<'a> {
    carre: bool,
    choix_unique: bool,
    mot_coche: &'a str,
}

/// Le cadre du menu (fond, bord, ombre), qui défile s'il ne tient pas ; rend le rang de la
/// ligne cliquée.
fn cadre(
    ui: &mut Ui,
    p: &Palette,
    menu: &mut MenuChoix,
    place: Rect,
    defile: bool,
    lignes: &[Ligne],
    forme: &Forme<'_>,
) -> Option<usize> {
    egui::Frame::new()
        .fill(couleur(p.fond_menu))
        .stroke(Stroke::new(1.0, couleur(p.bord_carte)))
        .corner_radius(CornerRadius::same(ARRONDI_MENU))
        .shadow(egui::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: egui::Color32::from_black_alpha(if p.sombre { 110 } else { 50 }),
        })
        .inner_margin(egui::Margin::same(MENU_MARGE as i8))
        .show(ui, |ui| {
            ui.set_width(place.width() - 2.0 * MENU_MARGE - 2.0);
            ui.spacing_mut().item_spacing = vec2(0.0, MENU_ECART);
            if defile {
                egui::ScrollArea::vertical()
                    .max_height(place.height() - 2.0 * MENU_MARGE - 2.0)
                    .show(ui, |ui| lignes_du_menu(ui, p, menu, lignes, forme))
                    .inner
            } else {
                lignes_du_menu(ui, p, menu, lignes, forme)
            }
        })
        .inner
}

/// Les lignes, l'une sous l'autre ; rend le rang de celle qu'on vient de cliquer.
fn lignes_du_menu(
    ui: &mut Ui,
    p: &Palette,
    menu: &mut MenuChoix,
    lignes: &[Ligne],
    forme: &Forme<'_>,
) -> Option<usize> {
    let mut clic = None;
    for (i, l) in lignes.iter().enumerate() {
        if une_ligne(ui, p, menu, i, l, forme) {
            clic = Some(i);
        }
    }
    clic
}

/// La ligne `i` : son fond actif, son carré, ses textes, sa coche, son contour de focus ;
/// rend vrai si on vient de la cliquer.
fn une_ligne(ui: &mut Ui, p: &Palette, menu: &mut MenuChoix, i: usize, l: &Ligne, forme: &Forme<'_>) -> bool {
    let (rect, r) = ui.allocate_exact_size(
        vec2(ui.available_width(), l.hauteur()),
        if l.grise { Sense::hover() } else { Sense::click() },
    );
    // La souris qui bouge sur une ligne libre la rend active (comme Windows).
    if !l.grise && r.hovered() && ui.input(|e| e.pointer.delta() != Vec2::ZERO) {
        menu.active = Some(i);
        menu.clavier = false;
    }
    let active = menu.active == Some(i) && !l.grise;
    if active {
        let f = if r.is_pointer_button_down_on() {
            p.appui
        } else {
            p.survol
        };
        ui.painter()
            .rect_filled(rect, CornerRadius::same(LIGNE_ARRONDI), couleur(f));
    }
    let cy = rect.center().y;
    let mut x = rect.left() + MARGE_GAUCHE;
    if forme.carre {
        let c = Rect::from_center_size(
            pos2(x + PASTILLE_LARGEUR / 2.0, cy),
            vec2(PASTILLE_LARGEUR, PASTILLE_HAUTEUR),
        );
        dessiner_carre(ui, p, c, l, l.grise);
        x += PASTILLE_LARGEUR + ESPACE_CARRE;
    }
    let droite = rect.right()
        - if forme.choix_unique {
            LIGNE_ECART + LIGNE_ICONE + LIGNE_MARGE_DROITE - LIGNE_ECART
        } else {
            LIGNE_MARGE_DROITE
        };
    dessiner_textes(ui, p, l, pos2(x, cy), droite);
    if l.coche {
        icone(
            ui,
            pos2(rect.right() - LIGNE_MARGE_DROITE - LIGNE_ICONE / 2.0, cy),
            icones::COCHE,
            LIGNE_ICONE,
            if l.grise { p.texte_desactive } else { p.accent },
        );
    }
    // Le contour de focus : seulement au clavier.
    if active && menu.clavier {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            CornerRadius::same(LIGNE_ARRONDI + 1),
            Stroke::new(2.0, couleur(p.texte)),
            StrokeKind::Outside,
        );
        ui.scroll_to_rect(rect, None);
    }
    let role = if forme.choix_unique {
        Role::MenuItemRadio
    } else {
        Role::MenuItem
    };
    let nom = nom_accessible(l, forme.mot_coche);
    r.widget_info(|| egui::WidgetInfo::selected(role, !l.grise, l.coche, &nom));
    !l.grise && r.clicked()
}

/// Le texte de la ligne (et son sous-texte dessous), à partir de `debut` (gauche, milieu de la
/// ligne), coupés (« … ») avant `droite`.
fn dessiner_textes(ui: &Ui, p: &Palette, l: &Ligne, debut: egui::Pos2, droite: f32) {
    let (texte, second) = if l.grise {
        (p.texte_desactive, p.texte_desactive)
    } else {
        (p.texte, p.texte_secondaire)
    };
    let (x, cy) = (debut.x, debut.y);
    let place = (droite - x).max(1.0);
    let coupe = |t: &str, taille: f32, c: Rgb| {
        let mut job = egui::text::LayoutJob::simple_singleline(
            t.to_string(),
            egui::FontId::proportional(taille),
            couleur(c),
        );
        job.wrap = egui::text::TextWrapping::truncate_at_width(place);
        ui.painter().layout_job(job)
    };
    if let Some(s) = l.sous_texte.as_deref() {
        let g = coupe(&l.texte, TAILLE_TEXTE, texte);
        ui.painter()
            .galley(pos2(x, cy - 9.0 - g.size().y / 2.0), g, couleur(texte));
        let g = coupe(s, TAILLE_PETIT, second);
        ui.painter()
            .galley(pos2(x, cy + 9.0 - g.size().y / 2.0), g, couleur(second));
    } else {
        let g = coupe(&l.texte, TAILLE_TEXTE, texte);
        ui.painter()
            .galley(pos2(x, cy - g.size().y / 2.0), g, couleur(texte));
    }
}

#[cfg(test)]
mod essais {
    use super::*;
    use crate::theme::{ChoixWindows, contraste};

    // Les touches (`suivre`) et la ligne initiale sont essayées dans le cœur
    // (`ic_coeur::gestes::choix`) ; ici, la reprise sur des lignes.
    #[test]
    fn ligne_initiale_est_la_cochee_sinon_la_premiere_libre() {
        let mut l = vec![Ligne::new("a", "A"), Ligne::new("b", "B"), Ligne::new("c", "C")];
        assert_eq!(ligne_initiale(&l), Some(0));
        l[1].coche = true;
        assert_eq!(ligne_initiale(&l), Some(1));
        l[1].grise = true;
        assert_eq!(ligne_initiale(&l), Some(0), "la cochée est grisée");
        l[0].grise = true;
        assert_eq!(ligne_initiale(&l), Some(2));
        l[2].grise = true;
        assert_eq!(ligne_initiale(&l), None);
        assert_eq!(ligne_initiale(&[]), None);
    }

    #[test]
    fn nom_pour_le_lecteur_d_ecran() {
        let mut l = Ligne::new("fixe", "Fixe");
        assert_eq!(nom_accessible(&l, "coché"), "Fixe");
        l.sous_texte = Some("Ce PC".into());
        assert_eq!(nom_accessible(&l, "coché"), "Fixe, Ce PC");
        l.coche = true;
        assert_eq!(nom_accessible(&l, "coché"), "Fixe, Ce PC, coché");
        // La couleur n'entre pas dans le nom.
        l.couleur = Some(Rgb(1, 2, 3));
        assert_eq!(nom_accessible(&l, "coché"), "Fixe, Ce PC, coché");
    }

    fn lignes() -> Vec<Ligne> {
        let mut a = Ligne::new("fixe", "Fixe");
        a.lettre = Some('F');
        a.couleur = Some(crate::pastilles::couleurs_pastilles()[0]);
        a.coche = true;
        let mut b = Ligne::new("portable", "Portable");
        b.sous_texte = Some("Hors ligne".into());
        b.grise = true;
        let mut c = Ligne::new("tour", "Tour");
        c.lettre = Some('T');
        vec![a, b, c]
    }

    /// Une image du menu avec ces événements ; rend ce qu'il a rendu.
    fn image(ctx: &egui::Context, menu: &mut MenuChoix, evts: Vec<egui::Event>) -> Option<Resultat> {
        let p = Palette::de(&ChoixWindows::default());
        let entree = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 300.0))),
            events: evts,
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let l = lignes();
        let mut rendu = None;
        let mut sortie = ctx.run_ui(entree, |ui| {
            let ancre = Rect::from_min_size(pos2(20.0, 20.0), vec2(120.0, 30.0));
            rendu = afficher(ui, &p, menu, Id::NULL.with("essai"), ancre, &l, "coché");
        });
        sortie.textures_delta.clear();
        rendu
    }

    fn touche(k: Key) -> Vec<egui::Event> {
        vec![egui::Event::Key {
            key: k,
            physical_key: Some(k),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }]
    }

    #[test]
    fn au_clavier_les_fleches_sautent_la_ligne_grisee_et_entree_choisit() {
        let ctx = egui::Context::default();
        let mut m = MenuChoix::default();
        m.ouvrir(&lignes());
        assert_eq!(m.active(), Some(0), "la cochée");
        assert_eq!(image(&ctx, &mut m, vec![]), None);
        // La touche d'ouverture ne choisit pas : la première image ne lit pas le clavier.
        assert_eq!(image(&ctx, &mut m, touche(Key::ArrowDown)), None);
        assert_eq!(m.active(), Some(2), "la ligne 1 est grisée");
        assert_eq!(image(&ctx, &mut m, touche(Key::ArrowDown)), None);
        assert_eq!(m.active(), Some(0), "fait le tour");
        assert_eq!(image(&ctx, &mut m, touche(Key::End)), None);
        assert_eq!(image(&ctx, &mut m, touche(Key::Enter)), Some(Resultat::Choisi(2)));
        assert!(!m.est_ouvert());
    }

    #[test]
    fn echap_ferme_le_menu() {
        let ctx = egui::Context::default();
        let mut m = MenuChoix::default();
        m.ouvrir(&lignes());
        image(&ctx, &mut m, vec![]);
        assert_eq!(image(&ctx, &mut m, touche(Key::Escape)), Some(Resultat::Ferme));
        assert!(!m.est_ouvert());
        // Fermé : il ne rend plus rien.
        assert_eq!(image(&ctx, &mut m, touche(Key::Enter)), None);
    }

    fn bouton(pos: egui::Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn un_clic_a_cote_ferme_et_un_clic_sur_la_case_ne_ferme_pas() {
        let ctx = egui::Context::default();
        let mut m = MenuChoix::default();
        m.ouvrir(&lignes());
        image(&ctx, &mut m, vec![]);
        // Sur la case qui l'ouvre : le menu reste (c'est à la case de le basculer).
        assert_eq!(image(&ctx, &mut m, vec![bouton(pos2(40.0, 30.0), true)]), None);
        assert!(m.est_ouvert());
        // À côté, loin de tout.
        assert_eq!(
            image(&ctx, &mut m, vec![bouton(pos2(380.0, 290.0), true)]),
            Some(Resultat::Ferme)
        );
        assert!(!m.est_ouvert());
    }

    #[test]
    fn un_clic_sur_une_ligne_la_choisit_et_la_grisee_ne_se_choisit_pas() {
        let ctx = egui::Context::default();
        let mut m = MenuChoix::default();
        m.ouvrir(&lignes());
        image(&ctx, &mut m, vec![]);
        // Le menu est sous la case : x = 20, y = 50 + 4, marge 4, puis les lignes de 32.
        let haut = 20.0 + 30.0 + ECART_ANCRE + MENU_MARGE + 1.0;
        let sur = |rang_y: f32| pos2(80.0, haut + rang_y);
        // La ligne 1 (grisée, double : 48) : un clic ne la choisit pas.
        let y1 = MENU_LIGNE + MENU_ECART + 10.0;
        image(&ctx, &mut m, vec![egui::Event::PointerMoved(sur(y1))]);
        image(&ctx, &mut m, vec![bouton(sur(y1), true)]);
        assert_eq!(image(&ctx, &mut m, vec![bouton(sur(y1), false)]), None);
        assert!(m.est_ouvert());
        // La ligne 2 (« Tour ») : après la ligne 0 (32) et la ligne 1 (48).
        let y2 = MENU_LIGNE + MENU_ECART + LIGNE_DOUBLE + MENU_ECART + 10.0;
        image(&ctx, &mut m, vec![egui::Event::PointerMoved(sur(y2))]);
        image(&ctx, &mut m, vec![bouton(sur(y2), true)]);
        assert_eq!(
            image(&ctx, &mut m, vec![bouton(sur(y2), false)]),
            Some(Resultat::Choisi(2))
        );
    }

    #[test]
    fn la_largeur_reste_entre_les_bornes() {
        let ctx = egui::Context::default();
        let mut sortie = ctx.run_ui(Default::default(), |ui| {
            let l = lignes();
            let w = largeur_menu(ui, &l);
            assert!((MENU_LARGEUR..=LARGEUR_MAX).contains(&w), "{w}");
            let mut long = Ligne::new("x", "x".repeat(400));
            long.coche = true;
            assert_eq!(largeur_menu(ui, &[long]), LARGEUR_MAX);
        });
        sortie.textures_delta.clear();
    }

    /// Le glyphe a la couleur de la lettre : contraste d'au moins 3:1 avec le carré, pour
    /// chacune des couleurs des pastilles, en clair comme en sombre (la lettre est même à 4,5:1).
    #[test]
    fn le_glyphe_est_lisible_sur_chaque_couleur() {
        for f in crate::pastilles::couleurs_pastilles() {
            assert!(contraste(f, couleur_lettre(f)) >= 3.0, "{f:?}");
        }
        for sombre in [false, true] {
            let p = Palette::de(&ChoixWindows {
                sombre,
                ..Default::default()
            });
            // Un carré neutre : le glyphe est le texte secondaire sur le fond des contrôles.
            assert!(
                contraste(p.texte_secondaire, p.controle) >= 3.0,
                "sombre : {sombre}"
            );
        }
    }
}
