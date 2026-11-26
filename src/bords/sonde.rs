//! La sonde des fils, côté bords (bloc O de l'organigramme) : les compteurs de battements, la
//! dernière étape de chaque fil, le journal branché, le fil du chien de garde. La décision
//! « bloqué », la veille du PC, les créations longues et le texte de chaque ligne sont dans le
//! cœur (`ic_coeur::gestes::sonde`), testés sans horloge ni fil ; ici, on lit l'heure et les
//! compteurs, on les passe au cœur et on écrit ce qu'il rend.
//!
//! Repris tel quel à `interface_commune::sonde` (`src/sonde.rs`), qui garde la documentation
//! pour les logiciels.
//!
//! - [`chien`] : la surveillance des fils et le chien des créations de fenêtre ;
//! - [`panique`] : le lieu de la dernière panique de chaque fil.

use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use ic_coeur::gestes::sonde as coeur;

mod chien;
mod panique;

pub use chien::{Creation, Surveillance, creation};
pub use ic_coeur::gestes::sonde::{Delais, SEUIL_DE_CREATION, est_une_veille};
pub use panique::{etape_du_fil, lieu_de_la_derniere_panique, retenir_les_paniques};

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

/// Écrit une ligne au journal branché, s'il y en a un. Le verrou est rendu avant l'appel : le
/// journal peut lui-même noter, ou être rebranché, sans attendre.
pub(crate) fn noter(ligne: String) {
    let s = SORTIE.read().unwrap_or_else(PoisonError::into_inner).clone();
    if let Some(s) = s {
        s(ligne);
    }
}

/// L'heure de maintenant en secondes UTC (le cœur n'a pas d'horloge).
fn secondes_utc(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Un clic reçu par l'icône (`"gauche"`, `"droit"`, `"double"`…), avec l'heure.
pub fn clic(quel: &'static str) {
    noter(coeur::ligne_du_clic(quel, secondes_utc(SystemTime::now())));
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

    /// Ce que le cœur lit de ce fil à un tour ; la clé est l'adresse du fil, unique tant qu'il
    /// est inscrit.
    fn releve(self: &Arc<Fil>) -> coeur::Releve<usize, Instant> {
        let e = self.etat();
        coeur::Releve {
            cle: Arc::as_ptr(self) as usize,
            nom: self.nom,
            battements: self.battements.load(Ordering::Acquire),
            etape: e.etape,
            imbriquee: e.imbriquee,
        }
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
    chien::demarrer_la_surveillance();
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
    noter(coeur::debut_d_imbriquee(nom));
    Imbriquee { nom, debut, fil }
}

impl Drop for Imbriquee {
    fn drop(&mut self) {
        if let Some((f, avant)) = self.fil.take() {
            f.sortir(avant);
        }
        noter(coeur::fin_d_imbriquee(self.nom, self.debut.elapsed()));
    }
}

#[cfg(test)]
mod essais;
