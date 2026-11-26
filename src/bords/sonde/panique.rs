//! Les paniques attrapées : le lieu (fichier court et ligne) de la dernière panique de chaque
//! fil, gardé pour le journal ; jamais le chemin du PC (`ic_coeur::gestes::sonde::lieu_court`).

use std::cell::RefCell;

use ic_coeur::gestes::sonde::lieu_court;

use super::COURANT;

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
