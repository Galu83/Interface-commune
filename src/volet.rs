//! La fenêtre du volet (clic gauche) et du petit menu (clic droit), à la façon de Windows 11
//! (règles : interface Windows 11, points 6 et 10).
//!
//! Les deux vivent dans **une seule fenêtre**, préparée au repos et gardée cachée
//! ([`veiller`]) : au clic, elle est mise à jour et montrée, jamais reconstruite (ni fenêtre,
//! ni OpenGL, ni polices). C'est la fenêtre principale de la seule boucle de fenêtres du
//! programme : les Paramètres et les autres fenêtres du logiciel sont dessinés avec elle
//! ([`crate::autres_fenetres`]), même quand elle est cachée. Elle monte depuis la barre des tâches en fondu (le volet) ou
//! apparaît en fondu (le petit menu), avec les durées de Windows (`theme`), et rien si
//! « Effets d'animation » est coupé. Elle se ferme par Échap, un clic à côté ou la perte du
//! focus, en fondu court.
//!
//! **Garde** : pendant les 250 premières ms ([`GARDE`]), un clic ou une touche ne choisit rien.
//! Un clic sur une ligne cache la fenêtre ; ce qu'il demande n'est fait qu'une fois la
//! fenêtre cachée : rien de lent dans le fil des fenêtres pendant qu'elle se montre.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use eframe::egui::{self, FontDefinitions, Key, ViewportCommand};

use crate::Logiciel;
use crate::apparence::{Apparence, couleur};
use crate::fenetres::{
    Demande, GARDE, GARDE_MAX, Garde, LENTE, Liens, Mesure, Ouvert, Reception, Suite, suite,
};
use crate::menu::Choix;
use crate::place::{Cote, Rect, cote_de_la_barre, placer, placer_au_point};
use crate::surface::{self, Choisi, LARGEUR, LARGEUR_MENU};
use crate::systeme::{self, hwnd_de};
use crate::theme::{self, Surface};

fn rang(s: Surface) -> usize {
    match s {
        Surface::Volet => 0,
        Surface::Menu => 1,
    }
}

/// Options communes des fenêtres : n'importe quel fil, fenêtre cachée jusqu'à sa mise en
/// place.
pub fn options(viewport: egui::ViewportBuilder) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport,
        event_loop_builder: Some(Box::new(|b| {
            use winit::platform::windows::EventLoopBuilderExtWindows;
            b.with_any_thread(true);
        })),
        centered: false,
        persist_window: false,
        ..Default::default()
    }
}

/// Fait ce qui a été choisi dans le volet ou le petit menu, une fois la fenêtre cachée ; rend
/// la fenêtre à ouvrir ensuite, s'il y en a une. N'attend jamais : le logiciel reçoit l'ordre
/// et rend la main.
pub fn faire(choisi: Option<Choisi>, logiciel: &Arc<dyn Logiciel>) -> Option<Demande> {
    match choisi? {
        Choisi::Rapide(cle) | Choisi::Menu(Choix::Action(cle)) => {
            logiciel.action(&cle);
            None
        }
        Choisi::Parametres | Choisi::Menu(Choix::Parametres) => Some(Demande::Parametres(None)),
        Choisi::Menu(Choix::RechercherMiseAJour) => {
            // La recherche part sur son propre fil ; sa réponse s'affiche dans les
            // Paramètres, page « Mise à jour ».
            logiciel.rechercher_mise_a_jour();
            Some(Demande::Parametres(Some(
                crate::parametres::modele::MISE_A_JOUR.into(),
            )))
        }
        Choisi::Menu(Choix::Quitter) => {
            logiciel.quitter();
            None
        }
    }
}

/// Écrit au journal sur un fil à part : le fil des fenêtres ne touche jamais au disque
/// pendant une ouverture.
fn noter(logiciel: &Arc<dyn Logiciel>, texte: String) {
    let l = logiciel.clone();
    let _ = std::thread::Builder::new()
        .name("journal du volet".into())
        .spawn(move || l.noter(texte));
}

/// La première ouverture de chaque surface, et la première préparation, depuis le lancement.
static DEJA_OUVERTE: [AtomicBool; 2] = [AtomicBool::new(false), AtomicBool::new(false)];
static DEJA_PREPAREE: AtomicBool = AtomicBool::new(false);

/// Une demande d'ouverture, reçue par la fenêtre.
#[derive(Debug, Clone, Copy)]
struct Montrer {
    surface: Surface,
    icone: Rect,
    /// Le point du clic droit (le petit menu) ; le centre de l'icône pour le volet.
    ancre: [i32; 2],
    clic: Instant,
    recue: Instant,
}

impl Montrer {
    fn de(d: &Demande) -> Option<Montrer> {
        let recue = Instant::now();
        match *d {
            Demande::Volet { icone, clic } => Some(Montrer {
                surface: Surface::Volet,
                icone,
                ancre: centre(icone),
                clic,
                recue,
            }),
            Demande::Menu { icone, ancre, clic } => Some(Montrer {
                surface: Surface::Menu,
                icone,
                ancre,
                clic,
                recue,
            }),
            _ => None,
        }
    }
}

fn centre(r: Rect) -> [i32; 2] {
    [r[0] / 2 + r[2] / 2, r[1] / 2 + r[3] / 2]
}

/// Où en est la fenêtre.
#[derive(Clone)]
enum Etape {
    /// Préparation au repos, fenêtre cachée : le volet, puis le petit menu, sont dessinés une
    /// fois (polices et images prêtes).
    Preparation(u8),
    /// Prête et cachée.
    Cachee,
    /// Montrée encore invisible (opacité 0) : écran, taille, place.
    MiseEnPlace { m: Montrer, images: u8 },
    /// Montrée : l'animation d'ouverture, puis l'attente d'un choix.
    Montree {
        m: Montrer,
        debut: Instant,
        place: [i32; 2],
        /// Vers la barre des tâches (le volet en part) : (x, y), de -1 à 1.
        sens: [f32; 2],
        finie: bool,
        /// La taille a changé (contenu nouveau) : la replacer contre la barre.
        replacer: bool,
        /// Clics et touches ignorés pendant les premiers instants.
        garde: Garde,
    },
    /// Le fondu de fermeture, puis ce qui a été choisi.
    Fermeture {
        surface: Surface,
        debut: Instant,
        puis: Option<Choisi>,
    },
}

/// Ce que la fenêtre rend au fil des fenêtres en se fermant.
#[derive(Default)]
struct Sortie {
    /// La demande pour une autre fenêtre (les Paramètres…).
    suite: Option<Demande>,
    /// La demande de volet ou de menu en cours (pour le menu de secours, en cas d'erreur).
    en_cours: Option<Demande>,
}

/// Ouvre les Paramètres à cette page, dans la boucle, ou les amène à cette page et devant
/// s'ils sont déjà ouverts.
fn ouvrir_parametres(liens: &Arc<Liens>, logiciel: &Arc<dyn Logiciel>, page: Option<String>) {
    crate::parametres::fenetre::ouvrir_dans(liens, logiciel, page);
}

struct Volet<'a> {
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    demandes: &'a Reception,
    apparence: Apparence,
    hwnd: Option<isize>,
    /// Windows accepte l'opacité de la fenêtre (fondu possible, préparation invisible).
    transparente: bool,
    /// La fenêtre est voilée (`systeme::voiler`) : toujours, sauf montrée pour de vrai.
    voilee: Option<bool>,
    etape: Etape,
    /// La taille du contenu de chaque surface (points), mesurée à chaque image.
    tailles: [egui::Vec2; 2],
    /// « Effets d'animation » de Windows, relu à chaque ouverture.
    effets: bool,
    a_eu_le_focus: bool,
    focus_redemande: bool,
    /// Le logiciel se ferme : on se ferme dès que la surface est cachée.
    sortie_voulue: Option<Demande>,
    /// La fenêtre se ferme pour de bon (la fermeture n'est pas annulée).
    fin_voulue: bool,

    sortie: Rc<RefCell<Sortie>>,
    creee_a: Instant,
    /// Un clic arrivé avant la préparation : montré dès qu'elle est finie.
    premiere: Option<Demande>,
}

impl Volet<'_> {
    fn dessinee(&self) -> Surface {
        match &self.etape {
            Etape::Preparation(1) => Surface::Menu,
            Etape::Preparation(_) | Etape::Cachee => Surface::Volet,
            Etape::MiseEnPlace { m, .. } | Etape::Montree { m, .. } => m.surface,
            Etape::Fermeture { surface, .. } => *surface,
        }
    }

    fn montree(&self) -> Option<Surface> {
        match &self.etape {
            Etape::MiseEnPlace { m, .. } | Etape::Montree { m, .. } => Some(m.surface),
            _ => None,
        }
    }

    /// Lit les demandes arrivées, sans attendre.
    fn lire(&mut self, ctx: &egui::Context) {
        while self.sortie_voulue.is_none() {
            match self.demandes.prendre() {
                Ok(Some(d)) => self.recevoir(ctx, d),
                Ok(None) => break,
                Err(()) => {
                    self.cacher_tout_de_suite(ctx);
                    self.sortir(ctx);
                    break;
                }
            }
        }
    }

    fn recevoir(&mut self, ctx: &egui::Context, d: Demande) {
        match suite(&d, self.montree()) {
            Suite::Ignorer => {}
            Suite::Montrer(s) => {
                let Some(m) = Montrer::de(&d) else { return };
                // Une autre surface montrée, ou en train de se fermer : cachée tout de suite.
                self.cacher_tout_de_suite(ctx);
                if self.sortie_voulue.is_some() {
                    // Ce que la fermeture avait choisi demande une autre fenêtre.
                    self.sortir(ctx);
                    return;
                }
                let liens = &self.liens;
                liens.fermer.store(false, Ordering::Release);
                liens.mettre(Ouvert::Surface(s));
                self.a_eu_le_focus = false;
                self.focus_redemande = false;
                self.sortie.borrow_mut().en_cours = Some(d);
                self.etape = Etape::MiseEnPlace { m, images: 0 };
                // Visible, mais encore invisible (opacité 0) : elle reçoit de quoi se
                // dessiner tout de suite.
                if let (Some(h), true) = (self.hwnd, self.transparente) {
                    systeme::transparente(h, 0);
                }
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                ctx.request_repaint();
            }
            Suite::Parametres => {
                // Le volet ou le petit menu montré se ferme ; les Paramètres s'ouvrent à côté,
                // dans la même boucle.
                if let Etape::Montree { .. } = self.etape {
                    self.commencer_fermeture(ctx, None);
                } else if let Etape::MiseEnPlace { .. } = self.etape {
                    self.cacher_tout_de_suite(ctx);
                }
                let page = match d {
                    Demande::Parametres(p) => p,
                    _ => None,
                };
                ouvrir_parametres(&self.liens, &self.logiciel, page);
            }
            Suite::Sortir => {
                self.sortie_voulue = Some(d);
                if let Etape::Montree { .. } = self.etape {
                    self.commencer_fermeture(ctx, None);
                } else if !matches!(self.etape, Etape::Fermeture { .. }) {
                    self.cacher_tout_de_suite(ctx);
                    self.sortir(ctx);
                }
            }
        }
    }

    /// Cache la fenêtre tout de suite, sans fondu ; une fermeture en cours est finie (ce
    /// qu'elle avait choisi est fait).
    fn cacher_tout_de_suite(&mut self, ctx: &egui::Context) {
        match std::mem::replace(&mut self.etape, Etape::Cachee) {
            Etape::Fermeture { surface, puis, .. } => {
                self.cacher(ctx, surface);
                self.faire(puis);
            }
            Etape::MiseEnPlace { m, .. } | Etape::Montree { m, .. } => self.cacher(ctx, m.surface),
            autre => self.etape = autre,
        }
    }

    /// Cache la fenêtre (invisible pour la prochaine fois) ; plus rien n'est montré.
    fn cacher(&mut self, ctx: &egui::Context, surface: Surface) {
        if let Some(h) = self.hwnd {
            if self.transparente {
                systeme::transparente(h, 0);
            }
            systeme::cacher(h);
        }
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        ctx.memory_mut(|m| {
            if let Some(id) = m.focused() {
                m.surrender_focus(id);
            }
        });
        let liens = &self.liens;
        liens.mettre(Ouvert::Rien);
        if let Ok(mut f) = liens.ferme_a.lock() {
            *f = Some((Instant::now(), surface));
        }
        self.sortie.borrow_mut().en_cours = None;
        self.etape = Etape::Cachee;
    }

    /// Rend la main au fil des fenêtres (la fenêtre est déjà cachée) : fin du programme, les
    /// autres fenêtres se ferment avec elle.
    fn sortir(&mut self, ctx: &egui::Context) {
        self.fin_voulue = true;
        self.liens.surface_prete.store(0, Ordering::Release);
        self.sortie.borrow_mut().suite = self.sortie_voulue.take();
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    /// Fait ce que l'utilisateur a choisi, une fois la fenêtre cachée. N'attend jamais (le
    /// service et le fil des recherches sont prévenus par leur file, sans attente).
    fn faire(&mut self, choisi: Option<Choisi>) {
        match faire(choisi, &self.logiciel) {
            Some(Demande::Parametres(page)) => ouvrir_parametres(&self.liens, &self.logiciel, page),
            Some(d) => {
                self.sortie_voulue.get_or_insert(d);
            }
            None => {}
        }
    }

    fn preparer(&mut self, ctx: &egui::Context, n: u8) {
        if n == 0
            && let Some(h) = self.hwnd
        {
            systeme::habiller(h, self.apparence.palette.sombre);
            self.transparente = systeme::transparente(h, 0);
        }
        if n < 2 {
            self.etape = Etape::Preparation(n + 1);
            ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint();
            return;
        }
        self.etape = Etape::Cachee;
        if self.transparente
            && let Some(h) = self.hwnd
        {
            self.liens.surface_prete.store(h, Ordering::Release);
        }
        let duree = self.creee_a.elapsed();
        let premiere = !DEJA_PREPAREE.swap(true, Ordering::AcqRel);
        if premiere || duree > LENTE {
            let fondu = if self.transparente {
                ""
            } else {
                " ; Windows refuse l'opacité de la fenêtre : ni fondu ni réveil immédiat"
            };
            noter(
                &self.logiciel,
                format!(
                    "Volet et petit menu préparés au repos en {} ms{fondu}.",
                    duree.as_millis()
                ),
            );
        }
        // Les demandes arrivées avant et pendant la préparation.
        if let Some(d) = self.premiere.take() {
            self.recevoir(ctx, d);
        }
        self.lire(ctx);
    }

    /// La place de la fenêtre pour cette image de l'animation.
    fn poser(&self, h: isize, place: [i32; 2], sens: [f32; 2], image: theme::Image, ppp: f32) {
        let d = image.decalage * ppp;
        let x = place[0] + (sens[0] * d).round() as i32;
        let y = place[1] + (sens[1] * d).round() as i32;
        systeme::deplacer(h, x, y, true);
        if self.transparente {
            systeme::transparente(h, image.opacite);
        }
    }

    /// La place finale (pixels) et le sens du glissement, d'après l'écran de l'icône.
    fn place(&self, m: &Montrer, taille: [i32; 2]) -> Option<([i32; 2], [f32; 2])> {
        let (e, t) = systeme::ecran_autour(m.ancre[0], m.ancre[1])?;
        let r = match m.surface {
            Surface::Volet => placer(m.icone, taille, e, t, 12),
            Surface::Menu => placer_au_point(m.ancre, m.icone, taille, e, t, 8),
        };
        let sens = match (m.surface, cote_de_la_barre(m.icone, e, t)) {
            (Surface::Menu, _) => [0.0, 0.0],
            (Surface::Volet, Cote::Bas) => [0.0, 1.0],
            (Surface::Volet, Cote::Haut) => [0.0, -1.0],
            (Surface::Volet, Cote::Gauche) => [-1.0, 0.0],
            (Surface::Volet, Cote::Droite) => [1.0, 0.0],
        };
        Some(([r[0], r[1]], sens))
    }

    /// La taille voulue (pixels) ; `None` si la fenêtre l'a déjà (à un pixel près).
    fn taille_a_changer(&self, ctx: &egui::Context, h: isize, surface: Surface) -> Option<egui::Vec2> {
        let ppp = ctx.pixels_per_point();
        let voulue = self.tailles[rang(surface)];
        let px = [(voulue.x * ppp).round() as i32, (voulue.y * ppp).round() as i32];
        match systeme::taille_fenetre(h) {
            Some(t) if (t[0] - px[0]).abs() <= 1 && (t[1] - px[1]).abs() <= 1 => None,
            _ => Some(voulue),
        }
    }

    /// Montrée encore invisible : d'abord sur le bon écran (elle prend son échelle), puis à
    /// la bonne taille, puis à sa place ; alors l'animation commence. Une étape par image,
    /// seulement si elle est nécessaire.
    fn mettre_en_place(&mut self, ctx: &egui::Context, m: Montrer, images: u8) {
        let images = images.saturating_add(1);
        if self.liens.fermer.swap(false, Ordering::AcqRel) {
            // Un nouveau clic sur l'icône avant même l'apparition : on n'ouvre pas.
            self.cacher(ctx, m.surface);
            return;
        }
        let Some(h) = self.hwnd else {
            self.cacher(ctx, m.surface);
            return;
        };
        if images == 1 {
            self.effets = systeme::effets_d_animation();
            if self.apparence.relire(ctx) {
                systeme::habiller(h, self.apparence.palette.sombre);
            }
            let voulu = systeme::ecran_autour(m.ancre[0], m.ancre[1]);
            let ici = systeme::rect_fenetre(h)
                .map(centre)
                .and_then(|c| systeme::ecran_autour(c[0], c[1]));
            if let Some((e, t)) = voulu
                && ici.map(|i| i.0) != Some(e)
            {
                systeme::deplacer(h, t[0] + 8, t[1] + 8, true);
                self.etape = Etape::MiseEnPlace { m, images };
                ctx.request_repaint();
                return;
            }
        }
        if images < 8
            && let Some(taille) = self.taille_a_changer(ctx, h, m.surface)
        {
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(taille));
            self.etape = Etape::MiseEnPlace { m, images };
            ctx.request_repaint();
            return;
        }
        let taille = systeme::taille_fenetre(h).unwrap_or([1, 1]);
        let (place, sens) = self.place(&m, taille).unwrap_or_else(|| {
            let r = systeme::rect_fenetre(h).unwrap_or_default();
            ([r[0], r[1]], [0.0, 0.0])
        });
        let image = theme::ouverture(m.surface, 0, self.effets);
        self.poser(h, place, sens, image, ctx.pixels_per_point());
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        systeme::premier_plan(h);
        let maintenant = Instant::now();
        let mesure = Mesure {
            attente: m.recue.saturating_duration_since(m.clic),
            mise_en_place: maintenant.saturating_duration_since(m.recue),
            images,
        };
        let premiere = !DEJA_OUVERTE[rang(m.surface)].swap(true, Ordering::AcqRel);
        if mesure.a_noter(premiere) {
            noter(&self.logiciel, mesure.phrase(m.surface, premiere));
        }
        self.etape = Etape::Montree {
            m,
            debut: maintenant,
            place,
            sens,
            finie: false,
            replacer: false,
            garde: Garde::default(),
        };
        ctx.request_repaint();
    }

    fn commencer_fermeture(&mut self, ctx: &egui::Context, puis: Option<Choisi>) {
        let surface = match &self.etape {
            Etape::Montree { m, .. } | Etape::MiseEnPlace { m, .. } => m.surface,
            _ => return,
        };
        self.etape = Etape::Fermeture {
            surface,
            debut: Instant::now(),
            puis,
        };
        self.fermer_pas_a_pas(ctx);
    }

    fn fermer_pas_a_pas(&mut self, ctx: &egui::Context) {
        let Etape::Fermeture {
            surface,
            debut,
            ref puis,
        } = self.etape
        else {
            return;
        };
        let ms = u32::try_from(debut.elapsed().as_millis()).unwrap_or(u32::MAX);
        let image = theme::fermeture(ms, self.effets);
        if image.finie {
            let puis = puis.clone();
            self.cacher(ctx, surface);
            self.faire(puis);
            if self.sortie_voulue.is_some() {
                self.sortir(ctx);
            } else {
                // Les demandes arrivées pendant le fondu.
                self.lire(ctx);
            }
        } else {
            if let (Some(h), true) = (self.hwnd, self.transparente) {
                systeme::transparente(h, image.opacite);
            }
            ctx.request_repaint();
        }
    }

    /// Montrée : l'animation, le clavier, la fermeture.
    fn montree_image(&mut self, ui: &mut egui::Ui, action: Option<Choisi>) {
        let ctx = ui.ctx().clone();
        let Etape::Montree {
            m,
            debut,
            place,
            sens,
            finie,
            replacer,
            garde,
        } = self.etape
        else {
            return;
        };
        let Some(h) = self.hwnd else { return };
        if !finie {
            let ms = u32::try_from(debut.elapsed().as_millis()).unwrap_or(u32::MAX);
            let image = theme::ouverture(m.surface, ms, self.effets);
            self.poser(h, place, sens, image, ctx.pixels_per_point());
            if let Etape::Montree { finie, .. } = &mut self.etape {
                *finie = image.finie;
            }
            if !image.finie {
                ctx.request_repaint();
            }
        } else if let Some(taille) = self.taille_a_changer(&ctx, h, m.surface) {
            // Le contenu a changé (un état, un message) : nouvelle taille, puis nouvelle place.
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(taille));
            if let Etape::Montree { replacer, .. } = &mut self.etape {
                *replacer = true;
            }
        } else if replacer {
            let taille = systeme::taille_fenetre(h).unwrap_or([1, 1]);
            if let Some((p, _)) = self.place(&m, taille) {
                systeme::deplacer(h, p[0], p[1], true);
                if let Etape::Montree { place, replacer, .. } = &mut self.etape {
                    *place = p;
                    *replacer = false;
                }
            }
        }

        // Clavier : flèches pour passer d'une ligne à l'autre, Échap pour fermer.
        let (echap, bas, haut) = ui.input(|i| {
            (
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
            )
        });
        if bas {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::Next));
        }
        if haut {
            ui.memory_mut(|m| m.move_focus(egui::FocusDirection::Previous));
        }
        let focus = ui.input(|i| i.viewport().focused);
        if focus == Some(true) {
            self.a_eu_le_focus = true;
        } else if !self.a_eu_le_focus && !self.focus_redemande && debut.elapsed() > Duration::from_millis(300)
        {
            // Windows n'a pas donné le focus : un seul nouvel essai (sans focus, un clic à
            // côté ne fermerait pas la fenêtre).
            self.focus_redemande = true;
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            systeme::premier_plan(h);
        }
        let perdu = self.a_eu_le_focus && focus == Some(false);
        let demande = self.liens.fermer.swap(false, Ordering::AcqRel);
        // Les premiers instants : un clic ou une touche ne choisit rien (Échap ferme quand
        // même).
        let mut garde = garde;
        let tenu = ui.input(|i| i.pointer.any_down() || !i.keys_down.is_empty());
        let action = garde.filtrer(action, debut.elapsed(), tenu);
        if let Etape::Montree { garde: g, .. } = &mut self.etape {
            *g = garde;
        }
        if !garde.levee() {
            // Une image de plus quand la garde peut se lever, même sans geste.
            let depuis = debut.elapsed();
            let prochaine = if depuis < GARDE { GARDE } else { GARDE_MAX };
            ctx.request_repaint_after(prochaine.saturating_sub(depuis));
        }
        if echap || perdu || demande {
            self.commencer_fermeture(&ctx, None);
        } else if action.is_some() {
            self.commencer_fermeture(&ctx, action);
        }
    }
}

impl eframe::App for Volet<'_> {
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
            // fin de la préparation.
            ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint();
            return;
        }
        self.lire(ctx);
        if self.liens.autres.a_faire() {
            // Une autre fenêtre à ouvrir, à amener devant ou à fermer : il faut une image
            // complète, même la fenêtre principale cachée (sans fenêtre visible, egui n'en
            // ferait aucune).
            ctx.send_viewport_cmd(ViewportCommand::RequestPaintWhileHidden);
            ctx.request_repaint();
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
        let p = self.apparence.palette.clone();
        let logiciel = self.logiciel.clone();
        let (action, taille) = surface::dessiner(ui, &p, surface, &*logiciel);
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
        let voile = voilee(&self.etape);
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

/// La fenêtre du volet est-elle voilée à cette étape ? Oui, sauf montrée pour de vrai
/// (animation d'ouverture, attente d'un choix, fermeture).
fn voilee(e: &Etape) -> bool {
    !matches!(e, Etape::Montree { .. } | Etape::Fermeture { .. })
}

/// Prépare au repos la fenêtre du volet et du petit menu (cachée, invisible), puis la garde :
/// chaque demande de volet ou de petit menu la montre sans la reconstruire ; les Paramètres
/// et les autres fenêtres s'ouvrent dans sa boucle. Rend la main quand le logiciel se ferme
/// (`Demande::Quitter` ou `None`).
pub fn veiller(
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    polices: FontDefinitions,
    demandes: &Reception,
    premiere: Option<Demande>,
) -> Result<Option<Demande>, (String, Option<Demande>)> {
    let sortie = Rc::new(RefCell::new(Sortie {
        suite: None,
        en_cours: premiere.clone(),
    }));
    let s2 = sortie.clone();
    let creee_a = Instant::now();
    let viewport = egui::ViewportBuilder::default()
        .with_title(format!("Volet de {}", logiciel.identite().nom))
        .with_inner_size([LARGEUR, 300.0])
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top()
        .with_taskbar(false)
        .with_visible(false);
    let (l2, liens2) = (logiciel.clone(), liens.clone());
    let nom = logiciel.identite().nom.clone();
    crate::sonde::etape("volet : création de la fenêtre et du contexte OpenGL");
    let r = eframe::run_native(
        &nom,
        options(viewport),
        Box::new(move |cc| {
            cc.egui_ctx.set_fonts(polices);
            let apparence = Apparence::new(None);
            apparence.appliquer(&cc.egui_ctx);
            if let Ok(mut c) = liens2.contexte.lock() {
                *c = Some(cc.egui_ctx.clone());
            }
            liens2.autres.lier(Some(cc.egui_ctx.clone()));
            // Une demande envoyée réveille cette fenêtre, même cachée.
            demandes.lecteur(Some(cc.egui_ctx.clone()));
            Ok(Box::new(Volet {
                logiciel: l2,
                liens: liens2,
                demandes,
                apparence,
                hwnd: None,
                transparente: false,
                voilee: None,
                etape: Etape::Preparation(0),
                tailles: [egui::vec2(LARGEUR, 300.0), egui::vec2(LARGEUR_MENU, 200.0)],
                effets: true,
                a_eu_le_focus: false,
                focus_redemande: false,
                sortie_voulue: None,
                fin_voulue: false,
                sortie: s2,
                creee_a,
                premiere,
            }))
        }),
    );
    demandes.lecteur(None);
    liens.autres.lier(None);
    liens.surface_prete.store(0, Ordering::Release);
    let mut s = sortie.borrow_mut();
    match r {
        Ok(()) => Ok(s.suite.take()),
        Err(e) => Err((e.to_string(), s.en_cours.take())),
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Préparée, cachée ou en train de se mettre en place : voilée (eframe la montre d'office
    /// à sa première image, et winit lui retire son opacité 0). Le voile ne se lève qu'à
    /// l'ouverture pour de vrai, et revient à la fermeture.
    #[test]
    fn voilee_sauf_montree_pour_de_vrai() {
        let maintenant = Instant::now();
        let m = Montrer {
            surface: Surface::Volet,
            icone: [0, 0, 16, 16],
            ancre: [8, 8],
            clic: maintenant,
            recue: maintenant,
        };
        assert!(voilee(&Etape::Preparation(0)));
        assert!(voilee(&Etape::Preparation(2)));
        assert!(voilee(&Etape::Cachee));
        assert!(voilee(&Etape::MiseEnPlace { m, images: 1 }));
        assert!(!voilee(&Etape::Fermeture {
            surface: Surface::Menu,
            debut: maintenant,
            puis: None,
        }));
    }
}
