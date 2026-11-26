//! Le dessin des Paramètres (bloc H de la refonte) : le squelette commun, d'après l'état que
//! tient le cœur (`ic_coeur::parametres`, bloc E). Ce module ne décide rien (le cœur le fait)
//! et n'ouvre aucune fenêtre (`crate::parametres::fenetre` le fait) ; il ne lit pas Windows :
//! les « effets d'animation » lui arrivent de [`crate::parametres`] ([`Parametres::dessiner`]).
//!
//! Un fichier par partie : la navigation à gauche (`navigation`), le contexte donné aux pages
//! du logiciel (`contexte`), les gestes rendus au logiciel (`gestes`), la bulle « Annuler » et
//! la carte « Conserver ? » (`bulles`), puis une page commune par fichier (`general`,
//! `mise_a_jour`, `a_propos`).

mod a_propos;
mod bulles;
mod contexte;
mod general;
mod gestes;
mod mise_a_jour;
mod navigation;

use crate::Reglages;
use crate::apparence::couleur;
use crate::mesures::NAVIGATION_MARGES as MARGE_NAVIGATION;
use crate::textes::{Textes, remplir};
use crate::theme::Palette;
use ic_coeur::parametres::{self as coeur, PageSuivie, Suivi};

use crate::parametres::modele;
pub use contexte::Contexte;
use modele::{Annulation, Confirmation, Page};

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
    /// « Conserver ces modifications ? » : ce qui attend la réponse.
    pub confirmation: Confirmation,
    /// Le focus est déjà posé sur « Revenir » (le bouton par défaut) pour cette carte.
    focus_conserver: bool,
    /// Le redémarrage demandé par un réglage (la raison), jusqu'au redémarrage.
    pub redemarrage: Option<String>,
    /// Le brouillon entier du logiciel au début de l'image en cours (pour « Annuler »).
    avant_image: Option<modele::Instantane>,
    /// « Rétablir les réglages par défaut » attend sa confirmation.
    pub confirmer_retablir: bool,
    /// « Effacer le journal » attend sa confirmation.
    pub confirmer_effacer: bool,
    /// Les textes avec lesquels la navigation a été faite (une autre langue la refait).
    textes: Textes,
    /// L'Accueil et les clés des pages propres avec lesquels la navigation a été faite :
    /// des pages qui changent (arrivées après l'ouverture) la refont.
    faite_avec: Suivi,
    logo: Option<Logo>,
    /// Le numéro de cette ouverture (les sections repliables repartent repliées).
    session: u64,
    /// Ce qui était tapé dans la recherche quand la page montrée a été choisie.
    recherche_page: String,
    /// Les « effets d'animation » de Windows, tels qu'ils étaient au début de l'image
    /// (donnés par [`crate::parametres`] : le dessin ne lit pas Windows).
    animations: bool,
}

/// Compte les ouvertures des Paramètres, pour que chacune reparte avec des sections repliées.
static OUVERTURES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Parametres {
    pub fn new(l: &dyn Reglages, page: Option<&str>) -> Parametres {
        let textes = l.textes().clone();
        let accueil = l.accueil();
        let propres = l.pages();
        let faite_avec = Suivi::de(accueil, &propres);
        let pages = modele::navigation(&textes, accueil, &propres);
        let page = modele::rang_de_page(&pages, page);
        Parametres {
            pages,
            page,
            sous_page: None,
            recherche: String::new(),
            annulation: Annulation::default(),
            confirmation: Confirmation::default(),
            focus_conserver: false,
            redemarrage: None,
            avant_image: None,
            confirmer_retablir: false,
            confirmer_effacer: false,
            textes,
            faite_avec,
            logo: None,
            session: OUVERTURES.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            recherche_page: String::new(),
            animations: true,
        }
    }

    /// Va à la page de cette clé, si elle existe.
    pub fn aller_a(&mut self, cle: &str) {
        if let Some(i) = coeur::rang_de_cle(&self.pages, cle) {
            self.page = i;
            self.sous_page = None;
            self.recherche_page.clear();
        }
    }

    /// Va à la page de cette clé comme si l'utilisateur avait tapé `tape` dans « Rechercher un
    /// paramètre » puis choisi cette page dans les résultats : les sections repliables qui
    /// contiennent ces mots s'ouvrent ([`Contexte::recherche`]).
    pub fn aller_a_par_recherche(&mut self, cle: &str, tape: &str) {
        self.aller_a(cle);
        self.recherche = tape.into();
        self.recherche_page = tape.trim().to_string();
    }

    /// La clé de la page montrée.
    pub fn cle(&self) -> &str {
        self.pages.get(self.page).map_or("", |p| p.cle.as_str())
    }

    /// Le titre de la fenêtre : « Paramètres de <logiciel> », dans la langue du logiciel.
    pub fn titre_fenetre(l: &dyn Reglages) -> String {
        remplir(l.textes().parametres_de, &[&l.identite().nom])
    }

    /// La langue, l'Accueil ou les pages propres ont changé (des pages envoyées après
    /// l'ouverture, par exemple) : la navigation est refaite. La page montrée reste si elle
    /// existe encore, sinon Général. Appelé à chaque image par [`Parametres::dessiner`] ; la
    /// comparaison ne porte que sur les clés.
    pub fn suivre(&mut self, l: &dyn Reglages) {
        let accueil = l.accueil();
        let propres = l.pages();
        let langue = *l.textes() != self.textes;
        let memes_pages = self.faite_avec.memes_pages(accueil, &propres);
        if !langue && memes_pages {
            return;
        }
        let cle = self.cle().to_string();
        if langue {
            self.textes = l.textes().clone();
        }
        if !memes_pages {
            self.faite_avec = Suivi::de(accueil, &propres);
        }
        self.pages = modele::navigation(&self.textes, accueil, &propres);
        match coeur::page_suivie(&self.pages, &cle) {
            PageSuivie::Gardee(i) => self.page = i,
            PageSuivie::Perdue(i) => {
                self.page = i;
                self.sous_page = None;
                self.recherche_page.clear();
            }
        }
    }

    /// Toute la fenêtre : navigation et page. `animations` : les « effets d'animation » de
    /// Windows (la rotation des chevrons) ; [`Parametres::dessiner`] les lit et appelle ceci.
    pub(crate) fn dessiner_avec(
        &mut self,
        ui: &mut egui::Ui,
        p: &Palette,
        l: &dyn Reglages,
        animations: bool,
    ) {
        self.animations = animations;
        self.suivre(l);
        self.tic(l);
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
        // « Conserver ces modifications ? », devant tout le reste.
        self.carte_conserver(ui, p, l);
    }

    fn page_montree(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let Some(page) = self.pages.get(self.page).cloned() else {
            return;
        };
        // Le bandeau de redémarrage, en haut de la page.
        self.bandeau_redemarrer(ui, p, l);
        self.entete(ui, p, &page.titre);
        self.avant_image = l.instantane();
        self.bulle_annuler(ui, p, l);
        let communs = l.communs();
        match page.cle.as_str() {
            modele::GENERAL => self.page_general(ui, p, l, &communs),
            modele::MISE_A_JOUR => self.page_mise_a_jour(ui, p, l, &communs),
            modele::A_PROPOS => self.page_a_propos(ui, p, l, &communs),
            cle => self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_page(cle, ui, c)),
        }
        // Le bas de la fenêtre, sous toutes les pages.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_pied(ui, c));
    }
}
