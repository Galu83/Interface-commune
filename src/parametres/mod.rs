//! Les Paramètres à la façon de Windows 11 (règles : interface Windows 11, point 7), le même
//! squelette dans tous les logiciels, dessiné comme ceux de PowerPeek (le modèle) :
//!
//! - barre de titre « Paramètres de <logiciel> » ;
//! - en haut de la navigation : le vrai logo du logiciel (32 points), son nom (16 points,
//!   semibold) et juste dessous « Version x.y.z » (12 points) ;
//! - la navigation à gauche, dont la largeur suit ses textes (250 points au moins, comme
//!   PowerPeek) : lignes de 38 points, icônes, petite barre d'accent sur la page choisie ;
//!   l'Accueil facultatif, les pages propres, puis, séparées par 6 points, toujours Général,
//!   Mise à jour et À propos (juste après, jamais collées au bas de la fenêtre) ;
//! - « Rechercher un paramètre » dès sept pages ;
//! - une carte par réglage, effet immédiat, bulle « Annuler », bandeau de redémarrage ;
//! - une sous-page s'ouvre par un chevron « › », avec le chemin « Page › Sous-page » ;
//! - la fenêtre retient sa taille et sa place (sous Windows, `fenetre`).
//!
//! Tous les mots viennent de [`crate::textes`] (français ou anglais) ou du logiciel. Le
//! dessin ne dépend pas de la fenêtre : [`Parametres::dessiner`] se teste sans rien ouvrir,
//! et un logiciel qui a sa propre boucle de fenêtres peut le dessiner dans la sienne.

#[cfg(windows)]
pub mod fenetre;
pub mod modele;

use std::time::Instant;

use egui::{Align2, CornerRadius, FontId, RichText, Sense, pos2, vec2};

use crate::apparence::couleur;
use crate::dessin::{
    self, Genre, bouton, carte, carte_info, carte_large, interrupteur_ecrit, titre_page, titre_section,
};
use crate::polices::semibold;
use crate::textes::{Textes, remplir};
use crate::theme::{Palette, TAILLE_PETIT};
use crate::{Communs, EtatMiseAJour, Reglages, cles, icones};
use modele::{Annulation, PAGES_POUR_RECHERCHE, Page, Valeur};

/// La clé de la fenêtre des Paramètres parmi les autres fenêtres de la boucle
/// ([`crate::autres_fenetres`]) : réservée.
pub const CLE_FENETRE: &str = "parametres";

// Les mesures, écrites une seule fois dans `crate::mesures`.
pub use crate::mesures::{
    ESPACE_COMMUNES, NAVIGATION_LARGEUR_MAX as LARGEUR_NAVIGATION_MAX,
    NAVIGATION_LARGEUR_MIN as LARGEUR_NAVIGATION_MIN, NAVIGATION_LIGNE as LIGNE_NAVIGATION,
    PARAMETRES_TAILLE as TAILLE, PARAMETRES_TAILLE_MIN as TAILLE_MIN, RAYON_ALERTE, TAILLE_LOGO, TAILLE_NOM,
};
use crate::mesures::{NAVIGATION_DROITE, NAVIGATION_MARGES as MARGE_NAVIGATION, NAVIGATION_TEXTE_X};

/// Ce qu'une page reçoit pour se dessiner : la palette, et de quoi signaler un changement,
/// un redémarrage nécessaire ou une sous-page.
pub struct Contexte<'a> {
    pub palette: &'a Palette,
    /// Les mots communs, dans la langue du logiciel.
    pub textes: &'a Textes,
    /// La sous-page ouverte dans cette page (son titre), s'il y en a une.
    pub sous_page: Option<String>,
    sous_page_voulue: Option<Option<String>>,
    changements: Vec<(String, String, Valeur, Valeur)>,
    redemarrage: Option<String>,
    aller_a: Option<String>,
}

impl Contexte<'_> {
    /// Un réglage vient d'être changé par l'utilisateur : la bibliothèque le rend au
    /// logiciel (`Reglages::changer`, effet immédiat) et ouvre la bulle « Annuler ».
    pub fn changer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        self.changements.push((cle.into(), titre.into(), avant, apres));
    }

    /// Ce changement ne prend effet qu'au redémarrage : le bandeau le dit en haut de la page.
    pub fn redemarrage_necessaire(&mut self, raison: &str) {
        self.redemarrage = Some(raison.into());
    }

    /// Ouvre une sous-page de la page (`None` : revient à la page).
    pub fn ouvrir_sous_page(&mut self, titre: Option<&str>) {
        self.sous_page_voulue = Some(titre.map(str::to_string));
    }

    /// Va à une autre page (sa clé).
    pub fn aller_a(&mut self, cle: &str) {
        self.aller_a = Some(cle.into());
    }
}

/// Le logo dessiné, gardé entre deux images (une texture par taille et par thème).
struct Logo {
    pixels: u32,
    sombre: bool,
    texture: Option<egui::TextureHandle>,
}

/// Ce que la fenêtre des Paramètres retient entre deux images.
pub struct Parametres {
    pub pages: Vec<Page>,
    pub page: usize,
    pub sous_page: Option<String>,
    pub recherche: String,
    pub annulation: Annulation,
    /// Le redémarrage demandé par un réglage (la raison), jusqu'au redémarrage.
    pub redemarrage: Option<String>,
    /// « Rétablir les réglages par défaut » attend sa confirmation.
    pub confirmer_retablir: bool,
    /// Les textes avec lesquels la navigation a été faite (une autre langue la refait).
    textes: Textes,
    logo: Option<Logo>,
}

impl Parametres {
    pub fn new(l: &dyn Reglages, page: Option<&str>) -> Parametres {
        let textes = l.textes().clone();
        let pages = modele::navigation(&textes, l.accueil(), &l.pages());
        let page = modele::rang_de_page(&pages, page);
        Parametres {
            pages,
            page,
            sous_page: None,
            recherche: String::new(),
            annulation: Annulation::default(),
            redemarrage: None,
            confirmer_retablir: false,
            textes,
            logo: None,
        }
    }

    /// Va à la page de cette clé, si elle existe.
    pub fn aller_a(&mut self, cle: &str) {
        if let Some(i) = self.pages.iter().position(|p| p.cle == cle) {
            self.page = i;
            self.sous_page = None;
        }
    }

    /// La clé de la page montrée.
    pub fn cle(&self) -> &str {
        self.pages.get(self.page).map_or("", |p| p.cle.as_str())
    }

    /// Un changement : rendu au logiciel tout de suite, et la bulle « Annuler ».
    pub fn changer(&mut self, l: &dyn Reglages, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        self.annulation.changer(cle, titre, avant, &apres, Instant::now());
        l.changer(cle, apres);
    }

    /// Le titre de la fenêtre : « Paramètres de <logiciel> », dans la langue du logiciel.
    pub fn titre_fenetre(l: &dyn Reglages) -> String {
        remplir(l.textes().parametres_de, &[&l.identite().nom])
    }

    /// La langue a changé : la navigation est refaite, sur la même page.
    fn suivre_la_langue(&mut self, l: &dyn Reglages) {
        if *l.textes() == self.textes {
            return;
        }
        let cle = self.cle().to_string();
        self.textes = l.textes().clone();
        self.pages = modele::navigation(&self.textes, l.accueil(), &l.pages());
        self.page = modele::rang_de_page(&self.pages, Some(&cle));
    }

    /// La largeur de la navigation (points) : ce qu'il faut à l'en-tête et aux noms des
    /// pages pour tenir sur une ligne, entre [`LARGEUR_NAVIGATION_MIN`] et
    /// [`LARGEUR_NAVIGATION_MAX`].
    pub fn largeur_navigation(&self, ui: &egui::Ui, l: &dyn Reglages) -> f32 {
        let mesure = |t: &str, f: FontId| {
            ui.painter()
                .layout_no_wrap(t.to_string(), f, egui::Color32::WHITE)
                .size()
                .x
        };
        let id = l.identite();
        let nom = mesure(&id.nom, semibold(TAILLE_NOM));
        let version = mesure(
            &remplir(self.textes.version, &[&id.version]),
            FontId::proportional(TAILLE_PETIT),
        );
        let entete = 8.0 + TAILLE_LOGO + 4.0 + ui.spacing().item_spacing.x + nom.max(version);
        let lignes = self
            .pages
            .iter()
            .map(|p| NAVIGATION_TEXTE_X + mesure(&p.titre, FontId::proportional(14.0)) + NAVIGATION_DROITE)
            .fold(0.0, f32::max);
        let marges = f32::from(MARGE_NAVIGATION[0]) + f32::from(MARGE_NAVIGATION[1]);
        (entete.max(lignes) + marges)
            .ceil()
            .clamp(LARGEUR_NAVIGATION_MIN, LARGEUR_NAVIGATION_MAX)
    }

    /// Toute la fenêtre : navigation et page.
    pub fn dessiner(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        self.suivre_la_langue(l);
        let largeur = self.largeur_navigation(ui, l);
        let [g, d, h, b] = MARGE_NAVIGATION;
        egui::Panel::left("navigation")
            .exact_size(largeur)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(couleur(p.fond))
                    .inner_margin(egui::Margin {
                        left: g,
                        right: d,
                        top: h,
                        bottom: b,
                    }),
            )
            .show(ui, |ui| self.navigation(ui, p, l));
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(couleur(p.fond))
                    .inner_margin(egui::Margin {
                        left: 18,
                        right: 4,
                        top: 22,
                        bottom: 0,
                    }),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin {
                                left: 0,
                                right: 24,
                                top: 0,
                                bottom: 24,
                            })
                            .show(ui, |ui| {
                                ui.set_max_width(ui.available_width().max(300.0));
                                self.page_montree(ui, p, l);
                            });
                    });
            });
    }

    /// Le logo à sa taille réelle (pixels de l'écran), sans flou.
    fn logo(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let ppp = ui.ctx().pixels_per_point();
        let px = (TAILLE_LOGO * ppp).round().clamp(16.0, 256.0) as u32;
        let a_refaire = self
            .logo
            .as_ref()
            .is_none_or(|g| g.pixels != px || (g.sombre != p.sombre && l.identite().logo.is_none()));
        if a_refaire {
            let texture = l.identite().logo_rvba(px, p.sombre).map(|rvba| {
                let image = egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &rvba);
                ui.ctx()
                    .load_texture("logo du logiciel", image, egui::TextureOptions::NEAREST)
            });
            self.logo = Some(Logo {
                pixels: px,
                sombre: p.sombre,
                texture,
            });
        }
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(px as f32 / ppp), Sense::hover());
        let coin = pos2((rect.min.x * ppp).round() / ppp, (rect.min.y * ppp).round() / ppp);
        if let Some(t) = self.logo.as_ref().and_then(|g| g.texture.as_ref()) {
            let uv = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(
                t.id(),
                egui::Rect::from_min_size(coin, rect.size()),
                uv,
                egui::Color32::WHITE,
            );
        }
    }

    fn navigation(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let id = l.identite();
        ui.add_space(4.0);
        // L'en-tête : le logo, puis le nom et « Version x.y.z » l'un sous l'autre.
        let version = remplir(self.textes.version, &[&id.version]);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            self.logo(ui, p, l);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(&id.nom).font(semibold(TAILLE_NOM)));
                ui.label(
                    RichText::new(version)
                        .size(TAILLE_PETIT)
                        .color(couleur(p.texte_secondaire)),
                );
            });
        });
        ui.add_space(18.0);
        let trouvees = if self.pages.len() >= PAGES_POUR_RECHERCHE {
            ui.add(
                egui::TextEdit::singleline(&mut self.recherche)
                    .hint_text(self.textes.rechercher_un_parametre)
                    .min_size(vec2(0.0, 32.0))
                    .desired_width(f32::INFINITY)
                    .char_limit(60),
            );
            ui.add_space(6.0);
            let t = modele::chercher(&self.pages, &self.recherche);
            (!self.recherche.trim().is_empty()).then_some(t)
        } else {
            None
        };
        let communes = self.pages.len().saturating_sub(3);
        for i in 0..self.pages.len() {
            if trouvees.as_ref().is_some_and(|t| !t.contains(&i)) {
                continue;
            }
            if i == communes && i > 0 && trouvees.is_none() {
                // Les trois pages communes, juste après les pages propres, séparées d'elles.
                ui.add_space(ESPACE_COMMUNES);
            }
            let (g, nom) = (self.pages[i].icone, self.pages[i].titre.clone());
            let choisie = self.page == i;
            let (rect, r) =
                ui.allocate_exact_size(vec2(ui.available_width(), LIGNE_NAVIGATION), Sense::click());
            if choisie || r.hovered() || r.has_focus() {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(5), couleur(p.survol));
            }
            if choisie {
                let barre =
                    egui::Rect::from_center_size(pos2(rect.left() + 1.5, rect.center().y), vec2(3.0, 16.0));
                ui.painter()
                    .rect_filled(barre, CornerRadius::same(2), couleur(p.accent));
            }
            dessin::icone(
                ui,
                pos2(rect.left() + 16.0 + 8.0, rect.center().y),
                g,
                16.0,
                p.texte,
            );
            ui.painter().text(
                pos2(rect.left() + NAVIGATION_TEXTE_X, rect.center().y),
                Align2::LEFT_CENTER,
                &nom,
                FontId::proportional(14.0),
                couleur(p.texte),
            );
            if l.alerte(&self.pages[i].cle) {
                // Le point d'alerte de PowerPeek : à droite de la ligne, couleur d'accent.
                ui.painter().circle_filled(
                    pos2(rect.right() - 16.0, rect.center().y),
                    RAYON_ALERTE,
                    couleur(p.accent),
                );
            }
            r.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, choisie, &nom));
            dessin::focus(ui, p, &r, 5);
            if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                self.page = i;
                self.sous_page = None;
                self.confirmer_retablir = false;
            }
            ui.add_space(2.0);
        }
    }

    /// Le chemin en haut de la page (« Page › Sous-page ») et le titre.
    fn entete(&mut self, ui: &mut egui::Ui, p: &Palette, titre: &str) {
        match self.sous_page.clone() {
            None => titre_page(ui, p, titre, ""),
            Some(s) => {
                ui.horizontal(|ui| {
                    if dessin::bouton_icone(ui, p, icones::RETOUR, self.textes.retour).clicked() {
                        self.sous_page = None;
                    }
                    ui.label(
                        RichText::new(format!("{titre} › {s}"))
                            .font(crate::polices::titre(crate::theme::TAILLE_TITRE)),
                    );
                });
                ui.add_space(8.0);
            }
        }
    }

    fn page_montree(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let Some(page) = self.pages.get(self.page).cloned() else {
            return;
        };
        // Le bandeau de redémarrage, en haut de la page.
        if let Some(r) = self.redemarrage.clone() {
            if dessin::bandeau(
                ui,
                p,
                p.orange,
                icones::RETABLIR,
                &r,
                self.textes.redemarrer_maintenant,
            ) {
                l.action(cles::REDEMARRER);
            }
            ui.add_space(8.0);
        }
        self.entete(ui, p, &page.titre);
        self.bulle_annuler(ui, p, l);
        let communs = l.communs();
        let t = self.textes.clone();
        match page.cle.as_str() {
            modele::GENERAL => self.page_general(ui, p, l, &communs),
            modele::MISE_A_JOUR => page_mise_a_jour(ui, p, &t, l, &communs),
            modele::A_PROPOS => self.page_a_propos(ui, p, l, &communs),
            cle => self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_page(cle, ui, c)),
        }
    }

    /// Fait dessiner le logiciel (une page propre, ses cartes de Général), puis applique ce
    /// qu'il a demandé : changements, redémarrage, sous-page, autre page.
    fn avec_contexte(
        &mut self,
        ui: &mut egui::Ui,
        p: &Palette,
        l: &dyn Reglages,
        dessin: impl FnOnce(&dyn Reglages, &mut egui::Ui, &mut Contexte<'_>),
    ) {
        let t = self.textes.clone();
        let mut c = Contexte {
            palette: p,
            textes: &t,
            sous_page: self.sous_page.clone(),
            sous_page_voulue: None,
            changements: Vec::new(),
            redemarrage: None,
            aller_a: None,
        };
        dessin(l, ui, &mut c);
        for (cle, titre, avant, apres) in c.changements {
            self.changer(l, &cle, &titre, avant, apres);
        }
        if let Some(r) = c.redemarrage {
            self.redemarrage = Some(r);
        }
        if let Some(s) = c.sous_page_voulue {
            self.sous_page = s;
        }
        if let Some(a) = c.aller_a {
            self.aller_a(&a);
        }
    }

    /// La bulle « Annuler », quelques secondes après un changement.
    fn bulle_annuler(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let maintenant = Instant::now();
        let Some(a) = self.annulation.a_annuler(maintenant).cloned() else {
            return;
        };
        if let Some(reste) = self.annulation.reste(maintenant) {
            ui.ctx().request_repaint_after(reste);
        }
        let mut annuler = false;
        let mut fermer = false;
        let t = &self.textes;
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(remplir(t.modifie, &[&a.titre]));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    fermer = dessin::bouton_icone(ui, p, icones::FERMER, t.fermer).clicked();
                    annuler = bouton(ui, p, Some(icones::ANNULER), t.annuler, Genre::Normal).clicked();
                });
            });
        });
        ui.add_space(8.0);
        if annuler && let Some((cle, avant)) = self.annulation.annuler(Instant::now()) {
            l.changer(&cle, avant);
        } else if fermer {
            self.annulation.oublier();
        }
    }

    fn page_general(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let t = self.textes.clone();
        if let Some(actif) = c.demarrage {
            let mut v = actif;
            carte(
                ui,
                p,
                icones::DEMARRAGE,
                t.demarrer_avec_windows,
                t.demarrer_explication,
                |ui| {
                    interrupteur_ecrit(ui, p, &t, &mut v);
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::DEMARRAGE,
                    t.demarrer_avec_windows,
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        if let Some((langue, langues)) = &c.langue {
            let mut choix = langue.clone();
            carte(ui, p, icones::LANGUE, t.langue, t.langue_explication, |ui| {
                egui::ComboBox::from_id_salt("langue")
                    .selected_text(choix.clone())
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        for x in langues {
                            ui.selectable_value(&mut choix, x.clone(), x);
                        }
                    });
            });
            if choix != *langue {
                self.changer(
                    l,
                    cles::LANGUE,
                    t.langue,
                    Valeur::Texte(langue.clone()),
                    Valeur::Texte(choix),
                );
            }
        }
        if let Some(actif) = c.notifications {
            let mut v = actif;
            carte(
                ui,
                p,
                icones::NOTIFICATIONS,
                t.notifications,
                t.notifications_explication,
                |ui| {
                    interrupteur_ecrit(ui, p, &t, &mut v);
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::NOTIFICATIONS,
                    t.notifications,
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        // Les cartes propres au logiciel, avant le journal.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_general(ui, c));
        carte(ui, p, icones::JOURNAL, t.journal, t.journal_explication, |ui| {
            if bouton(ui, p, None, t.ouvrir_le_journal, Genre::Normal).clicked() {
                l.action(cles::JOURNAL);
            }
        });
        titre_section(ui, t.remise_a_zero);
        if self.confirmer_retablir {
            carte_large(
                ui,
                p,
                icones::RETABLIR,
                t.retablir_question,
                t.retablir_question_explication,
                260.0,
                |ui| {
                    if bouton(ui, p, None, t.non, Genre::Normal).clicked() {
                        self.confirmer_retablir = false;
                    }
                    if bouton(ui, p, None, t.oui_retablir, Genre::Danger).clicked() {
                        self.confirmer_retablir = false;
                        l.action(cles::RETABLIR);
                    }
                },
            );
        } else {
            carte(
                ui,
                p,
                icones::RETABLIR,
                t.retablir,
                t.retablir_explication,
                |ui| {
                    if bouton(ui, p, None, t.retablir_bouton, Genre::Danger).clicked() {
                        self.confirmer_retablir = true;
                    }
                },
            );
        }
    }

    fn page_a_propos(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let id = l.identite();
        let t = self.textes.clone();
        // Le logo, le nom et la version, puis ce que fait le logiciel.
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.set_min_height(48.0);
                self.logo(ui, p, l);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("{} {}", id.nom, id.version)).font(semibold(14.0)));
                    if !id.description.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(dessin::insecables(&id.description))
                                    .size(TAILLE_PETIT)
                                    .color(couleur(p.texte_secondaire)),
                            )
                            .wrap(),
                        );
                    }
                });
            });
        });
        carte_info(
            ui,
            p,
            icones::LICENCE,
            t.licence,
            &remplir(t.sous_licence, &[&id.nom, &id.licence]),
        );
        if !c.credits.is_empty() {
            titre_section(ui, t.credits);
            dessin::cadre_carte(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                for cr in &c.credits {
                    ui.horizontal(|ui| {
                        ui.label(&cr.nom);
                        ui.label(RichText::new(&cr.licence).color(couleur(p.texte_secondaire)));
                    });
                }
            });
        }
    }
}

/// Le texte de l'état de la mise à jour, et sa couleur.
pub fn texte_mise_a_jour(e: &EtatMiseAJour, p: &Palette, t: &Textes) -> (char, crate::Rgb, String) {
    match e {
        EtatMiseAJour::Inconnu => (icones::MISE_A_JOUR, p.texte_secondaire, t.maj_pas_cherche.into()),
        EtatMiseAJour::AJour => (icones::COCHE, p.vert, t.maj_a_jour.into()),
        EtatMiseAJour::Recherche => (icones::MISE_A_JOUR, p.orange, t.maj_recherche.into()),
        EtatMiseAJour::Prete(v) => (icones::INSTALLER, p.accent, remplir(t.maj_prete, &[v])),
        EtatMiseAJour::Erreur(e) => (icones::PROBLEME, p.rouge, e.clone()),
    }
}

fn page_mise_a_jour(ui: &mut egui::Ui, p: &Palette, t: &Textes, l: &dyn Reglages, c: &Communs) {
    let (g, couleur_etat, texte) = texte_mise_a_jour(&c.mise_a_jour, p, t);
    dessin::cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                dessin::etat(ui, couleur_etat, g, &texte);
                ui.label(
                    RichText::new(remplir(t.version_actuelle, &[&l.identite().version]))
                        .size(TAILLE_PETIT)
                        .color(couleur(p.texte_secondaire)),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let EtatMiseAJour::Prete(_) = c.mise_a_jour {
                    if bouton(
                        ui,
                        p,
                        Some(icones::INSTALLER),
                        t.installer_maintenant,
                        Genre::Accent,
                    )
                    .clicked()
                    {
                        l.action(cles::INSTALLER);
                    }
                } else if bouton(
                    ui,
                    p,
                    Some(icones::MISE_A_JOUR),
                    t.rechercher_mise_a_jour,
                    Genre::Normal,
                )
                .clicked()
                {
                    l.rechercher_mise_a_jour();
                }
            });
        });
    });
    if !c.nouveautes.is_empty() {
        titre_section(ui, t.nouveautes);
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for n in &c.nouveautes {
                ui.add(egui::Label::new(format!("• {n}")).wrap());
            }
        });
    }
    titre_section(ui, t.reglages);
    carte_large(
        ui,
        p,
        icones::DOSSIER,
        t.dossier_installateurs,
        &c.dossier_installateurs,
        120.0,
        |ui| {
            if !c.dossier_fixe && bouton(ui, p, None, t.choisir, Genre::Normal).clicked() {
                l.action(cles::DOSSIER);
            }
        },
    );
}
