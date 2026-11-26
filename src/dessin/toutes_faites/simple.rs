//! La fenêtre « simple », dessinée sans fenêtre : titre, texte, nouveautés, état, barre
//! d'avancement, champ, choix, boutons (avec confirmation) ; sa hauteur suit son contenu,
//! jusqu'à un plafond ([`HAUTEUR_MAX`]) au-delà duquel on défile. Reprise de Vocalis
//! (`interface/src/simples.rs`).

use egui::{Align, RichText, Sense, TextureHandle, Ui, vec2};

use crate::apparence::couleur;
use crate::dessin::{self, Genre};
use crate::theme::{Palette, Rgb};
use crate::{FaireLogo, Textes, polices};

/// Hauteur au plus d'une fenêtre simple (points) : au-delà, le contenu défile.
pub const HAUTEUR_MAX: f32 = 640.0;
/// Largeur du contenu d'une fenêtre simple, par défaut (points).
pub const LARGEUR_SIMPLE: f32 = 440.0;
/// Marges d'une fenêtre simple (gauche et droite, haut et bas).
const MARGE: [i8; 2] = [24, 20];
/// Taille du logo d'une fenêtre simple (points).
const TAILLE_LOGO: f32 = 36.0;

/// La couleur d'un texte d'état.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ton {
    #[default]
    Normal,
    /// Bien fini (vert).
    Bon,
    /// En cours, en attente (orange).
    EnCours,
    /// Un problème (rouge).
    Probleme,
}

fn couleur_ton(t: Ton, p: &Palette) -> Rgb {
    match t {
        Ton::Normal => p.texte,
        Ton::Bon => p.vert,
        Ton::EnCours => p.orange,
        Ton::Probleme => p.rouge,
    }
}

/// Un bouton du bas d'une fenêtre simple.
#[derive(Debug, Clone, PartialEq)]
pub struct Bouton {
    /// Ce que le rappel reçoit quand on le clique.
    pub cle: String,
    pub texte: String,
    pub glyphe: Option<char>,
    pub genre: Genre,
    pub actif: bool,
    /// Non vide : un premier clic demande confirmation (ce texte, en rouge ; le bouton devient
    /// « Confirmer »), le second valide.
    pub confirmer: String,
}

impl Bouton {
    pub fn new(cle: &str, texte: &str) -> Bouton {
        Bouton {
            cle: cle.into(),
            texte: texte.into(),
            glyphe: None,
            genre: Genre::Normal,
            actif: true,
            confirmer: String::new(),
        }
    }

    /// Le bouton d'accent : celui qu'Entrée valide.
    pub fn accent(mut self) -> Bouton {
        self.genre = Genre::Accent;
        self
    }

    /// Une action qui détruit (texte rouge).
    pub fn danger(mut self) -> Bouton {
        self.genre = Genre::Danger;
        self
    }

    pub fn glyphe(mut self, g: char) -> Bouton {
        self.glyphe = Some(g);
        self
    }

    pub fn inactif(mut self) -> Bouton {
        self.actif = false;
        self
    }

    pub fn confirmer(mut self, question: &str) -> Bouton {
        self.confirmer = question.into();
        self
    }
}

/// Un champ d'une ligne.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Champ {
    /// Changer de clé remet le champ à `valeur` ; sinon ce que l'utilisateur écrit reste.
    pub cle: String,
    pub libelle: String,
    pub valeur: String,
}

/// Un choix parmi quelques options (boutons ronds).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Choix {
    pub cle: String,
    pub libelle: String,
    /// La clé de l'option choisie au départ.
    pub valeur: String,
    /// (clé, texte) de chaque option.
    pub options: Vec<(String, String)>,
}

/// Ce que montre une fenêtre simple ; un texte vide n'est pas dessiné.
#[derive(Debug, Clone, Default)]
pub struct Simple {
    pub titre: String,
    pub sous_titre: String,
    /// Le logo du logiciel à gauche du titre (`Identite::logo`).
    pub logo: Option<FaireLogo>,
    /// Une carte « Nouveautés » (elle défile au-delà de 180 points).
    pub nouveautes: String,
    pub statut: String,
    pub ton: Ton,
    /// La barre d'avancement, de 0 à 1.
    pub barre: Option<f32>,
    /// Une précision en petit, sous la barre.
    pub detail: String,
    pub champ: Option<Champ>,
    pub choix: Option<Choix>,
    /// De gauche à droite, alignés à droite.
    pub boutons: Vec<Bouton>,
    /// Largeur du contenu (points) ; 0 : [`LARGEUR_SIMPLE`].
    pub largeur: f32,
}

impl Simple {
    pub fn largeur(&self) -> f32 {
        if self.largeur > 0.0 {
            self.largeur.clamp(240.0, 900.0)
        } else {
            LARGEUR_SIMPLE
        }
    }

    /// La largeur de la fenêtre, marges comprises.
    pub fn largeur_fenetre(&self) -> f32 {
        self.largeur() + 2.0 * MARGE[0] as f32
    }
}

/// Un bouton cliqué, avec ce que contiennent le champ et le choix.
#[derive(Debug, Clone, PartialEq)]
pub struct Clic {
    pub cle: String,
    pub champ: Option<String>,
    pub choix: Option<String>,
}

/// Ce qui ne se dessine pas d'une image à l'autre : ce que l'utilisateur écrit et choisit,
/// une confirmation qui attend, la hauteur du contenu.
#[derive(Default)]
pub struct EtatSimple {
    /// (clé du champ, texte écrit).
    champ: Option<(String, String)>,
    /// (clé du choix, option choisie).
    choix: Option<(String, String)>,
    logo: Option<TextureHandle>,
    /// Le bouton qui attend sa confirmation.
    confirmer: Option<String>,
    /// La hauteur du contenu à la dernière image (points, marges comprises, au plus
    /// [`HAUTEUR_MAX`]).
    pub hauteur: f32,
    /// Les boutons dessinés à la dernière image (clé, place).
    pub boutons_dessines: Vec<(String, egui::Rect)>,
}

impl EtatSimple {
    fn synchroniser(&mut self, s: &Simple) {
        match &s.champ {
            Some(c) if self.champ.as_ref().map(|x| &x.0) != Some(&c.cle) => {
                self.champ = Some((c.cle.clone(), c.valeur.clone()));
            }
            None => self.champ = None,
            _ => {}
        }
        match &s.choix {
            Some(c) if self.choix.as_ref().map(|x| &x.0) != Some(&c.cle) => {
                self.choix = Some((c.cle.clone(), c.valeur.clone()));
            }
            None => self.choix = None,
            _ => {}
        }
        if self
            .confirmer
            .as_ref()
            .is_some_and(|c| !s.boutons.iter().any(|b| &b.cle == c && b.actif))
        {
            self.confirmer = None;
        }
    }

    /// Ce que contient le champ.
    pub fn champ(&self) -> Option<&str> {
        self.champ.as_ref().map(|c| c.1.as_str())
    }

    /// L'option choisie.
    pub fn choix(&self) -> Option<&str> {
        self.choix.as_ref().map(|c| c.1.as_str())
    }

    /// Un bouton attend-il sa confirmation ?
    pub fn confirmation(&self) -> Option<&str> {
        self.confirmer.as_deref()
    }

    /// Échap : abandonne la confirmation qui attend ; vrai s'il y en avait une.
    pub fn abandonner(&mut self) -> bool {
        self.confirmer.take().is_some()
    }

    fn clic(&self, cle: String) -> Clic {
        Clic {
            cle,
            champ: self.champ().map(Into::into),
            choix: self.choix().map(Into::into),
        }
    }
}

/// La barre d'avancement de Windows 11 : un rail fin, rempli de la couleur d'accent.
pub fn barre_avancement(ui: &mut Ui, p: &Palette, v: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
    let arrondi = egui::CornerRadius::same(3);
    ui.painter().rect_filled(r, arrondi, couleur(p.bord_controle));
    let mut plein = r;
    plein.set_width((r.width() * v.clamp(0.0, 1.0)).max(if v > 0.0 { 6.0 } else { 0.0 }));
    ui.painter().rect_filled(plein, arrondi, couleur(p.accent));
}

fn texture_logo(ui: &Ui, logo: FaireLogo) -> Option<TextureHandle> {
    let t = 64usize;
    let px = logo(t as u32).filter(|px| px.len() == t * t * 4)?;
    let image = egui::ColorImage::from_rgba_unmultiplied([t, t], &px);
    Some(
        ui.ctx()
            .load_texture("logo-simple", image, egui::TextureOptions::LINEAR),
    )
}

fn texte_long(ui: &mut Ui, texte: RichText) {
    ui.add(egui::Label::new(texte).wrap());
}

/// Un clic attendu : (clé du bouton, question de confirmation).
type ClicBrut = Option<(String, String)>;

/// Dessine une fenêtre simple dans ce `Ui` ; rend le bouton cliqué (au plus un par image).
/// Entrée valide le bouton d'accent actif.
pub fn dessiner_simple(ui: &mut Ui, p: &Palette, t: &Textes, s: &Simple, e: &mut EtatSimple) -> Option<Clic> {
    e.synchroniser(s);
    e.boutons_dessines.clear();
    let mut clic: ClicBrut = None;
    let largeur = s.largeur();
    let rep = egui::ScrollArea::vertical()
        .max_height(HAUTEUR_MAX)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(MARGE[0], MARGE[1]))
                .show(ui, |ui| {
                    ui.set_width(largeur);
                    ui.spacing_mut().item_spacing.y = 8.0;
                    clic = contenu(ui, p, t, s, e);
                })
                .response
                .rect
                .height()
        });
    e.hauteur = rep.inner.min(HAUTEUR_MAX);
    valider(ui, s, e, clic)
}

/// Tout le contenu, de haut en bas ; rend le bouton cliqué.
fn contenu(ui: &mut Ui, p: &Palette, t: &Textes, s: &Simple, e: &mut EtatSimple) -> ClicBrut {
    entete(ui, p, s, e);
    nouveautes(ui, p, t, s);
    etat(ui, p, s);
    champ(ui, s, e);
    choix(ui, s, e);
    boutons(ui, p, t, s, e)
}

/// Le logo, le titre et le sous-titre.
fn entete(ui: &mut Ui, p: &Palette, s: &Simple, e: &mut EtatSimple) {
    if s.logo.is_none() && s.titre.is_empty() && s.sous_titre.is_empty() {
        return;
    }
    ui.horizontal_top(|ui| {
        if let Some(l) = s.logo {
            if e.logo.is_none() {
                e.logo = texture_logo(ui, l);
            }
            if let Some(texture) = &e.logo {
                ui.add(egui::Image::new(texture).fit_to_exact_size(vec2(TAILLE_LOGO, TAILLE_LOGO)));
            }
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if !s.titre.is_empty() {
                texte_long(ui, RichText::new(&s.titre).font(polices::semibold(20.0)));
            }
            if !s.sous_titre.is_empty() {
                texte_long(
                    ui,
                    RichText::new(dessin::insecables(&s.sous_titre)).color(couleur(p.texte_secondaire)),
                );
            }
        });
    });
}

/// La carte « Nouveautés », qui défile au-delà de 180 points.
fn nouveautes(ui: &mut Ui, p: &Palette, t: &Textes, s: &Simple) {
    if s.nouveautes.trim().is_empty() {
        return;
    }
    dessin::cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(t.nouveautes).font(polices::semibold(14.0)));
        egui::ScrollArea::vertical()
            .id_salt("nouveautes")
            .max_height(180.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.add(egui::Label::new(s.nouveautes.trim()).wrap());
            });
    });
}

/// L'état (dans sa couleur), la barre d'avancement et la précision en petit.
fn etat(ui: &mut Ui, p: &Palette, s: &Simple) {
    if !s.statut.is_empty() {
        texte_long(
            ui,
            RichText::new(dessin::insecables(&s.statut)).color(couleur(couleur_ton(s.ton, p))),
        );
    }
    if let Some(b) = s.barre {
        barre_avancement(ui, p, b);
    }
    if !s.detail.is_empty() {
        texte_long(
            ui,
            RichText::new(&s.detail)
                .size(12.0)
                .color(couleur(p.texte_secondaire)),
        );
    }
}

/// Le champ d'une ligne ; il prend le focus à l'ouverture.
fn champ(ui: &mut Ui, s: &Simple, e: &mut EtatSimple) {
    let (Some(c), Some((_, texte))) = (&s.champ, e.champ.as_mut()) else {
        return;
    };
    if !c.libelle.is_empty() {
        ui.label(&c.libelle);
    }
    let r = ui.add(
        egui::TextEdit::singleline(texte)
            .desired_width(f32::INFINITY)
            .char_limit(200),
    );
    // Le champ prend le focus à l'ouverture, une seule fois.
    if !ui.memory(|m| m.data.get_temp::<bool>(r.id).unwrap_or(false)) {
        r.request_focus();
        ui.memory_mut(|m| m.data.insert_temp(r.id, true));
    }
}

/// Le choix, un bouton rond par option.
fn choix(ui: &mut Ui, s: &Simple, e: &mut EtatSimple) {
    let (Some(c), Some((_, v))) = (&s.choix, e.choix.as_mut()) else {
        return;
    };
    if !c.libelle.is_empty() {
        ui.label(&c.libelle);
    }
    for (k, nom) in &c.options {
        // Le rond est aligné sur le bord du champ (le dessin d'egui déborde de sa case à
        // gauche).
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.radio_value(v, k.clone(), dessin::insecables(nom));
        });
    }
}

/// Les boutons, alignés à droite, puis la question de confirmation qui attend (en rouge).
fn boutons(ui: &mut Ui, p: &Palette, t: &Textes, s: &Simple, e: &mut EtatSimple) -> ClicBrut {
    if s.boutons.is_empty() {
        return None;
    }
    let mut clic = None;
    ui.add_space(6.0);
    let attente = e.confirmer.clone();
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 32.0),
        egui::Layout::right_to_left(Align::Center),
        |ui| {
            // De droite à gauche : le dernier de la liste d'abord.
            for b in s.boutons.iter().rev() {
                let attend = attente.as_deref() == Some(b.cle.as_str());
                let (texte, g) = if attend {
                    (t.confirmer, Genre::Danger)
                } else {
                    (b.texte.as_str(), b.genre)
                };
                let r = ui
                    .add_enabled_ui(b.actif, |ui| dessin::bouton(ui, p, b.glyphe, texte, g))
                    .inner;
                e.boutons_dessines.push((b.cle.clone(), r.rect));
                if r.clicked() {
                    clic = Some((b.cle.clone(), b.confirmer.clone()));
                }
            }
        },
    );
    if let Some(q) = &attente
        && let Some(b) = s.boutons.iter().find(|b| &b.cle == q)
    {
        texte_long(
            ui,
            RichText::new(dessin::insecables(&b.confirmer)).color(couleur(p.rouge)),
        );
    }
    clic
}

/// Entrée valide le bouton d'accent actif ; un bouton à confirmer demande un second clic.
fn valider(ui: &Ui, s: &Simple, e: &mut EtatSimple, mut clic: ClicBrut) -> Option<Clic> {
    if clic.is_none() && e.confirmer.is_none() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        clic = s
            .boutons
            .iter()
            .find(|b| b.genre == Genre::Accent && b.actif)
            .map(|b| (b.cle.clone(), b.confirmer.clone()));
    }
    let (cle, question) = clic?;
    if !question.is_empty() && e.confirmer.as_deref() != Some(cle.as_str()) {
        e.confirmer = Some(cle);
        None
    } else {
        e.confirmer = None;
        Some(e.clic(cle))
    }
}

#[cfg(test)]
mod tests {
    // Dans un essai, un échec doit arrêter l'essai : `unwrap` et `expect` y sont voulus.
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::dessin::toutes_faites::essais::{cliquer, entree, palette, simple, une_image};
    use crate::textes::FRANCAIS;

    /// Un clic sur un bouton le rend avec le champ et le choix ; un bouton à confirmer
    /// demande un second clic ; Entrée valide le bouton d'accent.
    #[test]
    fn boutons_champ_choix_et_confirmation() {
        let ctx = egui::Context::default();
        let p = palette();
        let s = simple();
        let mut e = EtatSimple::default();
        let mut rendu = Vec::new();
        let mut f = |ui: &mut Ui| {
            if let Some(c) = dessiner_simple(ui, &p, &FRANCAIS, &s, &mut e) {
                rendu.push(c);
            }
        };
        une_image(&ctx, vec![], &mut f);
        drop(f);
        let place = |e: &EtatSimple, cle: &str| {
            e.boutons_dessines
                .iter()
                .find(|(c, _)| c == cle)
                .map(|(_, r)| r.center())
                .expect(cle)
        };
        let garder = place(&e, "garder");
        let effacer = place(&e, "effacer");
        assert!(effacer.x < garder.x, "dans l'ordre de la liste");
        let mut f = |ui: &mut Ui| {
            if let Some(c) = dessiner_simple(ui, &p, &FRANCAIS, &s, &mut e) {
                rendu.push(c);
            }
        };
        cliquer(&ctx, garder, &mut f);
        cliquer(&ctx, effacer, &mut f);
        drop(f);
        assert_eq!(e.confirmation(), Some("effacer"));
        let mut f = |ui: &mut Ui| {
            if let Some(c) = dessiner_simple(ui, &p, &FRANCAIS, &s, &mut e) {
                rendu.push(c);
            }
        };
        cliquer(&ctx, effacer, &mut f);
        une_image(&ctx, vec![entree()], &mut f);
        drop(f);
        let attendu = |cle: &str| Clic {
            cle: cle.into(),
            champ: Some("Lecture du matin".into()),
            choix: Some("b".into()),
        };
        assert_eq!(
            rendu,
            vec![attendu("garder"), attendu("effacer"), attendu("garder")]
        );
    }

    /// La hauteur suit le contenu, jusqu'au plafond.
    #[test]
    fn la_hauteur_suit_le_contenu() {
        let ctx = egui::Context::default();
        let p = palette();
        let hauteur = |s: &Simple| {
            let mut e = EtatSimple::default();
            for _ in 0..2 {
                une_image(&ctx, vec![], |ui| {
                    dessiner_simple(ui, &p, &FRANCAIS, s, &mut e);
                });
            }
            e.hauteur
        };
        let court = hauteur(&Simple {
            titre: "Un titre".into(),
            ..Default::default()
        });
        let moyen = hauteur(&simple());
        let long = hauteur(&Simple {
            detail: "Une ligne inventée.\n".repeat(200),
            ..Default::default()
        });
        assert!(court > 40.0 && court < moyen, "{court} / {moyen}");
        assert!(moyen < HAUTEUR_MAX);
        assert_eq!(long, HAUTEUR_MAX);
    }
}
