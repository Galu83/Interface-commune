//! Les fenêtres perdues, en calcul pur : compter les pertes récentes, reconnaître une rafale,
//! décider quand recréer (tout de suite, ou au clic suivant avec le menu de Windows pour le
//! clic en cours), et la ligne du journal. La bibliothèque (`interface_commune::fenetres`)
//! garde la boucle, la panique attrapée et les fenêtres ; elle reprend tout ceci à ses
//! chemins.

use std::time::Duration;

use super::temps::Horodatage;

/// Au-delà de ce nombre de pertes de la fenêtre du volet en [`RAFALE`], elle n'est plus
/// recréée tout de suite : le clic reçoit le menu de secours, et elle est recréée au clic
/// suivant.
pub const RECREATIONS_EN_RAFALE: usize = 3;
/// La fenêtre de temps des pertes en rafale.
pub const RAFALE: Duration = Duration::from_secs(60);

/// Les pertes récentes des fenêtres (la fenêtre du volet, ou toute la boucle des fenêtres).
/// `T` : l'instant ([`Horodatage`]).
#[derive(Debug, Clone)]
pub struct Pertes<T>(Vec<T>);

// À la main : `derive(Default)` demanderait `T: Default`, qu'`Instant` n'a pas.
impl<T> Default for Pertes<T> {
    fn default() -> Self {
        Pertes(Vec::new())
    }
}

impl<T: Horodatage> Pertes<T> {
    /// Compte une perte ; rend le nombre de pertes dans la dernière [`RAFALE`].
    pub fn compter(&mut self, maintenant: T) -> usize {
        self.0.retain(|t| maintenant.depuis_ou_zero(*t) < RAFALE);
        self.0.push(maintenant);
        self.0.len()
    }
}

impl<T> Pertes<T> {
    /// Le nombre de pertes gardées (sans en compter une de plus).
    pub fn nombre(&self) -> usize {
        self.0.len()
    }

    /// `n` pertes dans la dernière [`RAFALE`] font-elles une rafale (plus de
    /// [`RECREATIONS_EN_RAFALE`]) ?
    pub fn en_rafale(n: usize) -> bool {
        n > RECREATIONS_EN_RAFALE
    }
}

/// Pourquoi la boucle des fenêtres s'est arrêtée alors qu'elle devait durer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Perte {
    /// Une panique attrapée (par exemple eframe qui ne peut plus rendre son contexte OpenGL
    /// courant au réveil des écrans), avec sa cause pour le journal
    /// (`interface_commune::fenetres::cause_de_panique`).
    Panique(String),
    /// Elle s'est fermée d'elle-même (par Windows).
    Fermee,
    /// Elle n'a pas pu être recréée, alors qu'elle a déjà été lancée : un échec passager
    /// (écrans en veille, carte graphique qui se relance).
    Echec(String),
}

/// Quand les fenêtres perdues sont recréées.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reprise {
    /// Tout de suite ; le clic qui s'ouvrait est refait par les fenêtres recréées.
    Maintenant,
    /// Pertes en rafale (ou échec passager) : le menu de Windows pour le clic en cours, et
    /// les fenêtres sont recréées au clic suivant. Jamais de boucle folle, jamais bloqué
    /// jusqu'au prochain lancement.
    AuClicSuivant,
}

/// Quand recréer après `perte`, et le nombre de pertes dans la dernière [`RAFALE`]. Un échec
/// passager n'est pas une perte de plus : un nouvel essai au clic suivant, sans lire l'heure.
/// Sinon la perte est comptée (`maintenant` n'est appelé qu'alors) : au-delà de
/// [`RECREATIONS_EN_RAFALE`], au clic suivant ; sinon tout de suite.
pub fn decider_la_reprise<T: Horodatage>(
    perte: &Perte,
    pertes: &mut Pertes<T>,
    maintenant: impl FnOnce() -> T,
) -> (usize, Reprise) {
    if let Perte::Echec(_) = perte {
        return (pertes.nombre(), Reprise::AuClicSuivant);
    }
    let n = pertes.compter(maintenant());
    let reprise = if Pertes::<T>::en_rafale(n) {
        Reprise::AuClicSuivant
    } else {
        Reprise::Maintenant
    };
    (n, reprise)
}

/// Une perte des fenêtres, telle que la supervision la donne au logiciel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evenement {
    pub perte: Perte,
    /// Le nombre de pertes dans la dernière [`RAFALE`], celle-ci comprise.
    pub nombre: usize,
    pub reprise: Reprise,
}

impl Evenement {
    /// La ligne du journal (avec la cause, pour une panique).
    pub fn phrase(&self) -> String {
        let quoi = match &self.perte {
            Perte::Panique(cause) => format!("Fenêtres perdues ({cause})"),
            Perte::Fermee => "Fenêtres fermées par Windows".into(),
            Perte::Echec(e) => format!("Fenêtres impossibles à recréer ({e})"),
        };
        match (&self.perte, self.reprise) {
            (_, Reprise::Maintenant) => format!("{quoi} : recréées."),
            (Perte::Echec(_), Reprise::AuClicSuivant) => {
                format!("{quoi} : menu de Windows pour ce clic, nouvel essai au clic suivant.")
            }
            (_, Reprise::AuClicSuivant) => format!(
                "{quoi} : {} fois en {} s ; menu de Windows pour ce clic, recréées au clic suivant.",
                self.nombre,
                RAFALE.as_secs()
            ),
        }
    }
}

/// Comment finit la supervision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fin {
    /// Le logiciel se ferme.
    Quitter,
    /// La boucle n'a jamais pu être créée (ni OpenGL ni fenêtre possibles) : le menu de
    /// Windows pour de bon.
    Impossible(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(n: u64) -> Duration {
        Duration::from_secs(1_000 + n)
    }

    /// Les pertes se comptent sur la dernière minute ; plus de trois font une rafale.
    #[test]
    fn pertes_sur_la_derniere_rafale() {
        let mut p = Pertes::<Duration>::default();
        assert_eq!(p.nombre(), 0);
        assert_eq!(p.compter(s(0)), 1);
        assert_eq!(p.compter(s(10)), 2);
        assert_eq!(p.compter(s(20)), 3);
        assert!(!Pertes::<Duration>::en_rafale(3));
        assert_eq!(p.compter(s(30)), 4);
        assert!(Pertes::<Duration>::en_rafale(4));
        // Une minute après la première, elle ne compte plus.
        assert_eq!(p.compter(s(60)), 4);
        assert_eq!(p.nombre(), 4);
        // Une horloge qui recule ne fait rien oublier.
        assert_eq!(p.compter(s(5)), 5);
    }

    /// Un échec passager : au clic suivant, sans compter ni lire l'heure. Une panique ou une
    /// fermeture : tout de suite, puis au clic suivant au-delà de trois en une minute.
    #[test]
    fn reprise_selon_la_perte() {
        let mut p = Pertes::<Duration>::default();
        let echec = Perte::Echec("écrans en veille".into());
        let heure_lue = std::cell::Cell::new(0);
        let horloge = |t| {
            heure_lue.set(heure_lue.get() + 1);
            t
        };
        assert_eq!(
            decider_la_reprise(&echec, &mut p, || horloge(s(0))),
            (0, Reprise::AuClicSuivant)
        );
        assert_eq!(heure_lue.get(), 0, "un échec ne lit pas l'heure");
        let panique = Perte::Panique("contexte perdu".into());
        for n in 1..=3 {
            assert_eq!(
                decider_la_reprise(&panique, &mut p, || horloge(s(n))),
                (n as usize, Reprise::Maintenant)
            );
        }
        assert_eq!(
            decider_la_reprise(&Perte::Fermee, &mut p, || horloge(s(4))),
            (4, Reprise::AuClicSuivant)
        );
        assert_eq!(heure_lue.get(), 4);
        assert_eq!(
            decider_la_reprise(&echec, &mut p, || horloge(s(5))),
            (4, Reprise::AuClicSuivant)
        );
        assert_eq!(heure_lue.get(), 4);
    }

    /// Les lignes du journal, mot pour mot.
    #[test]
    fn phrases_des_pertes() {
        let e = |perte, nombre, reprise| Evenement {
            perte,
            nombre,
            reprise,
        };
        assert_eq!(
            e(Perte::Panique("p".into()), 1, Reprise::Maintenant).phrase(),
            "Fenêtres perdues (p) : recréées."
        );
        assert_eq!(
            e(Perte::Fermee, 4, Reprise::AuClicSuivant).phrase(),
            "Fenêtres fermées par Windows : 4 fois en 60 s ; menu de Windows pour ce clic, recréées au clic suivant."
        );
        assert_eq!(
            e(Perte::Echec("x".into()), 0, Reprise::AuClicSuivant).phrase(),
            "Fenêtres impossibles à recréer (x) : menu de Windows pour ce clic, nouvel essai au clic suivant."
        );
    }
}
