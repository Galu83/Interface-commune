//! Le faux moteur de fenêtres des essais (ORGANIGRAMME, §3.3 J et §5, case `FM`) : ce que la
//! boucle appelle pour préparer, montrer et perdre une fenêtre, sans eframe ni Windows.
//! - [`Faux`] tient la place de la vraie fenêtre du volet dans la boucle de la bibliothèque
//!   ([`super::boucle::Ouvreur`]) : il compte les préparations et ce qu'il montre, et perd la
//!   fenêtre (une panique) sur commande ;
//! - [`Supervisee`] tient la place de la boucle d'un logiciel (façon 2,
//!   [`super::supervision::Supervise`]) : chaque tour suit un script (panique, fermeture,
//!   échec, fin) sur une fausse horloge.
//!
//! - [`contexte`] et [`image`] tiennent la place d'eframe pour les autres fenêtres (Paramètres,
//!   fenêtres du logiciel, [`super::autres`]) : egui croit avoir de vraies fenêtres, rien ne
//!   s'ouvre.

// Ce faux moteur ne sert qu'aux essais : un échec doit y arrêter l'essai (`unwrap`,
// `expect` voulus).
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use ic_coeur::gestes::secours::{Evenement, Fin, Reprise};

use egui::{ViewportCommand, ViewportId};

use super::autres::AutresFenetres;
use super::boucle::Ouvreur;
use super::canal::Reception;
use super::liens::{Demande, Liens};
use super::supervision::{Supervise, superviser_les_fenetres};
use crate::theme::{Palette, Surface};

/// Un faux fil des fenêtres : il compte les préparations de la surface et ce qu'elle
/// montre, sans rien ouvrir.
#[derive(Default)]
pub(super) struct Faux {
    pub(super) preparations: usize,
    /// Chaque ouverture, avec le numéro de la préparation qui l'a montrée.
    pub(super) ouvertures: Vec<(Surface, usize)>,
    pub(super) parametres: Vec<Option<String>>,
    pub(super) secours: usize,
    pub(super) notes: Vec<String>,
    /// Comme la vraie fenêtre : le clic en cours est posé dans les liens jusqu'à ce qu'il
    /// soit montré.
    pub(super) liens: Option<Arc<Liens>>,
    /// Tant que ce nombre n'est pas épuisé, montrer le petit menu fait paniquer la
    /// boucle (la fenêtre est perdue, comme eframe qui ne retrouve plus son contexte
    /// OpenGL après la veille des écrans).
    pub(super) pertes_au_menu: usize,
}

impl Faux {
    pub(super) fn montrer(&mut self, d: &Demande) -> bool {
        let s = match d {
            Demande::Volet { .. } => Surface::Volet,
            Demande::Menu { .. } => Surface::Menu,
            Demande::Parametres(page) => {
                // Ouverts dans la même boucle, sans quitter la surface.
                self.parametres.push(page.clone());
                return true;
            }
            Demande::Quitter => return false,
        };
        if let Some(l) = &self.liens {
            l.poser_en_cours(Some(d.clone()));
        }
        if s == Surface::Menu && self.pertes_au_menu > 0 {
            self.pertes_au_menu -= 1;
            panic!("contexte OpenGL perdu (essai)");
        }
        self.ouvertures.push((s, self.preparations));
        if let Some(l) = &self.liens {
            l.poser_en_cours(None);
        }
        true
    }
}

impl Ouvreur for Faux {
    fn surface(
        &mut self,
        demandes: &Reception,
        premiere: Option<Demande>,
    ) -> Result<Option<Demande>, (String, Option<Demande>)> {
        self.preparations += 1;
        if let Some(d) = premiere {
            assert!(self.montrer(&d), "Quitter n'arrive jamais avant la préparation");
        }
        loop {
            match demandes.recevoir() {
                Some(d) if self.montrer(&d) => {}
                autre => return Ok(autre),
            }
        }
    }

    fn parametres(&mut self, page: Option<String>) -> Result<(), String> {
        self.parametres.push(page.map(|p| format!("seuls:{p}")));
        Ok(())
    }

    fn secours(&mut self, _: &Demande) {
        self.secours += 1;
    }

    fn noter(&mut self, ligne: String) {
        self.notes.push(ligne);
    }
}

pub(super) fn volet() -> Demande {
    Demande::Volet {
        icone: [0, 0, 16, 16],
        clic: Instant::now(),
    }
}

pub(super) fn menu() -> Demande {
    Demande::Menu {
        icone: [0, 0, 16, 16],
        ancre: [8, 8],
        clic: Instant::now(),
    }
}

/// Un tour de la fausse boucle supervisée.
#[derive(Debug, Clone, Copy)]
pub(super) enum Tour {
    Panique,
    Fermee,
    Erreur,
    /// Le logiciel se ferme pendant ce tour.
    Quitter,
}

/// Une fausse boucle de fenêtres pour la supervision : chaque tour suit le script, à
/// l'heure dite (en secondes depuis le départ), sans rien ouvrir.
pub(super) struct Supervisee {
    pub(super) script: std::collections::VecDeque<(u64, Tour)>,
    pub(super) horloge: Arc<std::sync::atomic::AtomicU64>,
    pub(super) tours: usize,
    pub(super) evenements: Vec<Evenement>,
    pub(super) attentes: usize,
    /// `attendre_un_clic` rend faux (le logiciel se ferme pendant la pause).
    pub(super) ferme_en_pause: bool,
    pub(super) fini: bool,
}

impl Supervisee {
    pub(super) fn new(script: &[(u64, Tour)]) -> Supervisee {
        Supervisee {
            script: script.iter().copied().collect(),
            horloge: Arc::default(),
            tours: 0,
            evenements: Vec::new(),
            attentes: 0,
            ferme_en_pause: false,
            fini: false,
        }
    }

    pub(super) fn superviser(&mut self) -> Fin {
        let depart = Instant::now();
        let h = self.horloge.clone();
        superviser_les_fenetres(self, move || {
            depart + Duration::from_secs(h.load(Ordering::Acquire))
        })
    }

    pub(super) fn reprises(&self) -> Vec<Reprise> {
        self.evenements.iter().map(|e| e.reprise).collect()
    }
}

impl Supervise for Supervisee {
    fn boucle(&mut self) -> Result<(), String> {
        self.tours += 1;
        let (seconde, tour) = self.script.pop_front().expect("script épuisé");
        self.horloge.store(seconde, Ordering::Release);
        match tour {
            Tour::Panique => panic!("contexte OpenGL perdu (essai)"),
            Tour::Fermee => Ok(()),
            Tour::Erreur => Err("OpenGL indisponible (essai)".into()),
            Tour::Quitter => {
                self.fini = true;
                Ok(())
            }
        }
    }

    fn perdues(&mut self, e: &Evenement) {
        self.evenements.push(e.clone());
    }

    fn attendre_un_clic(&mut self) -> bool {
        self.attentes += 1;
        !self.ferme_en_pause
    }

    fn quitter(&self) -> bool {
        self.fini
    }
}

// --- Le faux moteur egui des autres fenêtres (partagé : egui n'en garde qu'un pour tout le
// programme) : les Paramètres et les fenêtres du logiciel, dans la boucle de la bibliothèque.

// ÉTAT GLOBAL (essais seulement) : le faux moteur tient la place d'eframe, appelé sans poignée
// vers l'essai ; un par fil, pour que les essais parallèles ne se mêlent pas.
thread_local! {
    /// Les fenêtres que le faux moteur a dessinées à la dernière image.
    pub(super) static DESSINEES: RefCell<Vec<ViewportId>> = const { RefCell::new(Vec::new()) };
    /// Les fenêtres dont la croix est cliquée à la prochaine image.
    pub(super) static CROIX: RefCell<Vec<ViewportId>> = const { RefCell::new(Vec::new()) };
    /// Le faux moteur rate la création des fenêtres (il rend la main sans dessiner, comme
    /// eframe quand Windows refuse la fenêtre ou son contexte OpenGL).
    pub(super) static CREATION_RATEE: Cell<bool> = const { Cell::new(false) };
    /// Les ordres que les fenêtres ont donnés (taille, titre…), depuis le début.
    pub(super) static ORDRES: RefCell<Vec<(ViewportId, ViewportCommand)>> = const { RefCell::new(Vec::new()) };
}

/// Un faux moteur de fenêtres : egui croit avoir de vraies fenêtres, rien ne s'ouvre.
pub(super) fn contexte() -> egui::Context {
    let ctx = egui::Context::default();
    ctx.set_embed_viewports(false);
    egui::Context::set_immediate_viewport_renderer(|ctx, mut vp| {
        if CREATION_RATEE.with(Cell::get) {
            return;
        }
        DESSINEES.with(|d| d.borrow_mut().push(vp.ids.this));
        let mut input = egui::RawInput {
            viewport_id: vp.ids.this,
            ..Default::default()
        };
        let mut info = egui::ViewportInfo::default();
        if CROIX.with(|c| c.borrow().contains(&vp.ids.this)) {
            info.events.push(egui::ViewportEvent::Close);
        }
        input.viewports.insert(vp.ids.this, info);
        let mut sortie = ctx.run_ui(input, |ui| (vp.viewport_ui_cb)(ui));
        sortie.textures_delta.clear();
    });
    ctx
}

/// Une image de la fenêtre principale ; rend les fenêtres dessinées et celles qu'egui
/// garde.
pub(super) fn image(ctx: &egui::Context, f: &AutresFenetres) -> (Vec<ViewportId>, Vec<ViewportId>) {
    DESSINEES.with(|d| d.borrow_mut().clear());
    let p = Palette::de(&crate::theme::ChoixWindows::default());
    let mut sortie = ctx.run_ui(egui::RawInput::default(), |ui| f.montrer(ui.ctx(), &p));
    sortie.textures_delta.clear();
    ORDRES.with(|o| {
        for (id, v) in &sortie.viewport_output {
            o.borrow_mut().extend(v.commands.iter().map(|c| (*id, c.clone())));
        }
    });
    let dessinees = DESSINEES.with(|d| d.borrow().clone());
    let gardees = sortie
        .viewport_output
        .keys()
        .copied()
        .filter(|i| *i != ViewportId::ROOT)
        .collect();
    (dessinees, gardees)
}
