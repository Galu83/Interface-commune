//! Le chien de garde de la sonde : l'unique fil de surveillance des fils inscrits, et celui
//! des créations de fenêtre. Chacun lit l'heure et l'état, demande au cœur
//! (`ic_coeur::gestes::sonde`) ce qu'il faut en dire, et l'écrit au journal.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant, SystemTime};

use ic_coeur::gestes::sonde as coeur;

use super::{FILS, Fil, delais, noter, secondes_utc};

// ---------------------------------------------------------------------------------------
// La surveillance des fils

/// La surveillance des fils : la décision « bloqué » du cœur, nourrie des compteurs des
/// [`Fil`]s. L'heure est donnée à chaque tour.
pub struct Surveillance {
    decision: coeur::Surveillance<usize, Instant>,
}

impl Surveillance {
    /// Une surveillance sans pas connu : un grand écart entre deux tours reste un blocage.
    pub fn new(blocage: Duration) -> Surveillance {
        Surveillance {
            decision: coeur::Surveillance::new(blocage),
        }
    }

    /// Comme [`Surveillance::new`], pour une boucle qui tourne tous les `pas` : un écart bien
    /// plus long entre deux tours est une veille du PC ([`coeur::est_une_veille`]) ; les repères
    /// de tous les fils repartent de zéro, sans alerte, avec une seule ligne au journal.
    pub fn avec_pas(blocage: Duration, pas: Duration) -> Surveillance {
        Surveillance {
            decision: coeur::Surveillance::avec_pas(blocage, pas),
        }
    }

    /// Lit les compteurs de `fils` à `maintenant` ; `ecrire` reçoit les lignes à journaliser.
    pub(super) fn tour(&mut self, maintenant: Instant, fils: &[Arc<Fil>], ecrire: &mut dyn FnMut(String)) {
        let releves: Vec<_> = fils.iter().map(Fil::releve).collect();
        for c in self.decision.tour(maintenant, &releves) {
            ecrire(c.ligne());
        }
    }

    /// Un tour, puis le signal de réveil de chaque fil (il a un pas entier pour battre avant le
    /// tour suivant). Le signal n'attend jamais : un réveil bloqué n'arrête que lui.
    pub(super) fn tour_et_reveils(
        &mut self,
        maintenant: Instant,
        fils: &[Arc<Fil>],
        ecrire: &mut dyn FnMut(String),
    ) {
        self.tour(maintenant, fils, ecrire);
        for f in fils {
            if let Some(r) = &f.reveil {
                let _ = r.try_send(());
            }
        }
    }
}

/// Lance l'unique fil de surveillance (une seule fois pour le programme).
pub(super) fn demarrer_la_surveillance() {
    static UNE_FOIS: OnceLock<()> = OnceLock::new();
    UNE_FOIS.get_or_init(|| {
        let d = delais();
        let _ = std::thread::Builder::new().name("sonde".into()).spawn(move || {
            let mut s = Surveillance::avec_pas(d.blocage, d.pas);
            let mut avant = Instant::now();
            loop {
                std::thread::sleep(d.pas);
                let maintenant = Instant::now();
                if coeur::est_une_veille(maintenant.saturating_duration_since(avant), d.pas) {
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

/// Le seuil en millisecondes (les essais le raccourcissent).
static SEUIL_CREATION_MS: AtomicU64 = AtomicU64::new(3_000);

#[cfg(test)]
pub(super) fn regler_le_seuil_de_creation(seuil: Duration) {
    SEUIL_CREATION_MS.store(
        u64::try_from(seuil.as_millis()).unwrap_or(u64::MAX),
        Ordering::Release,
    );
}

fn seuil_de_creation() -> Duration {
    Duration::from_millis(SEUIL_CREATION_MS.load(Ordering::Acquire))
}

static CREATIONS: Mutex<Vec<coeur::EnCreation<Instant>>> = Mutex::new(Vec::new());
static PROCHAINE_CREATION: AtomicU64 = AtomicU64::new(1);

/// Une création de fenêtre en cours : à garder pendant tout l'appel qui crée la fenêtre.
/// Si l'appel dure plus de [`coeur::SEUIL_DE_CREATION`] (il ne rend jamais la main : un
/// blocage dans la création de la fenêtre ou de son contexte OpenGL, sans panique ni erreur),
/// un fil à part écrit au journal le nom de la fenêtre et l'heure du début ; puis une ligne
/// quand l'appel finit, avec la durée. Le nom est celui que le logiciel a donné à sa fenêtre.
pub struct Creation {
    id: u64,
}

/// Commence la surveillance de la création de la fenêtre `nom`.
pub fn creation(nom: &str) -> Creation {
    let id = PROCHAINE_CREATION.fetch_add(1, Ordering::AcqRel);
    CREATIONS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(coeur::EnCreation {
            id,
            nom: nom.to_string(),
            debut: Instant::now(),
            debut_utc: secondes_utc(SystemTime::now()),
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
        if let Some(ligne) = fini.and_then(|c| coeur::fin_de_creation(&c, Instant::now())) {
            noter(ligne);
        }
    }
}

/// Lance l'unique fil du chien de garde des créations (une seule fois pour le programme). Il
/// regarde souvent ([`coeur::pas_des_creations`]) et ne fait rien d'autre.
fn demarrer_le_chien_des_creations() {
    static UNE_FOIS: OnceLock<()> = OnceLock::new();
    UNE_FOIS.get_or_init(|| {
        let _ = std::thread::Builder::new()
            .name("chien-des-creations".into())
            .spawn(|| {
                let mut dernier = Instant::now();
                loop {
                    let seuil = seuil_de_creation();
                    let pas = coeur::pas_des_creations(seuil);
                    std::thread::sleep(pas);
                    let lignes = {
                        let mut l = CREATIONS.lock().unwrap_or_else(PoisonError::into_inner);
                        coeur::tour_des_creations(&mut l, Instant::now(), &mut dernier, pas, seuil)
                    };
                    for ligne in lignes {
                        noter(ligne);
                    }
                }
            });
    });
}
