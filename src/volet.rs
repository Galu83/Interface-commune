//! La fenêtre du volet (clic gauche) et du petit menu (clic droit), à la façon de Windows 11
//! (règles : interface Windows 11, points 6 et 10).
//!
//! Les deux vivent dans **une seule fenêtre**, préparée au repos et gardée cachée
//! ([`veiller`]) : au clic, elle est mise à jour et montrée, jamais reconstruite (ni fenêtre,
//! ni OpenGL, ni polices). C'est la fenêtre principale de la seule boucle de fenêtres du
//! programme : les Paramètres et les autres fenêtres du logiciel sont dessinés avec elle
//! ([`crate::autres_fenetres`]), même quand elle est cachée. Elle apparaît à sa place, prête
//! et déjà visible dès sa première image, en un fondu court (`theme`), et sans fondu si
//! « Effets d'animation » est coupé. Elle se ferme par Échap, un clic à côté ou la perte du
//! focus, en fondu court. Le clic à côté ne dépend pas du focus ([`Veilleur`]) : même si
//! Windows refuse le focus à la fenêtre, un clic en dehors d'elle la ferme toujours.
//!
//! **Garde** : pendant les 250 premières ms ([`GARDE`]), un clic ou une touche ne choisit rien.
//! Un clic sur une ligne cache la fenêtre ; ce qu'il demande n'est fait qu'une fois la
//! fenêtre cachée : rien de lent dans le fil des fenêtres pendant qu'elle se montre.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use eframe::egui::{self, FontDefinitions, Key, ViewportCommand};

use crate::Logiciel;
use crate::apparence::{Apparence, couleur};
use crate::fenetres::{
    Appuis, Demande, GARDE, GARDE_MAX, Garde, LENTE, Liens, Mesure, Ouvert, Reception, Suite,
    fermer_sur_appui, suite,
};
use crate::menu::Choix;
use crate::place::{Rect, placer, placer_au_point};
use crate::surface::{self, Choisi, LARGEUR, LARGEUR_MENU};
use crate::systeme::{self, hwnd_de};
use crate::theme::{self, Surface};

fn rang(s: Surface) -> usize {
    match s {
        Surface::Volet => 0,
        Surface::Menu => 1,
    }
}

/// Écart entre deux images d'un fondu (ouverture ou fermeture) : environ 60 par seconde.
/// Sans synchronisation verticale, rien d'autre ne cadence le dessin.
pub const IMAGE_DE_FONDU: Duration = Duration::from_millis(16);

/// Options communes des fenêtres : n'importe quel fil, fenêtre cachée jusqu'à sa mise en
/// place.
///
/// Sans synchronisation verticale (`glow_options.vsync: false`) : avec elle, SwapBuffers attend le
/// prochain rafraîchissement de l'écran. Quand un écran est retiré ou rendu (il passe à un
/// autre PC, il se met en veille), Windows refait l'affichage et cette attente a duré des
/// dizaines de secondes, voire des minutes : le fil des fenêtres restait bloqué dans
/// SwapBuffers, et le volet comme le petit menu ne répondaient plus. Ces fenêtres ne
/// dessinent qu'à la demande : elles n'ont pas besoin d'être cadencées par l'écran, et les
/// fondus se cadencent eux-mêmes ([`IMAGE_DE_FONDU`]).
pub fn options(viewport: egui::ViewportBuilder) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport,
        glow_options: eframe::egui_glow::GlowConfiguration {
            vsync: false,
            ..Default::default()
        },
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

/// Une image du volet ou du petit menu pendant son ouverture, pour les mesures
/// ([`observer`]).
#[derive(Debug, Clone, Copy)]
pub struct ImageVue {
    pub surface: Surface,
    /// Du clic sur l'icône à la fin du dessin de cette image (avant l'échange des tampons).
    pub depuis_clic: Duration,
    /// L'image est encore une image de mise en place (fenêtre voilée, rien à l'écran).
    pub mise_en_place: bool,
    /// L'opacité posée pour cette image (0 à 255).
    pub opacite: u8,
    /// L'animation d'ouverture est finie : la fenêtre est à sa place, opaque.
    pub finie: bool,
}

type Observateur = Box<dyn FnMut(ImageVue) + Send>;

/// Le mode d'essai des mesures : chaque image d'une ouverture est rendue à l'observateur ;
/// la fenêtre reste voilée et ne prend jamais le premier plan (rien ne s'ouvre à l'écran,
/// le clavier reste à l'application en cours).
static OBSERVATEUR: std::sync::Mutex<Option<Observateur>> = std::sync::Mutex::new(None);
static DISCRET: AtomicBool = AtomicBool::new(false);

/// Pour les mesures (`examples/mesure_ouverture.rs`) seulement : `Some`, chaque image d'une
/// ouverture est rendue à `f`, la fenêtre reste voilée (invisible) et ne prend jamais le
/// focus ; `None` revient au fonctionnement normal.
pub fn observer(f: Option<Observateur>) {
    DISCRET.store(f.is_some(), Ordering::Release);
    if let Ok(mut o) = OBSERVATEUR.lock() {
        *o = f;
    }
}

fn discret() -> bool {
    DISCRET.load(Ordering::Acquire)
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
        /// L'animation déjà jouée, avancée image par image ([`avancer`]) : un retard de
        /// l'écran ne la fait jamais sauter à la fin.
        anime: Duration,
        /// La fin de l'image précédente.
        derniere: Instant,
        /// La première image montrée (celle qui lève le voile) a été présentée.
        presentee: bool,
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

/// Ce que le fil de veille de la souris lit : la fenêtre à surveiller (0 : aucune) et une
/// génération, qui change à chaque nouvelle surveillance.
#[derive(Default)]
struct Veille {
    fenetre: AtomicIsize,
    generation: AtomicU64,
    fin: AtomicBool,
}

/// Le filet « clic en dehors » : tant que le volet ou le petit menu est montré, un fil lit
/// l'état des boutons de la souris toutes les [`PAS_DE_VEILLE`] et ferme la fenêtre à tout
/// appui (gauche, droit ou milieu) **en dehors** d'elle. Il ne dépend pas du focus : si
/// Windows refuse le focus (règles du premier plan), la fenêtre ne perd jamais un focus
/// qu'elle n'a jamais eu, et resterait affichée. Pas de crochet système : un crochet souris
/// bas niveau gèle la souris de tout le PC quand le fil qui le pose est bloqué (par exemple
/// dans SwapBuffers). Le fil dort tant que rien n'est montré. Respecte la garde des premiers
/// instants ([`fermer_sur_appui`]) ; un clic dedans ne ferme jamais.
struct Veilleur {
    veille: Arc<Veille>,
    fil: Option<std::thread::JoinHandle<()>>,
}

/// Le temps entre deux relevés de la souris (un clic dure bien plus longtemps).
const PAS_DE_VEILLE: Duration = Duration::from_millis(15);

impl Veilleur {
    fn new(liens: Arc<Liens>) -> Veilleur {
        let veille = Arc::new(Veille::default());
        let v2 = veille.clone();
        let fil = std::thread::Builder::new()
            .name("clic-en-dehors".into())
            .spawn(move || veiller_la_souris(&v2, &liens))
            .ok();
        Veilleur { veille, fil }
    }

    /// La fenêtre `h` vient d'être montrée : surveiller les clics en dehors d'elle.
    fn activer(&self, h: isize) {
        self.veille.fenetre.store(h, Ordering::Release);
        self.veille.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(f) = &self.fil {
            f.thread().unpark();
        }
    }

    /// La fenêtre est fermée ou cachée : plus rien à surveiller.
    fn desactiver(&self) {
        self.veille.fenetre.store(0, Ordering::Release);
    }
}

impl Drop for Veilleur {
    fn drop(&mut self) {
        self.veille.fin.store(true, Ordering::Release);
        self.veille.fenetre.store(0, Ordering::Release);
        if let Some(f) = &self.fil {
            f.thread().unpark();
        }
    }
}

fn veiller_la_souris(veille: &Veille, liens: &Liens) {
    // La surveillance en cours : sa génération, les appuis déjà vus et son début.
    let mut suivi: Option<(u64, Appuis, Instant)> = None;
    while !veille.fin.load(Ordering::Acquire) {
        let generation = veille.generation.load(Ordering::Acquire);
        let h = veille.fenetre.load(Ordering::Acquire);
        if h == 0 {
            suivi = None;
            std::thread::park();
            continue;
        }
        if suivi.as_ref().is_none_or(|s| s.0 != generation) {
            suivi = Some((generation, Appuis::default(), Instant::now()));
        }
        if let Some((_, appuis, depuis)) = suivi.as_mut()
            && appuis.nouvel_appui(systeme::bouton_enfonce())
            && let (Some(p), Some(r)) = (systeme::position_souris(), systeme::rect_fenetre(h))
            && fermer_sur_appui(p, r, depuis.elapsed())
        {
            liens.fermer_la_fenetre();
        }
        std::thread::sleep(PAS_DE_VEILLE);
    }
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
    /// L'opacité posée à la dernière image (pour les mesures).
    vue: u8,
    /// Le filet « clic en dehors » (indépendant du focus).
    veilleur: Veilleur,
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
                // Refait par le fil des fenêtres si la fenêtre est perdue avant de se montrer.
                liens.poser_en_cours(Some(d.clone()));
                self.sortie.borrow_mut().en_cours = Some(d);
                self.etape = Etape::MiseEnPlace { m, images: 0 };
                // Visible, mais encore invisible (opacité 0) : elle reçoit de quoi se
                // dessiner tout de suite.
                if let (Some(h), true) = (self.hwnd, self.transparente) {
                    systeme::transparente(h, 0);
                }
                self.vue = 0;
                match (discret(), self.hwnd) {
                    (true, Some(h)) => systeme::montrer_sans_activer(h),
                    _ => ctx.send_viewport_cmd(ViewportCommand::Visible(true)),
                }
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
        self.veilleur.desactiver();
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
        liens.poser_en_cours(None);
        self.etape = Etape::Cachee;
    }

    /// Rend la main au fil des fenêtres (la fenêtre est déjà cachée) : fin du programme, les
    /// autres fenêtres se ferment avec elle.
    fn sortir(&mut self, ctx: &egui::Context) {
        self.fin_voulue = true;
        self.liens.surface_prete.store(0, Ordering::Release);
        self.liens.autres.racine(0);
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
            self.liens.autres.racine(h);
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

    /// L'opacité de la fenêtre pour cette image de l'animation (elle ne bouge pas : posée
    /// une fois à sa place).
    fn opacite(&mut self, h: isize, image: theme::Image) {
        self.vue = image.opacite;
        if self.transparente {
            systeme::transparente(h, image.opacite);
        }
    }

    /// La place de la fenêtre (pixels), d'après l'écran de l'icône.
    fn place(&self, m: &Montrer, taille: [i32; 2]) -> Option<[i32; 2]> {
        let (e, t) = systeme::ecran_autour(m.ancre[0], m.ancre[1])?;
        let r = match m.surface {
            Surface::Volet => placer(m.icone, taille, e, t, 12),
            Surface::Menu => placer_au_point(m.ancre, m.icone, taille, e, t, 8),
        };
        Some([r[0], r[1]])
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
            noter(
                &self.logiciel,
                "Ouverture impossible : la fenêtre du volet n'a pas de fenêtre Windows.".into(),
            );
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
        let place = self.place(&m, taille).unwrap_or_else(|| {
            let r = systeme::rect_fenetre(h).unwrap_or_default();
            [r[0], r[1]]
        });
        // À sa place, déjà visible (jamais une première image transparente), avant que le
        // voile se lève à la fin de cette image.
        systeme::deplacer(h, place[0], place[1], true);
        self.opacite(h, theme::ouverture(0, self.effets));
        if !discret() {
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            systeme::premier_plan(h);
        }
        let maintenant = Instant::now();
        let mesure = Mesure {
            attente: m.recue.saturating_duration_since(m.clic),
            mise_en_place: maintenant.saturating_duration_since(m.recue),
            images,
        };
        self.liens.poser_en_cours(None);
        let premiere = !DEJA_OUVERTE[rang(m.surface)].swap(true, Ordering::AcqRel);
        if mesure.a_noter(premiere) {
            noter(&self.logiciel, mesure.phrase(m.surface, premiere));
        }
        // Le filet « clic en dehors », qui ne dépend pas du focus (jamais en essai discret :
        // la fenêtre n'y est pas montrée pour de vrai).
        if !discret() {
            self.veilleur.activer(h);
        }
        self.etape = Etape::Montree {
            m,
            debut: maintenant,
            anime: Duration::ZERO,
            derniere: maintenant,
            presentee: false,
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
        self.veilleur.desactiver();
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
            ctx.request_repaint_after(IMAGE_DE_FONDU);
        }
    }

    /// Montrée : l'animation, le clavier, la fermeture.
    fn montree_image(&mut self, ui: &mut egui::Ui, action: Option<Choisi>) {
        let ctx = ui.ctx().clone();
        let Etape::Montree {
            m,
            debut,
            anime,
            derniere,
            presentee,
            finie,
            replacer,
            garde,
        } = self.etape
        else {
            return;
        };
        let Some(h) = self.hwnd else { return };
        // Cette image commence quand la précédente a été échangée (SwapBuffers) : l'écart dit
        // si l'écran a fait attendre.
        let maintenant = Instant::now();
        let ecart = maintenant.saturating_duration_since(derniere);
        if !presentee {
            // La première image montrée vient d'être présentée : le vrai délai du clic à
            // l'écran.
            let delai = maintenant.saturating_duration_since(m.clic);
            if delai > LENTE {
                noter(
                    &self.logiciel,
                    format!(
                        "{} présenté à l'écran {} ms après le clic (l'échange de sa première image a pris {} ms).",
                        quoi(m.surface),
                        delai.as_millis(),
                        ecart.as_millis()
                    ),
                );
            }
        } else if !finie && ecart > BLOQUEE {
            noter(
                &self.logiciel,
                format!(
                    "{} : une image de l'ouverture bloquée {} ms (échange des tampons).",
                    quoi(m.surface),
                    ecart.as_millis()
                ),
            );
        }
        let anime = if presentee { avancer(anime, ecart) } else { anime };
        if let Etape::Montree {
            anime: a,
            derniere: d,
            presentee: p,
            ..
        } = &mut self.etape
        {
            *a = anime;
            *d = maintenant;
            *p = true;
        }
        if !finie {
            let ms = u32::try_from(anime.as_millis()).unwrap_or(u32::MAX);
            let image = theme::ouverture(ms, self.effets);
            self.opacite(h, image);
            if let Etape::Montree { finie, .. } = &mut self.etape {
                *finie = image.finie;
            }
            if !image.finie {
                ctx.request_repaint_after(IMAGE_DE_FONDU);
            }
        } else if let Some(taille) = self.taille_a_changer(&ctx, h, m.surface) {
            // Le contenu a changé (un état, un message) : nouvelle taille, puis nouvelle place.
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(taille));
            if let Etape::Montree { replacer, .. } = &mut self.etape {
                *replacer = true;
            }
        } else if replacer {
            let taille = systeme::taille_fenetre(h).unwrap_or([1, 1]);
            if let Some(p) = self.place(&m, taille) {
                systeme::deplacer(h, p[0], p[1], true);
                if let Etape::Montree { replacer, .. } = &mut self.etape {
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
        } else if !self.a_eu_le_focus
            && !self.focus_redemande
            && !discret()
            && debut.elapsed() > Duration::from_millis(300)
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

/// Le plus long pas de l'animation d'une image à la suivante : une image d'un écran à 50 Hz.
/// Une image en retard (écran qui fait attendre) avance l'animation d'un pas seulement :
/// l'ouverture se joue toujours en entier, jamais d'un coup à la fin.
const PAS_MAX: Duration = Duration::from_millis(20);
/// Une image de l'ouverture plus longue que ce temps est notée au journal.
const BLOQUEE: Duration = Duration::from_millis(100);

/// L'animation jouée après une image de plus, arrivée `ecart` après la précédente.
fn avancer(anime: Duration, ecart: Duration) -> Duration {
    anime + ecart.min(PAS_MAX)
}

fn quoi(s: Surface) -> &'static str {
    match s {
        Surface::Volet => "Volet",
        Surface::Menu => "Petit menu",
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
        let observee = match &self.etape {
            Etape::MiseEnPlace { m, .. } => Some((m.clic, true)),
            Etape::Montree { m, finie: false, .. } => Some((m.clic, false)),
            _ => None,
        };
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
        if let Some((clic, mise_en_place)) = observee
            && discret()
            && let Ok(mut o) = OBSERVATEUR.lock()
            && let Some(f) = o.as_mut()
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
            let veilleur = Veilleur::new(liens2.clone());
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
                vue: 0,
                veilleur,
            }))
        }),
    );
    demandes.lecteur(None);
    liens.autres.lier(None);
    liens.surface_prete.store(0, Ordering::Release);
    liens.autres.racine(0);
    let mut s = sortie.borrow_mut();
    match r {
        Ok(()) => Ok(s.suite.take()),
        Err(e) => Err((e.to_string(), s.en_cours.take())),
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Volet, petit menu et Paramètres sans synchronisation verticale : avec elle, le fil des
    /// fenêtres restait bloqué dans SwapBuffers pendant des minutes après le retrait d'un
    /// écran (il passait à un autre PC), et rien ne s'ouvrait plus.
    #[test]
    fn fenetres_sans_synchronisation_verticale() {
        let o = options(egui::ViewportBuilder::default());
        assert!(!o.glow_options.vsync, "SwapBuffers ne doit pas attendre l'écran");
        // Les fondus se cadencent eux-mêmes, sans tourner à vide.
        assert!(IMAGE_DE_FONDU >= Duration::from_millis(10));
        assert!(IMAGE_DE_FONDU <= Duration::from_millis(20));
    }

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
        assert!(!voilee(&Etape::Montree {
            m,
            debut: maintenant,
            anime: Duration::ZERO,
            derniere: maintenant,
            presentee: false,
            finie: false,
            replacer: false,
            garde: Garde::default(),
        }));
        assert!(!voilee(&Etape::Fermeture {
            surface: Surface::Menu,
            debut: maintenant,
            puis: None,
        }));
    }

    /// Après un retard de l'écran (une image échangée 300 ms trop tard), l'animation ne saute
    /// pas à la fin : elle n'avance que d'un pas, et se joue ensuite en entier.
    #[test]
    fn un_retard_ne_fait_pas_sauter_l_animation() {
        let retard = avancer(Duration::ZERO, Duration::from_millis(300));
        assert_eq!(retard, PAS_MAX);
        let ms = u32::try_from(retard.as_millis()).unwrap_or(u32::MAX);
        assert!(!theme::ouverture(ms, true).finie);
        // Des images régulières à 60 Hz : finie en 5 ou 6 images, comme prévu.
        let mut anime = Duration::ZERO;
        let mut images = 0;
        while !theme::ouverture(u32::try_from(anime.as_millis()).unwrap_or(u32::MAX), true).finie {
            anime = avancer(anime, Duration::from_micros(16_667));
            images += 1;
        }
        assert!((5..=6).contains(&images), "{images}");
    }

    /// Prête avant d'être montrée : le voile se lève à la fin de la dernière image de mise en
    /// place, quand la fenêtre est déjà à sa taille, à sa place et à l'opacité de la première
    /// image de l'ouverture. Cette image-là est déjà bien visible (l'ancienne ouverture
    /// montrait d'abord une fenêtre transparente, puis la faisait monter et apparaître en
    /// 250 ms : une attente ressentie à chaque clic).
    #[test]
    fn premiere_image_montree_deja_visible() {
        let premiere = theme::ouverture(0, true);
        assert!(premiere.opacite >= 128, "{}", premiere.opacite);
        const { assert!(theme::OUVERTURE_FONDU_MS <= 100) };
        // « Effets d'animation » coupés : opaque tout de suite.
        assert_eq!(theme::ouverture(0, false).opacite, 255);
        assert!(theme::ouverture(0, false).finie);
    }
}
