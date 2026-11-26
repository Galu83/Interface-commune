//! Le filet « clic en dehors » du volet et du petit menu ([`Veilleur`]), public pour un
//! logiciel qui a son propre volet.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::fenetres::{Appuis, fermer_sur_appui};
use crate::systeme;

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
///
/// Public pour un logiciel qui a **son propre volet ou menu** (sa fenêtre à lui) : il crée un
/// `Veilleur` avec ce qu'il faut faire pour fermer sa fenêtre, appelle [`Veilleur::activer`]
/// quand elle est montrée (et placée) et [`Veilleur::desactiver`] quand elle est cachée ou
/// fermée ; le `Veilleur` se retire seul quand il est détruit. `fermer` est appelée sur le fil
/// de veille : elle doit seulement demander la fermeture (lever un drapeau, réveiller la
/// fenêtre), jamais la faire elle-même.
///
/// ```no_run
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use interface_commune::volet::Veilleur;
///
/// let a_fermer = Arc::new(AtomicBool::new(false));
/// let drapeau = a_fermer.clone();
/// let veilleur = Veilleur::new(move || drapeau.store(true, Ordering::Release));
/// # let hwnd = 0isize;
/// // La fenêtre (son HWND) vient d'être montrée :
/// veilleur.activer(hwnd);
/// // À chaque image : `a_fermer.swap(false, Ordering::AcqRel)` dit de la fermer.
/// // Elle est cachée ou fermée :
/// veilleur.desactiver();
/// ```
pub struct Veilleur {
    veille: Arc<Veille>,
    fil: Option<std::thread::JoinHandle<()>>,
}

/// Le temps entre deux relevés de la souris (un clic dure bien plus longtemps).
pub const PAS_DE_VEILLE: Duration = Duration::from_millis(15);

impl Veilleur {
    /// Crée le veilleur (son fil dort tant que rien n'est surveillé). `fermer` : ce qu'il faut
    /// faire pour fermer la fenêtre quand un appui de souris tombe en dehors d'elle.
    pub fn new(fermer: impl Fn() + Send + 'static) -> Veilleur {
        let veille = Arc::new(Veille::default());
        let v2 = veille.clone();
        let fil = std::thread::Builder::new()
            .name("clic-en-dehors".into())
            .spawn(move || veiller_la_souris(&v2, &fermer))
            .ok();
        Veilleur { veille, fil }
    }

    /// La fenêtre `h` (son HWND) vient d'être montrée : surveiller les clics en dehors d'elle.
    /// Les appuis des [`GARDE`](crate::fenetres::GARDE) premières millisecondes, et un bouton déjà tenu, ne comptent pas.
    pub fn activer(&self, h: isize) {
        self.veille.fenetre.store(h, Ordering::Release);
        self.veille.generation.fetch_add(1, Ordering::AcqRel);
        if let Some(f) = &self.fil {
            f.thread().unpark();
        }
    }

    /// La fenêtre est fermée ou cachée : plus rien à surveiller.
    pub fn desactiver(&self) {
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

fn veiller_la_souris(veille: &Veille, fermer: &dyn Fn()) {
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
            fermer();
        }
        std::thread::sleep(PAS_DE_VEILLE);
    }
}
