//! La fenêtre des Paramètres (sous Windows) : coins arrondis et barre de titre sombre par
//! Windows, aucune animation à nous (Windows anime déjà les fenêtres), taille et place
//! retenues d'une ouverture à l'autre et toujours en entier sur un écran, taille minimale.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use eframe::egui::{self, FontDefinitions, ViewportCommand};

use super::{Parametres, TAILLE, TAILLE_MIN};
use crate::Logiciel;
use crate::apparence::{Apparence, couleur};
use crate::fenetres::Liens;
use crate::place::{Place, garder_visible, place_valable};
use crate::systeme::{self, hwnd_de};

/// L'icône de la fenêtre (barre des tâches, Alt+Tab), en RVBA.
pub fn icone_fenetre(l: &dyn Logiciel, sombre: bool) -> egui::IconData {
    let t = 32;
    let mut px = (l.identite().icone)(t, sombre);
    if px.len() != (t * t * 4) as usize {
        px = vec![0; (t * t * 4) as usize];
    }
    for p in px.chunks_mut(4) {
        p.swap(0, 2);
    }
    egui::IconData {
        rgba: px,
        width: t,
        height: t,
    }
}

struct Fenetre {
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    apparence: Apparence,
    parametres: Parametres,
    hwnd: Option<isize>,
    montree: bool,
    /// La place retenue à poser une fois la fenêtre créée (pixels).
    place: Option<Place>,
}

impl Fenetre {
    /// La place de la fenêtre, à retenir.
    fn retenir(&self) {
        if let Some(h) = self.hwnd
            && let Some(r) = systeme::rect_fenetre(h)
            && let Some(p) = place_valable([r[0], r[1], r[2] - r[0], r[3] - r[1]])
        {
            self.logiciel.retenir_place(p);
        }
    }
}

impl eframe::App for Fenetre {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        couleur(self.apparence.palette.fond).to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.apparence.suivre(&ctx)
            && let Some(h) = self.hwnd
        {
            systeme::habiller(h, self.apparence.palette.sombre);
            let icone = icone_fenetre(&*self.logiciel, self.apparence.palette.sombre);
            ctx.send_viewport_cmd(ViewportCommand::Icon(Some(Arc::new(icone))));
        }
        if self.hwnd.is_none() {
            self.hwnd = hwnd_de(frame);
            if let Some(h) = self.hwnd {
                systeme::habiller(h, self.apparence.palette.sombre);
                if let Some(p) = self.place.take() {
                    systeme::poser(h, p);
                }
            }
        }
        if !self.montree {
            // Barre de titre habillée avant de montrer la fenêtre : pas d'éclair blanc.
            self.montree = true;
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
        if let Some(page) = self.liens.page_voulue.lock().ok().and_then(|mut p| p.take()) {
            self.parametres.aller_a(&page);
        }
        if self.liens.devant.swap(false, Ordering::AcqRel) {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            if let Some(h) = self.hwnd {
                systeme::premier_plan(h);
            }
        }
        if self.liens.fermer.swap(false, Ordering::AcqRel) {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if ui.input(|i| i.viewport().close_requested()) {
            self.retenir();
        }
        let p = self.apparence.palette.clone();
        let l = self.logiciel.clone();
        self.parametres.dessiner(ui, &p, &*l);
    }
}

/// Ouvre les Paramètres à cette page (sa clé) ; rend la main quand la fenêtre se ferme.
pub fn ouvrir(
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    page: Option<String>,
    polices: FontDefinitions,
) -> Result<(), String> {
    let apparence = Apparence::new(None);
    let nom = logiciel.identite().nom.clone();
    // La place retenue, ramenée sur l'écran le plus proche (il a pu changer).
    let place = logiciel.place_parametres().and_then(place_valable).and_then(|p| {
        let travail = systeme::zone_de_travail_pres_de([p[0], p[1], p[0] + p[2], p[1] + p[3]])?;
        Some(garder_visible(
            p,
            travail,
            [TAILLE_MIN[0] as i32, TAILLE_MIN[1] as i32],
        ))
    });
    let viewport = egui::ViewportBuilder::default()
        .with_title(format!("Paramètres de {nom}"))
        .with_inner_size(TAILLE)
        .with_min_inner_size(TAILLE_MIN)
        .with_icon(icone_fenetre(&*logiciel, apparence.palette.sombre))
        .with_visible(false);
    let mut options = crate::volet::options(viewport);
    options.centered = place.is_none();
    let l2 = logiciel.clone();
    eframe::run_native(
        &nom,
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_fonts(polices);
            apparence.appliquer(&cc.egui_ctx);
            if let Ok(mut c) = liens.contexte.lock() {
                *c = Some(cc.egui_ctx.clone());
            }
            let parametres = Parametres::new(&*l2, page.as_deref());
            Ok(Box::new(Fenetre {
                logiciel: l2,
                liens,
                apparence,
                parametres,
                hwnd: None,
                montree: false,
                place,
            }))
        }),
    )
    .map_err(|e| e.to_string())
}
