//! Le fil des fenêtres, sans Windows (tout est testé) : les demandes que l'icône lui envoie,
//! ce que partagent les fils, et sa boucle.
//!
//! Un fil à part tient **la seule boucle de fenêtres du programme** (winit n'en accepte
//! qu'une). Sa fenêtre principale est celle du volet et du petit menu, préparée une fois, au
//! repos, et gardée cachée jusqu'à la fin : un clic la met à jour et la montre, sans rien
//! reconstruire (ni fenêtre, ni OpenGL, ni polices ; règles : interface Windows 11, point 10,
//! « la première ouverture aussi rapide que les suivantes »). Les Paramètres et les autres
//! fenêtres du logiciel s'ouvrent et se ferment dans cette même boucle
//! ([`crate::autres_fenetres`]). Sans elle (polices ou OpenGL absents), le menu gris de
//! Windows sert de secours.
//!
//! **Une fenêtre perdue est recréée.** Une panique pendant la boucle des fenêtres (par
//! exemple eframe qui ne peut plus rendre son contexte OpenGL courant après une mise en
//! veille des écrans) détruit la fenêtre du volet. Sans garde, elle tuait le fil des
//! fenêtres en silence : plus rien ne s'ouvrait au clic. Elle est maintenant attrapée, notée
//! au journal avec sa cause, et la fenêtre est recréée aussitôt ; le clic en cours est refait
//! après la recréation. Des pertes en rafale (plus de [`RECREATIONS_EN_RAFALE`] en
//! [`RAFALE`]) donnent le menu de secours pour ce clic, et la fenêtre n'est recréée qu'au clic
//! suivant (jamais de boucle folle). Si le fil s'arrête quand même, [`GardeDuFil`] fait
//! passer l'icône au menu de secours.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::{Receiver, SendError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::autres_fenetres::AutresFenetres;
use crate::place::Rect;
use crate::theme::Surface;

/// Ce que l'icône (fil principal) demande au fil des fenêtres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Demande {
    /// Ouvrir le volet contre l'icône (rectangle de l'icône, en pixels) : clic gauche,
    /// Entrée ou Espace. `clic` : quand l'icône a été cliquée (le temps d'ouverture).
    Volet {
        icone: Rect,
        clic: Instant,
    },
    /// Ouvrir le petit menu au point `ancre` (pixels) : clic droit, touche menu, Maj+F10.
    Menu {
        icone: Rect,
        ancre: [i32; 2],
        clic: Instant,
    },
    /// Les Paramètres, à cette page (sa clé) ou à la première : une fenêtre de plus de la
    /// boucle, à côté de la fenêtre du volet.
    Parametres(Option<String>),
    Quitter,
}

/// Ce qui est ouvert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ouvert {
    #[default]
    Rien,
    /// Le volet (clic gauche) ou le petit menu (clic droit) est montré.
    Surface(Surface),
    Parametres,
}

/// Ce que partagent le fil principal (l'icône), le fil des fenêtres et le logiciel.
#[derive(Default)]
pub struct Liens {
    /// La fenêtre cachée du fil principal (0 tant qu'elle n'existe pas).
    pub fenetre: AtomicIsize,
    /// Le menu moderne n'a pas pu s'ouvrir : le menu de secours prend sa place.
    pub egui_impossible: AtomicBool,
    /// Le contexte egui de la fenêtre ouverte (pour la redessiner).
    pub contexte: Mutex<Option<egui::Context>>,
    ouvert: Mutex<Ouvert>,
    /// Le clic (volet ou petit menu) en train d'être ouvert : refait si la fenêtre est perdue
    /// avant de s'être montrée.
    en_cours: Mutex<Option<Demande>>,
    /// Quand le volet ou le petit menu s'est fermé, et lequel (un clic sur l'icône qui vient
    /// de le fermer, par la perte du focus, ne le rouvre pas).
    pub ferme_a: Mutex<Option<(Instant, Surface)>>,
    /// La fenêtre du volet et du petit menu, prête et cachée (0 : pas prête). Le fil
    /// principal la montre lui-même au clic, encore transparente, pour qu'elle se dessine
    /// tout de suite.
    pub surface_prete: AtomicIsize,
    /// Le logiciel se ferme : le fil des fenêtres ne prépare plus rien.
    pub quitter: AtomicBool,
    /// Le volet ou le petit menu montré doit se fermer.
    pub fermer: AtomicBool,
    /// Les Paramètres ouverts doivent montrer cette page.
    pub page_voulue: Arc<Mutex<Option<String>>>,
    /// Les Paramètres et les autres fenêtres du logiciel, dans la boucle des fenêtres.
    pub autres: AutresFenetres,
}

impl Liens {
    /// Ce qui est ouvert : le volet ou le petit menu s'il est montré ; sinon les Paramètres
    /// s'ils sont ouverts.
    pub fn ouvert(&self) -> Ouvert {
        let o = self.ouvert.lock().map(|o| *o).unwrap_or_default();
        if o == Ouvert::Rien && self.autres.ouverte(crate::parametres::CLE_FENETRE) {
            Ouvert::Parametres
        } else {
            o
        }
    }

    pub fn mettre(&self, o: Ouvert) {
        if let Ok(mut v) = self.ouvert.lock() {
            *v = o;
        }
    }

    /// Le clic en train d'être ouvert (`None` : montré, ou abandonné).
    pub fn poser_en_cours(&self, d: Option<Demande>) {
        if let Ok(mut v) = self.en_cours.lock() {
            *v = d;
        }
    }

    /// Le clic qui était en train d'être ouvert (retiré).
    pub fn prendre_en_cours(&self) -> Option<Demande> {
        self.en_cours.lock().ok().and_then(|mut v| v.take())
    }

    /// La fenêtre du volet n'existe plus : plus rien n'est montré ni prêt, aucune fenêtre ne
    /// lit les demandes.
    fn oublier_la_surface(&self, demandes: &Reception) {
        self.surface_prete.store(0, Ordering::Release);
        self.mettre(Ouvert::Rien);
        self.fermer.store(false, Ordering::Release);
        if let Ok(mut c) = self.contexte.lock() {
            *c = None;
        }
        self.autres.lier(None);
        self.autres.racine(0);
        demandes.lecteur(None);
    }

    /// `surface` s'est fermée il y a moins d'[`ANTI_REBOND`] (le clic sur l'icône qui l'a
    /// fermée en lui prenant le focus ne doit pas la rouvrir aussitôt).
    pub fn juste_fermee(&self, surface: Surface) -> bool {
        self.ferme_a
            .lock()
            .ok()
            .and_then(|f| *f)
            .is_some_and(|(t, s)| s == surface && t.elapsed() < ANTI_REBOND)
    }

    /// Redessine les fenêtres ouvertes : à appeler quand l'état du logiciel change (le volet,
    /// les Paramètres et les autres fenêtres le montrent aussitôt : elles sont dessinées avec
    /// la fenêtre principale).
    pub fn redessiner(&self) {
        if let Ok(c) = self.contexte.lock()
            && let Some(c) = c.as_ref()
        {
            c.request_repaint();
        }
    }

    pub fn fermer_la_fenetre(&self) {
        self.fermer.store(true, Ordering::Release);
        self.redessiner();
    }

    /// Les Paramètres ouverts passent devant.
    pub fn mettre_devant(&self) {
        self.autres.devant(crate::parametres::CLE_FENETRE);
    }

    /// Montre les Paramètres sur cette page : ouverts, ils passent devant et changent de
    /// page ; sinon, ils s'ouvrent.
    pub fn montrer_page(&self, demandes: &Demandes, page: Option<String>) {
        if self.ouvert() == Ouvert::Parametres {
            if page.is_some()
                && let Ok(mut p) = self.page_voulue.lock()
            {
                *p = page;
            }
            self.mettre_devant();
        } else {
            let _ = demandes.send(Demande::Parametres(page));
        }
    }

    /// Fin du programme : fermer la fenêtre ouverte, quelle qu'elle soit, et ne plus rien
    /// préparer.
    pub fn tout_fermer(&self) {
        self.quitter.store(true, Ordering::Release);
        self.surface_prete.store(0, Ordering::Release);
        self.fermer.store(true, Ordering::Release);
        if let Ok(c) = self.contexte.lock()
            && let Some(c) = c.as_ref()
        {
            c.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

/// Le réveil du fil des fenêtres entre deux fenêtres. Sous Windows, il répond en attendant
/// aux messages que Windows envoie aux fenêtres du fil (sinon Windows le croit bloqué).
#[cfg(windows)]
pub type Reveil = crate::systeme::Reveil;

/// Le réveil du fil des fenêtres entre deux fenêtres.
#[cfg(not(windows))]
#[derive(Default)]
pub struct Reveil {
    sonne: Mutex<bool>,
    cv: std::sync::Condvar,
}

#[cfg(not(windows))]
impl Reveil {
    pub fn sonner(&self) {
        if let Ok(mut s) = self.sonne.lock() {
            *s = true;
        }
        self.cv.notify_all();
    }

    pub fn attendre(&self, ms: u32) {
        if let Ok(s) = self.sonne.lock()
            && let Ok((mut s, _)) = self
                .cv
                .wait_timeout_while(s, Duration::from_millis(u64::from(ms)), |s| !*s)
        {
            *s = false;
        }
    }
}

/// Le contexte egui de la fenêtre qui lit les demandes pendant qu'elle est ouverte (la
/// fenêtre du volet et du petit menu, même cachée) : une demande la réveille.
type Lecteur = Arc<Mutex<Option<egui::Context>>>;

/// L'envoi des demandes au fil des fenêtres : chaque demande sonne aussi son réveil, et
/// réveille la fenêtre qui lit les demandes, s'il y en a une.
#[derive(Clone)]
pub struct Demandes {
    envoi: Sender<Demande>,
    reveil: Arc<Reveil>,
    lecteur: Lecteur,
}

impl Demandes {
    pub fn send(&self, d: Demande) -> Result<(), SendError<Demande>> {
        let r = self.envoi.send(d);
        self.reveil.sonner();
        if let Ok(l) = self.lecteur.lock()
            && let Some(c) = l.as_ref()
        {
            c.request_repaint();
        }
        r
    }
}

/// La réception des demandes, côté fil des fenêtres.
pub struct Reception {
    reception: Receiver<Demande>,
    reveil: Arc<Reveil>,
    lecteur: Lecteur,
}

impl Reception {
    /// La prochaine demande déjà arrivée, sans attendre. `Err(())` : plus personne ne peut
    /// en envoyer.
    #[allow(clippy::result_unit_err)]
    pub fn prendre(&self) -> Result<Option<Demande>, ()> {
        match self.reception.try_recv() {
            Ok(d) => Ok(Some(d)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(()),
        }
    }

    /// La fenêtre qui lit les demandes (`None` : plus aucune).
    pub fn lecteur(&self, c: Option<egui::Context>) {
        if let Ok(mut l) = self.lecteur.lock() {
            *l = c;
        }
    }

    /// La prochaine demande, en l'attendant. `None` quand plus personne ne peut en envoyer.
    pub fn recevoir(&self) -> Option<Demande> {
        loop {
            match self.reception.try_recv() {
                Ok(d) => return Some(d),
                Err(TryRecvError::Disconnected) => return None,
                Err(TryRecvError::Empty) => {
                    crate::sonde::battement();
                    self.reveil.attendre(1_000);
                }
            }
        }
    }
}

/// Le canal des demandes vers le fil des fenêtres.
pub fn canal() -> (Demandes, Reception) {
    let (envoi, reception) = std::sync::mpsc::channel();
    let reveil = Arc::new(Reveil::default());
    let lecteur = Lecteur::default();
    (
        Demandes {
            envoi,
            reveil: reveil.clone(),
            lecteur: lecteur.clone(),
        },
        Reception {
            reception,
            reveil,
            lecteur,
        },
    )
}

/// Ce que le fil des fenêtres ouvre : les vraies fenêtres (egui), ou un faux dans les tests.
pub trait Ouvreur {
    /// Prépare au repos la fenêtre du volet et du petit menu, cachée, et la garde jusqu'à la
    /// fin : chaque demande de volet ou de petit menu la montre sans la reconstruire ; une
    /// demande de Paramètres les ouvre dans la même boucle. Rend `Demande::Quitter`, ou
    /// `None` (plus personne n'envoie, le logiciel se ferme, ou la fenêtre a été fermée).
    /// `premiere` : une demande arrivée avant la préparation, faite dès qu'elle est finie. En
    /// erreur, rend aussi la demande de volet ou de menu en cours, s'il y en a une (pour le
    /// menu de secours).
    fn surface(
        &mut self,
        demandes: &Reception,
        premiere: Option<Demande>,
    ) -> Result<Option<Demande>, (String, Option<Demande>)>;
    /// Secours, quand la fenêtre du volet n'a pas pu être préparée : les Paramètres seuls, à
    /// cette page ; rend la main à leur fermeture.
    fn parametres(&mut self, page: Option<String>) -> Result<(), String>;
    /// Le menu gris de Windows, à la place du volet ou du petit menu.
    fn secours(&mut self, d: &Demande);
    /// Une ligne pour le journal du logiciel.
    fn noter(&mut self, ligne: String);
}

/// Au-delà de ce nombre de pertes de la fenêtre du volet en [`RAFALE`], elle n'est plus
/// recréée tout de suite : le clic reçoit le menu de secours, et elle est recréée au clic
/// suivant.
pub const RECREATIONS_EN_RAFALE: usize = 3;
/// La fenêtre de temps des pertes en rafale.
pub const RAFALE: Duration = Duration::from_secs(60);

/// Les pertes récentes de la fenêtre du volet.
#[derive(Default)]
struct Pertes(Vec<Instant>);

impl Pertes {
    /// Compte une perte ; rend le nombre de pertes dans la dernière [`RAFALE`].
    fn compter(&mut self, maintenant: Instant) -> usize {
        self.0
            .retain(|t| maintenant.saturating_duration_since(*t) < RAFALE);
        self.0.push(maintenant);
        self.0.len()
    }
}

/// Le texte d'une panique attrapée : son message, son lieu et la dernière étape du fil
/// (de quoi trouver la cause dans le journal).
pub fn cause_de_panique(p: &(dyn std::any::Any + Send)) -> String {
    let message = p
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| p.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "sans message".into());
    let mut message: String = message.chars().take(300).collect();
    message = message.replace(['\r', '\n'], " ");
    let mut t = format!("panique : {message}");
    if let Some(lieu) = crate::sonde::lieu_de_la_derniere_panique() {
        t.push_str(&format!(", à {lieu}"));
    }
    if let Some(e) = crate::sonde::etape_du_fil() {
        t.push_str(&format!(", étape « {e} »"));
    }
    t
}

fn nom_du_clic(d: &Demande) -> &'static str {
    match d {
        Demande::Volet { .. } => "le volet",
        Demande::Menu { .. } => "le petit menu",
        Demande::Parametres(_) => "les Paramètres",
        Demande::Quitter => "la fermeture",
    }
}

/// La boucle du fil des fenêtres : la surface (volet et petit menu) préparée au repos, qui
/// ouvre aussi les Paramètres et les autres fenêtres, jusqu'à `Demande::Quitter`. Elle est
/// refaite si elle a été fermée (par Windows) ou perdue (une panique, attrapée ici : voir le
/// haut du module).
pub fn boucle_des_fenetres(liens: &Liens, ouvreur: &mut impl Ouvreur, demandes: &Reception) {
    let mut pertes = Pertes::default();
    // Le clic à refaire après la recréation de la fenêtre perdue.
    let mut reprise: Option<Demande> = None;
    // Pertes en rafale : la fenêtre n'est recréée qu'au prochain clic.
    let mut en_pause = false;
    // Une première préparation a été tentée (un échec ensuite est passager).
    let mut deja_preparee = false;
    loop {
        crate::sonde::battement();
        crate::sonde::etape("boucle des fenêtres");
        if liens.quitter.load(Ordering::Acquire) {
            break;
        }
        // D'abord ce qui attend déjà (premier lancement : les Paramètres) ; sinon, la
        // surface préparée au repos, qui garde la main jusqu'à une autre demande.
        let deja = match reprise.take() {
            Some(d) => Some(d),
            None => match demandes.prendre() {
                Ok(d) => d,
                Err(()) => break,
            },
        };
        let impossible = liens.egui_impossible.load(Ordering::Acquire);
        let deja = match deja {
            None if en_pause && !impossible => match demandes.recevoir() {
                Some(d) => Some(d),
                None => break,
            },
            autre => autre,
        };
        let d = match deja {
            None if impossible => match demandes.recevoir() {
                Some(d) => d,
                None => break,
            },
            Some(d) if impossible || d == Demande::Quitter => d,
            // Rien (ou une demande arrivée avant la préparation, qu'elle fera) : la surface.
            premiere => {
                liens.poser_en_cours(premiere.clone());
                let r = catch_unwind(AssertUnwindSafe(|| ouvreur.surface(demandes, premiere)));
                let premiere_preparation = !deja_preparee;
                deja_preparee = true;
                let en_cours = liens.prendre_en_cours();
                liens.surface_prete.store(0, Ordering::Release);
                liens.mettre(Ouvert::Rien);
                if let Ok(mut c) = liens.contexte.lock() {
                    *c = None;
                }
                let r = match r {
                    Ok(r) => r,
                    Err(p) => {
                        let cause = cause_de_panique(&*p);
                        liens.oublier_la_surface(demandes);
                        if liens.quitter.load(Ordering::Acquire) {
                            break;
                        }
                        let n = pertes.compter(Instant::now());
                        let refaire = en_cours
                            .as_ref()
                            .map(|d| format!(" ; {} demandé est rouvert", nom_du_clic(d)))
                            .unwrap_or_default();
                        if n <= RECREATIONS_EN_RAFALE {
                            ouvreur.noter(format!(
                                "Fenêtre du volet et du petit menu perdue ({cause}) : recréée{refaire}."
                            ));
                            reprise = en_cours;
                            en_pause = false;
                        } else {
                            ouvreur.noter(format!(
                                "Fenêtre du volet et du petit menu perdue ({cause}) : {n} fois en {} s ; menu de secours pour ce clic, recréée au clic suivant.",
                                RAFALE.as_secs()
                            ));
                            if let Some(d) = en_cours {
                                ouvreur.secours(&d);
                            }
                            en_pause = true;
                        }
                        continue;
                    }
                };
                en_pause = false;
                match r {
                    Ok(Some(d)) => d,
                    Ok(None) => {
                        if liens.quitter.load(Ordering::Acquire) {
                            break;
                        }
                        let n = pertes.compter(Instant::now());
                        en_pause = n > RECREATIONS_EN_RAFALE;
                        ouvreur.noter(if en_pause {
                            format!(
                                "Fenêtre du volet et du petit menu fermée par Windows : {n} fois en {} s ; recréée au clic suivant.",
                                RAFALE.as_secs()
                            )
                        } else {
                            "Fenêtre du volet et du petit menu fermée par Windows : recréée.".into()
                        });
                        continue;
                    }
                    Err((e, en_cours)) if premiere_preparation => {
                        // Jamais préparée (ni OpenGL ni fenêtre possibles) : le menu de
                        // secours pour de bon.
                        ouvreur.noter(format!(
                            "Volet impossible à préparer ({e}) ; menu de secours de Windows."
                        ));
                        liens.egui_impossible.store(true, Ordering::Release);
                        if let Some(d) = en_cours {
                            ouvreur.secours(&d);
                        }
                        continue;
                    }
                    Err((e, en_cours)) => {
                        // Elle a déjà marché : un échec passager (écrans en veille, carte
                        // graphique qui se relance). Le menu de secours pour ce clic, un
                        // nouvel essai au clic suivant.
                        ouvreur.noter(format!(
                            "Volet impossible à recréer ({e}) ; menu de secours pour ce clic, nouvel essai au clic suivant."
                        ));
                        if let Some(d) = en_cours {
                            ouvreur.secours(&d);
                        }
                        en_pause = true;
                        continue;
                    }
                }
            }
        };
        let resultat = match d {
            Demande::Quitter => break,
            Demande::Volet { .. } | Demande::Menu { .. } => {
                // Ici seulement quand la surface est impossible : le menu de secours.
                ouvreur.secours(&d);
                Ok(())
            }
            Demande::Parametres(page) => {
                // Ici seulement quand la surface est impossible : les Paramètres seuls.
                liens.fermer.store(false, Ordering::Release);
                liens.mettre(Ouvert::Parametres);
                catch_unwind(AssertUnwindSafe(|| ouvreur.parametres(page)))
                    .unwrap_or_else(|p| Err(cause_de_panique(&*p)))
            }
        };
        liens.mettre(Ouvert::Rien);
        if let Ok(mut c) = liens.contexte.lock() {
            *c = None;
        }
        if let Err(e) = resultat {
            ouvreur.noter(format!(
                "Fenêtre impossible à ouvrir ({e}) ; menu de secours de Windows."
            ));
            liens.egui_impossible.store(true, Ordering::Release);
        }
    }
}

/// Le dernier filet du fil des fenêtres : s'il s'arrête alors que le logiciel ne se ferme pas
/// (une panique hors de la boucle), l'icône passe au menu de secours de Windows au lieu de ne
/// plus rien ouvrir, et le journal le dit. À créer au début du fil, gardée jusqu'à sa fin.
pub struct GardeDuFil {
    liens: Arc<Liens>,
    noter: Box<dyn Fn(String) + Send>,
}

impl GardeDuFil {
    pub fn new(liens: Arc<Liens>, noter: impl Fn(String) + Send + 'static) -> GardeDuFil {
        GardeDuFil {
            liens,
            noter: Box::new(noter),
        }
    }
}

impl Drop for GardeDuFil {
    fn drop(&mut self) {
        let l = &self.liens;
        l.surface_prete.store(0, Ordering::Release);
        l.mettre(Ouvert::Rien);
        if l.quitter.load(Ordering::Acquire) {
            return;
        }
        l.egui_impossible.store(true, Ordering::Release);
        let pourquoi = if std::thread::panicking() {
            "panique"
        } else {
            "fin inattendue"
        };
        (self.noter)(format!(
            "Fil des fenêtres arrêté ({pourquoi}) : l'icône ouvre le menu de secours de Windows jusqu'au prochain lancement."
        ));
    }
}

// ---------------------------------------------------------------------------------------
// Ce que la fenêtre du volet fait d'une demande, sa garde et la mesure de son ouverture

/// Après la fermeture du volet ou du petit menu, un clic sur l'icône ne le rouvre pas
/// pendant ce temps (ce clic est celui qui l'a fermé).
pub const ANTI_REBOND: Duration = Duration::from_millis(400);

/// Une ouverture plus longue que ce temps est notée au journal, avec le détail.
pub const LENTE: Duration = Duration::from_millis(150);

/// Pendant ce temps après l'ouverture, clics et touches sont ignorés : un programme qui
/// ouvre le volet ou le petit menu ne peut pas cliquer une ligne dans la foulée, ni une
/// frappe destinée à une autre fenêtre choisir quelque chose.
pub const GARDE: Duration = Duration::from_millis(250);
/// La garde se lève au plus tard après ce temps.
pub const GARDE_MAX: Duration = Duration::from_secs(1);

/// La garde des premiers instants : ce qui est cliqué ou tapé pendant [`GARDE`] après
/// l'ouverture ne choisit rien. Elle se lève ensuite dès qu'aucun bouton ni aucune touche
/// n'est tenu : un appui commencé pendant la garde ne compte pas, même relâché après. Au bout
/// de [`GARDE_MAX`], elle se lève quoi qu'il arrive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Garde {
    levee: bool,
}

impl Garde {
    pub fn levee(&self) -> bool {
        self.levee
    }

    /// Ce qui est fait de `action` à cette image : rien tant que la garde n'est pas levée.
    /// `depuis` : le temps depuis l'ouverture ; `tenu` : un bouton ou une touche est enfoncé.
    pub fn filtrer<A>(&mut self, action: Option<A>, depuis: Duration, tenu: bool) -> Option<A> {
        let permise = if self.levee { action } else { None };
        if depuis >= GARDE_MAX || (depuis >= GARDE && !tenu) {
            self.levee = true;
        }
        permise
    }
}

/// Ce que la fenêtre du volet fait d'une demande reçue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    /// La montrer avec cette surface (mise à jour, jamais reconstruite).
    Montrer(Surface),
    /// Déjà montrée avec cette surface : rien.
    Ignorer,
    /// Les Paramètres sont demandés : le volet se cache, ils s'ouvrent à côté.
    Parametres,
    /// Le logiciel se ferme : se cacher et rendre la main au fil des fenêtres.
    Sortir,
}

/// La suite d'une demande, quand `montree` est la surface montrée (ou en train de l'être).
pub fn suite(d: &Demande, montree: Option<Surface>) -> Suite {
    let voulue = match d {
        Demande::Volet { .. } => Surface::Volet,
        Demande::Menu { .. } => Surface::Menu,
        Demande::Parametres(_) => return Suite::Parametres,
        Demande::Quitter => return Suite::Sortir,
    };
    if montree == Some(voulue) {
        Suite::Ignorer
    } else {
        Suite::Montrer(voulue)
    }
}

/// Le temps d'une ouverture, par étape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mesure {
    /// Du clic à la demande lue par le fil des fenêtres.
    pub attente: Duration,
    /// De la demande lue à la fenêtre montrée (taille, place, première image).
    pub mise_en_place: Duration,
    /// Les images dessinées pendant la mise en place.
    pub images: u8,
}

impl Mesure {
    pub fn total(&self) -> Duration {
        self.attente + self.mise_en_place
    }

    /// Faut-il la noter ? La première ouverture, et toute ouverture lente.
    pub fn a_noter(&self, premiere: bool) -> bool {
        premiere || self.total() > LENTE
    }

    /// La ligne du journal.
    pub fn phrase(&self, surface: Surface, premiere: bool) -> String {
        let quoi = match surface {
            Surface::Volet => "Volet",
            Surface::Menu => "Petit menu",
        };
        let pourquoi = if premiere {
            "première ouverture"
        } else {
            "ouverture lente"
        };
        format!(
            "{quoi} ouvert en {} ms ({pourquoi}) : {} ms pour arriver au fil des fenêtres, {} ms de mise en place ({} image(s)).",
            self.total().as_millis(),
            self.attente.as_millis(),
            self.mise_en_place.as_millis(),
            self.images
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un faux fil des fenêtres : il compte les préparations de la surface et ce qu'elle
    /// montre, sans rien ouvrir.
    #[derive(Default)]
    struct Faux {
        preparations: usize,
        /// Chaque ouverture, avec le numéro de la préparation qui l'a montrée.
        ouvertures: Vec<(Surface, usize)>,
        parametres: Vec<Option<String>>,
        secours: usize,
        notes: Vec<String>,
        /// Comme la vraie fenêtre : le clic en cours est posé dans les liens jusqu'à ce qu'il
        /// soit montré.
        liens: Option<Arc<Liens>>,
        /// Tant que ce nombre n'est pas épuisé, montrer le petit menu fait paniquer la
        /// boucle (la fenêtre est perdue, comme eframe qui ne retrouve plus son contexte
        /// OpenGL après la veille des écrans).
        pertes_au_menu: usize,
    }

    impl Faux {
        fn montrer(&mut self, d: &Demande) -> bool {
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

    fn volet() -> Demande {
        Demande::Volet {
            icone: [0, 0, 16, 16],
            clic: Instant::now(),
        }
    }

    fn menu() -> Demande {
        Demande::Menu {
            icone: [0, 0, 16, 16],
            ancre: [8, 8],
            clic: Instant::now(),
        }
    }

    /// Règle : la première ouverture aussi rapide que les suivantes. La surface (volet et
    /// petit menu) est préparée une fois au repos ; deux ouvertures (et plus) ne la
    /// reconstruisent pas, ni les Paramètres, ouverts dans la même boucle. Une demande
    /// arrivée avant la préparation n'est pas perdue.
    #[test]
    fn deux_ouvertures_ne_reconstruisent_pas_le_volet() {
        let liens = Liens::default();
        let (envoi, reception) = canal();
        for d in [
            Demande::Parametres(None),
            volet(),
            volet(),
            menu(),
            volet(),
            Demande::Parametres(Some("general".into())),
            menu(),
            volet(),
            Demande::Quitter,
        ] {
            envoi.send(d).expect("demande");
        }
        let mut faux = Faux::default();
        boucle_des_fenetres(&liens, &mut faux, &reception);
        assert_eq!(faux.preparations, 1, "{:?}", faux.ouvertures);
        assert_eq!(
            faux.ouvertures,
            vec![
                (Surface::Volet, 1),
                (Surface::Volet, 1),
                (Surface::Menu, 1),
                (Surface::Volet, 1),
                (Surface::Menu, 1),
                (Surface::Volet, 1),
            ]
        );
        assert_eq!(faux.parametres, vec![None, Some("general".to_string())]);
    }

    /// Le défaut corrigé : la fenêtre du volet perdue (une panique dans la boucle des
    /// fenêtres, après une nuit d'écrans en veille) tuait le fil des fenêtres en silence, et
    /// plus aucun clic n'ouvrait rien. Maintenant elle est recréée tout de suite, le clic en
    /// cours est refait, les clics suivants marchent, et le journal dit la perte et sa cause.
    #[test]
    fn fenetre_perdue_recreee_et_le_clic_ouvre_quand_meme() {
        let liens = Arc::new(Liens::default());
        let (envoi, reception) = canal();
        for d in [volet(), menu(), volet(), menu(), Demande::Quitter] {
            envoi.send(d).expect("demande");
        }
        let mut faux = Faux {
            liens: Some(liens.clone()),
            pertes_au_menu: 1,
            ..Faux::default()
        };
        boucle_des_fenetres(&liens, &mut faux, &reception);
        assert_eq!(faux.preparations, 2, "recréée une fois");
        assert_eq!(
            faux.ouvertures,
            vec![
                (Surface::Volet, 1),
                (Surface::Menu, 2),
                (Surface::Volet, 2),
                (Surface::Menu, 2),
            ]
        );
        assert_eq!(faux.secours, 0);
        assert!(!liens.egui_impossible.load(Ordering::Acquire));
        assert_eq!(faux.notes.len(), 1, "{:?}", faux.notes);
        let note = &faux.notes[0];
        assert!(note.contains("perdue"), "{note}");
        assert!(note.contains("contexte OpenGL perdu (essai)"), "{note}");
        assert!(note.contains("recréée"), "{note}");
        assert!(note.contains("le petit menu demandé est rouvert"), "{note}");
    }

    /// Des pertes en rafale ne font jamais tourner la boucle à vide : au-delà de trois en une
    /// minute, le clic reçoit le menu de secours, la fenêtre attend le clic suivant pour être
    /// recréée, et ce clic-là ouvre le volet. Chaque perte est notée.
    #[test]
    fn pertes_en_rafale_menu_de_secours_puis_recreee_au_clic_suivant() {
        let liens = Arc::new(Liens::default());
        let (envoi, reception) = canal();
        for d in [menu(), volet(), Demande::Quitter] {
            envoi.send(d).expect("demande");
        }
        let mut faux = Faux {
            liens: Some(liens.clone()),
            pertes_au_menu: usize::MAX,
            ..Faux::default()
        };
        boucle_des_fenetres(&liens, &mut faux, &reception);
        // Le menu : perdu, puis refait trois fois (trois recréations), puis le secours.
        assert_eq!(faux.secours, 1);
        assert_eq!(faux.notes.len(), RECREATIONS_EN_RAFALE + 1, "{:?}", faux.notes);
        assert!(faux.notes[RECREATIONS_EN_RAFALE].contains("menu de secours pour ce clic"));
        // Le volet, au clic suivant : la fenêtre est recréée et l'ouvre.
        assert_eq!(faux.preparations, RECREATIONS_EN_RAFALE + 2);
        assert_eq!(faux.ouvertures, vec![(Surface::Volet, RECREATIONS_EN_RAFALE + 2)]);
        assert!(!liens.egui_impossible.load(Ordering::Acquire));
    }

    /// Une panique hors de la boucle arrête le fil des fenêtres : la garde fait passer l'icône
    /// au menu de secours (un clic ouvre toujours quelque chose) et le note. À la fermeture
    /// normale du logiciel, rien de tel.
    #[test]
    fn fil_des_fenetres_arrete_menu_de_secours() {
        let liens = Arc::new(Liens::default());
        liens
            .surface_prete
            .store(1234, std::sync::atomic::Ordering::Release);
        liens.mettre(Ouvert::Surface(Surface::Volet));
        let notes = Arc::new(Mutex::new(Vec::<String>::new()));
        let (l2, n2) = (liens.clone(), notes.clone());
        let fil = std::thread::spawn(move || {
            let _garde = GardeDuFil::new(l2, move |t| n2.lock().expect("notes").push(t));
            panic!("panique hors de la boucle (essai)");
        });
        assert!(fil.join().is_err());
        assert!(liens.egui_impossible.load(Ordering::Acquire));
        assert_eq!(liens.surface_prete.load(Ordering::Acquire), 0);
        assert_eq!(liens.ouvert(), Ouvert::Rien);
        let notes = notes.lock().expect("notes");
        assert_eq!(notes.len(), 1);
        assert!(notes[0].contains("panique"), "{}", notes[0]);

        let liens = Arc::new(Liens::default());
        liens.tout_fermer();
        drop(GardeDuFil::new(liens.clone(), |_| panic!("rien à noter")));
        assert!(!liens.egui_impossible.load(Ordering::Acquire));
    }

    /// Le logiciel se ferme : plus rien n'est préparé.
    #[test]
    fn rien_n_est_prepare_apres_quitter() {
        let liens = Liens::default();
        liens.tout_fermer();
        let (_envoi, reception) = canal();
        let mut faux = Faux::default();
        boucle_des_fenetres(&liens, &mut faux, &reception);
        assert_eq!(faux.preparations, 0);
    }

    /// Sans OpenGL ni polices : jamais de surface, le menu de secours à la place.
    #[test]
    fn sans_opengl_le_menu_de_secours() {
        let liens = Liens::default();
        liens.egui_impossible.store(true, Ordering::Release);
        let (envoi, reception) = canal();
        for d in [volet(), menu(), Demande::Parametres(None), Demande::Quitter] {
            envoi.send(d).expect("demande");
        }
        let mut faux = Faux::default();
        boucle_des_fenetres(&liens, &mut faux, &reception);
        assert_eq!(faux.preparations, 0);
        assert_eq!(faux.secours, 2);
        assert_eq!(faux.parametres, vec![None]);
    }

    /// Les Paramètres ouverts dans la boucle comptent comme « ouverts » pour l'icône (un
    /// clic les ramène devant).
    #[test]
    fn parametres_ouverts_dans_la_boucle() {
        struct Rien;
        impl crate::autres_fenetres::Contenu for Rien {
            fn dessiner(&mut self, _: &mut egui::Ui, _: &mut crate::autres_fenetres::Cadre<'_>) {}
        }
        let liens = Liens::default();
        assert_eq!(liens.ouvert(), Ouvert::Rien);
        liens.autres.ouvrir(
            crate::parametres::CLE_FENETRE,
            crate::autres_fenetres::Ouverture::new("Paramètres", [100.0, 100.0]),
            Rien,
        );
        assert_eq!(liens.ouvert(), Ouvert::Parametres);
        liens.mettre(Ouvert::Surface(Surface::Volet));
        assert_eq!(liens.ouvert(), Ouvert::Surface(Surface::Volet));
    }

    /// Clic gauche : le volet ; clic droit : le petit menu ; une autre fenêtre : on rend la
    /// main. Un deuxième clic pendant l'ouverture ne rouvre rien.
    #[test]
    fn suite_des_demandes() {
        assert_eq!(suite(&volet(), None), Suite::Montrer(Surface::Volet));
        assert_eq!(suite(&menu(), None), Suite::Montrer(Surface::Menu));
        assert_eq!(suite(&volet(), Some(Surface::Volet)), Suite::Ignorer);
        assert_eq!(
            suite(&menu(), Some(Surface::Volet)),
            Suite::Montrer(Surface::Menu)
        );
        assert_eq!(suite(&Demande::Parametres(None), None), Suite::Parametres);

        assert_eq!(suite(&Demande::Quitter, Some(Surface::Menu)), Suite::Sortir);
    }

    /// Le journal note la première ouverture et toute ouverture de plus de 150 ms, avec le
    /// détail ; pas les autres.
    #[test]
    fn temps_d_ouverture_au_journal() {
        let rapide = Mesure {
            attente: Duration::from_millis(3),
            mise_en_place: Duration::from_millis(30),
            images: 2,
        };
        assert!(rapide.a_noter(true));
        assert!(!rapide.a_noter(false));
        let lente = Mesure {
            mise_en_place: Duration::from_millis(200),
            ..rapide
        };
        assert!(lente.a_noter(false));
        let phrase = lente.phrase(Surface::Volet, false);
        assert!(phrase.contains("203 ms"), "{phrase}");
        assert!(phrase.contains("200 ms de mise en place"), "{phrase}");
        assert!(
            rapide
                .phrase(Surface::Menu, true)
                .starts_with("Petit menu ouvert")
        );
    }

    /// Rien n'est choisi pendant les premiers instants, ni par un appui commencé pendant la
    /// garde et relâché après ; ensuite, tout marche normalement.
    #[test]
    fn garde_des_premiers_instants() {
        let mut g = Garde::default();
        let ms = Duration::from_millis;
        assert_eq!(g.filtrer(Some(1), ms(10), false), None);
        assert_eq!(g.filtrer(None::<i32>, ms(300), true), None);
        assert!(!g.levee());
        assert_eq!(g.filtrer(Some(2), ms(320), false), None);
        assert!(g.levee());
        assert_eq!(g.filtrer(Some(3), ms(400), false), Some(3));
        let mut g = Garde::default();
        assert_eq!(g.filtrer(None::<i32>, ms(900), true), None);
        assert!(!g.levee());
        assert_eq!(g.filtrer(None::<i32>, GARDE_MAX, true), None);
        assert!(g.levee());
    }
}
