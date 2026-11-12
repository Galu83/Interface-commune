//! Les Paramètres à la façon de Windows 11 (règles : interface Windows 11, point 7), le même
//! squelette dans tous les logiciels :
//!
//! - en haut de la navigation : l'icône du logiciel, son nom, et « Version x.y.z » ;
//! - la navigation à gauche : icônes, petite barre d'accent sur la page choisie ; l'Accueil
//!   facultatif, les pages propres, puis toujours Général, Mise à jour et À propos ;
//! - « Rechercher un paramètre » quand il y a plus de six pages ;
//! - une carte par réglage, effet immédiat, bulle « Annuler », bandeau de redémarrage ;
//! - une sous-page s'ouvre par un chevron « › », avec le chemin « Page › Sous-page » ;
//! - la fenêtre retient sa taille et sa place (sous Windows, `fenetre`).
//!
//! Le dessin ne dépend pas de la fenêtre : [`Parametres::dessiner`] se teste sans rien
//! ouvrir.

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
use crate::theme::{Palette, TAILLE_PETIT};
use crate::{Communs, EtatMiseAJour, Logiciel, cles, icones};
use modele::{Annulation, PAGES_POUR_RECHERCHE, Page, Valeur};

/// Taille de la fenêtre au premier lancement, et taille minimale (points).
pub const TAILLE: [f32; 2] = [940.0, 660.0];
pub const TAILLE_MIN: [f32; 2] = [760.0, 480.0];
/// Largeur de la navigation (points).
pub const LARGEUR_NAVIGATION: f32 = 280.0;

/// Ce qu'une page reçoit pour se dessiner : la palette, et de quoi signaler un changement,
/// un redémarrage nécessaire ou une sous-page.
pub struct Contexte<'a> {
    pub palette: &'a Palette,
    /// La sous-page ouverte dans cette page (son titre), s'il y en a une.
    pub sous_page: Option<String>,
    sous_page_voulue: Option<Option<String>>,
    changements: Vec<(String, String, Valeur, Valeur)>,
    redemarrage: Option<String>,
    aller_a: Option<String>,
}

impl Contexte<'_> {
    /// Un réglage vient d'être changé par l'utilisateur : la bibliothèque le rend au
    /// logiciel (`Logiciel::changer`, effet immédiat) et ouvre la bulle « Annuler ».
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
}

impl Parametres {
    pub fn new(l: &dyn Logiciel, page: Option<&str>) -> Parametres {
        let pages = modele::navigation(l.accueil(), &l.pages());
        let page = modele::rang_de_page(&pages, page);
        Parametres {
            pages,
            page,
            sous_page: None,
            recherche: String::new(),
            annulation: Annulation::default(),
            redemarrage: None,
            confirmer_retablir: false,
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
    pub fn changer(&mut self, l: &dyn Logiciel, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        self.annulation.changer(cle, titre, avant, &apres, Instant::now());
        l.changer(cle, apres);
    }

    /// Toute la fenêtre : navigation et page.
    pub fn dessiner(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) {
        egui::Panel::left("navigation")
            .exact_size(LARGEUR_NAVIGATION)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(couleur(p.fond))
                    .inner_margin(egui::Margin {
                        left: 8,
                        right: 8,
                        top: 14,
                        bottom: 8,
                    }),
            )
            .show(ui, |ui| self.navigation(ui, p, l));
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(couleur(p.fond))
                    .inner_margin(egui::Margin {
                        left: 20,
                        right: 8,
                        top: 18,
                        bottom: 0,
                    }),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width((ui.available_width() - 20.0).max(300.0));
                        self.page_montree(ui, p, l);
                        ui.add_space(16.0);
                    });
            });
    }

    fn navigation(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) {
        let id = l.identite();
        // L'en-tête : icône, nom, et « Version x.y.z » juste dessous.
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let (r, _) = ui.allocate_exact_size(vec2(32.0, 40.0), Sense::hover());
            dessin::icone(ui, r.center(), id.glyphe, 24.0, p.accent);
            ui.vertical(|ui| {
                ui.label(RichText::new(&id.nom).font(semibold(15.0)));
                ui.label(
                    RichText::new(format!("Version {}", id.version))
                        .size(TAILLE_PETIT)
                        .color(couleur(p.texte_secondaire)),
                );
            });
        });
        ui.add_space(10.0);
        let trouvees = if self.pages.len() >= PAGES_POUR_RECHERCHE {
            ui.add(
                egui::TextEdit::singleline(&mut self.recherche)
                    .hint_text("Rechercher un paramètre")
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
                // Les trois pages communes, en bas, séparées des pages propres.
                ui.add_space(6.0);
            }
            let (g, nom) = (self.pages[i].icone, self.pages[i].titre.clone());
            let choisie = self.page == i;
            let (rect, r) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
            if choisie || r.hovered() || r.has_focus() {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(4), couleur(p.survol));
            }
            if choisie {
                let barre =
                    egui::Rect::from_center_size(pos2(rect.left() + 2.0, rect.center().y), vec2(3.0, 16.0));
                ui.painter()
                    .rect_filled(barre, CornerRadius::same(2), couleur(p.accent));
            }
            dessin::icone(ui, pos2(rect.left() + 22.0, rect.center().y), g, 16.0, p.texte);
            ui.painter().text(
                pos2(rect.left() + 46.0, rect.center().y),
                Align2::LEFT_CENTER,
                &nom,
                if choisie {
                    semibold(14.0)
                } else {
                    FontId::proportional(14.0)
                },
                couleur(p.texte),
            );
            r.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, choisie, &nom));
            dessin::focus(ui, p, &r, 4);
            if r.clicked() {
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
                    if dessin::bouton_icone(ui, p, icones::RETOUR, titre).clicked() {
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

    fn page_montree(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) {
        let Some(page) = self.pages.get(self.page).cloned() else {
            return;
        };
        // Le bandeau de redémarrage, en haut de la page.
        if let Some(r) = self.redemarrage.clone() {
            if dessin::bandeau(ui, p, p.orange, icones::RETABLIR, &r, "Redémarrer maintenant") {
                l.action(cles::REDEMARRER);
            }
            ui.add_space(8.0);
        }
        self.entete(ui, p, &page.titre);
        self.bulle_annuler(ui, p, l);
        let communs = l.communs();
        match page.cle.as_str() {
            modele::GENERAL => self.page_general(ui, p, l, &communs),
            modele::MISE_A_JOUR => page_mise_a_jour(ui, p, l, &communs),
            modele::A_PROPOS => page_a_propos(ui, p, l, &communs),
            cle => {
                let mut c = Contexte {
                    palette: p,
                    sous_page: self.sous_page.clone(),
                    sous_page_voulue: None,
                    changements: Vec::new(),
                    redemarrage: None,
                    aller_a: None,
                };
                l.dessiner_page(cle, ui, &mut c);
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
        }
    }

    /// La bulle « Annuler », quelques secondes après un changement.
    fn bulle_annuler(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel) {
        let maintenant = Instant::now();
        let Some(a) = self.annulation.a_annuler(maintenant).cloned() else {
            return;
        };
        if let Some(reste) = self.annulation.reste(maintenant) {
            ui.ctx().request_repaint_after(reste);
        }
        let mut annuler = false;
        let mut fermer = false;
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(format!("« {} » modifié.", a.titre));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    fermer = dessin::bouton_icone(ui, p, icones::FERMER, "Fermer").clicked();
                    annuler = bouton(ui, p, Some(icones::ANNULER), "Annuler", Genre::Normal).clicked();
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

    fn page_general(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel, c: &Communs) {
        if let Some(actif) = c.demarrage {
            let mut v = actif;
            carte(
                ui,
                p,
                icones::DEMARRAGE,
                "Démarrer avec Windows",
                "Le logiciel s'ouvre tout seul à l'ouverture de la session.",
                |ui| {
                    interrupteur_ecrit(ui, p, &mut v);
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::DEMARRAGE,
                    "Démarrer avec Windows",
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        if let Some((langue, langues)) = &c.langue {
            let mut choix = langue.clone();
            carte(
                ui,
                p,
                icones::LANGUE,
                "Langue",
                "La langue de l'interface.",
                |ui| {
                    egui::ComboBox::from_id_salt("langue")
                        .selected_text(choix.clone())
                        .width(170.0)
                        .show_ui(ui, |ui| {
                            for x in langues {
                                ui.selectable_value(&mut choix, x.clone(), x);
                            }
                        });
                },
            );
            if choix != *langue {
                self.changer(
                    l,
                    cles::LANGUE,
                    "Langue",
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
                "Notifications",
                "Une bulle près de l'horloge quand quelque chose demande ton attention.",
                |ui| {
                    interrupteur_ecrit(ui, p, &mut v);
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::NOTIFICATIONS,
                    "Notifications",
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        carte(
            ui,
            p,
            icones::JOURNAL,
            "Journal",
            "Ce que le logiciel a fait, pour comprendre un souci.",
            |ui| {
                if bouton(ui, p, None, "Ouvrir le journal", Genre::Normal).clicked() {
                    l.action(cles::JOURNAL);
                }
            },
        );
        titre_section(ui, "Remise à zéro");
        if self.confirmer_retablir {
            carte_large(
                ui,
                p,
                icones::RETABLIR,
                "Rétablir les réglages par défaut ?",
                "Tous les réglages reviennent à ceux du premier lancement.",
                260.0,
                |ui| {
                    if bouton(ui, p, None, "Non", Genre::Normal).clicked() {
                        self.confirmer_retablir = false;
                    }
                    if bouton(ui, p, None, "Oui, rétablir", Genre::Danger).clicked() {
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
                "Rétablir les réglages par défaut",
                "Une confirmation est demandée avant.",
                |ui| {
                    if bouton(ui, p, None, "Rétablir…", Genre::Danger).clicked() {
                        self.confirmer_retablir = true;
                    }
                },
            );
        }
    }
}

/// Le texte de l'état de la mise à jour, et sa couleur.
pub fn texte_mise_a_jour(e: &EtatMiseAJour, p: &Palette) -> (char, crate::Rgb, String) {
    match e {
        EtatMiseAJour::Inconnu => (
            icones::MISE_A_JOUR,
            p.texte_secondaire,
            "Pas encore cherché".into(),
        ),
        EtatMiseAJour::AJour => (icones::COCHE, p.vert, "Le logiciel est à jour".into()),
        EtatMiseAJour::Recherche => (icones::MISE_A_JOUR, p.orange, "Recherche en cours…".into()),
        EtatMiseAJour::Prete(v) => (icones::INSTALLER, p.accent, format!("Version {v} prête")),
        EtatMiseAJour::Erreur(e) => (icones::PROBLEME, p.rouge, e.clone()),
    }
}

fn page_mise_a_jour(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel, c: &Communs) {
    let (g, couleur_etat, texte) = texte_mise_a_jour(&c.mise_a_jour, p);
    dessin::cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                dessin::etat(ui, couleur_etat, g, &texte);
                ui.label(
                    RichText::new(format!("Version actuelle : {}", l.identite().version))
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
                        "Installer maintenant",
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
                    "Rechercher une mise à jour",
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
        titre_section(ui, "Nouveautés");
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for n in &c.nouveautes {
                ui.add(egui::Label::new(format!("• {n}")).wrap());
            }
        });
    }
    titre_section(ui, "Réglages");
    carte_large(
        ui,
        p,
        icones::DOSSIER,
        "Dossier des installateurs",
        &c.dossier_installateurs,
        120.0,
        |ui| {
            if bouton(ui, p, None, "Choisir…", Genre::Normal).clicked() {
                l.action(cles::DOSSIER);
            }
        },
    );
}

fn page_a_propos(ui: &mut egui::Ui, p: &Palette, l: &dyn Logiciel, c: &Communs) {
    let id = l.identite();
    carte_info(
        ui,
        p,
        id.glyphe,
        &format!("{} {}", id.nom, id.version),
        &id.description,
    );
    carte_info(
        ui,
        p,
        icones::LICENCE,
        "Licence",
        &format!("{} est sous licence {}.", id.nom, id.licence),
    );
    if !c.credits.is_empty() {
        titre_section(ui, "Crédits");
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
