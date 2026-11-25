//! « Conserver ces modifications ? » : un réglage qui peut couper l'écran, la souris, le
//! clavier ou la liaison est appliqué tout de suite (le logiciel s'en charge), puis la carte
//! demande de le garder ; sans réponse en [`DUREE_CONSERVER`], tout revient comme avant (comme
//! Windows après un changement de résolution d'écran). **Une seule confirmation à la fois** :
//! un nouveau changement pendant le compte à rebours garde l'état d'avant le **premier**, et le
//! compte à rebours repart de zéro.

use std::time::{Duration, Instant};

use super::annuler::Changement;
use crate::contrat::{Instantane, Valeur};
use crate::gestes::Horodatage;

/// « Conserver ces modifications ? » attend la réponse ce temps ; sans réponse, retour au
/// réglage d'avant.
pub const DUREE_CONSERVER: Duration = Duration::from_secs(15);

/// Ce qui attend la réponse « Conserver » ou « Revenir ».
#[derive(Debug, Clone, PartialEq)]
pub struct AConfirmer<T = Instant> {
    /// Le nom du réglage changé en dernier, montré dans la carte (« Mode d'affichage »).
    pub titre: String,
    /// Les réglages changés (clé, valeur d'**avant le premier changement**), dans l'ordre où
    /// ils ont changé.
    pub valeurs: Vec<(String, Valeur)>,
    /// Un brouillon entier : son état d'avant le premier changement.
    pub brouillon: Option<Instantane>,
    /// Le dernier changement : le compte à rebours repart de là.
    quand: T,
}

/// Ce que « Revenir » (ou le délai écoulé, ou la fermeture) rend au logiciel : le brouillon
/// d'avant, à remettre d'abord par `Reglages::remettre`, puis les valeurs d'avant, à remettre
/// par `Reglages::changer` (le même chemin que « Annuler ») ; voir [`super::Envoi`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Retour {
    pub valeurs: Vec<(String, Valeur)>,
    pub brouillon: Option<Instantane>,
}

impl Retour {
    /// Rien à remettre.
    pub fn est_vide(&self) -> bool {
        self.valeurs.is_empty() && self.brouillon.is_none()
    }
}

/// « Conserver ces modifications ? » : ce qui attend la réponse, une seule à la fois.
#[derive(Debug)]
pub struct Confirmation<T = Instant> {
    attente: Option<AConfirmer<T>>,
}

impl<T> Default for Confirmation<T> {
    fn default() -> Self {
        Confirmation { attente: None }
    }
}

impl<T: Horodatage> Confirmation<T> {
    /// Ouvre la confirmation, ou la continue (le titre du dernier changement ; compte à
    /// rebours remis à [`DUREE_CONSERVER`]).
    fn continuer(&mut self, titre: &str, maintenant: T) -> &mut AConfirmer<T> {
        let a = self.attente.get_or_insert_with(|| AConfirmer {
            titre: String::new(),
            valeurs: Vec::new(),
            brouillon: None,
            quand: maintenant,
        });
        a.titre = titre.to_string();
        a.quand = maintenant;
        a
    }

    /// Garde la valeur d'avant d'une clé (celle d'avant le premier changement) ; revenue à
    /// cette valeur, la clé n'a plus rien à confirmer.
    fn noter(&mut self, cle: &str, avant: Valeur, apres: &Valeur) {
        let Some(a) = &mut self.attente else { return };
        if !a.valeurs.iter().any(|(k, _)| k == cle) {
            a.valeurs.push((cle.to_string(), avant));
        }
        if let Some(i) = a.valeurs.iter().position(|(k, v)| k == cle && v == apres) {
            a.valeurs.remove(i);
        }
    }

    /// Plus rien à confirmer : la carte disparaît.
    fn nettoyer(&mut self) {
        if self
            .attente
            .as_ref()
            .is_some_and(|a| a.valeurs.is_empty() && a.brouillon.is_none())
        {
            self.attente = None;
        }
    }

    /// La carte attend-elle encore la réponse ?
    fn vivante(a: &AConfirmer<T>, maintenant: T) -> bool {
        maintenant.depuis_ou_zero(a.quand) < DUREE_CONSERVER
    }

    /// Le réglage `cle` vient de passer de `avant` à `apres`, et doit être confirmé.
    /// Appeler [`Confirmation::echu`] avant (la bibliothèque le fait).
    pub fn changer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: &Valeur, maintenant: T) {
        if avant == *apres {
            return;
        }
        self.continuer(titre, maintenant);
        self.noter(cle, avant, apres);
        self.nettoyer();
    }

    /// Plusieurs réglages changés d'un seul geste (clé, avant, après), à confirmer d'un seul
    /// coup. Ceux qui ne changent rien sont oubliés.
    pub fn changer_groupe(&mut self, titre: &str, changements: Vec<Changement>, maintenant: T) {
        let changements: Vec<Changement> = changements.into_iter().filter(|(_, a, b)| a != b).collect();
        if changements.is_empty() {
            return;
        }
        self.continuer(titre, maintenant);
        for (cle, avant, apres) in changements {
            self.noter(&cle, avant, &apres);
        }
        self.nettoyer();
    }

    /// Un brouillon entier vient d'être changé : `avant` est son état avant ce changement.
    /// Pendant une confirmation déjà ouverte, l'état d'avant le premier est gardé.
    pub fn changer_brouillon(&mut self, titre: &str, avant: Instantane, maintenant: T) {
        let a = self.continuer(titre, maintenant);
        if a.brouillon.is_none() {
            a.brouillon = Some(avant);
        }
    }

    /// La confirmation attendue, tant que le temps n'est pas écoulé.
    pub fn en_attente(&self, maintenant: T) -> Option<&AConfirmer<T>> {
        self.attente.as_ref().filter(|a| Self::vivante(a, maintenant))
    }

    /// Le temps qu'il reste avant le retour automatique.
    pub fn reste(&self, maintenant: T) -> Option<Duration> {
        self.en_attente(maintenant)
            .map(|a| DUREE_CONSERVER.saturating_sub(maintenant.depuis_ou_zero(a.quand)))
    }

    /// Les secondes montrées (15, 14… 1) : le temps qui reste, arrondi au-dessus.
    pub fn secondes(&self, maintenant: T) -> Option<u64> {
        self.reste(maintenant)
            .map(|r| r.as_secs() + u64::from(r.subsec_nanos() > 0))
    }

    /// Dans combien de temps la carte change de seconde (pour redessiner juste à ce
    /// moment-là, pas plus souvent).
    pub fn prochain_changement(&self, maintenant: T) -> Option<Duration> {
        self.reste(maintenant).map(|r| {
            let dans_la_seconde = Duration::from_nanos(u64::from(r.subsec_nanos()));
            let pas = if dans_la_seconde.is_zero() {
                Duration::from_secs(1)
            } else {
                dans_la_seconde
            };
            // 5 ms de marge : on redessine juste après le changement de seconde.
            pas + Duration::from_millis(5)
        })
    }

    /// « Conserver » : les changements restent. `false` si rien n'attendait, ou si le délai
    /// était déjà écoulé (le retour automatique l'emporte : voir [`Confirmation::echu`]).
    pub fn conserver(&mut self, maintenant: T) -> bool {
        if self.en_attente(maintenant).is_none() {
            return false;
        }
        self.attente = None;
        true
    }

    /// « Revenir » (ou la fermeture des Paramètres) : tout ce qui attendait, avec les valeurs
    /// d'avant le premier changement, à rendre au logiciel. Vide si rien n'attendait.
    pub fn revenir(&mut self) -> Retour {
        match self.attente.take() {
            Some(a) => Retour {
                valeurs: a.valeurs,
                brouillon: a.brouillon,
            },
            None => Retour::default(),
        }
    }

    /// Le délai est écoulé : le retour automatique, à rendre au logiciel comme « Revenir ».
    /// `None` tant qu'il reste du temps (et quand rien n'attend).
    pub fn echu(&mut self, maintenant: T) -> Option<Retour> {
        let ecoule = self
            .attente
            .as_ref()
            .is_some_and(|a| !Self::vivante(a, maintenant));
        ecoule.then(|| self.revenir())
    }
}
