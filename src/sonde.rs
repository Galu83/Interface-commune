//! La sonde des fils : dit au journal quand un fil du logiciel reste pris (l'icône qui ne
//! réagit plus, le volet qui ne s'ouvre plus) et où il est pris. Sans Windows : tout est testé.
//!
//! Trois choses, toutes écrites au journal du logiciel par [`crate::Logiciel::noter`] :
//! - **les clics** de l'icône (gauche, droit, double), avec l'heure ([`clic`]) ;
//! - **les boucles imbriquées** : un appel qui fait tourner sa propre boucle de messages
//!   (menu de Windows, sélecteur de dossier…) est entouré de [`imbriquee`] : une ligne au
//!   début (le nom), une à la fin (la durée) ;
//! - **le chien de garde** : un seul fil de surveillance pour tous les fils inscrits
//!   ([`inscrire`]). Chaque fil a un compteur de battements ([`battement`]) qui avance à chaque
//!   tour de SA boucle. Toutes les 5 s, la surveillance lit les compteurs ; un compteur immobile
//!   depuis 15 s donne une ligne (une seule par blocage), puis une ligne quand le fil repart,
//!   avec la durée totale. La ligne dit la **dernière étape** du fil ([`etape`]) et, s'il est
//!   dans une boucle imbriquée, laquelle et depuis combien de temps.
//!
//! Les lignes ne contiennent jamais rien de l'utilisateur (frappes, textes, chemins) : seulement
//! des noms écrits dans le code (`&'static str`).
//!
//! # Utilisation par un projet
//!
//! La bibliothèque inscrit ses propres fils (l'icône, les fenêtres) et branche le journal
//! d'elle-même. Un projet peut seulement changer les délais, **avant** `Horloge::lancer` :
//!
//! ```no_run
//! use std::time::Duration;
//! interface_commune::sonde::regler(interface_commune::sonde::Delais {
//!     pas: Duration::from_secs(2),
//!     blocage: Duration::from_secs(10),
//! });
//! ```
//!
//! Un fil du projet qui a sa propre boucle s'inscrit pareil : `let _fil = sonde::inscrire(
//! "mon fil", None);` puis `sonde::battement()` à chaque tour, `sonde::etape("…")` avant un
//! appel à risque et `let _g = sonde::imbriquee("…");` autour d'un appel à boucle propre.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex, OnceLock, PoisonError, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Les délais de la surveillance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delais {
    /// Le temps entre deux lectures des compteurs.
    pub pas: Duration,
    /// Le temps sans battement après lequel un fil est dit bloqué.
    pub blocage: Duration,
}

impl Default for Delais {
    fn default() -> Delais {
        Delais {
            pas: Duration::from_secs(5),
            blocage: Duration::from_secs(15),
        }
    }
}

static DELAIS: Mutex<Option<Delais>> = Mutex::new(None);

/// Change les délais de la surveillance ; à appeler avant le lancement (la surveillance les lit
/// une fois, à son démarrage).
pub fn regler(d: Delais) {
    *DELAIS.lock().unwrap_or_else(PoisonError::into_inner) = Some(d);
}

fn delais() -> Delais {
    DELAIS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------------------
// Le journal

type Sortie = Arc<dyn Fn(String) + Send + Sync>;

static SORTIE: RwLock<Option<Sortie>> = RwLock::new(None);

/// Branche le journal : `f` reçoit chaque ligne de la sonde (la bibliothèque y met
/// `Logiciel::noter`). Sans journal branché, les lignes sont perdues.
pub fn brancher_le_journal(f: impl Fn(String) + Send + Sync + 'static) {
    *SORTIE.write().unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(f));
}

pub(crate) fn noter(ligne: String) {
    let s = SORTIE.read().unwrap_or_else(PoisonError::into_inner).clone();
    if let Some(s) = s {
        s(ligne);
    }
}

/// L'heure UTC du jour, `14:03:22`.
fn heure_utc(t: SystemTime) -> String {
    let s = t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// Un clic reçu par l'icône (`"gauche"`, `"droit"`, `"double"`…), avec l'heure.
pub fn clic(quel: &'static str) {
    noter(format!(
        "Icône : clic {quel} reçu à {} UTC.",
        heure_utc(SystemTime::now())
    ));
}

// ---------------------------------------------------------------------------------------
// Les fils surveillés

/// Une boucle imbriquée en cours : son nom et son début.
type EnImbrication = Option<(&'static str, Instant)>;

#[derive(Clone, Copy, Default)]
struct EtatDuFil {
    /// La dernière étape posée (un nom écrit dans le code).
    etape: &'static str,
    /// La boucle imbriquée en cours : son nom et son début.
    imbriquee: EnImbrication,
}

/// Un fil surveillé : un compteur de battements, sa dernière étape, de quoi le réveiller.
pub struct Fil {
    nom: &'static str,
    battements: AtomicU64,
    etat: Mutex<EtatDuFil>,
    /// Le signal qui réveille le fil s'il dort (un fil à l'arrêt entre deux messages ne bat pas
    /// tout seul). Le réveil lui-même tourne sur un petit fil à part : la surveillance envoie
    /// le signal (`try_send`) sans jamais attendre, même si ce réveil reste pris dans un verrou.
    reveil: Option<SyncSender<()>>,
}

impl Fil {
    fn nouveau(nom: &'static str, reveil: Option<Box<dyn Fn() + Send + Sync>>) -> Arc<Fil> {
        // Le petit fil du réveil s'arrête quand le fil est lâché (le canal se ferme).
        let reveil = reveil.and_then(|r| {
            let (tx, rx) = sync_channel::<()>(1);
            std::thread::Builder::new()
                .name(format!("sonde : réveil {nom}"))
                .spawn(move || {
                    while rx.recv().is_ok() {
                        r();
                    }
                })
                .ok()
                .map(|_| tx)
        });
        Arc::new(Fil {
            nom,
            battements: AtomicU64::new(0),
            etat: Mutex::new(EtatDuFil::default()),
            reveil,
        })
    }

    fn battre(&self) {
        self.battements.fetch_add(1, Ordering::Release);
    }

    fn poser(&self, etape: &'static str) {
        self.etat.lock().unwrap_or_else(PoisonError::into_inner).etape = etape;
    }

    fn entrer(&self, nom: &'static str, maintenant: Instant) -> EnImbrication {
        let mut e = self.etat.lock().unwrap_or_else(PoisonError::into_inner);
        let avant = e.imbriquee;
        e.imbriquee = Some((nom, maintenant));
        avant
    }

    fn sortir(&self, avant: EnImbrication) {
        self.etat.lock().unwrap_or_else(PoisonError::into_inner).imbriquee = avant;
    }

    fn etat(&self) -> EtatDuFil {
        *self.etat.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

static FILS: Mutex<Vec<Arc<Fil>>> = Mutex::new(Vec::new());

thread_local! {
    static COURANT: RefCell<Option<Arc<Fil>>> = const { RefCell::new(None) };
}

/// Ce que rend [`inscrire`] : le fil sort de la surveillance quand elle est lâchée.
#[must_use = "le fil n'est surveillé que tant que cette valeur vit"]
pub struct Inscription {
    fil: Arc<Fil>,
}

impl Drop for Inscription {
    fn drop(&mut self) {
        FILS.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|f| !Arc::ptr_eq(f, &self.fil));
        COURANT.with(|c| *c.borrow_mut() = None);
    }
}

/// Inscrit le fil appelant à la surveillance (lancée au premier fil inscrit). `reveil` : de
/// quoi le tirer de son attente pour qu'il batte (un message posté à sa fenêtre, une image
/// demandée) ; `None` si sa boucle tourne toujours.
pub fn inscrire(nom: &'static str, reveil: Option<Box<dyn Fn() + Send + Sync>>) -> Inscription {
    let fil = Fil::nouveau(nom, reveil);
    FILS.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(fil.clone());
    COURANT.with(|c| *c.borrow_mut() = Some(fil.clone()));
    demarrer_la_surveillance();
    Inscription { fil }
}

/// Le fil appelant a fait un tour de sa boucle. Sans effet pour un fil non inscrit.
pub fn battement() {
    COURANT.with(|c| {
        if let Some(f) = c.borrow().as_ref() {
            f.battre();
        }
    });
}

/// Pose la dernière étape du fil appelant, **avant** un appel à risque (changement de
/// contexte OpenGL, échange des tampons, changement d'écrans, création ou destruction d'une
/// fenêtre). Un nom écrit dans le code, jamais un texte venu de l'utilisateur.
pub fn etape(nom: &'static str) {
    COURANT.with(|c| {
        if let Some(f) = c.borrow().as_ref() {
            f.poser(nom);
        }
    });
}

/// Garde d'une boucle imbriquée : voir [`imbriquee`].
#[must_use = "la boucle imbriquée est notée jusqu'à ce que cette valeur soit lâchée"]
pub struct Imbriquee {
    nom: &'static str,
    debut: Instant,
    fil: Option<(Arc<Fil>, EnImbrication)>,
}

/// À mettre autour d'un appel qui fait tourner sa propre boucle de messages (menu de Windows,
/// sélecteur de dossier, boîte de message…) : pendant ce temps le fil ne bat pas, et il ne
/// traite plus les clics de l'icône. Une ligne au début (le nom), une à la fin (la durée).
pub fn imbriquee(nom: &'static str) -> Imbriquee {
    let debut = Instant::now();
    let fil = COURANT.with(|c| c.borrow().clone()).map(|f| {
        let avant = f.entrer(nom, debut);
        (f, avant)
    });
    noter(format!("Boucle imbriquée « {nom} » : début."));
    Imbriquee { nom, debut, fil }
}

impl Drop for Imbriquee {
    fn drop(&mut self) {
        if let Some((f, avant)) = self.fil.take() {
            f.sortir(avant);
        }
        noter(format!(
            "Boucle imbriquée « {} » : fin après {} ms.",
            self.nom,
            self.debut.elapsed().as_millis()
        ));
    }
}

// ---------------------------------------------------------------------------------------
// Les paniques attrapées

thread_local! {
    /// Le lieu (fichier et ligne) de la dernière panique de ce fil, gardé pour le journal.
    static DERNIERE_PANIQUE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Garde, pour chaque fil, le lieu de sa dernière panique ([`lieu_de_la_derniere_panique`]),
/// puis passe la main au crochet de panique déjà en place (qui garde son rôle). À appeler une
/// fois ; sans effet ensuite.
pub fn retenir_les_paniques() {
    static UNE_FOIS: std::sync::Once = std::sync::Once::new();
    UNE_FOIS.call_once(|| {
        let ancien = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if let Some(l) = info.location() {
                let lieu = lieu_court(l.file(), l.line());
                let _ = DERNIERE_PANIQUE.try_with(|d| {
                    if let Ok(mut d) = d.try_borrow_mut() {
                        *d = Some(lieu);
                    }
                });
            }
            ancien(info);
        }));
    });
}

/// Le lieu de la dernière panique du fil appelant (une fois : il est retiré).
pub fn lieu_de_la_derniere_panique() -> Option<String> {
    DERNIERE_PANIQUE
        .try_with(|d| d.try_borrow_mut().ok().and_then(|mut d| d.take()))
        .ok()
        .flatten()
}

/// La dernière étape posée par le fil appelant, s'il est surveillé.
pub fn etape_du_fil() -> Option<&'static str> {
    COURANT
        .try_with(|c| c.borrow().as_ref().map(|f| f.etat().etape))
        .ok()
        .flatten()
        .filter(|e| !e.is_empty())
}

/// Les deux derniers éléments du chemin d'un fichier source (jamais le chemin du PC).
fn lieu_court(fichier: &str, ligne: u32) -> String {
    let morceaux: Vec<&str> = fichier.split(['/', '\\']).filter(|m| !m.is_empty()).collect();
    let debut = morceaux.len().saturating_sub(2);
    format!("{}:{ligne}", morceaux[debut..].join("/"))
}

// ---------------------------------------------------------------------------------------
// La surveillance

struct Suivi {
    dernier: u64,
    /// Le moment où le compteur a changé pour la dernière fois (ou où on l'a vu la première
    /// fois).
    depuis: Instant,
    signale: bool,
}

/// La logique de la surveillance, sans horloge ni fil : l'heure est donnée à chaque tour.
pub struct Surveillance {
    blocage: Duration,
    suivis: HashMap<usize, Suivi>,
    /// Le pas entre deux tours, et l'heure du dernier tour : un écart bien plus long que le pas
    /// veut dire que la boucle de surveillance elle-même n'a pas tourné (le PC dormait). `None`
    /// sans pas : aucune veille n'est cherchée.
    pas: Option<Duration>,
    dernier_tour: Option<Instant>,
}

/// Un écart entre deux tours d'une boucle de surveillance qui dépasse largement son `pas`
/// (plus de trois pas, et plus de 3 s) : la boucle elle-même n'a pas tourné, le PC dormait. Les
/// horloges de Windows (`Instant`) continuent de compter pendant la veille : sans cela, au
/// réveil, chaque fil paraîtrait bloqué depuis toute la durée du sommeil.
pub fn est_une_veille(ecart: Duration, pas: Duration) -> bool {
    ecart > (pas * 3).max(Duration::from_secs(3))
}

impl Surveillance {
    pub fn new(blocage: Duration) -> Surveillance {
        Surveillance {
            blocage,
            suivis: HashMap::new(),
            pas: None,
            dernier_tour: None,
        }
    }

    /// Comme [`Surveillance::new`], pour une boucle qui tourne tous les `pas` : un écart bien plus
    /// long entre deux tours est une veille du PC ([`est_une_veille`]) ; les repères de tous les
    /// fils repartent de zéro, sans alerte, avec une seule ligne au journal.
    pub fn avec_pas(blocage: Duration, pas: Duration) -> Surveillance {
        Surveillance {
            pas: Some(pas),
            ..Surveillance::new(blocage)
        }
    }

    /// Lit les compteurs de `fils` à `maintenant` ; `ecrire` reçoit les lignes à journaliser.
    fn tour(&mut self, maintenant: Instant, fils: &[Arc<Fil>], ecrire: &mut dyn FnMut(String)) {
        let precedent = self.dernier_tour.replace(maintenant);
        if let (Some(pas), Some(precedent)) = (self.pas, precedent) {
            let ecart = maintenant.saturating_duration_since(precedent);
            if est_une_veille(ecart, pas) {
                // La boucle a dormi : aucun fil n'est bloqué pour autant. Les repères repartent
                // de zéro (chaque fil est revu comme la première fois), sans alerte.
                self.suivis.clear();
                ecrire(format!(
                    "Reprise après une veille d'environ {} s : les mesures de blocage repartent de zéro.",
                    ecart.as_secs()
                ));
            }
        }
        let cles: Vec<usize> = fils.iter().map(|f| Arc::as_ptr(f) as usize).collect();
        self.suivis.retain(|k, _| cles.contains(k));
        for (f, k) in fils.iter().zip(cles) {
            let c = f.battements.load(Ordering::Acquire);
            let s = self.suivis.entry(k).or_insert(Suivi {
                dernier: c,
                depuis: maintenant,
                signale: false,
            });
            if c != s.dernier {
                if s.signale {
                    ecrire(format!(
                        "Le fil « {} » repart après {} s de blocage.",
                        f.nom,
                        maintenant.saturating_duration_since(s.depuis).as_secs()
                    ));
                }
                *s = Suivi {
                    dernier: c,
                    depuis: maintenant,
                    signale: false,
                };
                continue;
            }
            let immobile = maintenant.saturating_duration_since(s.depuis);
            if immobile >= self.blocage && !s.signale {
                s.signale = true;
                let e = f.etat();
                let dans = e.imbriquee.map_or(String::new(), |(n, d)| {
                    format!(
                        " dans « {n} » depuis {} s,",
                        maintenant.saturating_duration_since(d).as_secs()
                    )
                });
                let etape = if e.etape.is_empty() { "aucune" } else { e.etape };
                ecrire(format!(
                    "Le fil « {} » ne répond plus depuis {} s :{dans} dernière étape : {etape}.",
                    f.nom,
                    immobile.as_secs()
                ));
            }
        }
    }
}

impl Surveillance {
    /// Un tour, puis le signal de réveil de chaque fil (il a un pas entier pour battre avant le
    /// tour suivant). Le signal n'attend jamais : un réveil bloqué n'arrête que lui.
    fn tour_et_reveils(&mut self, maintenant: Instant, fils: &[Arc<Fil>], ecrire: &mut dyn FnMut(String)) {
        self.tour(maintenant, fils, ecrire);
        for f in fils {
            if let Some(r) = &f.reveil {
                let _ = r.try_send(());
            }
        }
    }
}

/// Lance l'unique fil de surveillance (une seule fois pour le programme).
fn demarrer_la_surveillance() {
    static UNE_FOIS: OnceLock<()> = OnceLock::new();
    UNE_FOIS.get_or_init(|| {
        let d = delais();
        let _ = std::thread::Builder::new().name("sonde".into()).spawn(move || {
            let mut s = Surveillance::avec_pas(d.blocage, d.pas);
            let mut avant = Instant::now();
            loop {
                std::thread::sleep(d.pas);
                let maintenant = Instant::now();
                if est_une_veille(maintenant.saturating_duration_since(avant), d.pas) {
                    // Une reprise que Windows n'aurait pas signalée (WM_POWERBROADCAST) : le
                    // dessin des fenêtres attend aussi la fin du réveil.
                    crate::reprise::reveil();
                }
                avant = maintenant;
                let fils: Vec<Arc<Fil>> = FILS.lock().unwrap_or_else(PoisonError::into_inner).clone();
                s.tour_et_reveils(maintenant, &fils, &mut noter);
            }
        });
    });
}

// ---------------------------------------------------------------------------------------
// Le chien de garde des créations de fenêtre

/// Une création de fenêtre qui dure plus longtemps que cela est écrite au journal.
pub const SEUIL_DE_CREATION: Duration = Duration::from_secs(3);

/// Le seuil en millisecondes (les essais le raccourcissent).
static SEUIL_CREATION_MS: AtomicU64 = AtomicU64::new(3_000);

#[cfg(test)]
pub(crate) fn regler_le_seuil_de_creation(seuil: Duration) {
    SEUIL_CREATION_MS.store(
        u64::try_from(seuil.as_millis()).unwrap_or(u64::MAX),
        Ordering::Release,
    );
}

fn seuil_de_creation() -> Duration {
    Duration::from_millis(SEUIL_CREATION_MS.load(Ordering::Acquire))
}

/// Une création de fenêtre en cours, vue par le chien de garde.
struct EnCreation {
    id: u64,
    nom: String,
    debut: Instant,
    debut_utc: SystemTime,
    signalee: bool,
}

static CREATIONS: Mutex<Vec<EnCreation>> = Mutex::new(Vec::new());
static PROCHAINE_CREATION: AtomicU64 = AtomicU64::new(1);

/// Une création de fenêtre en cours : à garder pendant tout l'appel qui crée la fenêtre.
/// Si l'appel dure plus de [`SEUIL_DE_CREATION`] (il ne rend jamais la main : un blocage dans
/// la création de la fenêtre ou de son contexte OpenGL, sans panique ni erreur), un fil à part
/// écrit au journal le nom de la fenêtre et l'heure du début ; puis une ligne quand l'appel
/// finit, avec la durée. Le nom est celui que le logiciel a donné à sa fenêtre.
pub struct Creation {
    id: u64,
}

/// Commence la surveillance de la création de la fenêtre `nom`.
pub fn creation(nom: &str) -> Creation {
    let id = PROCHAINE_CREATION.fetch_add(1, Ordering::AcqRel);
    CREATIONS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(EnCreation {
            id,
            nom: nom.to_string(),
            debut: Instant::now(),
            debut_utc: SystemTime::now(),
            signalee: false,
        });
    demarrer_le_chien_des_creations();
    Creation { id }
}

impl Drop for Creation {
    fn drop(&mut self) {
        let fini = {
            let mut l = CREATIONS.lock().unwrap_or_else(PoisonError::into_inner);
            l.iter().position(|c| c.id == self.id).map(|i| l.remove(i))
        };
        if let Some(c) = fini
            && c.signalee
        {
            noter(format!(
                "Fenêtre « {} » : la création, commencée à {} UTC, s'est enfin terminée après {} s.",
                c.nom,
                heure_utc(c.debut_utc),
                c.debut.elapsed().as_secs()
            ));
        }
    }
}

/// Les lignes à écrire à `maintenant` : une, une seule fois, pour chaque création qui dure
/// depuis plus de `seuil`.
fn lignes_des_creations(l: &mut [EnCreation], maintenant: Instant, seuil: Duration) -> Vec<String> {
    let mut lignes = Vec::new();
    for c in l.iter_mut() {
        let duree = maintenant.saturating_duration_since(c.debut);
        if duree >= seuil && !c.signalee {
            c.signalee = true;
            lignes.push(format!(
                "Fenêtre « {} » : sa création dure depuis {} s (commencée à {} UTC) ; le fil des fenêtres ne rend plus la main.",
                c.nom,
                duree.as_secs(),
                heure_utc(c.debut_utc)
            ));
        }
    }
    lignes
}

/// Un tour du chien de garde des créations à `maintenant` : si l'écart avec le tour précédent
/// (`dernier`) est une veille du PC ([`est_une_veille`]), les créations en cours repartent de
/// zéro, sans alerte (la sonde écrit la ligne de reprise) ; sinon, les lignes de
/// [`lignes_des_creations`].
fn tour_des_creations(
    l: &mut [EnCreation],
    maintenant: Instant,
    dernier: &mut Instant,
    pas: Duration,
    seuil: Duration,
) -> Vec<String> {
    let ecart = maintenant.saturating_duration_since(*dernier);
    *dernier = maintenant;
    if est_une_veille(ecart, pas) {
        for c in l.iter_mut() {
            c.debut = maintenant;
            c.signalee = false;
        }
        return Vec::new();
    }
    lignes_des_creations(l, maintenant, seuil)
}

/// Lance l'unique fil du chien de garde des créations (une seule fois pour le programme). Il
/// regarde souvent (un tiers du seuil, entre 20 ms et 1 s) et ne fait rien d'autre.
fn demarrer_le_chien_des_creations() {
    static UNE_FOIS: OnceLock<()> = OnceLock::new();
    UNE_FOIS.get_or_init(|| {
        let _ = std::thread::Builder::new()
            .name("chien-des-creations".into())
            .spawn(|| {
                let mut dernier = Instant::now();
                loop {
                    let seuil = seuil_de_creation();
                    let pas = (seuil / 3).clamp(Duration::from_millis(20), Duration::from_secs(1));
                    std::thread::sleep(pas);
                    let lignes = {
                        let mut l = CREATIONS.lock().unwrap_or_else(PoisonError::into_inner);
                        tour_des_creations(&mut l, Instant::now(), &mut dernier, pas, seuil)
                    };
                    for ligne in lignes {
                        noter(ligne);
                    }
                }
            });
    });
}

#[cfg(test)]
mod essais {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Les essais qui touchent à l'état global (journal, fils inscrits, délais) se suivent, avec
    /// des délais courts : la surveillance réelle n'est lancée qu'une fois pour le programme.
    static GLOBAL: Mutex<()> = Mutex::new(());

    fn courts() {
        regler(Delais {
            pas: Duration::from_millis(50),
            blocage: Duration::from_millis(200),
        });
    }

    fn tour(s: &mut Surveillance, t: Instant, fils: &[Arc<Fil>]) -> Vec<String> {
        let mut v = Vec::new();
        s.tour(t, fils, &mut |l| v.push(l));
        v
    }

    #[test]
    fn un_fil_bloque_donne_une_ligne_puis_une_de_reprise() {
        let t0 = Instant::now();
        let f = Fil::nouveau("icône", None);
        f.battre();
        f.poser("Shell_NotifyIcon");
        let fils = [f.clone()];
        let mut s = Surveillance::new(Duration::from_secs(15));
        assert!(tour(&mut s, t0, &fils).is_empty());
        assert!(tour(&mut s, t0 + Duration::from_secs(5), &fils).is_empty());
        assert!(tour(&mut s, t0 + Duration::from_secs(10), &fils).is_empty());
        let l = tour(&mut s, t0 + Duration::from_secs(15), &fils);
        assert_eq!(l.len(), 1, "{l:?}");
        assert!(l[0].contains("« icône »") && l[0].contains("15 s"), "{l:?}");
        assert!(l[0].contains("Shell_NotifyIcon"), "{l:?}");
        // Une seule ligne par blocage, même longtemps après.
        assert!(tour(&mut s, t0 + Duration::from_secs(20), &fils).is_empty());
        assert!(tour(&mut s, t0 + Duration::from_secs(7800), &fils).is_empty());
        f.battre();
        let l = tour(&mut s, t0 + Duration::from_secs(7805), &fils);
        assert_eq!(l.len(), 1, "{l:?}");
        assert!(l[0].contains("repart") && l[0].contains("7805 s"), "{l:?}");
        // Reparti : rien de plus tant qu'il bat.
        f.battre();
        assert!(tour(&mut s, t0 + Duration::from_secs(7810), &fils).is_empty());
    }

    #[test]
    fn un_fil_dans_une_boucle_imbriquee_est_dit_comme_tel() {
        let t0 = Instant::now();
        let f = Fil::nouveau("icône", None);
        f.battre();
        let fils = [f.clone()];
        let mut s = Surveillance::new(Duration::from_secs(15));
        assert!(tour(&mut s, t0, &fils).is_empty());
        let avant = f.entrer("TrackPopupMenu", t0 + Duration::from_secs(2));
        assert_eq!(avant, None);
        let l = tour(&mut s, t0 + Duration::from_secs(20), &fils);
        assert_eq!(l.len(), 1, "{l:?}");
        assert!(l[0].contains("dans « TrackPopupMenu » depuis 18 s"), "{l:?}");
        // Sortie de la boucle imbriquée : l'état d'avant est rendu.
        f.sortir(avant);
        assert_eq!(f.etat().imbriquee, None);
    }

    #[test]
    fn un_fil_qui_avance_n_ecrit_rien() {
        let t0 = Instant::now();
        let f = Fil::nouveau("fenêtres", None);
        let g = Fil::nouveau("icône", None);
        let fils = [f.clone(), g.clone()];
        let mut s = Surveillance::new(Duration::from_secs(15));
        for i in 0..40u64 {
            f.battre();
            g.battre();
            assert!(tour(&mut s, t0 + Duration::from_secs(5 * i), &fils).is_empty());
        }
    }

    #[test]
    fn deux_fils_un_seul_bloque() {
        let t0 = Instant::now();
        let a = Fil::nouveau("icône", None);
        let b = Fil::nouveau("fenêtres", None);
        let fils = [a.clone(), b.clone()];
        let mut s = Surveillance::new(Duration::from_secs(15));
        let mut lignes = Vec::new();
        for i in 0..5u64 {
            a.battre();
            lignes.extend(tour(&mut s, t0 + Duration::from_secs(5 * i), &fils));
        }
        assert_eq!(lignes.len(), 1, "{lignes:?}");
        assert!(
            lignes[0].contains("« fenêtres »") && lignes[0].contains("aucune"),
            "{lignes:?}"
        );
    }

    /// La garde d'une boucle imbriquée écrit ses deux lignes et rend l'état ; un fil qui sort
    /// de la surveillance ne laisse rien derrière lui. (Le seul essai qui touche au journal
    /// global.)
    #[test]
    fn la_garde_ecrit_debut_et_fin_et_rend_l_etat() {
        let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
        courts();
        let lignes: Arc<Mutex<Vec<String>>> = Arc::default();
        let l2 = lignes.clone();
        brancher_le_journal(move |l| l2.lock().unwrap().push(l));
        let inscription = inscrire("essai", None);
        etape("avant le menu");
        {
            let _g = imbriquee("menu d'essai");
            assert_eq!(
                inscription.fil.etat().imbriquee.map(|(n, _)| n),
                Some("menu d'essai")
            );
        }
        assert_eq!(inscription.fil.etat().imbriquee, None);
        assert_eq!(inscription.fil.etat().etape, "avant le menu");
        battement();
        assert_eq!(inscription.fil.battements.load(Ordering::Acquire), 1);
        clic("droit");
        let v = lignes.lock().unwrap().clone();
        assert!(v.iter().any(|l| l.contains("« menu d'essai » : début")), "{v:?}");
        assert!(
            v.iter().any(|l| l.contains("« menu d'essai » : fin après")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|l| l.contains("clic droit reçu à") && l.ends_with("UTC.")),
            "{v:?}"
        );
        let fil = inscription.fil.clone();
        drop(inscription);
        assert!(!FILS.lock().unwrap().iter().any(|f| Arc::ptr_eq(f, &fil)));
        *SORTIE.write().unwrap() = None;
    }

    /// Un réveil qui reste pris pour toujours (un verrou jamais rendu) n'empêche pas la
    /// surveillance de dire le blocage d'un autre fil, et ne la fait jamais attendre.
    #[test]
    fn un_reveil_bloque_ne_bloque_pas_la_surveillance() {
        let t0 = Instant::now();
        let verrou: Arc<Mutex<()>> = Arc::default();
        let garde = verrou.lock().unwrap();
        let v2 = verrou.clone();
        let a = Fil::nouveau("fenêtres", Some(Box::new(move || drop(v2.lock()))));
        let b = Fil::nouveau("icône", None);
        b.battre();
        b.poser("attente d'un message");
        let fils = [a.clone(), b.clone()];
        let mut s = Surveillance::new(Duration::from_secs(15));
        let mut lignes = Vec::new();
        let debut = Instant::now();
        for i in 0..5u64 {
            a.battre();
            s.tour_et_reveils(t0 + Duration::from_secs(5 * i), &fils, &mut |l| lignes.push(l));
        }
        assert!(
            debut.elapsed() < Duration::from_secs(5),
            "la surveillance a attendu"
        );
        assert_eq!(lignes.len(), 1, "{lignes:?}");
        assert!(lignes[0].contains("« icône »"), "{lignes:?}");
        drop(garde);
    }

    /// La vraie boucle de surveillance, avec des délais courts : un fil inscrit qui ne bat
    /// plus est dit une fois, puis dit reparti. (Seul essai qui lance la boucle réelle.)
    #[test]
    fn la_boucle_reelle_dit_le_blocage_et_la_reprise() {
        let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
        courts();
        let lignes: Arc<Mutex<Vec<String>>> = Arc::default();
        let l2 = lignes.clone();
        brancher_le_journal(move |l| l2.lock().unwrap().push(l));
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let fil = std::thread::spawn(move || {
            let _i = inscrire("fil d'essai", None);
            battement();
            etape("étape d'essai");
            // Bloqué : plus de battement, jusqu'au signal.
            let _ = rx.recv();
            // Reparti : il bat de nouveau, jusqu'à ce que l'essai ferme le canal.
            while !matches!(rx.try_recv(), Err(std::sync::mpsc::TryRecvError::Disconnected)) {
                battement();
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        let attendre = |f: &dyn Fn(&[String]) -> bool| {
            for _ in 0..100 {
                if f(&lignes.lock().unwrap()) {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            false
        };
        assert!(attendre(&|v| v.iter().any(|l| l
            .contains("« fil d'essai » ne répond plus")
            && l.contains("étape d'essai"))));
        tx.send(()).unwrap();
        assert!(attendre(&|v| v
            .iter()
            .any(|l| l.contains("« fil d'essai » repart"))));
        drop(tx);
        fil.join().unwrap();
        let v = lignes.lock().unwrap().clone();
        let n = v
            .iter()
            .filter(|l| l.contains("« fil d'essai » ne répond plus"))
            .count();
        assert_eq!(n, 1, "{v:?}");
        *SORTIE.write().unwrap() = None;
    }

    #[test]
    fn l_heure_se_lit() {
        assert_eq!(
            heure_utc(UNIX_EPOCH + Duration::from_secs(14 * 3600 + 3 * 60 + 22)),
            "14:03:22"
        );
    }

    /// Le lieu d'une panique, au journal : les deux derniers éléments du chemin seulement
    /// (jamais le chemin du PC), quel que soit le séparateur.
    #[test]
    fn lieu_de_panique_sans_le_chemin_du_pc() {
        assert_eq!(
            lieu_court(
                r"C:\Users\exemple\.cargo\registry\eframe-0.37.0\src\native\glow_integration.rs",
                1162
            ),
            "native/glow_integration.rs:1162"
        );
        assert_eq!(lieu_court("src/volet.rs", 7), "src/volet.rs:7");
        assert_eq!(lieu_court("lib.rs", 1), "lib.rs:1");
    }

    /// Une panique attrapée : son lieu est gardé pour ce fil, et lu une seule fois.
    #[test]
    fn lieu_de_la_panique_garde_pour_le_fil() {
        retenir_les_paniques();
        let r = std::thread::spawn(|| {
            let _ = std::panic::catch_unwind(|| panic!("essai"));
            (lieu_de_la_derniere_panique(), lieu_de_la_derniere_panique())
        })
        .join()
        .unwrap();
        assert!(r.0.as_deref().is_some_and(|l| l.contains("sonde.rs:")), "{r:?}");
        assert_eq!(r.1, None);
    }

    /// Une création de fenêtre qui dure : une seule ligne, avec le nom et l'heure du début ;
    /// rien pour une création courte.
    #[test]
    fn creation_longue_donne_une_ligne() {
        let t0 = Instant::now();
        let utc = SystemTime::now();
        let mut l = vec![
            EnCreation {
                id: 1,
                nom: "Paramètres".into(),
                debut: t0,
                debut_utc: utc,
                signalee: false,
            },
            EnCreation {
                id: 2,
                nom: "Courte".into(),
                debut: t0 + Duration::from_secs(5),
                debut_utc: utc,
                signalee: false,
            },
        ];
        let seuil = Duration::from_secs(3);
        assert!(lignes_des_creations(&mut l, t0 + Duration::from_secs(2), seuil).is_empty());
        let v = lignes_des_creations(&mut l, t0 + Duration::from_secs(6), seuil);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(
            v[0].contains("« Paramètres »") && v[0].contains("depuis 6 s"),
            "{v:?}"
        );
        assert!(v[0].contains(&heure_utc(utc)) && v[0].contains("UTC"), "{v:?}");
        // Une seule ligne par création, même longtemps après.
        assert!(
            lignes_des_creations(&mut l, t0 + Duration::from_secs(60), seuil)
                .iter()
                .all(|x| !x.contains("Paramètres"))
        );
    }

    /// Le vrai fil : une création qui ne finit pas est écrite au journal, puis sa fin aussi.
    #[test]
    fn le_chien_ecrit_la_creation_qui_dure() {
        let _g = GLOBAL.lock().unwrap_or_else(PoisonError::into_inner);
        let lignes = Arc::new(Mutex::new(Vec::<String>::new()));
        let l2 = lignes.clone();
        brancher_le_journal(move |l| l2.lock().unwrap().push(l));
        regler_le_seuil_de_creation(Duration::from_millis(100));
        let c = creation("Fenêtre d'essai du chien");
        let attendre = |texte: &str| {
            let fin = Instant::now() + Duration::from_secs(5);
            while Instant::now() < fin {
                if lignes.lock().unwrap().iter().any(|l| l.contains(texte)) {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            false
        };
        assert!(
            attendre("sa création dure depuis"),
            "{:?}",
            lignes.lock().unwrap()
        );
        drop(c);
        assert!(attendre("s'est enfin terminée"), "{:?}", lignes.lock().unwrap());
        regler_le_seuil_de_creation(SEUIL_DE_CREATION);
    }

    /// Le PC dort : au réveil, la boucle de surveillance (qui a dormi aussi) ne signale aucun
    /// fil comme bloqué ; une seule ligne dit la reprise ; un vrai blocage ensuite est toujours
    /// signalé. L'heure est donnée à la main : aucune attente réelle.
    #[test]
    fn la_veille_du_pc_n_est_pas_un_blocage() {
        let t0 = Instant::now();
        let pas = Duration::from_secs(5);
        let sec = Duration::from_secs;
        let (a, b) = (Fil::nouveau("écrans", None), Fil::nouveau("placement", None));
        a.battre();
        b.battre();
        let fils = [a.clone(), b.clone()];
        let mut s = Surveillance::avec_pas(Duration::from_secs(15), pas);
        assert!(tour(&mut s, t0, &fils).is_empty());
        assert!(tour(&mut s, t0 + sec(5), &fils).is_empty());
        // Quatre heures de sommeil : les fils n'ont pas battu (le PC dormait).
        let reveil = t0 + sec(5) + sec(15_900);
        let l = tour(&mut s, reveil, &fils);
        assert_eq!(l.len(), 1, "{l:?}");
        assert!(l[0].contains("veille") && l[0].contains("15900 s"), "{l:?}");
        assert!(!l[0].contains("ne répond plus"), "{l:?}");
        // Les tours suivants : aucune alerte, aucune « reprise » de fil, comme un fil qui bat.
        for i in 1..=2 {
            a.battre();
            b.battre();
            assert!(tour(&mut s, reveil + sec(5 * i), &fils).is_empty());
        }
        // Un vrai blocage après la veille est signalé comme avant (15 s sans battement).
        a.battre();
        let t = reveil + sec(10);
        assert!(tour(&mut s, t + sec(5), &fils).is_empty());
        assert!(tour(&mut s, t + sec(10), &fils).is_empty());
        let l = tour(&mut s, t + sec(15), &fils);
        assert_eq!(l.len(), 1, "{l:?}");
        assert!(
            l[0].contains("« placement »") && l[0].contains("ne répond plus"),
            "{l:?}"
        );
    }

    /// Sans pas connu (`Surveillance::new`), un grand écart reste un blocage : l'horloge est
    /// celle de l'appelant.
    #[test]
    fn sans_pas_un_grand_ecart_reste_un_blocage() {
        let t0 = Instant::now();
        let f = Fil::nouveau("icône", None);
        f.battre();
        let fils = [f.clone()];
        let mut s = Surveillance::new(Duration::from_secs(15));
        assert!(tour(&mut s, t0, &fils).is_empty());
        let l = tour(&mut s, t0 + Duration::from_secs(15_900), &fils);
        assert_eq!(l.len(), 1, "{l:?}");
        assert!(l[0].contains("ne répond plus"), "{l:?}");
    }

    /// Une veille se reconnaît à un écart de plus de trois pas, et de plus de 3 s.
    #[test]
    fn reconnaitre_une_veille() {
        let ms = Duration::from_millis;
        assert!(!est_une_veille(Duration::from_secs(5), Duration::from_secs(5)));
        assert!(!est_une_veille(Duration::from_secs(15), Duration::from_secs(5)));
        assert!(est_une_veille(Duration::from_secs(16), Duration::from_secs(5)));
        // Un pas court (le chien des créations) : jamais moins de 3 s.
        assert!(!est_une_veille(ms(2_500), ms(100)));
        assert!(est_une_veille(ms(3_100), ms(100)));
    }

    /// Le chien des créations : une création qui dure vraiment est signalée ; après une veille du
    /// PC, elle repart de zéro, sans alerte.
    #[test]
    fn la_veille_n_allonge_pas_une_creation() {
        let t0 = Instant::now();
        let pas = Duration::from_secs(1);
        let seuil = Duration::from_secs(3);
        let sec = Duration::from_secs;
        let utc = SystemTime::now();
        let mut l = vec![EnCreation {
            id: 1,
            nom: "Paramètres".into(),
            debut: t0,
            debut_utc: utc,
            signalee: false,
        }];
        let mut dernier = t0;
        assert!(tour_des_creations(&mut l, t0 + sec(1), &mut dernier, pas, seuil).is_empty());
        // Le PC dort une heure : pas de ligne, la création repart de zéro.
        let reveil = t0 + sec(3_600);
        assert!(tour_des_creations(&mut l, reveil, &mut dernier, pas, seuil).is_empty());
        assert!(tour_des_creations(&mut l, reveil + sec(1), &mut dernier, pas, seuil).is_empty());
        assert!(tour_des_creations(&mut l, reveil + sec(2), &mut dernier, pas, seuil).is_empty());
        // Une vraie création qui dure plus que le seuil est signalée, une fois.
        let v = tour_des_creations(&mut l, reveil + sec(3), &mut dernier, pas, seuil);
        assert_eq!(v.len(), 1, "{v:?}");
        assert!(v[0].contains("« Paramètres »"), "{v:?}");
        assert!(tour_des_creations(&mut l, reveil + sec(4), &mut dernier, pas, seuil).is_empty());
    }
}
