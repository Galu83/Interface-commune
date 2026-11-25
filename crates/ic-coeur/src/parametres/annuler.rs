//! La bulle « Annuler » (C-70) : chaque changement s'applique et s'enregistre tout de suite (le
//! logiciel s'en charge) ; la bulle garde la valeur d'avant [`DUREE_ANNULER`]. Deux changements
//! de suite du même réglage gardent l'« avant » du premier ; un brouillon entier revient d'un
//! coup. L'instant arrive en paramètre (`Instant` dans la bibliothèque, une fausse horloge dans
//! les essais).

use std::time::{Duration, Instant};

use crate::contrat::{Instantane, Valeur};
use crate::gestes::Horodatage;

/// Un changement d'un groupe : la clé, la valeur d'avant, celle d'après.
pub type Changement = (String, Valeur, Valeur);

/// La bulle « Annuler » reste ce temps après un changement.
pub const DUREE_ANNULER: Duration = Duration::from_secs(6);

/// Le dernier changement, que la bulle « Annuler » peut défaire.
#[derive(Debug, Clone, PartialEq)]
pub struct Annulable<T = Instant> {
    pub cle: String,
    /// Le nom du réglage, montré dans la bulle (« Thème changé »).
    pub titre: String,
    pub avant: Valeur,
    /// Les autres réglages d'un groupe de changements (clé, valeur d'avant), défaits avec lui.
    pub autres: Vec<(String, Valeur)>,
    /// Un brouillon entier : son état d'avant (la clé et `avant` ne servent alors pas).
    pub instantane: Option<Instantane>,
    quand: T,
}

/// La bulle « Annuler » : le dernier changement, tant qu'il peut être défait.
#[derive(Debug)]
pub struct Annulation<T = Instant> {
    annulable: Option<Annulable<T>>,
}

impl<T> Default for Annulation<T> {
    fn default() -> Self {
        Annulation { annulable: None }
    }
}

impl<T: Horodatage> Annulation<T> {
    /// La bulle de ce changement est-elle encore montrée ?
    fn vivante(a: &Annulable<T>, maintenant: T) -> bool {
        maintenant.depuis_ou_zero(a.quand) < DUREE_ANNULER
    }

    /// L'utilisateur vient de changer `cle` de `avant` à `apres`. Plusieurs changements de
    /// suite du même réglage (un curseur, une frappe) ne font qu'un : « Annuler » revient à
    /// la valeur d'avant le premier.
    pub fn changer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: &Valeur, maintenant: T) {
        let suite = self
            .annulable
            .as_ref()
            .is_some_and(|a| a.cle == cle && Self::vivante(a, maintenant));
        if suite {
            if let Some(a) = &mut self.annulable {
                a.quand = maintenant;
                if a.avant == *apres {
                    // Revenu à la valeur de départ : plus rien à annuler.
                    self.annulable = None;
                }
            }
        } else if avant != *apres {
            self.annulable = Some(Annulable {
                cle: cle.to_string(),
                titre: titre.to_string(),
                avant,
                autres: Vec::new(),
                instantane: None,
                quand: maintenant,
            });
        }
    }

    /// Un brouillon entier vient d'être changé : `avant` est son état au début de l'image.
    /// Plusieurs changements de suite (une frappe) ne font qu'un : « Annuler » revient à l'état
    /// d'avant le premier.
    pub fn changer_brouillon(&mut self, titre: &str, avant: Instantane, maintenant: T) {
        if let Some(a) = &mut self.annulable
            && a.instantane.is_some()
            && Self::vivante(a, maintenant)
        {
            a.quand = maintenant;
            return;
        }
        self.annulable = Some(Annulable {
            cle: String::new(),
            titre: titre.to_string(),
            avant: Valeur::Booleen(false),
            autres: Vec::new(),
            instantane: Some(avant),
            quand: maintenant,
        });
    }

    /// « Annuler » est cliqué sur un brouillon entier : son état d'avant, à rendre au logiciel
    /// (`Reglages::remettre`). Rien si la bulle n'est pas celle d'un brouillon, ou a déjà
    /// disparu.
    pub fn annuler_brouillon(&mut self, maintenant: T) -> Option<Instantane> {
        let est_brouillon = self.a_annuler(maintenant).is_some_and(|a| a.instantane.is_some());
        if !est_brouillon {
            return None;
        }
        self.annulable.take().and_then(|a| a.instantane)
    }

    /// Plusieurs réglages changés d'un seul geste (clé, avant, après) : une seule bulle, et
    /// « Annuler » les remet tous. Ceux qui ne changent rien sont oubliés.
    pub fn changer_groupe(&mut self, titre: &str, changements: Vec<Changement>, maintenant: T) {
        let mut avants = changements
            .into_iter()
            .filter(|(_, avant, apres)| avant != apres)
            .map(|(cle, avant, _)| (cle, avant));
        let Some((cle, avant)) = avants.next() else {
            return;
        };
        self.annulable = Some(Annulable {
            cle,
            titre: titre.to_string(),
            avant,
            autres: avants.collect(),
            instantane: None,
            quand: maintenant,
        });
    }

    /// Ce que la bulle propose, tant qu'elle est montrée.
    pub fn a_annuler(&self, maintenant: T) -> Option<&Annulable<T>> {
        self.annulable.as_ref().filter(|a| Self::vivante(a, maintenant))
    }

    /// Le temps qu'il reste à la bulle (pour redessiner quand elle disparaît).
    pub fn reste(&self, maintenant: T) -> Option<Duration> {
        self.a_annuler(maintenant)
            .map(|a| DUREE_ANNULER.saturating_sub(maintenant.depuis_ou_zero(a.quand)))
    }

    /// « Annuler » est cliqué : rend les clés et les valeurs d'avant, à remettre par le
    /// logiciel (une seule, sauf pour un groupe). Rien si la bulle a déjà disparu.
    pub fn annuler(&mut self, maintenant: T) -> Vec<(String, Valeur)> {
        match self.annulable.take() {
            Some(a) if a.instantane.is_some() => {
                // Un brouillon entier ne se défait que par `annuler_brouillon`.
                self.annulable = Some(a);
                Vec::new()
            }
            Some(a) if Self::vivante(&a, maintenant) => {
                let mut tout = vec![(a.cle, a.avant)];
                tout.extend(a.autres);
                tout
            }
            _ => Vec::new(),
        }
    }

    /// La bulle est fermée sans annuler.
    pub fn oublier(&mut self) {
        self.annulable = None;
    }
}
