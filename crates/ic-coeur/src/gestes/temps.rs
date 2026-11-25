//! L'instant, sans horloge : le cœur ne lit jamais l'heure (`tests/etages.rs`). Une décision
//! qui compte le temps reçoit l'instant en paramètre, d'un type qui sait dire le temps passé
//! depuis un autre ([`Horodatage`]) : `std::time::Instant` dans la bibliothèque, une
//! `Duration` comptée depuis une origine inventée dans les essais (une fausse horloge).

use std::time::{Duration, Instant};

/// Un instant : la bibliothèque passe un `Instant`, les essais une `Duration` depuis une
/// origine de leur choix.
pub trait Horodatage: Copy + Ord {
    /// Le temps passé depuis `avant` ; `None` si `avant` est après `self`.
    fn depuis(self, avant: Self) -> Option<Duration>;

    /// Le temps passé depuis `avant` ; zéro si `avant` est après `self` (comme
    /// `Instant::saturating_duration_since`).
    fn depuis_ou_zero(self, avant: Self) -> Duration {
        self.depuis(avant).unwrap_or_default()
    }
}

impl Horodatage for Instant {
    fn depuis(self, avant: Self) -> Option<Duration> {
        self.checked_duration_since(avant)
    }
}

impl Horodatage for Duration {
    fn depuis(self, avant: Self) -> Option<Duration> {
        self.checked_sub(avant)
    }
}
