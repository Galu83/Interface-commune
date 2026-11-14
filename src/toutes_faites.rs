//! Les fenêtres toutes faites, ouvertes dans la boucle du logiciel ([`AutresFenetres`]) :
//! - la fenêtre « simple » ([`Simple`]) : titre, texte, nouveautés, état, barre
//!   d'avancement, champ, choix, boutons (avec confirmation) ; sa hauteur suit son contenu,
//!   jusqu'à un plafond ([`HAUTEUR_MAX`]) au-delà duquel on défile ;
//! - la fenêtre de lecture ([`Lecture`]) : un texte lu à voix haute, le mot lu surligné et
//!   suivi ; un clic sur un mot le rend au logiciel (reprendre la lecture là).
//!
//! Reprises de Vocalis (`interface/src/simples.rs`), sans son canal : le logiciel décrit la
//! fenêtre par une structure, la change par sa [`Poignee`], et reçoit ce que l'utilisateur fait
//! par un rappel. Les fonctions de dessin ([`dessiner_simple`], [`dessiner_lecture`]) servent
//! aussi seules, dans une fenêtre du logiciel.
//!
//! Le rappel est appelé pendant le dessin de la fenêtre : il peut changer la fenêtre par sa
//! poignée ([`Poignee::changer`], [`Poignee::fermer`]) et ouvrir ou fermer d'autres
//! fenêtres, mais ne doit pas appeler `AutresFenetres::fermer` sur sa propre clé (la poignée
//! le fait sans attente).

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};

use eframe::egui::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use eframe::egui::text_selection::LabelSelectionState;
use eframe::egui::{
    self, Align, Color32, FontId, Galley, RichText, Sense, Stroke, TextureHandle, Ui, ViewportCommand, vec2,
};

use crate::apparence::couleur;
use crate::autres_fenetres::{AutresFenetres, Cadre, Contenu, Ouverture};
use crate::dessin::{self, Genre};
use crate::place::Place;
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

fn verrou<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ======================================================================================
// La poignée commune
// ======================================================================================

struct Partage<T> {
    donnees: T,
    fermer: bool,
}

type Partagee<T> = Arc<Mutex<Partage<T>>>;

/// Ce qui reste au logiciel d'une fenêtre toute faite ouverte : la changer, la fermer. Se
/// clone et s'utilise depuis n'importe quel fil.
pub struct Poignee<T> {
    partage: Partagee<T>,
    autres: AutresFenetres,
    cle: String,
}

impl<T> Clone for Poignee<T> {
    fn clone(&self) -> Self {
        Poignee {
            partage: self.partage.clone(),
            autres: self.autres.clone(),
            cle: self.cle.clone(),
        }
    }
}

impl<T> Poignee<T> {
    /// Change ce que montre la fenêtre ; elle se redessine.
    pub fn changer(&self, f: impl FnOnce(&mut T)) {
        f(&mut verrou(&self.partage).donnees);
        self.autres.redessiner();
    }

    /// Lit ce que montre la fenêtre.
    pub fn lire<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&verrou(&self.partage).donnees)
    }

    /// Ferme la fenêtre (à sa prochaine image).
    pub fn fermer(&self) {
        verrou(&self.partage).fermer = true;
        self.autres.redessiner();
    }

    /// La fenêtre est-elle encore ouverte ?
    pub fn ouverte(&self) -> bool {
        !verrou(&self.partage).fermer && self.autres.ouverte(&self.cle)
    }

    pub fn cle(&self) -> &str {
        &self.cle
    }
}

fn nouvelle<T>(autres: &AutresFenetres, cle: &str, donnees: T) -> Poignee<T> {
    Poignee {
        partage: Arc::new(Mutex::new(Partage {
            donnees,
            fermer: false,
        })),
        autres: autres.clone(),
        cle: cle.into(),
    }
}

// ======================================================================================
// La fenêtre simple
// ======================================================================================

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

/// La clé du bouton « Plus tard » de [`Simple::version_prete`] (« Installer maintenant » rend
/// [`crate::cles::INSTALLER`]).
pub const PLUS_TARD: &str = "plus_tard";

impl Simple {
    /// La fenêtre « <Logiciel> x.y.z est prête » : le logo, le titre, les nouveautés (une
    /// par ligne), « Plus tard » et « Installer maintenant » (bouton d'accent, Entrée).
    pub fn version_prete(
        identite: &crate::Identite,
        t: &Textes,
        version: &str,
        nouveautes: &[String],
    ) -> Simple {
        Simple {
            titre: crate::textes::remplir(t.prete_titre, &[&identite.nom, version]),
            sous_titre: t.prete_explication.into(),
            logo: identite.logo,
            nouveautes: nouveautes
                .iter()
                .map(|n| format!("• {n}"))
                .collect::<Vec<_>>()
                .join("\n"),
            boutons: vec![
                Bouton::new(PLUS_TARD, t.plus_tard),
                Bouton {
                    glyphe: Some(crate::icones::INSTALLER),
                    ..Bouton::new(crate::cles::INSTALLER, t.installer_maintenant).accent()
                },
            ],
            ..Default::default()
        }
    }

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

/// Dessine une fenêtre simple dans ce `Ui` ; rend le bouton cliqué (au plus un par image).
/// Entrée valide le bouton d'accent actif.
pub fn dessiner_simple(ui: &mut Ui, p: &Palette, t: &Textes, s: &Simple, e: &mut EtatSimple) -> Option<Clic> {
    e.synchroniser(s);
    e.boutons_dessines.clear();
    let mut clic: Option<(String, String)> = None; // (clé, question de confirmation)
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
                    if s.logo.is_some() || !s.titre.is_empty() || !s.sous_titre.is_empty() {
                        ui.horizontal_top(|ui| {
                            if let Some(l) = s.logo {
                                if e.logo.is_none() {
                                    e.logo = texture_logo(ui, l);
                                }
                                if let Some(t) = &e.logo {
                                    ui.add(
                                        egui::Image::new(t).fit_to_exact_size(vec2(TAILLE_LOGO, TAILLE_LOGO)),
                                    );
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
                                        RichText::new(dessin::insecables(&s.sous_titre))
                                            .color(couleur(p.texte_secondaire)),
                                    );
                                }
                            });
                        });
                    }
                    if !s.nouveautes.trim().is_empty() {
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
                    if !s.statut.is_empty() {
                        texte_long(
                            ui,
                            RichText::new(dessin::insecables(&s.statut))
                                .color(couleur(couleur_ton(s.ton, p))),
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
                    if let (Some(c), Some((_, texte))) = (&s.champ, e.champ.as_mut()) {
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
                    if let (Some(c), Some((_, v))) = (&s.choix, e.choix.as_mut()) {
                        if !c.libelle.is_empty() {
                            ui.label(&c.libelle);
                        }
                        for (k, nom) in &c.options {
                            // Le rond est aligné sur le bord du champ (le dessin d'egui
                            // déborde de sa case à gauche).
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                ui.radio_value(v, k.clone(), dessin::insecables(nom));
                            });
                        }
                    }
                    if !s.boutons.is_empty() {
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
                                        .add_enabled_ui(b.actif, |ui| {
                                            dessin::bouton(ui, p, b.glyphe, texte, g)
                                        })
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
                    }
                })
                .response
                .rect
                .height()
        });
    e.hauteur = rep.inner.min(HAUTEUR_MAX);
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

/// Ce que l'utilisateur fait d'une fenêtre simple.
#[derive(Debug, Clone, PartialEq)]
pub enum Reponse {
    Clic(Clic),
    /// La fenêtre s'est fermée (croix, Échap, ou la poignée).
    Fermee,
}

pub type FenetreSimple = Poignee<Simple>;

struct ContenuSimple {
    partage: Partagee<Simple>,
    textes: Textes,
    etat: EtatSimple,
    rappel: Box<dyn FnMut(Reponse) + Send>,
    /// La hauteur demandée à Windows (points).
    hauteur_posee: f32,
}

impl Contenu for ContenuSimple {
    fn dessiner(&mut self, ui: &mut Ui, cadre: &mut Cadre<'_>) {
        let p = cadre.palette;
        let echap = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let (clic, largeur) = {
            let g = verrou(&self.partage);
            if g.fermer {
                cadre.fermer();
                return;
            }
            let largeur = g.donnees.largeur_fenetre();
            let mut clic = None;
            egui::Frame::new().fill(couleur(p.fond)).show(ui, |ui| {
                clic = dessiner_simple(ui, p, &self.textes, &g.donnees, &mut self.etat);
            });
            (clic, largeur)
        };
        if echap && !self.etat.abandonner() {
            cadre.fermer();
        }
        let h = self.etat.hauteur;
        if h > 0.0 && (h - self.hauteur_posee).abs() > 1.0 {
            // La fenêtre prend la hauteur de son contenu.
            self.hauteur_posee = h;
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::InnerSize(vec2(largeur, h)));
        }
        if let Some(c) = clic {
            (self.rappel)(Reponse::Clic(c));
        }
    }

    fn fermee(&mut self, _place: Option<Place>) {
        verrou(&self.partage).fermer = true;
        (self.rappel)(Reponse::Fermee);
    }
}

/// Ouvre une fenêtre simple parmi les fenêtres de la boucle. `ouverture` donne son titre (la
/// barre de titre) et son icône ; sa taille suit le contenu. `None` si une fenêtre de cette
/// clé est déjà ouverte (elle passe devant, inchangée).
pub fn ouvrir_simple(
    autres: &AutresFenetres,
    cle: &str,
    ouverture: Ouverture,
    textes: &Textes,
    simple: Simple,
    rappel: impl FnMut(Reponse) + Send + 'static,
) -> Option<FenetreSimple> {
    let mut o = ouverture.fixe();
    o.taille = [simple.largeur_fenetre(), 160.0];
    let poignee = nouvelle(autres, cle, simple);
    let contenu = ContenuSimple {
        partage: poignee.partage.clone(),
        textes: textes.clone(),
        etat: EtatSimple::default(),
        rappel: Box::new(rappel),
        hauteur_posee: 0.0,
    };
    autres.ouvrir(cle, o, contenu).then_some(poignee)
}

// ======================================================================================
// La fenêtre de lecture
// ======================================================================================

/// Ce que montre la fenêtre de lecture.
#[derive(Debug, Clone, PartialEq)]
pub struct Lecture {
    pub texte: String,
    /// Chaque mot : [premier caractère, fin] (en caractères, pas en octets).
    pub mots: Vec<[u32; 2]>,
    /// Le mot lu (son rang dans `mots`) : surligné, et la page défile jusqu'à lui.
    pub mot: Option<usize>,
    /// Taille du texte (points).
    pub taille: f32,
    /// Couleur et opacité (0 à 1) du surlignage.
    pub surlignage: Rgb,
    pub opacite: f32,
}

impl Default for Lecture {
    fn default() -> Self {
        Lecture {
            texte: String::new(),
            mots: Vec::new(),
            mot: None,
            taille: 14.0,
            surlignage: Rgb(255, 214, 10),
            opacite: 0.4,
        }
    }
}

impl Lecture {
    /// Un texte et ses mots, découpés aux espaces.
    pub fn de(texte: &str) -> Lecture {
        Lecture {
            texte: texte.into(),
            mots: mots(texte),
            ..Default::default()
        }
    }

    /// Le mot dont la fin est la première à ne pas être avant ce caractère (un clic entre
    /// deux mots donne le suivant).
    pub fn mot_au_caractere(&self, car: usize) -> Option<usize> {
        self.mots.iter().position(|m| car as u32 <= m[1])
    }

    fn surlignage(&self) -> Color32 {
        let Rgb(r, g, b) = self.surlignage;
        let a = (self.opacite + 0.2).clamp(0.15, 1.0);
        Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0) as u8)
    }
}

/// Les mots d'un texte, découpés aux espaces : [premier caractère, fin] de chacun.
pub fn mots(texte: &str) -> Vec<[u32; 2]> {
    let mut v = Vec::new();
    let mut debut: Option<u32> = None;
    let mut n = 0u32;
    for c in texte.chars() {
        if c.is_whitespace() {
            if let Some(d) = debut.take() {
                v.push([d, n]);
            }
        } else if debut.is_none() {
            debut = Some(n);
        }
        n += 1;
    }
    if let Some(d) = debut {
        v.push([d, n]);
    }
    v
}

struct Paragraphe {
    /// Premier caractère du paragraphe, et sa fin.
    debut: usize,
    fin: usize,
    galley: Option<Arc<Galley>>,
    surligne: bool,
}

/// Ce que la fenêtre de lecture garde d'une image à l'autre : la mise en page de chaque
/// paragraphe (refaite seulement pour ceux dont le surlignage change), le mot suivi.
#[derive(Default)]
pub struct EtatLecture {
    /// Empreinte du texte, largeur, taille, thème de la dernière mise en page.
    clef: (u64, i32, i32, bool),
    /// Début de chaque caractère en octets (un de plus que le nombre de caractères).
    octets: Vec<usize>,
    paragraphes: Vec<Paragraphe>,
    mot_dessine: Option<usize>,
    /// Le mot lu à la dernière image : s'il change, la page défile jusqu'à lui.
    mot_suivi: Option<usize>,
    /// Où est le mot lu à l'écran à la dernière image.
    pub mot_vu: Option<egui::Rect>,
    /// La zone visible du texte à la dernière image.
    pub zone_vue: Option<egui::Rect>,
}

fn empreinte(texte: &str) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    texte.hash(&mut h);
    h.finish()
}

impl EtatLecture {
    fn preparer(&mut self, l: &Lecture, largeur: f32, sombre: bool) {
        let clef = (
            empreinte(&l.texte),
            (largeur * 4.0) as i32,
            (l.taille * 4.0) as i32,
            sombre,
        );
        if clef == self.clef && !(self.paragraphes.is_empty() && !l.texte.is_empty()) {
            return;
        }
        if clef.0 != self.clef.0 || self.octets.is_empty() {
            self.octets = l
                .texte
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(l.texte.len()))
                .collect();
        }
        self.clef = clef;
        self.mot_dessine = None;
        self.paragraphes.clear();
        let mut debut = 0usize;
        for ligne in l.texte.split('\n') {
            let n = ligne.chars().count();
            self.paragraphes.push(Paragraphe {
                debut,
                fin: debut + n,
                galley: None,
                surligne: false,
            });
            debut += n + 1;
        }
    }

    fn tranche<'a>(&self, l: &'a Lecture, a: usize, b: usize) -> &'a str {
        let o = |c: usize| self.octets.get(c).copied().unwrap_or(l.texte.len());
        l.texte.get(o(a)..o(b)).unwrap_or("")
    }

    fn mise_en_page(
        &self,
        ui: &Ui,
        p: &Palette,
        l: &Lecture,
        par: &Paragraphe,
        largeur: f32,
        mot: Option<[u32; 2]>,
    ) -> Arc<Galley> {
        let normal = TextFormat {
            font_id: FontId::proportional(l.taille),
            color: couleur(p.texte),
            ..Default::default()
        };
        let mut job = LayoutJob {
            wrap: TextWrapping::wrap_at_width(largeur),
            ..Default::default()
        };
        match mot {
            Some([a, b]) if (a as usize) >= par.debut && (b as usize) <= par.fin && a < b => {
                let (a, b) = (a as usize, b as usize);
                job.append(self.tranche(l, par.debut, a), 0.0, normal.clone());
                let mut vif = normal.clone();
                vif.background = l.surlignage();
                job.append(self.tranche(l, a, b), 0.0, vif);
                job.append(self.tranche(l, b, par.fin), 0.0, normal);
            }
            _ => job.append(self.tranche(l, par.debut, par.fin), 0.0, normal),
        }
        ui.painter().layout_job(job)
    }
}

/// Dessine le texte lu dans ce `Ui`, le mot lu surligné (la page le suit quand il change) ;
/// rend le mot cliqué. Glisser sélectionne le texte (Ctrl+C le copie).
pub fn dessiner_lecture(ui: &mut Ui, p: &Palette, l: &Lecture, e: &mut EtatLecture) -> Option<usize> {
    let marge = 20.0;
    let largeur = (ui.available_width() - 2.0 * marge).max(120.0);
    e.preparer(l, largeur, p.sombre);
    let mot = l.mot.and_then(|i| l.mots.get(i).copied());
    let change = e.mot_dessine != l.mot;
    let contient =
        |par: &Paragraphe| mot.is_some_and(|[a, b]| a as usize >= par.debut && b as usize <= par.fin);
    for i in 0..e.paragraphes.len() {
        let veut = contient(&e.paragraphes[i]);
        let par = &e.paragraphes[i];
        if par.galley.is_none() || (change && (veut || par.surligne)) {
            let g = e.mise_en_page(ui, p, l, par, largeur, if veut { mot } else { None });
            let par = &mut e.paragraphes[i];
            par.galley = Some(g);
            par.surligne = veut;
        }
    }
    e.mot_dessine = l.mot;
    let suivre = e.mot_suivi != l.mot;
    e.mot_suivi = l.mot;
    e.mot_vu = None;
    let mut clic = None;
    let mut zone = None;
    let mut mot_vu = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(marge - 2.0);
            for par in &e.paragraphes {
                let Some(g) = &par.galley else { continue };
                let (rect, r) = ui.allocate_exact_size(
                    vec2(largeur, g.size().y.max(l.taille * 1.3)),
                    Sense::click_and_drag(),
                );
                let rect = rect.translate(vec2(marge, 0.0));
                if ui.is_rect_visible(rect) {
                    LabelSelectionState::label_text_selection(
                        ui,
                        &r,
                        rect.min,
                        g.clone(),
                        couleur(p.texte),
                        Stroke::NONE,
                    );
                }
                if par.surligne
                    && let Some([a, _]) = mot
                {
                    let dans = (a as usize).saturating_sub(par.debut);
                    let c = g
                        .pos_from_cursor(egui::text::CCursor::new(dans))
                        .translate(rect.min.to_vec2());
                    mot_vu = Some(c);
                    if suivre {
                        ui.scroll_to_rect(c, Some(Align::Center));
                    }
                }
                // Un double-clic ne reprend qu'une fois (au premier clic).
                if r.clicked()
                    && !r.double_clicked()
                    && let Some(pos) = r.interact_pointer_pos()
                {
                    let dans: usize = g.cursor_from_pos(pos - rect.min).index.into();
                    clic = l.mot_au_caractere(dans + par.debut);
                }
            }
            zone = Some(ui.clip_rect());
            ui.add_space(marge);
        });
    e.mot_vu = mot_vu;
    e.zone_vue = zone;
    clic
}

/// Ce que l'utilisateur fait de la fenêtre de lecture.
#[derive(Debug, Clone, PartialEq)]
pub enum ReponseLecture {
    /// Un clic sur ce mot (son rang) : reprendre la lecture là.
    Mot(usize),
    /// La fenêtre s'est fermée (croix, Échap, ou la poignée).
    Fermee,
}

pub type FenetreLecture = Poignee<Lecture>;

/// Une barre au-dessus du texte, dessinée par le logiciel (pause, vitesse, voix…).
pub type BarreLecture = Box<dyn FnMut(&mut Ui, &Palette) + Send>;

struct ContenuLecture {
    partage: Partagee<Lecture>,
    etat: EtatLecture,
    barre: Option<BarreLecture>,
    rappel: Box<dyn FnMut(ReponseLecture) + Send>,
}

impl Contenu for ContenuLecture {
    fn dessiner(&mut self, ui: &mut Ui, cadre: &mut Cadre<'_>) {
        let p = cadre.palette;
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            cadre.fermer();
        }
        let mut clic = None;
        egui::Frame::new().fill(couleur(p.fond)).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            if let Some(b) = self.barre.as_mut() {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_centered(|ui| b(ui, p));
                    });
                ui.separator();
            }
            let g = verrou(&self.partage);
            if g.fermer {
                cadre.fermer();
                return;
            }
            clic = dessiner_lecture(ui, p, &g.donnees, &mut self.etat);
        });
        if let Some(m) = clic {
            (self.rappel)(ReponseLecture::Mot(m));
        }
    }

    fn fermee(&mut self, _place: Option<Place>) {
        verrou(&self.partage).fermer = true;
        (self.rappel)(ReponseLecture::Fermee);
    }
}

/// Ouvre la fenêtre de lecture parmi les fenêtres de la boucle. `None` si une fenêtre de
/// cette clé est déjà ouverte (elle passe devant, inchangée).
pub fn ouvrir_lecture(
    autres: &AutresFenetres,
    cle: &str,
    ouverture: Ouverture,
    lecture: Lecture,
    barre: Option<BarreLecture>,
    rappel: impl FnMut(ReponseLecture) + Send + 'static,
) -> Option<FenetreLecture> {
    let poignee = nouvelle(autres, cle, lecture);
    let contenu = ContenuLecture {
        partage: poignee.partage.clone(),
        etat: EtatLecture::default(),
        barre,
        rappel: Box::new(rappel),
    };
    autres.ouvrir(cle, ouverture, contenu).then_some(poignee)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::autres_fenetres::essais::{CROIX, ORDRES, contexte, image};
    use crate::autres_fenetres::id_de;
    use crate::textes::FRANCAIS;

    fn palette() -> Palette {
        Palette::de(&crate::theme::ChoixWindows::default())
    }

    /// Une image d'un dessin, sans fenêtre ; `evts` : ce que l'utilisateur fait.
    fn une_image(ctx: &egui::Context, evts: Vec<egui::Event>, f: impl FnMut(&mut Ui)) {
        let entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(600.0, 900.0))),
            events: evts,
            // Le temps avance : les défilements animés vont au bout.
            time: Some(ctx.input(|i| i.time) + 0.1),
            ..Default::default()
        };
        let mut f = f;
        let mut sortie = ctx.run_ui(entree, |ui| f(ui));
        sortie.textures_delta.clear();
    }

    fn cliquer(ctx: &egui::Context, pos: egui::Pos2, f: &mut impl FnMut(&mut Ui)) {
        une_image(ctx, vec![egui::Event::PointerMoved(pos)], &mut *f);
        let bouton = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        une_image(ctx, vec![bouton(true)], &mut *f);
        une_image(ctx, vec![bouton(false)], &mut *f);
    }

    fn simple() -> Simple {
        Simple {
            titre: "Garder comme action".into(),
            statut: "Prêt.".into(),
            ton: Ton::Bon,
            barre: Some(0.5),
            champ: Some(Champ {
                cle: "nom".into(),
                libelle: "Nom".into(),
                valeur: "Lecture du matin".into(),
            }),
            choix: Some(Choix {
                cle: "ou".into(),
                libelle: String::new(),
                valeur: "b".into(),
                options: vec![("a".into(), "Ici".into()), ("b".into(), "Là".into())],
            }),
            boutons: vec![
                Bouton::new("effacer", "Effacer")
                    .danger()
                    .confirmer("Effacer pour de bon ?"),
                Bouton::new("garder", "Garder").accent(),
            ],
            ..Default::default()
        }
    }

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
        une_image(
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            &mut f,
        );
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

    /// « <Logiciel> x.y.z est prête » : titre, nouveautés, « Plus tard » puis « Installer
    /// maintenant », qu'Entrée choisit.
    #[test]
    fn version_prete_toute_faite() {
        let id = crate::Identite {
            nom: "Exemple".into(),
            version: "1.0.0".into(),
            licence: "MIT".into(),
            description: String::new(),
            icone: |_, _| Vec::new(),
            logo: None,
        };
        let s = Simple::version_prete(&id, &FRANCAIS, "2.0.0", &["Un ajout inventé.".into()]);
        assert_eq!(s.titre, "Exemple 2.0.0 est prête");
        assert_eq!(s.nouveautes, "• Un ajout inventé.");
        let cles: Vec<&str> = s.boutons.iter().map(|b| b.cle.as_str()).collect();
        assert_eq!(cles, [PLUS_TARD, crate::cles::INSTALLER]);
        let en = Simple::version_prete(&id, &crate::textes::ANGLAIS, "2.0.0", &[]);
        assert_eq!(en.titre, "Exemple 2.0.0 is ready");
        let ctx = egui::Context::default();
        let p = palette();
        let mut e = EtatSimple::default();
        let mut rendu = Vec::new();
        let mut f = |ui: &mut Ui| {
            if let Some(c) = dessiner_simple(ui, &p, &FRANCAIS, &s, &mut e) {
                rendu.push(c.cle);
            }
        };
        une_image(&ctx, vec![], &mut f);
        une_image(
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Default::default(),
            }],
            &mut f,
        );
        drop(f);
        assert_eq!(rendu, [crate::cles::INSTALLER]);
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

    #[test]
    fn mots_et_clic_entre_deux_mots() {
        let l = Lecture::de("Un  été\nchaud");
        assert_eq!(l.mots, vec![[0, 2], [4, 7], [8, 13]]);
        assert_eq!(l.mot_au_caractere(3), Some(1));
        assert_eq!(l.mot_au_caractere(9), Some(2));
        assert_eq!(l.mot_au_caractere(20), None);
    }

    fn surligne(e: &EtatLecture) -> Vec<String> {
        e.paragraphes
            .iter()
            .filter_map(|p| p.galley.as_ref())
            .flat_map(|g| {
                g.job
                    .sections
                    .iter()
                    .filter(|s| s.format.background != Color32::TRANSPARENT)
                    .map(|s| g.job.text[s.byte_range.start.0..s.byte_range.end.0].to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Le mot lu est surligné (et seulement lui), la page le suit ; un clic sur un mot le
    /// rend.
    #[test]
    fn mot_surligne_suivi_et_clique() {
        let ctx = egui::Context::default();
        let p = palette();
        let paragraphe = "Le soleil se lève sur la colline et les oiseaux chantent.";
        let texte = vec![paragraphe; 80].join("\n");
        let mut l = Lecture::de(&texte);
        let mut e = EtatLecture::default();
        let mut cliques = Vec::new();
        l.mot = Some(1);
        for _ in 0..2 {
            une_image(&ctx, vec![], |ui| {
                cliques.extend(dessiner_lecture(ui, &p, &l, &mut e));
            });
        }
        assert_eq!(surligne(&e), vec!["soleil"]);
        // Un mot loin dans le texte : la page défile jusqu'à lui.
        let loin = l.mots.len() - 2;
        l.mot = Some(loin);
        for _ in 0..20 {
            une_image(&ctx, vec![], |ui| {
                cliques.extend(dessiner_lecture(ui, &p, &l, &mut e));
            });
        }
        assert_eq!(surligne(&e), vec!["oiseaux"]);
        let (vu, zone) = (e.mot_vu.expect("vu"), e.zone_vue.expect("zone"));
        assert!(zone.contains_rect(vu), "{vu:?} dans {zone:?}");
        cliquer(&ctx, vu.center() + vec2(2.0, 0.0), &mut |ui: &mut Ui| {
            cliques.extend(dessiner_lecture(ui, &p, &l, &mut e));
        });
        assert_eq!(cliques, vec![loin]);
    }

    /// Une fenêtre simple et une fenêtre de lecture s'ouvrent dans la boucle, se changent et
    /// se ferment par leur poignée ou par la croix ; le rappel l'apprend. La fenêtre simple
    /// demande la hauteur de son contenu.
    #[test]
    fn fenetres_toutes_faites_dans_la_boucle() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        let recu = Arc::new(Mutex::new(Vec::<String>::new()));
        let r1 = recu.clone();
        let simple = ouvrir_simple(
            &f,
            "simple",
            Ouverture::new("Démo", [0.0, 0.0]),
            &FRANCAIS,
            simple(),
            move |r| r1.lock().unwrap().push(format!("{r:?}")),
        )
        .expect("ouverte");
        let r2 = recu.clone();
        let lecture = ouvrir_lecture(
            &f,
            "lecture",
            Ouverture::new("Lecture", [500.0, 400.0]),
            Lecture::de("Un texte inventé."),
            Some(Box::new(|ui: &mut Ui, _: &Palette| {
                ui.label("Pause");
            })),
            move |r| r2.lock().unwrap().push(format!("{r:?}")),
        )
        .expect("ouverte");
        assert!(
            ouvrir_lecture(
                &f,
                "lecture",
                Ouverture::new("Lecture", [500.0, 400.0]),
                Lecture::default(),
                None,
                |_| {}
            )
            .is_none()
        );
        let (d, _) = image(&ctx, &f);
        assert_eq!(d.len(), 2);
        let _ = image(&ctx, &f);
        let largeur = simple.lire(|s| s.largeur_fenetre());
        assert_eq!(largeur, LARGEUR_SIMPLE + 48.0);
        let tailles: Vec<egui::Vec2> = ORDRES.with(|o| {
            o.borrow()
                .iter()
                .filter_map(|(id, c)| match c {
                    ViewportCommand::InnerSize(t) if *id == id_de("simple") => Some(*t),
                    _ => None,
                })
                .collect()
        });
        let t = tailles
            .last()
            .expect("la fenêtre prend la hauteur de son contenu");
        assert_eq!(t.x, largeur);
        assert!(t.y > 100.0 && t.y < HAUTEUR_MAX, "{t:?}");
        lecture.changer(|l| l.mot = Some(1));
        simple.changer(|s| s.barre = Some(1.0));
        simple.fermer();
        assert!(!simple.ouverte());
        let _ = image(&ctx, &f);
        assert_eq!(f.cles(), vec!["lecture".to_string()]);
        CROIX.with(|c| c.borrow_mut().push(id_de("lecture")));
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().clear());
        assert!(f.cles().is_empty());
        assert!(!lecture.ouverte());
        assert_eq!(*recu.lock().unwrap(), vec!["Fermee", "Fermee"]);
    }
}
