//! La fenêtre du volet dans eframe : une image (lire les demandes, dessiner la surface,
//! avancer l'étape), le voile, les autres fenêtres dessinées avec elle.

use std::sync::atomic::Ordering;

use eframe::egui::{self, ViewportCommand};

use super::fenetre::Volet;
use super::{Etape, ImageVue, OBSERVATEUR, discret, fleches_en_tab, rang, voilee};
use crate::apparence::couleur;
use crate::bords::verrou::verrou;
use crate::surface;
use crate::systeme::{self, hwnd_de};

impl eframe::App for Volet<'_> {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        fleches_en_tab(&mut raw_input.events);
    }

    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        couleur(self.apparence.palette.fond_menu).to_normalized_gamma_f32()
    }

    /// Appelé avant chaque image, et seul quand la fenêtre est cachée : lit les demandes.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::sonde::battement();
        crate::sonde::etape("volet : début de l'image");
        if self.liens.quitter.load(Ordering::Acquire) {
            self.cacher_tout_de_suite(ctx);
            self.sortie_voulue = None;
            self.sortir(ctx);
            return;
        }
        if let Etape::Preparation(_) = self.etape {
            // Une fenêtre cachée n'est dessinée que sur demande ; les demandes attendent la
            // fin de la préparation. Pendant un réveil de Windows, la préparation attend.
            match crate::bords::reprise::attente_avant_dessin() {
                Some(d) => ctx.request_repaint_after(d),
                None => {
                    ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
                    ctx.request_repaint();
                }
            }
            return;
        }
        self.lire(ctx);
        if let Some(dans) = self.liens.autres.prochaine_image() {
            // Une autre fenêtre à ouvrir, à amener devant ou à fermer : il faut une image
            // complète, même la fenêtre principale cachée (sans fenêtre visible, egui n'en
            // ferait aucune). Une création ratée attend son nouvel essai. Pendant un réveil
            // de Windows (`crate::reprise`), aucune image complète : OpenGL et SwapBuffers
            // ont bloqué ce fil plus d'une minute au sortir d'une veille.
            let dans = match crate::bords::reprise::attente_avant_dessin() {
                Some(r) => dans.max(r),
                None => dans,
            };
            if dans.is_zero() {
                ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
                ctx.request_repaint();
            } else {
                // Une création différée est reprise par un réveil par message, jamais par
                // une repeinture hors du contexte de winit (la fenêtre ne pourrait pas être
                // créée) : voir `AutresFenetres::reveiller_dans`.
                self.liens.autres.reveiller_dans(dans);
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        crate::sonde::etape("volet : dessin de l'image");
        let ctx = ui.ctx().clone();
        if self.hwnd.is_none() {
            self.hwnd = hwnd_de(frame);
        }
        if ui.input(|i| i.viewport().close_requested()) && !self.fin_voulue {
            // Alt+F4 sur le volet ou le petit menu : il se ferme, la boucle (et les autres
            // fenêtres) restent.
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            if let Etape::Montree { .. } = self.etape {
                self.commencer_fermeture(&ctx, None);
            }
        }
        if self.apparence.suivre(&ctx)
            && let Some(h) = self.hwnd
        {
            systeme::habiller(h, self.apparence.palette.sombre);
        }
        let surface = self.dessinee();
        let observee = match &self.etape {
            Etape::MiseEnPlace { m, .. } => Some((m.clic, true)),
            Etape::Montree { m, finie: false, .. } => Some((m.clic, false)),
            _ => None,
        };
        let p = self.apparence.palette.clone();
        let logiciel = self.logiciel.clone();
        self.borner(&ctx);
        let (action, taille) = surface::dessiner_borne(ui, &p, surface, &*logiciel, self.borne);
        self.tailles[rang(surface)] = taille;
        match self.etape.clone() {
            Etape::Preparation(n) => self.preparer(&ctx, n),
            Etape::Cachee => {
                // Montrée sans demande (le fil principal l'a réveillée pour rien) : cachée.
                if let Some(h) = self.hwnd {
                    systeme::cacher(h);
                }
            }
            Etape::MiseEnPlace { m, images } => self.mettre_en_place(&ctx, m, images),
            Etape::Montree { .. } => self.montree_image(ui, action),
            Etape::Fermeture { .. } => self.fermer_pas_a_pas(&ctx),
        }
        // Voilée tant qu'elle n'est pas ouverte pour de bon : eframe montre d'office une
        // fenêtre à sa première image, même préparée cachée. Fait avant la fin de l'image,
        // donc avant que la demande de la montrer soit appliquée.
        crate::sonde::etape("volet : voile et fenêtre (appels à Windows)");
        if let Some((clic, mise_en_place)) = observee
            && discret()
            && let Some(f) = verrou(&OBSERVATEUR).as_mut()
        {
            let finie = matches!(self.etape, Etape::Montree { finie: true, .. });
            f(ImageVue {
                surface,
                depuis_clic: clic.elapsed(),
                mise_en_place: mise_en_place && !matches!(self.etape, Etape::Montree { .. }),
                opacite: self.vue,
                finie,
            });
        }
        let voile = voilee(&self.etape) || discret();
        if let Some(h) = self.hwnd
            && self.voilee != Some(voile)
            && systeme::voiler(h, voile)
        {
            self.voilee = Some(voile);
        }
        // Les Paramètres et les autres fenêtres, à chaque image de la fenêtre principale
        // (une fenêtre qui n'y est pas dessinée est détruite par egui).
        crate::sonde::etape("autres fenêtres : création, dessin, destruction");
        self.liens.autres.montrer(&ctx, &p);
        // La suite est à eframe : dessin OpenGL et échange des tampons (SwapBuffers).
        crate::sonde::etape("volet : après le dessin, OpenGL et SwapBuffers");
    }
}
