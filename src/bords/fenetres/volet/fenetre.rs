//! La fenêtre du volet et du petit menu ([`Volet`]) : sa préparation au repos, sa mise en
//! place, son animation d'ouverture, son clavier et son fondu de fermeture.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use eframe::egui::{self, Key, ViewportCommand};
use ic_coeur::theme::avancer;

use super::super::verrou;
use super::{
    DEJA_OUVERTE, DEJA_PREPAREE, Etape, IMAGE_DE_FONDU, Montrer, Sortie, Veilleur, centre, discret, faire,
    noter, ouvrir_parametres, rang,
};
use crate::Logiciel;
use crate::apparence::Apparence;
use crate::fenetres::{
    Demande, GARDE, GARDE_MAX, Garde, LENTE, Liens, Mesure, Ouvert, Reception, Suite, suite,
};
use crate::place::placer_au_point;
use crate::surface::Choisi;
use crate::systeme;
use crate::theme::{self, Surface};

pub(super) struct Volet<'a> {
    pub(super) logiciel: Arc<dyn Logiciel>,
    pub(super) liens: Arc<Liens>,
    pub(super) demandes: &'a Reception,
    pub(super) apparence: Apparence,
    pub(super) hwnd: Option<isize>,
    /// Windows accepte l'opacité de la fenêtre (fondu possible, préparation invisible).
    pub(super) transparente: bool,
    /// La fenêtre est voilée (`systeme::voiler`) : toujours, sauf montrée pour de vrai.
    pub(super) voilee: Option<bool>,
    pub(super) etape: Etape,
    /// La taille du contenu de chaque surface (points), mesurée à chaque image.
    pub(super) tailles: [egui::Vec2; 2],
    /// La hauteur que le volet ne dépasse pas (points), d'après `Logiciel::placement_volet`
    /// et la zone de travail de son écran ; `None` : aussi haut que son contenu.
    pub(super) borne: Option<f32>,
    /// « Effets d'animation » de Windows, relu à chaque ouverture.
    pub(super) effets: bool,
    pub(super) a_eu_le_focus: bool,
    pub(super) focus_redemande: bool,
    /// Le logiciel se ferme : on se ferme dès que la surface est cachée.
    pub(super) sortie_voulue: Option<Demande>,
    /// La fenêtre se ferme pour de bon (la fermeture n'est pas annulée).
    pub(super) fin_voulue: bool,

    pub(super) sortie: Rc<RefCell<Sortie>>,
    pub(super) creee_a: Instant,
    /// Un clic arrivé avant la préparation : montré dès qu'elle est finie.
    pub(super) premiere: Option<Demande>,
    /// L'opacité posée à la dernière image (pour les mesures).
    pub(super) vue: u8,
    /// Le filet « clic en dehors » (indépendant du focus).
    pub(super) veilleur: Veilleur,
}

impl Volet<'_> {
    pub(super) fn dessinee(&self) -> Surface {
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
    pub(super) fn lire(&mut self, ctx: &egui::Context) {
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
    pub(super) fn cacher_tout_de_suite(&mut self, ctx: &egui::Context) {
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
        *verrou(&liens.ferme_a) = Some((Instant::now(), surface));
        self.sortie.borrow_mut().en_cours = None;
        liens.poser_en_cours(None);
        self.etape = Etape::Cachee;
    }

    /// Rend la main au fil des fenêtres (la fenêtre est déjà cachée) : fin du programme, les
    /// autres fenêtres se ferment avec elle.
    pub(super) fn sortir(&mut self, ctx: &egui::Context) {
        self.fin_voulue = true;
        self.liens.surface_prete.store(0, Ordering::Release);
        self.liens.autres.racine(0);
        self.demandes.racine(0);
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

    pub(super) fn preparer(&mut self, ctx: &egui::Context, n: u8) {
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
            self.demandes.racine(h);
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

    /// La place de la fenêtre (pixels), d'après l'écran de l'icône ; `ppp` : l'échelle de cet
    /// écran (pixels par point), pour l'écart choisi par le logiciel.
    fn place(&self, m: &Montrer, taille: [i32; 2], ppp: f32) -> Option<[i32; 2]> {
        let (e, t) = systeme::ecran_autour(m.ancre[0], m.ancre[1])?;
        let r = match m.surface {
            Surface::Volet => self.logiciel.placement_volet().placer(m.icone, taille, e, t, ppp),
            Surface::Menu => placer_au_point(m.ancre, m.icone, taille, e, t, 8),
        };
        Some([r[0], r[1]])
    }

    /// La hauteur que le volet ne dépasse pas à cette image (points) : la hauteur maximale du
    /// logiciel, bornée par la zone de travail de l'écran de l'icône dès qu'il est connu.
    pub(super) fn borner(&mut self, ctx: &egui::Context) {
        let placement = self.logiciel.placement_volet();
        self.borne = match &self.etape {
            Etape::MiseEnPlace { m, .. } | Etape::Montree { m, .. } => {
                match systeme::ecran_autour(m.ancre[0], m.ancre[1]) {
                    Some((_, t)) => placement.hauteur_max_points(t, ctx.pixels_per_point()),
                    None => placement.hauteur_max,
                }
            }
            // Pendant le fondu de fermeture : la même qu'à l'ouverture.
            Etape::Fermeture { .. } => self.borne,
            Etape::Preparation(_) | Etape::Cachee => placement.hauteur_max,
        }
        .filter(|h| h.is_finite());
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
    pub(super) fn mettre_en_place(&mut self, ctx: &egui::Context, m: Montrer, images: u8) {
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
        let encore = (images == 1 && self.aller_sur_son_ecran(ctx, h, &m))
            || (images < 8 && self.mettre_a_sa_taille(ctx, h, m.surface));
        if encore {
            self.etape = Etape::MiseEnPlace { m, images };
            ctx.request_repaint();
            return;
        }
        self.montrer_a_sa_place(ctx, h, m, images);
    }

    /// La première image de la mise en place : relit les effets d'animation et le thème ; la
    /// fenêtre qui n'est pas sur l'écran de l'icône y est emmenée (`true` : une image de plus).
    fn aller_sur_son_ecran(&mut self, ctx: &egui::Context, h: isize, m: &Montrer) -> bool {
        self.effets = systeme::effets_d_animation();
        if self.apparence.relire(ctx) {
            systeme::habiller(h, self.apparence.palette.sombre);
        }
        let voulu = systeme::ecran_autour(m.ancre[0], m.ancre[1]);
        let ici = systeme::rect_fenetre(h)
            .map(centre)
            .and_then(|c| systeme::ecran_autour(c[0], c[1]));
        match voulu {
            Some((e, t)) if ici.map(|i| i.0) != Some(e) => {
                systeme::deplacer(h, t[0] + 8, t[1] + 8, true);
                true
            }
            _ => false,
        }
    }

    /// Demande la taille voulue si la fenêtre ne l'a pas encore (`true` : une image de plus).
    fn mettre_a_sa_taille(&self, ctx: &egui::Context, h: isize, surface: Surface) -> bool {
        match self.taille_a_changer(ctx, h, surface) {
            Some(taille) => {
                ctx.send_viewport_cmd(ViewportCommand::InnerSize(taille));
                true
            }
            None => false,
        }
    }

    /// À la bonne taille sur le bon écran : la fenêtre est posée à sa place, déjà visible, et
    /// l'animation d'ouverture commence (le délai est noté, le filet du clic en dehors posé).
    fn montrer_a_sa_place(&mut self, ctx: &egui::Context, h: isize, m: Montrer, images: u8) {
        let taille = systeme::taille_fenetre(h).unwrap_or([1, 1]);
        let place = self.place(&m, taille, ctx.pixels_per_point()).unwrap_or_else(|| {
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

    pub(super) fn commencer_fermeture(&mut self, ctx: &egui::Context, puis: Option<Choisi>) {
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

    pub(super) fn fermer_pas_a_pas(&mut self, ctx: &egui::Context) {
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
    pub(super) fn montree_image(&mut self, ui: &mut egui::Ui, action: Option<Choisi>) {
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
        self.noter_l_image(&m, presentee, finie, ecart, maintenant);
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
        self.animer(ui.ctx(), h, &m, anime, finie, replacer);
        self.clavier(ui, h, debut, garde, action);
    }

    /// Au journal : la première image montrée présentée tard après le clic, ou une image de
    /// l'ouverture bloquée.
    fn noter_l_image(&self, m: &Montrer, presentee: bool, finie: bool, ecart: Duration, maintenant: Instant) {
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
    }

    /// L'animation d'ouverture ; une fois finie, la nouvelle taille et la nouvelle place quand
    /// le contenu change.
    fn animer(
        &mut self,
        ctx: &egui::Context,
        h: isize,
        m: &Montrer,
        anime: Duration,
        finie: bool,
        replacer: bool,
    ) {
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
        } else if let Some(taille) = self.taille_a_changer(ctx, h, m.surface) {
            // Le contenu a changé (un état, un message) : nouvelle taille, puis nouvelle place.
            ctx.send_viewport_cmd(ViewportCommand::InnerSize(taille));
            if let Etape::Montree { replacer, .. } = &mut self.etape {
                *replacer = true;
            }
        } else if replacer {
            let taille = systeme::taille_fenetre(h).unwrap_or([1, 1]);
            if let Some(p) = self.place(m, taille, ctx.pixels_per_point()) {
                systeme::deplacer(h, p[0], p[1], true);
                if let Etape::Montree { replacer, .. } = &mut self.etape {
                    *replacer = false;
                }
            }
        }
    }

    /// Le clavier (Échap ferme), le focus, la garde des premiers instants et la fermeture.
    fn clavier(&mut self, ui: &egui::Ui, h: isize, debut: Instant, garde: Garde, action: Option<Choisi>) {
        let ctx = ui.ctx();
        // Clavier : Échap pour fermer ; les flèches passent d'une ligne à l'autre (elles sont
        // changées en Tab avant egui : `raw_input_hook`).
        let echap = ui.input(|i| i.key_pressed(Key::Escape));
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
            self.commencer_fermeture(ctx, None);
        } else if action.is_some() {
            self.commencer_fermeture(ctx, action);
        }
    }
}

/// Une image de l'ouverture plus longue que ce temps est notée au journal.
const BLOQUEE: Duration = Duration::from_millis(100);

fn quoi(s: Surface) -> &'static str {
    match s {
        Surface::Volet => "Volet",
        Surface::Menu => "Petit menu",
    }
}
