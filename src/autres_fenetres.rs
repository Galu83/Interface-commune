//! Les autres fenêtres du logiciel, ouvertes et fermées **dans la boucle de fenêtres de la
//! bibliothèque**.
//!
//! winit 0.30 n'accepte qu'une boucle de fenêtres par programme : une deuxième, créée dans un
//! autre fil, échoue (« EventLoop can't be recreated »). La bibliothèque garde donc une seule
//! boucle, toute la vie du logiciel, dont la fenêtre principale est celle du volet (cachée au
//! repos) ; chaque autre fenêtre (les Paramètres, une fenêtre simple, une fenêtre de lecture…)
//! y est une fenêtre de plus d'egui (un « viewport » immédiat), dessinée pendant l'image de la
//! fenêtre principale.
//!
//! Pour un logiciel, depuis n'importe quel fil :
//!
//! ```no_run
//! # fn exemple(fenetres: interface_commune::autres_fenetres::AutresFenetres) {
//! use interface_commune::autres_fenetres::{Cadre, Contenu, Ouverture};
//! struct Bonjour;
//! impl Contenu for Bonjour {
//!     fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
//!         egui::CentralPanel::default().show(ui, |ui| {
//!             if ui.button("Fermer").clicked() {
//!                 cadre.fermer();
//!             }
//!         });
//!     }
//! }
//! fenetres.ouvrir("bonjour", Ouverture::new("Bonjour", [320.0, 160.0]), Bonjour);
//! // … plus tard, depuis n'importe où :
//! fenetres.fermer("bonjour");
//! # }
//! ```
//!
//! Chaque fenêtre s'ouvre **cachée**, se dessine deux fois, puis se montre : jamais une
//! fenêtre vide ni un éclair blanc. Elle suit le thème de Windows (barre de titre sombre,
//! coins arrondis). Elle se ferme par sa croix, Alt+F4, [`Cadre::fermer`] ou
//! [`AutresFenetres::fermer`] ; [`Contenu::fermee`] reçoit alors sa place, à retenir.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use egui::{ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};

use crate::place::Place;
use crate::theme::Palette;

/// Comment ouvrir une fenêtre : son titre, sa taille (points), sa place…
#[derive(Clone, Debug)]
pub struct Ouverture {
    /// Le titre de la barre de titre (aussi celui de la barre des tâches).
    pub titre: String,
    /// La taille intérieure au premier affichage (points).
    pub taille: [f32; 2],
    /// La taille minimale (points), si elle se redimensionne.
    pub taille_min: Option<[f32; 2]>,
    pub redimensionnable: bool,
    /// L'icône de la barre de titre et de la barre des tâches.
    pub icone: Option<Arc<egui::IconData>>,
    /// La place (gauche, haut, largeur, hauteur, en pixels) ; sans elle, la fenêtre est
    /// centrée sur l'écran de la souris.
    pub place: Option<Place>,
    pub toujours_devant: bool,
    pub barre_des_taches: bool,
    /// La barre de titre et le cadre de Windows.
    pub cadre: bool,
    /// Prendre le focus en se montrant.
    pub focus: bool,
    /// Sans `place`, la bibliothèque centre la fenêtre sur l'écran de la souris à chaque
    /// image cachée ; `false` : le contenu la pose lui-même (près d'une zone, par exemple).
    pub centree: bool,
}

impl Ouverture {
    /// Une fenêtre ordinaire : barre de titre, redimensionnable, centrée, dans la barre des
    /// tâches, qui prend le focus.
    pub fn new(titre: &str, taille: [f32; 2]) -> Ouverture {
        Ouverture {
            titre: titre.into(),
            taille,
            taille_min: None,
            redimensionnable: true,
            icone: None,
            place: None,
            toujours_devant: false,
            barre_des_taches: true,
            cadre: true,
            focus: true,
            centree: true,
        }
    }

    pub fn taille_min(mut self, t: [f32; 2]) -> Ouverture {
        self.taille_min = Some(t);
        self
    }

    pub fn fixe(mut self) -> Ouverture {
        self.redimensionnable = false;
        self
    }

    pub fn icone(mut self, icone: egui::IconData) -> Ouverture {
        self.icone = Some(Arc::new(icone));
        self
    }

    /// La place retenue (pixels) ; `None` : centrée.
    pub fn place(mut self, place: Option<Place>) -> Ouverture {
        self.place = place;
        self
    }

    pub fn toujours_devant(mut self) -> Ouverture {
        self.toujours_devant = true;
        self
    }

    pub fn hors_barre_des_taches(mut self) -> Ouverture {
        self.barre_des_taches = false;
        self
    }

    /// Sans barre de titre ni cadre (une bulle, un bandeau).
    pub fn sans_cadre(mut self) -> Ouverture {
        self.cadre = false;
        self
    }

    /// Se montre sans prendre le focus (une fenêtre de lecture qui ne doit pas voler la
    /// frappe en cours).
    pub fn sans_focus(mut self) -> Ouverture {
        self.focus = false;
        self
    }

    /// Le contenu pose lui-même la fenêtre (`systeme::poser` sur le `hwnd` du `Cadre`) : la
    /// bibliothèque ne la centre pas avant de la montrer.
    pub fn pose_par_le_contenu(mut self) -> Ouverture {
        self.centree = false;
        self
    }

    /// Ce qu'egui reçoit : la fenêtre est toujours créée cachée.
    pub fn constructeur(&self) -> ViewportBuilder {
        let mut b = ViewportBuilder::default()
            .with_title(&self.titre)
            .with_inner_size(self.taille)
            .with_resizable(self.redimensionnable)
            .with_decorations(self.cadre)
            .with_taskbar(self.barre_des_taches)
            .with_active(self.focus)
            .with_visible(false);
        if let Some(t) = self.taille_min {
            b = b.with_min_inner_size(t);
        }
        if let Some(i) = &self.icone {
            b = b.with_icon(i.clone());
        }
        if self.toujours_devant {
            b = b.with_always_on_top();
        }
        b
    }
}

/// Ce qu'une fenêtre montre. Ses méthodes sont appelées sur le fil des fenêtres : elles
/// dessinent ce que le logiciel a déjà en mémoire, sans disque, réseau ni attente.
pub trait Contenu: Send + 'static {
    /// Dessine la fenêtre, à chaque image. `ui` n'a ni marge ni fond : un
    /// `egui::CentralPanel` (ou [`crate::dessin::fond`]) les donne.
    fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>);

    /// La fenêtre va se fermer : le contenu enregistre ce qui attend, et peut **refuser**
    /// (`false`) si l'enregistrement a échoué : la fenêtre reste ouverte, et le contenu montre
    /// pourquoi. Toujours appelé, quelle que soit la façon de fermer, même fenêtre réduite.
    ///
    /// `imposee` : la fermeture ne se discute pas ([`AutresFenetres::fermer`] ou
    /// [`AutresFenetres::tout_fermer`] : le logiciel se ferme, la session se termine) ; la
    /// réponse est alors ignorée, mais le contenu a pu enregistrer. Pour la croix, Alt+F4 ou
    /// la barre des tâches, `imposee` est `false` et le refus garde la fenêtre.
    fn peut_fermer(&mut self, _imposee: bool) -> bool {
        true
    }

    /// Le contenu est prêt à être vu (sa taille est connue, ce qu'il attend est arrivé) ?
    /// Tant que non, la fenêtre reste cachée, même après ses deux images ; on la redessine
    /// jusqu'à ce que oui. Par défaut : toujours prêt.
    fn prete(&self) -> bool {
        true
    }

    /// La fenêtre se ferme ; `place` : sa dernière place (pixels), à retenir.
    fn fermee(&mut self, _place: Option<Place>) {}
}

/// Ce qu'une fenêtre reçoit à chaque image, et ce qu'elle peut demander.
pub struct Cadre<'a> {
    pub palette: &'a Palette,
    /// La fenêtre de Windows (HWND), dès qu'elle est trouvée.
    pub hwnd: Option<isize>,
    /// Le nombre d'images dessinées depuis l'ouverture (1 à la première).
    pub image: u32,
    /// La fenêtre est montrée (les premières images sont dessinées cachées).
    pub montree: bool,
    fermer: bool,
    titre: Option<String>,
}

impl<'a> Cadre<'a> {
    /// Un cadre pour dessiner un contenu ailleurs que dans la boucle (fenêtre principale
    /// d'un logiciel, essais).
    pub fn new(palette: &'a Palette, hwnd: Option<isize>, image: u32, montree: bool) -> Cadre<'a> {
        Cadre {
            palette,
            hwnd,
            image,
            montree,
            fermer: false,
            titre: None,
        }
    }

    /// Le contenu a demandé la fermeture.
    pub fn fermeture_demandee(&self) -> bool {
        self.fermer
    }

    /// Le titre demandé par le contenu, s'il en a demandé un.
    pub fn titre_demande(&self) -> Option<&str> {
        self.titre.as_deref()
    }

    /// Ferme la fenêtre à la fin de cette image.
    pub fn fermer(&mut self) {
        self.fermer = true;
    }

    /// Change le titre de la fenêtre.
    pub fn titre(&mut self, titre: &str) {
        self.titre = Some(titre.into());
    }
}

/// Une fenêtre ouverte.
struct Fenetre {
    ouverture: Ouverture,
    contenu: Box<dyn Contenu>,
    /// Le titre en cours (la fenêtre de Windows est retrouvée par lui).
    titre: String,
    images: u32,
    montree: bool,
    hwnd: Option<isize>,
    /// Le thème avec lequel la barre de titre a été habillée.
    sombre: Option<bool>,
    /// Voilée (`systeme::voiler`) tant qu'elle n'est pas montrée.
    voilee: Option<bool>,
    devant: bool,
    fermer: bool,
    /// Le contenu a déjà été consulté sur cette fermeture (`Contenu::peut_fermer`).
    consulte: bool,
    /// Les créations ratées de suite (eframe n'a pas pu créer la fenêtre de Windows ou son
    /// contexte OpenGL) : voir [`CREATIONS_MAX`].
    ratees: u32,
    /// Pas de nouvel essai de création avant cet instant (après une création ratée).
    pas_avant: Option<std::time::Instant>,
}

/// Les créations ratées de suite après lesquelles une fenêtre est abandonnée (fermée).
pub const CREATIONS_MAX: u32 = 3;

/// L'attente avant un nouvel essai de création.
const NOUVEL_ESSAI: std::time::Duration = std::time::Duration::from_millis(500);

/// Après un changement d'affichage de Windows (`WM_DISPLAYCHANGE` : écran retiré, rendu, autre
/// résolution), une fenêtre qui n'existe pas encore n'est créée qu'au bout de ce temps : le
/// pilote graphique refait alors son travail, et créer une fenêtre OpenGL au même moment a
/// fait échouer la création, ou bloqué le fil dans SwapBuffers pendant des dizaines de
/// secondes. Une fenêtre déjà créée n'attend jamais (egui détruit celle qu'on ne dessine pas).
pub const REPRISE: std::time::Duration = std::time::Duration::from_secs(1);

/// Un premier dessin après un changement d'affichage est mesuré au journal pendant ce temps.
const SUIVI_DE_LA_REPRISE: std::time::Duration = std::time::Duration::from_secs(10);

/// Une fenêtre retirée n'est détruite par egui qu'à la fin de l'image qui ne la dessine plus.
/// Créer une autre fenêtre aussitôt après a fait bloquer sans fin `show_viewport_immediate`
/// (un processeur très chargé, plusieurs fenêtres fermées puis ouvertes de suite). Aucune
/// fenêtre n'est donc créée tant qu'une fenêtre retirée existe encore, ni pendant
/// [`APRES_DESTRUCTION`] après la dernière destruction. Au plus [`DESTRUCTION_MAX`] d'attente
/// pour une fenêtre qui ne disparaît pas.
pub const DESTRUCTION_MAX: std::time::Duration = std::time::Duration::from_secs(2);

/// Le temps laissé à Windows et au pilote graphique après la destruction d'une fenêtre,
/// avant d'en créer une autre.
pub const APRES_DESTRUCTION: std::time::Duration = std::time::Duration::from_millis(100);

/// Le temps entre deux vérifications qu'une fenêtre retirée a disparu.
const VERIFICATION: std::time::Duration = std::time::Duration::from_millis(10);

/// Les fenêtres retirées (leur HWND et l'heure du retrait) qui existent encore, et l'heure
/// de la dernière destruction constatée.
#[derive(Default)]
struct Destructions {
    en_attente: Vec<(isize, std::time::Instant)>,
    derniere: Option<std::time::Instant>,
}

static DESTRUCTIONS: Mutex<Destructions> = Mutex::new(Destructions {
    en_attente: Vec::new(),
    derniere: None,
});

impl Destructions {
    /// Une fenêtre vient d'être retirée : egui va la détruire.
    fn retiree(&mut self, h: isize, maintenant: std::time::Instant) {
        self.en_attente.push((h, maintenant));
        self.derniere = Some(maintenant);
    }

    /// Constate les destructions (`existe` : la fenêtre existe-t-elle encore ?) et rend
    /// l'attente avant de pouvoir créer une fenêtre (`None` : maintenant).
    fn attente(
        &mut self,
        maintenant: std::time::Instant,
        existe: impl Fn(isize) -> bool,
    ) -> Option<std::time::Duration> {
        let mut derniere = self.derniere;
        self.en_attente.retain(|&(h, retiree)| {
            let depuis = maintenant.saturating_duration_since(retiree);
            if depuis >= DESTRUCTION_MAX {
                // Elle ne disparaît pas : on ne bloque pas pour toujours.
                return false;
            }
            if existe(h) {
                return true;
            }
            // Détruite. Constatée longtemps après son retrait (personne ne regardait) : la
            // destruction date du retrait, pas d'aujourd'hui.
            let quand = if depuis > APRES_DESTRUCTION {
                retiree
            } else {
                maintenant
            };
            derniere = Some(derniere.map_or(quand, |d| d.max(quand)));
            false
        });
        self.derniere = derniere;
        attente_apres_destruction(
            !self.en_attente.is_empty(),
            derniere.map(|d| maintenant.saturating_duration_since(d)),
        )
    }
}

/// L'attente avant de créer une fenêtre après des destructions : tant qu'une fenêtre retirée
/// existe (`en_attente`), le temps de la prochaine vérification ; sinon, ce qui reste de
/// [`APRES_DESTRUCTION`] depuis la dernière destruction (`depuis`) ; `None` : créer.
pub fn attente_apres_destruction(
    en_attente: bool,
    depuis: Option<std::time::Duration>,
) -> Option<std::time::Duration> {
    if en_attente {
        return Some(VERIFICATION);
    }
    depuis
        .and_then(|d| APRES_DESTRUCTION.checked_sub(d))
        .filter(|d| !d.is_zero())
}

/// L'attente due aux destructions en cours, maintenant.
fn attente_destruction() -> Option<std::time::Duration> {
    #[cfg(windows)]
    let existe = crate::systeme::fenetre_existe;
    #[cfg(not(windows))]
    let existe = |_: isize| false;
    verrou(&DESTRUCTIONS).attente(std::time::Instant::now(), existe)
}

/// L'heure du dernier changement d'affichage de Windows.
static AFFICHAGE_CHANGE: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// Windows vient de changer l'affichage (`WM_DISPLAYCHANGE`) : les fenêtres pas encore créées
/// attendent [`REPRISE`]. À appeler depuis le fil de l'icône.
pub fn affichage_change() {
    *verrou(&AFFICHAGE_CHANGE) = Some(std::time::Instant::now());
}

fn depuis_le_changement() -> Option<std::time::Duration> {
    (*verrou(&AFFICHAGE_CHANGE)).map(|t| t.elapsed())
}

/// Combien de temps une fenêtre pas encore créée attend avant d'être dessinée (`None` :
/// maintenant). `depuis` : le temps écoulé depuis le dernier changement d'affichage ;
/// `essai` : ce qui reste avant le nouvel essai d'une création ratée ; `destruction` :
/// l'attente due aux fenêtres qui viennent d'être détruites ([`attente_apres_destruction`]).
/// Elle attend le plus long des trois.
pub fn attente_avant_creation(
    depuis: Option<std::time::Duration>,
    essai: Option<std::time::Duration>,
    destruction: Option<std::time::Duration>,
) -> Option<std::time::Duration> {
    let reprise = depuis.and_then(|d| REPRISE.checked_sub(d));
    [reprise, essai, destruction]
        .into_iter()
        .flatten()
        .filter(|d| !d.is_zero())
        .max()
}

/// Ce que la fenêtre devient après une image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Apres {
    Garder,
    Fermer,
}

/// Les images dessinées cachées avant de montrer la fenêtre : la première pour la mettre en
/// page, la deuxième pour la dessiner à sa place et à sa taille.
pub const IMAGES_CACHEES: u32 = 2;

impl Fenetre {
    /// Combien de temps la création attend encore avant son nouvel essai (`None` : elle peut
    /// être tentée). Seule une fenêtre jamais dessinée attend : une fenêtre déjà créée est
    /// dessinée à chaque image, egui détruirait celle qui ne l'est pas.
    fn attente(&self, maintenant: std::time::Instant) -> Option<std::time::Duration> {
        if self.images > 0 || self.montree {
            return None;
        }
        let essai = self.pas_avant.and_then(|t| t.checked_duration_since(maintenant));
        attente_avant_creation(depuis_le_changement(), essai, attente_destruction())
    }

    fn image(&mut self, ui: &mut egui::Ui, classe: ViewportClass, palette: &Palette) -> Apres {
        let ctx = ui.ctx().clone();
        if self.fermer {
            return Apres::Fermer;
        }
        if ui.input(|i| i.viewport().close_requested()) {
            // La croix, Alt+F4, la barre des tâches : le contenu enregistre, ou refuse.
            if self.contenu.peut_fermer(false) {
                self.consulte = true;
                return Apres::Fermer;
            }
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        self.images = self.images.saturating_add(1);
        let reelle = !matches!(classe, ViewportClass::EmbeddedWindow);
        #[cfg(windows)]
        if reelle {
            self.windows(palette);
        }
        if !self.montree {
            if reelle && (self.images < IMAGES_CACHEES || !self.contenu.prete()) {
                ctx.request_repaint();
            } else {
                self.montree = true;
                #[cfg(windows)]
                self.voiler(false);
                #[cfg(windows)]
                let sans_focus = !self.ouverture.focus && self.hwnd.is_some();
                #[cfg(not(windows))]
                let sans_focus = false;
                if sans_focus {
                    // Jamais le premier plan : `Visible(true)` activerait la fenêtre.
                    #[cfg(windows)]
                    if let Some(h) = self.hwnd {
                        crate::systeme::montrer_sans_activer(h);
                    }
                } else {
                    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    if self.ouverture.focus {
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    }
                }
            }
        }
        if std::mem::take(&mut self.devant) {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            #[cfg(windows)]
            if let Some(h) = self.hwnd {
                crate::systeme::premier_plan(h);
            }
        }
        let mut cadre = Cadre {
            palette,
            hwnd: self.hwnd,
            image: self.images,
            montree: self.montree,
            fermer: false,
            titre: None,
        };
        self.contenu.dessiner(ui, &mut cadre);
        if let Some(t) = cadre.titre
            && t != self.titre
        {
            ctx.send_viewport_cmd(ViewportCommand::Title(t.clone()));
            self.titre = t;
        }
        if cadre.fermer {
            Apres::Fermer
        } else {
            Apres::Garder
        }
    }

    /// Sous Windows : la fenêtre retrouvée par son titre, habillée (coins, barre de titre
    /// sombre), posée à sa place ou centrée, avant d'être montrée.
    #[cfg(windows)]
    fn windows(&mut self, palette: &Palette) {
        use crate::systeme;
        if self.hwnd.is_none() {
            let mut prises = verrou(&PRISES);
            self.hwnd = systeme::fenetre_du_fil(&self.titre, &prises);
            if let Some(h) = self.hwnd {
                prises.push(h);
                // Glissée vers un écran d'une autre échelle : elle suit la souris
                // (`crate::echelle`).
                crate::echelle::suivre_windows(h);
            }
        }
        let Some(h) = self.hwnd else { return };
        // winit réécrit les styles de la fenêtre à chaque changement de ses réglages (visibilité,
        // boutons…) : ceux qu'il ne connaît pas sont donc reposés à chaque image (sans effet si
        // déjà faits), avant de la montrer.
        if !self.ouverture.focus {
            systeme::sans_focus(h, !self.ouverture.barre_des_taches);
        }
        if !self.ouverture.cadre {
            systeme::sans_cadre(h);
        }
        if !self.montree {
            // Préparée cachée : voilée dès qu'elle existe (eframe la montrerait d'office).
            self.voiler(true);
        }
        if self.sombre != Some(palette.sombre) {
            self.sombre = Some(palette.sombre);
            systeme::habiller(h, palette.sombre);
        }
        if !self.montree {
            // Posée à chaque image cachée : si Windows change son échelle en l'amenant sur un
            // autre écran, la fois suivante lui rend sa taille.
            match self.ouverture.place {
                Some(p) => systeme::poser(h, p),
                None if self.ouverture.centree => {
                    if let Some(t) = systeme::taille_fenetre(h)
                        && let Some(p) = systeme::centree(t)
                    {
                        systeme::deplacer(h, p[0], p[1], false);
                    }
                }
                None => {}
            }
        }
    }

    /// Voile la fenêtre ou lève le voile (une seule fois par changement).
    #[cfg(windows)]
    fn voiler(&mut self, voile: bool) {
        if let Some(h) = self.hwnd
            && self.voilee != Some(voile)
            && crate::systeme::voiler(h, voile)
        {
            self.voilee = Some(voile);
        }
    }

    /// La place de la fenêtre (pixels), pour la retenir.
    fn place(&self) -> Option<Place> {
        #[cfg(windows)]
        {
            let r = crate::systeme::rect_fenetre(self.hwnd?)?;
            crate::place::place_valable([r[0], r[1], r[2] - r[0], r[3] - r[1]])
        }
        #[cfg(not(windows))]
        {
            None
        }
    }
}

type Partagee = Arc<Mutex<Fenetre>>;

/// Les fenêtres de Windows déjà attribuées à une fenêtre du logiciel (deux fenêtres du même titre
/// ne se confondent pas).
static PRISES: Mutex<Vec<isize>> = Mutex::new(Vec::new());

#[derive(Default)]
struct Registre {
    fenetres: BTreeMap<String, Partagee>,
}

/// Les autres fenêtres du logiciel. Se clone et s'utilise depuis n'importe quel fil ; la
/// fenêtre principale de la boucle (le volet) les dessine par [`AutresFenetres::montrer`].
#[derive(Clone, Default)]
pub struct AutresFenetres {
    registre: Arc<Mutex<Registre>>,
    contexte: Arc<Mutex<Option<egui::Context>>>,
    /// La fenêtre principale de la boucle (le volet, cachée au repos), 0 tant qu'elle n'est pas
    /// prête : une demande venue d'un autre fil la réveille.
    racine: Arc<std::sync::atomic::AtomicIsize>,
}

fn verrou<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Un verrou empoisonné (un dessin a paniqué) ne bloque pas les autres fenêtres.
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// L'identifiant egui de la fenêtre de cette clé.
pub fn id_de(cle: &str) -> ViewportId {
    ViewportId::from_hash_of(("interface-commune", cle))
}

impl AutresFenetres {
    /// Le contexte egui de la boucle (fixé par la bibliothèque) : une ouverture ou une
    /// fermeture la réveille.
    pub fn lier(&self, ctx: Option<egui::Context>) {
        *verrou(&self.contexte) = ctx;
        // Les fenêtres demandées avant que la boucle soit prête s'ouvrent maintenant.
        self.reveiller();
    }

    /// La fenêtre principale de la boucle (0 : aucune), fixée par le volet.
    pub(crate) fn racine(&self, h: isize) {
        self.racine.store(h, std::sync::atomic::Ordering::Release);
    }

    /// Réveille la fenêtre principale, pour qu'elle ouvre, ferme ou redessine. Cachée au repos,
    /// elle ne répond pas à une simple demande de dessin venue d'un autre fil : elle est
    /// montrée voilée (rien n'apparaît), dessine, puis se recache.
    pub fn reveiller(&self) {
        if let Some(c) = verrou(&self.contexte).as_ref() {
            c.request_repaint_of(ViewportId::ROOT);
        }
        #[cfg(windows)]
        crate::systeme::reveiller(self.racine.load(std::sync::atomic::Ordering::Acquire));
    }

    /// Ouvre la fenêtre de cette clé (une seule à la fois par clé). Déjà ouverte : elle passe
    /// devant, son contenu ne change pas, et la réponse est `false`.
    pub fn ouvrir(&self, cle: &str, ouverture: Ouverture, contenu: impl Contenu) -> bool {
        self.ouvrir_boite(cle, ouverture, Box::new(contenu))
    }

    /// Comme [`AutresFenetres::ouvrir`], pour un contenu déjà en boîte.
    pub fn ouvrir_boite(&self, cle: &str, ouverture: Ouverture, contenu: Box<dyn Contenu>) -> bool {
        let nouvelle = {
            let mut r = verrou(&self.registre);
            if let Some(f) = r.fenetres.get(cle) {
                let mut f = verrou(f);
                if f.fermer {
                    // Fermée mais pas encore partie : elle reste ouverte.
                    f.fermer = false;
                }
                f.devant = true;
                false
            } else {
                let titre = ouverture.titre.clone();
                r.fenetres.insert(
                    cle.into(),
                    Arc::new(Mutex::new(Fenetre {
                        ouverture,
                        contenu,
                        titre,
                        images: 0,
                        montree: false,
                        hwnd: None,
                        sombre: None,
                        voilee: None,
                        devant: false,
                        fermer: false,
                        consulte: false,
                        ratees: 0,
                        pas_avant: None,
                    })),
                );
                true
            }
        };
        self.reveiller();
        nouvelle
    }

    /// Ferme la fenêtre de cette clé (rien si elle n'est pas ouverte).
    pub fn fermer(&self, cle: &str) {
        if let Some(f) = verrou(&self.registre).fenetres.get(cle) {
            verrou(f).fermer = true;
        }
        self.reveiller();
    }

    /// Ferme toutes les fenêtres.
    pub fn tout_fermer(&self) {
        for f in verrou(&self.registre).fenetres.values() {
            verrou(f).fermer = true;
        }
        self.reveiller();
    }

    pub fn ouverte(&self, cle: &str) -> bool {
        verrou(&self.registre)
            .fenetres
            .get(cle)
            .is_some_and(|f| !verrou(f).fermer)
    }

    /// Les clés des fenêtres ouvertes.
    pub fn cles(&self) -> Vec<String> {
        verrou(&self.registre).fenetres.keys().cloned().collect()
    }

    /// Fait passer la fenêtre devant (et la sort de la barre des tâches si elle y est
    /// réduite).
    pub fn devant(&self, cle: &str) {
        if let Some(f) = verrou(&self.registre).fenetres.get(cle) {
            verrou(f).devant = true;
        }
        self.reveiller();
    }

    /// Redessine les fenêtres (l'état du logiciel a changé).
    pub fn redessiner(&self) {
        self.reveiller();
    }

    /// Faut-il une image complète de la fenêtre principale même cachée ? Oui tant qu'une
    /// fenêtre doit s'ouvrir (encore cachée), passer devant ou se fermer : une fenêtre cachée
    /// sans fenêtre visible n'aurait sinon aucune image.
    pub fn a_faire(&self) -> bool {
        self.prochaine_image().is_some()
    }

    /// Dans combien de temps la fenêtre principale doit faire une image complète, même
    /// cachée (`None` : rien à faire). `Some(0)` : tout de suite ; plus : une fenêtre dont la
    /// création a raté attend son nouvel essai ([`NOUVEL_ESSAI`]) au lieu de le refaire à
    /// chaque image.
    pub fn prochaine_image(&self) -> Option<std::time::Duration> {
        let maintenant = std::time::Instant::now();
        verrou(&self.registre)
            .fenetres
            .values()
            .filter_map(|f| {
                let f = verrou(f);
                if f.fermer {
                    return Some(std::time::Duration::ZERO);
                }
                if let Some(d) = f.attente(maintenant) {
                    return Some(d);
                }
                (!f.montree || f.devant).then_some(std::time::Duration::ZERO)
            })
            .min()
    }

    /// Essais : les fenêtres n'attendent plus leur nouvel essai de création.
    #[cfg(test)]
    fn sans_attente(&self) {
        for f in verrou(&self.registre).fenetres.values() {
            verrou(f).pas_avant = None;
        }
    }

    /// Dessine les fenêtres ouvertes, à chaque image de la fenêtre principale, et ferme celles
    /// qui le demandent. Aucun verrou n'est tenu pendant le dessin d'un contenu : il peut
    /// ouvrir ou fermer d'autres fenêtres.
    pub fn montrer(&self, ctx: &egui::Context, palette: &Palette) {
        // Une fenêtre retirée qui n'a pas encore disparu : constatée dès qu'elle disparaît, sans
        // attendre un clic.
        if let Some(d) = attente_destruction() {
            ctx.request_repaint_after_for(d, ViewportId::ROOT);
        }
        let liste: Vec<(String, Partagee)> = verrou(&self.registre)
            .fenetres
            .iter()
            .map(|(c, f)| (c.clone(), f.clone()))
            .collect();
        for (cle, f) in liste {
            let builder = verrou(&f).ouverture.constructeur();
            let apres = if verrou(&f).fermer {
                Apres::Fermer
            } else if let Some(d) = verrou(&f).attente(std::time::Instant::now()) {
                // Création ratée : pas de nouvel essai avant l'heure (une fenêtre jamais
                // dessinée n'existe pas encore pour egui : rien n'est détruit).
                ctx.request_repaint_after_for(d, ViewportId::ROOT);
                continue;
            } else {
                let premiere = {
                    let g = verrou(&f);
                    g.images == 0 && !g.montree
                };
                let debut = std::time::Instant::now();
                // La création de la fenêtre se passe dans cet appel : le chien de garde écrit
                // au journal celle qui ne finit pas.
                let _creation = premiere.then(|| {
                    let titre = verrou(&f).titre.clone();
                    crate::sonde::creation(&titre)
                });
                let apres = self.image_ou_essai(ctx, &cle, &f, builder, palette);
                // Le premier dessin après un changement d'affichage, mesuré (la reprise s'est
                // bien passée, ou non).
                if premiere
                    && let Some(depuis) = depuis_le_changement()
                    && depuis < SUIVI_DE_LA_REPRISE
                {
                    let g = verrou(&f);
                    if g.images > 0 {
                        crate::sonde::noter(format!(
                            "Fenêtre « {} » : premier dessin après le changement d'affichage de Windows : {} ms (le changement date de {} ms).",
                            g.titre,
                            debut.elapsed().as_millis(),
                            depuis.as_millis()
                        ));
                    }
                }
                apres
            };
            if apres == Apres::Fermer {
                self.retirer(&cle, &f);
                // La fenêtre principale refait une image : la fenêtre fermée n'y est plus,
                // egui la détruit.
                ctx.request_repaint_of(ViewportId::ROOT);
            }
        }
    }

    /// Une image de la fenêtre ; si eframe n'a pas pu la créer, un nouvel essai plus tard.
    ///
    /// Quand eframe ne peut pas créer la fenêtre de Windows ou son contexte OpenGL (Windows
    /// refait l'affichage : un écran retiré ou rendu, une carte graphique qui se relance), il
    /// rend la main sans dessiner, et egui panique (« the user callback was never called »).
    /// Sans ce garde-fou, la panique emportait toute la boucle des fenêtres (le volet et le
    /// petit menu étaient recréés, à froid, au milieu du changement d'affichage). Ici, la
    /// fenêtre attend [`NOUVEL_ESSAI`] et réessaie ; après [`CREATIONS_MAX`] échecs de suite,
    /// elle est fermée. Une panique du contenu lui-même (son dessin a commencé) suit son cours.
    fn image_ou_essai(
        &self,
        ctx: &egui::Context,
        cle: &str,
        f: &Partagee,
        builder: ViewportBuilder,
        palette: &Palette,
    ) -> Apres {
        let dessinee = std::cell::Cell::new(false);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.show_viewport_immediate(id_de(cle), builder, |ui, classe| {
                dessinee.set(true);
                verrou(f).image(ui, classe, palette)
            })
        }));
        match r {
            Ok(apres) => {
                let mut g = verrou(f);
                g.ratees = 0;
                g.pas_avant = None;
                apres
            }
            Err(p) if dessinee.get() => std::panic::resume_unwind(p),
            Err(_) => {
                // Panique rattrapée : son lieu ne doit pas être pris, plus tard, pour celui
                // d'une autre panique (la sonde le garde pour le journal).
                let _ = crate::sonde::lieu_de_la_derniere_panique();
                let (n, titre) = {
                    let mut g = verrou(f);
                    g.ratees = g.ratees.saturating_add(1);
                    g.pas_avant = Some(std::time::Instant::now() + NOUVEL_ESSAI);
                    (g.ratees, g.titre.clone())
                };
                if n >= CREATIONS_MAX {
                    crate::sonde::noter(format!(
                        "Fenêtre « {titre} » : Windows n'a pas pu la créer ({n} essais de suite) ; fermée."
                    ));
                    Apres::Fermer
                } else {
                    crate::sonde::noter(format!(
                        "Fenêtre « {titre} » : Windows n'a pas pu la créer (essai {n} sur {CREATIONS_MAX}) ; nouvel essai dans {} ms.",
                        NOUVEL_ESSAI.as_millis()
                    ));
                    ctx.request_repaint_after_for(NOUVEL_ESSAI, ViewportId::ROOT);
                    Apres::Garder
                }
            }
        }
    }

    fn retirer(&self, cle: &str, f: &Partagee) {
        {
            let mut r = verrou(&self.registre);
            if r.fenetres.get(cle).is_some_and(|g| Arc::ptr_eq(g, f)) {
                r.fenetres.remove(cle);
            }
        }
        let mut f = verrou(f);
        if !f.consulte {
            // Fermée par le logiciel ou par son contenu : le refus ne compte pas.
            let _ = f.contenu.peut_fermer(true);
        }
        if let Some(h) = f.hwnd {
            verrou(&PRISES).retain(|x| *x != h);
            // egui détruit la fenêtre à la fin de l'image : les créations suivantes attendent.
            verrou(&DESTRUCTIONS).retiree(h, std::time::Instant::now());
        }
        let place = f.place();
        f.contenu.fermee(place);
    }
}

/// Le faux moteur de fenêtres des essais (partagé : egui n'en garde qu'un pour tout le
/// programme).
#[cfg(test)]
pub(crate) mod essais {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        /// Les fenêtres que le faux moteur a dessinées à la dernière image.
        pub(crate) static DESSINEES: RefCell<Vec<ViewportId>> = const { RefCell::new(Vec::new()) };
        /// Les fenêtres dont la croix est cliquée à la prochaine image.
        pub(crate) static CROIX: RefCell<Vec<ViewportId>> = const { RefCell::new(Vec::new()) };
        /// Le faux moteur rate la création des fenêtres (il rend la main sans dessiner, comme
        /// eframe quand Windows refuse la fenêtre ou son contexte OpenGL).
        pub(crate) static CREATION_RATEE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        /// Les ordres que les fenêtres ont donnés (taille, titre…), depuis le début.
        pub(crate) static ORDRES: RefCell<Vec<(ViewportId, ViewportCommand)>> = const { RefCell::new(Vec::new()) };
    }

    /// Un faux moteur de fenêtres : egui croit avoir de vraies fenêtres, rien ne s'ouvre.
    pub(crate) fn contexte() -> egui::Context {
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(false);
        egui::Context::set_immediate_viewport_renderer(|ctx, mut vp| {
            if CREATION_RATEE.with(std::cell::Cell::get) {
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
    pub(crate) fn image(ctx: &egui::Context, f: &AutresFenetres) -> (Vec<ViewportId>, Vec<ViewportId>) {
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
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::essais::{CREATION_RATEE, CROIX, contexte, image};
    use super::*;

    #[derive(Clone, Default)]
    struct Trace(Arc<Mutex<Vec<String>>>);

    struct Essai {
        nom: &'static str,
        trace: Trace,
        fermer_a: Option<u32>,
    }

    impl Contenu for Essai {
        fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
            ui.label(self.nom);
            self.trace.0.lock().unwrap().push(format!(
                "{} image {} montrée {}",
                self.nom, cadre.image, cadre.montree
            ));
            if Some(cadre.image) == self.fermer_a {
                cadre.fermer();
            }
        }

        fn fermee(&mut self, _place: Option<Place>) {
            self.trace.0.lock().unwrap().push(format!("{} fermée", self.nom));
        }
    }

    /// Le défaut corrigé : eframe ne peut pas créer la fenêtre (Windows refait l'affichage) et
    /// ne dessine rien ; egui paniquait (« the user callback was never called ») et la panique
    /// emportait la boucle des fenêtres. Maintenant : aucune panique, la fenêtre reste et
    /// réessaie ; dessinée dès que la création réussit ; après trois échecs de suite, fermée.
    #[test]
    fn creation_ratee_sans_panique_puis_nouvel_essai() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        let trace = Trace::default();
        f.ouvrir(
            "p",
            Ouverture::new("P", [200.0, 100.0]),
            Essai {
                nom: "p",
                trace: trace.clone(),
                fermer_a: None,
            },
        );
        CREATION_RATEE.with(|c| c.set(true));
        let (dessinees, _) = image(&ctx, &f);
        assert!(dessinees.is_empty());
        assert!(
            f.ouverte("p"),
            "une création ratée : la fenêtre attend un nouvel essai"
        );
        // Elle attend ~500 ms : les images qui suivent ne refont pas l'essai (ni ne le
        // comptent comme un échec de plus), et la fenêtre principale est réveillée après
        // l'attente, pas tout de suite.
        let attente = f.prochaine_image().expect("un nouvel essai est à faire");
        assert!(attente > std::time::Duration::from_millis(100), "{attente:?}");
        assert!(attente <= NOUVEL_ESSAI, "{attente:?}");
        for _ in 0..5 {
            let _ = image(&ctx, &f);
        }
        assert!(f.ouverte("p"), "pas d'essai avant la fin de l'attente");
        // La création réussit à l'essai suivant : dessinée, le compte repart.
        f.sans_attente();
        CREATION_RATEE.with(|c| c.set(false));
        let (dessinees, _) = image(&ctx, &f);
        assert_eq!(dessinees, vec![id_de("p")]);
        assert!(trace.0.lock().unwrap().iter().any(|l| l.starts_with("p image")));
        // Trois échecs de suite : fermée (son contenu l'apprend), la boucle continue.
        CREATION_RATEE.with(|c| c.set(true));
        for _ in 0..CREATIONS_MAX {
            f.sans_attente();
            let _ = image(&ctx, &f);
        }
        CREATION_RATEE.with(|c| c.set(false));
        assert!(!f.ouverte("p"), "abandonnée après {CREATIONS_MAX} échecs");
        assert!(trace.0.lock().unwrap().iter().any(|l| l == "p fermée"));
    }

    /// L'attente avant la création d'une fenêtre : après un changement d'affichage, jusqu'à la
    /// fin de la reprise ; après une création ratée, jusqu'au nouvel essai ; le plus long des
    /// deux ; rien sinon (dessiner tout de suite).
    #[test]
    fn attendre_ou_dessiner() {
        use std::time::Duration as D;
        let ms = D::from_millis;
        // Aucun changement d'affichage, aucun échec : dessiner.
        assert_eq!(attente_avant_creation(None, None, None), None);
        // Un changement d'affichage tout juste arrivé : attendre la reprise entière.
        assert_eq!(attente_avant_creation(Some(D::ZERO), None, None), Some(REPRISE));
        // À mi-chemin : attendre le reste.
        assert_eq!(attente_avant_creation(Some(ms(400)), None, None), Some(ms(600)));
        // Reprise finie (pile, ou bien après) : dessiner.
        assert_eq!(attente_avant_creation(Some(REPRISE), None, None), None);
        assert_eq!(attente_avant_creation(Some(D::from_secs(30)), None, None), None);
        // Une création ratée attend son nouvel essai ; avec un changement d'affichage : le plus
        // long des deux.
        assert_eq!(attente_avant_creation(None, Some(ms(500)), None), Some(ms(500)));
        assert_eq!(
            attente_avant_creation(Some(ms(900)), Some(ms(500)), None),
            Some(ms(500))
        );
        assert_eq!(
            attente_avant_creation(Some(ms(100)), Some(ms(500)), None),
            Some(ms(900))
        );
        // Un reste nul n'est pas une attente.
        assert_eq!(attente_avant_creation(None, Some(D::ZERO), None), None);
        // Jamais plus d'une seconde d'attente pour un changement d'affichage.
        assert!(REPRISE <= D::from_secs(1));
    }

    /// La création après destruction : rien ne se crée tant qu'une fenêtre retirée existe,
    /// ni pendant 100 ms après la dernière destruction ; une fenêtre qui ne disparaît pas
    /// n'attend pas plus de 2 s ; une destruction constatée tard date de son retrait.
    #[test]
    fn creation_apres_destruction() {
        use std::time::{Duration as D, Instant};
        let t0 = Instant::now();
        let ms = D::from_millis;
        // La décision pure.
        assert_eq!(attente_apres_destruction(false, None), None);
        assert_eq!(attente_apres_destruction(true, None), Some(VERIFICATION));
        assert_eq!(attente_apres_destruction(false, Some(ms(30))), Some(ms(70)));
        assert_eq!(attente_apres_destruction(false, Some(APRES_DESTRUCTION)), None);
        assert_eq!(attente_apres_destruction(false, Some(D::from_secs(9))), None);

        // Rien de retiré : créer.
        let mut d = Destructions::default();
        assert_eq!(d.attente(t0, |_| true), None);

        // Deux fenêtres retirées, encore là : attendre la vérification suivante.
        d.retiree(10, t0);
        d.retiree(11, t0);
        assert_eq!(d.attente(t0 + ms(5), |_| true), Some(VERIFICATION));
        // L'une détruite, l'autre encore là : toujours attendre.
        assert_eq!(d.attente(t0 + ms(20), |h| h == 11), Some(VERIFICATION));
        // Les deux détruites à t0 + 30 ms : encore 100 ms de calme.
        assert_eq!(d.attente(t0 + ms(30), |_| false), Some(APRES_DESTRUCTION));
        assert_eq!(d.attente(t0 + ms(80), |_| false), Some(ms(50)));
        // Le calme passé : créer.
        assert_eq!(d.attente(t0 + ms(130), |_| false), None);

        // Une fenêtre qui ne disparaît jamais n'arrête pas la création pour toujours.
        let mut d = Destructions::default();
        d.retiree(20, t0);
        assert_eq!(d.attente(t0 + ms(500), |_| true), Some(VERIFICATION));
        assert_eq!(d.attente(t0 + DESTRUCTION_MAX + ms(1), |_| true), None);

        // Détruite il y a longtemps (personne ne regardait) : pas de calme de plus.
        let mut d = Destructions::default();
        d.retiree(30, t0);
        assert_eq!(d.attente(t0 + D::from_secs(1), |_| false), None);
    }

    /// Une panique du contenu lui-même (son dessin a commencé) n'est pas prise pour une
    /// création ratée : elle suit son cours (la boucle la rattrape plus haut).
    #[test]
    fn panique_du_contenu_suit_son_cours() {
        struct Panique;
        impl Contenu for Panique {
            fn dessiner(&mut self, _: &mut egui::Ui, _: &mut Cadre<'_>) {
                panic!("dessin (essai)");
            }
        }
        let ctx = contexte();
        let f = AutresFenetres::default();
        f.ouvrir("q", Ouverture::new("Q", [200.0, 100.0]), Panique);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| image(&ctx, &f)));
        assert!(r.is_err());
    }

    /// Un contenu qui refuse de se fermer tant que son enregistrement échoue.
    struct Refuse {
        trace: Trace,
        enregistre: Arc<std::sync::atomic::AtomicBool>,
    }

    impl Contenu for Refuse {
        fn dessiner(&mut self, ui: &mut egui::Ui, _: &mut Cadre<'_>) {
            ui.label("refuse");
        }

        fn peut_fermer(&mut self, imposee: bool) -> bool {
            let ok = self.enregistre.load(std::sync::atomic::Ordering::Relaxed);
            self.trace
                .0
                .lock()
                .unwrap()
                .push(format!("peut_fermer imposée {imposee} → {ok}"));
            ok
        }

        fn fermee(&mut self, _place: Option<Place>) {
            self.trace.0.lock().unwrap().push("fermée".into());
        }
    }

    /// La croix passe par le crochet : refusée, la fenêtre reste ouverte (et l'ordre
    /// d'annuler la fermeture part) ; acceptée, elle se ferme, sans second appel. Fermée par
    /// le logiciel, le crochet est appelé quand même (imposée), et son refus est ignoré.
    #[test]
    fn crochet_de_fermeture_avec_refus() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        let trace = Trace::default();
        let enregistre = Arc::new(std::sync::atomic::AtomicBool::new(false));
        f.ouvrir(
            "p",
            Ouverture::new("P", [200.0, 100.0]),
            Refuse {
                trace: trace.clone(),
                enregistre: enregistre.clone(),
            },
        );
        let _ = image(&ctx, &f);
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().push(id_de("p")));
        let _ = image(&ctx, &f);
        assert!(f.ouverte("p"), "refusée : reste ouverte");
        assert!(
            super::essais::ORDRES.with(|o| o
                .borrow()
                .iter()
                .any(|(i, c)| *i == id_de("p") && matches!(c, ViewportCommand::CancelClose))),
            "l'ordre d'annuler la fermeture part"
        );
        enregistre.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = image(&ctx, &f);
        assert!(f.cles().is_empty(), "acceptée : fermée");
        CROIX.with(|c| c.borrow_mut().clear());
        assert_eq!(
            trace.0.lock().unwrap().clone(),
            vec![
                "peut_fermer imposée false → false",
                "peut_fermer imposée false → true",
                "fermée"
            ]
        );
        // Imposée : le contenu est consulté, sa réponse ne retient rien.
        enregistre.store(false, std::sync::atomic::Ordering::Relaxed);
        let trace = Trace::default();
        f.ouvrir(
            "q",
            Ouverture::new("Q", [200.0, 100.0]),
            Refuse {
                trace: trace.clone(),
                enregistre,
            },
        );
        let _ = image(&ctx, &f);
        f.tout_fermer();
        let _ = image(&ctx, &f);
        assert!(f.cles().is_empty());
        assert_eq!(
            trace.0.lock().unwrap().clone(),
            vec!["peut_fermer imposée true → false", "fermée"]
        );
    }

    /// Un contenu pas prêt garde sa fenêtre cachée, même après ses deux images ; elle se montre
    /// à l'image qui suit le moment où il l'est.
    #[test]
    fn un_contenu_pas_pret_garde_sa_fenetre_cachee() {
        struct Attend(Arc<std::sync::atomic::AtomicBool>, Trace);
        impl Contenu for Attend {
            fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
                ui.label("attend");
                self.1
                    .0
                    .lock()
                    .unwrap()
                    .push(format!("image {} montrée {}", cadre.image, cadre.montree));
            }
            fn prete(&self) -> bool {
                self.0.load(std::sync::atomic::Ordering::Acquire)
            }
        }
        let ctx = contexte();
        let f = AutresFenetres::default();
        f.lier(Some(ctx.clone()));
        let (pret, trace) = (
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            Trace::default(),
        );
        f.ouvrir(
            "carte",
            Ouverture::new("Carte", [300.0, 200.0]).sans_focus(),
            Attend(pret.clone(), trace.clone()),
        );
        for _ in 0..4 {
            let _ = image(&ctx, &f);
        }
        assert!(f.a_faire(), "pas prête : encore cachée");
        pret.store(true, std::sync::atomic::Ordering::Release);
        let _ = image(&ctx, &f);
        assert!(!f.a_faire(), "prête : montrée");
        let t = trace.0.lock().unwrap().clone();
        assert_eq!(t[3], "image 4 montrée false", "{t:?}");
        assert_eq!(t[4], "image 5 montrée true", "{t:?}");
    }

    /// Une deuxième fenêtre s'ouvre puis se ferme dans la même boucle, sans en créer une
    /// autre ; elle est dessinée cachée deux fois avant d'être montrée.
    #[test]
    fn deuxieme_fenetre_ouverte_puis_fermee_dans_la_meme_boucle() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        f.lier(Some(ctx.clone()));
        let trace = Trace::default();
        let id = id_de("lecture");
        assert!(f.ouvrir(
            "lecture",
            Ouverture::new("Lecture", [300.0, 200.0]),
            Essai {
                nom: "lecture",
                trace: trace.clone(),
                fermer_a: None,
            },
        ));
        assert!(
            f.a_faire(),
            "à ouvrir : une image même la fenêtre principale cachée"
        );
        let (d, g) = image(&ctx, &f);
        assert_eq!(d, vec![id]);
        assert_eq!(g, vec![id]);
        let _ = image(&ctx, &f);
        assert!(!f.a_faire(), "montrée : plus rien à faire");
        assert!(f.ouverte("lecture"));
        // Une deuxième ouverture de la même clé ne la recrée pas : elle passe devant.
        assert!(!f.ouvrir(
            "lecture",
            Ouverture::new("Lecture", [300.0, 200.0]),
            Essai {
                nom: "autre",
                trace: trace.clone(),
                fermer_a: None,
            },
        ));
        f.fermer("lecture");
        assert!(!f.ouverte("lecture"));
        assert!(f.a_faire());
        let (d, g) = image(&ctx, &f);
        assert!(d.is_empty(), "{d:?}");
        assert!(g.is_empty(), "egui détruit la fenêtre : {g:?}");
        assert!(f.cles().is_empty());
        let t = trace.0.lock().unwrap().clone();
        assert_eq!(
            t,
            vec![
                "lecture image 1 montrée false",
                "lecture image 2 montrée true",
                "lecture fermée",
            ]
        );
    }

    /// Plusieurs fenêtres à la fois ; la croix ou le contenu en ferment une, les autres
    /// restent.
    #[test]
    fn plusieurs_fenetres_et_croix() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        let trace = Trace::default();
        for (cle, fermer_a) in [("a", None), ("b", Some(3)), ("c", None)] {
            f.ouvrir(
                cle,
                Ouverture::new(cle, [200.0, 100.0]),
                Essai {
                    nom: cle,
                    trace: trace.clone(),
                    fermer_a,
                },
            );
        }
        let (d, _) = image(&ctx, &f);
        assert_eq!(d.len(), 3);
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().push(id_de("a")));
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().clear());
        let (d, g) = image(&ctx, &f);
        assert_eq!(f.cles(), vec!["c".to_string()]);
        assert_eq!(d, vec![id_de("c")]);
        assert_eq!(g, vec![id_de("c")]);
        let t = trace.0.lock().unwrap().clone();
        assert!(t.contains(&"a fermée".to_string()), "{t:?}");
        assert!(t.contains(&"b fermée".to_string()), "{t:?}");
        f.tout_fermer();
        let (_, g) = image(&ctx, &f);
        assert!(g.is_empty());
    }

    /// Un contenu peut ouvrir une autre fenêtre pendant son dessin (aucun verrou tenu).
    #[test]
    fn un_contenu_ouvre_une_autre_fenetre() {
        struct Ouvre(AutresFenetres);
        impl Contenu for Ouvre {
            fn dessiner(&mut self, _: &mut egui::Ui, _: &mut Cadre<'_>) {
                self.0.ouvrir(
                    "fille",
                    Ouverture::new("Fille", [100.0, 100.0]),
                    Essai {
                        nom: "fille",
                        trace: Trace::default(),
                        fermer_a: None,
                    },
                );
            }
        }
        let ctx = contexte();
        let f = AutresFenetres::default();
        f.ouvrir("mere", Ouverture::new("Mère", [100.0, 100.0]), Ouvre(f.clone()));
        let _ = image(&ctx, &f);
        let (d, _) = image(&ctx, &f);
        assert_eq!(d.len(), 2);
    }
}
