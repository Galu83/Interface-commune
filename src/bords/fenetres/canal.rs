//! Le canal des demandes vers le fil des fenêtres : l'envoi ([`Demandes`], de tout fil), la
//! réception ([`Reception`], sur le fil des fenêtres) et le réveil entre deux fenêtres.

use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, SendError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use super::liens::Demande;
use super::verrou;

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
        *verrou(&self.sonne) = true;
        self.cv.notify_all();
    }

    pub fn attendre(&self, ms: u32) {
        let attente = std::time::Duration::from_millis(u64::from(ms));
        let (mut s, _) = self
            .cv
            .wait_timeout_while(verrou(&self.sonne), attente, |s| !*s)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *s = false;
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
        if let Some(c) = verrou(&self.lecteur).as_ref() {
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
        *verrou(&self.lecteur) = c;
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
