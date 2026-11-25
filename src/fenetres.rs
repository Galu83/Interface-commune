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
//!
//! Un logiciel qui tient sa propre boucle eframe (au lieu de [`boucle_des_fenetres`]) suit
//! les mêmes règles avec [`superviser_les_fenetres`] : il dit comment lancer sa boucle et quoi
//! faire de ses fenêtres perdues ([`Supervise`]), la supervision compte les pertes
//! ([`Pertes`]) et décide quand les recréer ([`Reprise`]).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::{Receiver, SendError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::autres_fenetres::AutresFenetres;
use crate::place::Rect;
use crate::theme::Surface;

// La décision d'un clic, la garde, les pertes et leur reprise : calcul pur, dans le cœur
// (bloc D), repris ici à leurs chemins d'avant.
pub use ic_coeur::gestes::ouverture::{
    ANTI_REBOND, Appuis, Clic, GARDE, GARDE_MAX, Garde, LENTE, Mesure, Ouvert, decider_le_clic, est_dehors,
    fermer_sur_appui,
};
pub use ic_coeur::gestes::secours::{Evenement, Fin, Perte, RAFALE, RECREATIONS_EN_RAFALE, Reprise};
/// Les pertes récentes des fenêtres, à l'heure de Windows ([`ic_coeur::gestes::secours::Pertes`]).
pub type Pertes = ic_coeur::gestes::secours::Pertes<Instant>;

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

/// Ce qu'est devenu un clic sur une icône ([`envoyer_le_clic`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aiguillage {
    /// Rien à faire de plus : le clic ferme la surface montrée, ou il est ignoré.
    Rien,
    /// La demande est partie au fil des fenêtres : réveiller la fenêtre prête
    /// ([`Liens::surface_prete`]) pour qu'elle se montre tout de suite.
    Envoye,
    /// Les fenêtres modernes sont impossibles (ou le fil des fenêtres ne répond plus) : le
    /// menu de secours de Windows, au point du clic.
    Secours,
}

/// Le clic sur une icône pour `surface` (le volet, ou le petit menu avec son point `ancre`),
/// par les mécanismes communs : ouvert devant les Paramètres, refermé par un second clic,
/// ignoré juste après une fermeture ([`decider_le_clic`]), envoyé au fil des fenêtres
/// ([`Demandes`]). Si le fil des fenêtres ne répond plus, l'icône passe au menu de secours pour
/// de bon ([`Liens::egui_impossible`]). Le même pour l'icône unique (`horloge`) et pour les
/// icônes d'une `zone`.
pub fn envoyer_le_clic(
    liens: &Liens,
    demandes: &Demandes,
    surface: Surface,
    icone: Rect,
    ancre: Option<[i32; 2]>,
    clic: Instant,
) -> Aiguillage {
    if liens.egui_impossible.load(Ordering::Acquire) {
        return Aiguillage::Secours;
    }
    match decider_le_clic(liens.ouvert(), surface, liens.juste_fermee(surface)) {
        Clic::Fermer => {
            liens.fermer_la_fenetre();
            Aiguillage::Rien
        }
        Clic::Ignorer => Aiguillage::Rien,
        Clic::Ouvrir => {
            let d = match (surface, ancre) {
                (Surface::Menu, Some(ancre)) => Demande::Menu { icone, ancre, clic },
                (Surface::Menu, None) => Demande::Menu {
                    icone,
                    ancre: crate::place::centre_de(icone),
                    clic,
                },
                (Surface::Volet, _) => Demande::Volet { icone, clic },
            };
            if demandes.send(d).is_err() {
                liens.egui_impossible.store(true, Ordering::Release);
                return Aiguillage::Secours;
            }
            Aiguillage::Envoye
        }
    }
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
        demandes.racine(0);
        demandes.lecteur(None);
    }

    /// `surface` s'est fermée il y a moins d'[`ANTI_REBOND`] (le clic sur l'icône qui l'a
    /// fermée en lui prenant le focus ne doit pas la rouvrir aussitôt).
    pub fn juste_fermee(&self, surface: Surface) -> bool {
        let fermee = self.ferme_a.lock().ok().and_then(|f| *f);
        ic_coeur::gestes::ouverture::juste_fermee(fermee, surface, Instant::now())
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
            && let Ok((mut s, _)) =
                self.cv
                    .wait_timeout_while(s, std::time::Duration::from_millis(u64::from(ms)), |s| !*s)
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
    /// La fenêtre principale de la boucle (HWND ; 0 tant qu'elle n'est pas prête).
    racine: Arc<std::sync::atomic::AtomicIsize>,
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
        // Un message à la fenêtre principale cachée : l'image qui lit la demande se fait dans le
        // contexte de winit, le seul où une fenêtre (les Paramètres…) peut être créée. Un
        // simple `request_repaint` fait peindre la fenêtre cachée hors de ce contexte (minuterie
        // d'eframe) : la création ratait au premier essai. Pendant un réveil de Windows
        // (`crate::reprise`), pas de message : la demande est lue par une image sans dessin,
        // et la fenêtre principale, réveillée plus tard (`AutresFenetres::reveiller_dans`).
        #[cfg(windows)]
        if crate::reprise::attente_avant_dessin().is_none() {
            crate::systeme::reveiller(self.racine.load(Ordering::Acquire));
        }
        r
    }
}

/// La réception des demandes, côté fil des fenêtres.
pub struct Reception {
    reception: Receiver<Demande>,
    reveil: Arc<Reveil>,
    lecteur: Lecteur,
    racine: Arc<std::sync::atomic::AtomicIsize>,
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

    /// La fenêtre principale de la boucle, prête (HWND ; 0 : plus aucune) : chaque demande
    /// envoyée lui adresse un message pour la réveiller ([`Demandes::send`]).
    pub fn racine(&self, h: isize) {
        self.racine.store(h, Ordering::Release);
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
    let racine = Arc::new(std::sync::atomic::AtomicIsize::new(0));
    (
        Demandes {
            envoi,
            reveil: reveil.clone(),
            lecteur: lecteur.clone(),
            racine: racine.clone(),
        },
        Reception {
            reception,
            reveil,
            lecteur,
            racine,
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

// ---------------------------------------------------------------------------------------
// La supervision d'une boucle de fenêtres tenue par le logiciel

/// Ce que la supervision fait tourner : la boucle des fenêtres du logiciel, et ce qu'il fait
/// quand elle est perdue. Pour un logiciel qui tient sa propre boucle eframe ; la boucle
/// toute faite de la bibliothèque ([`boucle_des_fenetres`]) suit les mêmes règles.
pub trait Supervise {
    /// Un tour de la boucle des fenêtres (par exemple `eframe::run_native` avec
    /// `run_and_return`). `Ok` quand elle s'est fermée, `Err` quand elle n'a pas pu être
    /// créée. Une panique est attrapée par la supervision.
    fn boucle(&mut self) -> Result<(), String>;
    /// Les fenêtres sont perdues : le logiciel note `e.phrase()` au journal et oublie ce qui
    /// était ouvert. [`Reprise::Maintenant`] : il refait le clic qui s'ouvrait, pour les
    /// fenêtres recréées. [`Reprise::AuClicSuivant`] : il ouvre le menu de Windows pour ce
    /// clic ; le clic suivant revient aux fenêtres.
    fn perdues(&mut self, e: &Evenement);
    /// Après [`Reprise::AuClicSuivant`] : attend le clic suivant (ou toute autre demande aux
    /// fenêtres), puis rend `true` pour les recréer ; `false` : le logiciel se ferme.
    fn attendre_un_clic(&mut self) -> bool;
    /// Le logiciel se ferme : plus rien n'est recréé.
    fn quitter(&self) -> bool {
        false
    }
}

/// Fait tourner la boucle des fenêtres de `s` et la recrée quand elle est perdue : une
/// panique (attrapée ici, sa cause et son lieu gardés pour le journal), une fermeture par
/// Windows ou un échec passager. Jusqu'à [`RECREATIONS_EN_RAFALE`] pertes en [`RAFALE`],
/// elle est recréée tout de suite ; au-delà, le logiciel ouvre le menu de Windows pour le
/// clic en cours, et elle est recréée au clic suivant. Seul un échec au tout premier tour
/// (rien n'a jamais marché) arrête la supervision pour de bon. `horloge` : l'heure
/// (injectable pour les essais ; `Instant::now` sinon).
pub fn superviser_les_fenetres(s: &mut impl Supervise, horloge: impl Fn() -> Instant) -> Fin {
    crate::sonde::retenir_les_paniques();
    let mut pertes = Pertes::default();
    let mut premier_tour = true;
    loop {
        if s.quitter() {
            return Fin::Quitter;
        }
        let tour = catch_unwind(AssertUnwindSafe(|| s.boucle()));
        let premier = std::mem::replace(&mut premier_tour, false);
        if s.quitter() {
            return Fin::Quitter;
        }
        let perte = match tour {
            Ok(Ok(())) => Perte::Fermee,
            Ok(Err(e)) if premier => return Fin::Impossible(e),
            Ok(Err(e)) => Perte::Echec(e),
            Err(p) => Perte::Panique(cause_de_panique(&*p)),
        };
        // Un échec passager n'est pas une perte de plus : un nouvel essai au clic suivant.
        let (nombre, reprise) = ic_coeur::gestes::secours::decider_la_reprise(&perte, &mut pertes, &horloge);
        s.perdues(&Evenement {
            perte,
            nombre,
            reprise,
        });
        if reprise == Reprise::AuClicSuivant && !s.attendre_un_clic() {
            return Fin::Quitter;
        }
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
                        if !Pertes::en_rafale(n) {
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
                        en_pause = Pertes::en_rafale(n);
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
// Ce que la fenêtre du volet fait d'une demande (sa garde et la mesure de son ouverture :
// `ic_coeur::gestes::ouverture`)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

    /// Un tour de la fausse boucle supervisée.
    #[derive(Debug, Clone, Copy)]
    enum Tour {
        Panique,
        Fermee,
        Erreur,
        /// Le logiciel se ferme pendant ce tour.
        Quitter,
    }

    /// Une fausse boucle de fenêtres pour la supervision : chaque tour suit le script, à
    /// l'heure dite (en secondes depuis le départ), sans rien ouvrir.
    struct Supervisee {
        script: std::collections::VecDeque<(u64, Tour)>,
        horloge: Arc<std::sync::atomic::AtomicU64>,
        tours: usize,
        evenements: Vec<Evenement>,
        attentes: usize,
        /// `attendre_un_clic` rend faux (le logiciel se ferme pendant la pause).
        ferme_en_pause: bool,
        fini: bool,
    }

    impl Supervisee {
        fn new(script: &[(u64, Tour)]) -> Supervisee {
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

        fn superviser(&mut self) -> Fin {
            let depart = Instant::now();
            let h = self.horloge.clone();
            superviser_les_fenetres(self, move || {
                depart + Duration::from_secs(h.load(Ordering::Acquire))
            })
        }

        fn reprises(&self) -> Vec<Reprise> {
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

    /// La boucle d'un logiciel perdue (panique) est recréée tout de suite, trois fois en une
    /// minute ; la quatrième perte fait une rafale : le menu de Windows pour ce clic, puis la
    /// supervision attend le clic suivant et la recrée (jamais bloquée jusqu'au prochain
    /// lancement). La cause de la panique est dans le journal.
    #[test]
    fn supervision_rafale_puis_reprise_au_clic_suivant() {
        let mut s = Supervisee::new(&[
            (0, Tour::Panique),
            (1, Tour::Panique),
            (2, Tour::Panique),
            (3, Tour::Panique),
            // Recréée au clic suivant : elle tient, puis le logiciel se ferme.
            (10, Tour::Quitter),
        ]);
        assert_eq!(s.superviser(), Fin::Quitter);
        assert_eq!(s.tours, 5);
        assert_eq!(
            s.reprises(),
            vec![
                Reprise::Maintenant,
                Reprise::Maintenant,
                Reprise::Maintenant,
                Reprise::AuClicSuivant
            ]
        );
        assert_eq!(s.attentes, 1);
        let e = &s.evenements[3];
        assert_eq!(e.nombre, RECREATIONS_EN_RAFALE + 1);
        assert!(
            matches!(&e.perte, Perte::Panique(c) if c.contains("contexte OpenGL perdu (essai)")),
            "{e:?}"
        );
        let phrase = e.phrase();
        assert!(phrase.contains("contexte OpenGL perdu (essai)"), "{phrase}");
        assert!(phrase.contains("4 fois en 60 s"), "{phrase}");
        assert!(phrase.contains("recréées au clic suivant"), "{phrase}");
        assert!(s.evenements[0].phrase().ends_with(": recréées."));
    }

    /// Après une rafale, une nouvelle perte dans la même minute redonne le menu de Windows et
    /// attend encore le clic suivant (jamais de boucle folle) ; une minute plus tard, les
    /// vieilles pertes sont oubliées et la boucle est de nouveau recréée tout de suite.
    #[test]
    fn supervision_reprise_apres_rafale() {
        let mut s = Supervisee::new(&[
            (0, Tour::Panique),
            (0, Tour::Panique),
            (0, Tour::Panique),
            (0, Tour::Panique),
            (5, Tour::Panique),
            (120, Tour::Panique),
            (121, Tour::Quitter),
        ]);
        assert_eq!(s.superviser(), Fin::Quitter);
        assert_eq!(
            s.reprises(),
            vec![
                Reprise::Maintenant,
                Reprise::Maintenant,
                Reprise::Maintenant,
                Reprise::AuClicSuivant,
                Reprise::AuClicSuivant,
                Reprise::Maintenant,
            ]
        );
        assert_eq!(s.attentes, 2);
        assert_eq!(s.evenements[5].nombre, 1);
    }

    /// Des pertes espacées de plus d'une minute ne font jamais de rafale : toutes recréées
    /// tout de suite. Une fermeture par Windows compte comme une perte.
    #[test]
    fn supervision_pertes_espacees_toujours_recreees() {
        let mut script: Vec<(u64, Tour)> = (0..10)
            .map(|i| {
                let tour = if i % 2 == 0 { Tour::Panique } else { Tour::Fermee };
                (i * 61, tour)
            })
            .collect();
        script.push((1_000, Tour::Quitter));
        let mut s = Supervisee::new(&script);
        assert_eq!(s.superviser(), Fin::Quitter);
        assert_eq!(s.tours, 11);
        assert_eq!(s.reprises(), vec![Reprise::Maintenant; 10]);
        assert_eq!(s.attentes, 0);
        assert_eq!(s.evenements[1].perte, Perte::Fermee);
        assert!(s.evenements[1].phrase().contains("fermées par Windows"));
    }

    /// Un échec au tout premier tour (ni OpenGL ni fenêtre) : le menu de Windows pour de bon.
    /// Un échec après un premier tour est passager : le menu de Windows pour ce clic, un
    /// nouvel essai au clic suivant, sans compter comme une perte.
    #[test]
    fn supervision_echec_premier_tour_ou_passager() {
        let mut s = Supervisee::new(&[(0, Tour::Erreur)]);
        assert_eq!(
            s.superviser(),
            Fin::Impossible("OpenGL indisponible (essai)".into())
        );
        assert!(s.evenements.is_empty());

        let mut s = Supervisee::new(&[(0, Tour::Panique), (1, Tour::Erreur), (2, Tour::Quitter)]);
        assert_eq!(s.superviser(), Fin::Quitter);
        assert_eq!(s.reprises(), vec![Reprise::Maintenant, Reprise::AuClicSuivant]);
        assert_eq!(s.evenements[1].nombre, 1, "l'échec ne compte pas");
        assert_eq!(s.attentes, 1);
        let phrase = s.evenements[1].phrase();
        assert!(phrase.contains("OpenGL indisponible (essai)"), "{phrase}");
        assert!(phrase.contains("nouvel essai au clic suivant"), "{phrase}");
    }

    /// Le logiciel se ferme : pendant la pause d'une rafale, ou à la fin d'un tour, rien
    /// n'est recréé.
    #[test]
    fn supervision_quitter() {
        let mut s = Supervisee::new(&[(0, Tour::Panique); 4]);
        s.ferme_en_pause = true;
        assert_eq!(s.superviser(), Fin::Quitter);
        assert_eq!((s.tours, s.attentes), (4, 1));

        let mut s = Supervisee::new(&[(0, Tour::Quitter)]);
        assert_eq!(s.superviser(), Fin::Quitter);
        assert!(s.evenements.is_empty());
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

    /// Le volet ouvert devant les Paramètres : `ouvert()` rend la surface ; une fois la surface
    /// fermée, il redonne les Paramètres (restés ouverts derrière).
    #[test]
    fn surface_devant_les_parametres_puis_retour() {
        struct Rien;
        impl crate::autres_fenetres::Contenu for Rien {
            fn dessiner(&mut self, _: &mut egui::Ui, _: &mut crate::autres_fenetres::Cadre<'_>) {}
        }
        let liens = Liens::default();
        liens.autres.ouvrir(
            crate::parametres::CLE_FENETRE,
            crate::autres_fenetres::Ouverture::new("Paramètres", [100.0, 100.0]),
            Rien,
        );
        assert_eq!(liens.ouvert(), Ouvert::Parametres);
        let clic = decider_le_clic(liens.ouvert(), Surface::Menu, liens.juste_fermee(Surface::Menu));
        assert_eq!(clic, Clic::Ouvrir);
        liens.mettre(Ouvert::Surface(Surface::Menu));
        assert_eq!(liens.ouvert(), Ouvert::Surface(Surface::Menu));
        assert!(liens.autres.ouverte(crate::parametres::CLE_FENETRE));
        // Fermeture de la surface (perte du focus comprise) : les Paramètres sont toujours là.
        liens.mettre(Ouvert::Rien);
        assert_eq!(liens.ouvert(), Ouvert::Parametres);
        assert!(liens.autres.ouverte(crate::parametres::CLE_FENETRE));
    }

    /// Les Paramètres ouverts dans la boucle comptent comme « ouverts » pour l'icône.
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
}
