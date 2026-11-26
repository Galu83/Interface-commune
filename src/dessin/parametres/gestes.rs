//! Les gestes des Paramètres rendus au logiciel : un changement (bulle « Annuler »), un
//! changement à confirmer (« Conserver ces modifications ? »), « Revenir », le délai écoulé
//! et la fermeture. Le cœur décide (`ic_coeur::parametres::envois`) ; ici, on passe au
//! logiciel ce qu'il a décidé, dans l'ordre.

use std::time::Instant;

#[cfg(doc)]
use super::Contexte;
use super::Parametres;
use crate::Reglages;
use crate::parametres::modele::{self, Changement, Valeur};
use ic_coeur::parametres::{Envoi, envois};

impl Parametres {
    /// Passe au logiciel, dans l'ordre, ce que le cœur a décidé de lui rendre.
    pub(super) fn envoyer(l: &dyn Reglages, envois: Vec<Envoi>) {
        for e in envois {
            match e {
                Envoi::Changer(cle, valeur) => l.changer(&cle, valeur),
                Envoi::Remettre(avant) => l.remettre(&avant),
            }
        }
    }

    /// Un changement : rendu au logiciel tout de suite, et la bulle « Annuler ».
    pub fn changer(&mut self, l: &dyn Reglages, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        let e = envois::changer(&mut self.annulation, cle, titre, avant, apres, Instant::now());
        Self::envoyer(l, e);
    }

    /// Un groupe de changements : tous rendus au logiciel, une seule bulle « Annuler ».
    pub fn changer_groupe(&mut self, l: &dyn Reglages, titre: &str, changements: Vec<Changement>) {
        let e = envois::changer_groupe(&mut self.annulation, titre, changements, Instant::now());
        Self::envoyer(l, e);
    }

    /// Un changement **à confirmer** : rendu au logiciel tout de suite, puis la carte
    /// « Conserver ces modifications ? » ([`Contexte::changer_a_confirmer`]).
    pub fn changer_a_confirmer(
        &mut self,
        l: &dyn Reglages,
        cle: &str,
        titre: &str,
        avant: Valeur,
        apres: Valeur,
    ) {
        let (an, co) = (&mut self.annulation, &mut self.confirmation);
        let e = envois::changer_a_confirmer(an, co, cle, titre, avant, apres, Instant::now());
        Self::envoyer(l, e);
    }

    /// Un groupe de changements **à confirmer**, d'un seul coup.
    pub fn changer_groupe_a_confirmer(
        &mut self,
        l: &dyn Reglages,
        titre: &str,
        changements: Vec<Changement>,
    ) {
        let (an, co) = (&mut self.annulation, &mut self.confirmation);
        let e = envois::changer_groupe_a_confirmer(an, co, titre, changements, Instant::now());
        Self::envoyer(l, e);
    }

    /// Un brouillon entier **à confirmer** : `avant` est son état avant ce changement (le
    /// logiciel l'a déjà modifié).
    pub fn brouillon_a_confirmer(&mut self, l: &dyn Reglages, titre: &str, avant: modele::Instantane) {
        let (an, co) = (&mut self.annulation, &mut self.confirmation);
        let e = envois::brouillon_a_confirmer(an, co, titre, avant, Instant::now());
        Self::envoyer(l, e);
    }

    /// « Conserver » : les changements restent.
    pub fn conserver(&mut self) {
        self.confirmation.conserver(Instant::now());
    }

    /// « Revenir » : tout revient comme avant le premier changement à confirmer. Appelé aussi
    /// par la fermeture des Paramètres ([`Parametres::fermer`]).
    pub fn revenir(&mut self, l: &dyn Reglages) {
        let e = envois::revenir(&mut self.annulation, &mut self.confirmation);
        Self::envoyer(l, e);
    }

    /// À appeler à chaque image (la bibliothèque le fait) : le délai de « Conserver » est-il
    /// écoulé ? Alors tout revient comme avant. Rend `true` si cela vient d'arriver.
    pub fn tic(&mut self, l: &dyn Reglages) -> bool {
        self.tic_a(l, Instant::now())
    }

    /// Comme [`Parametres::tic`], à une heure donnée (pour les essais : pas d'attente de 15 s).
    pub fn tic_a(&mut self, l: &dyn Reglages, maintenant: Instant) -> bool {
        match envois::echu(&mut self.annulation, &mut self.confirmation, maintenant) {
            Some(e) => {
                Self::envoyer(l, e);
                true
            }
            None => false,
        }
    }

    /// Les Paramètres se ferment : ce qui attendait « Conserver » revient comme avant
    /// ([`Parametres::revenir`]), avant que le logiciel enregistre ([`Reglages::peut_fermer`]).
    pub fn fermer(&mut self, l: &dyn Reglages) {
        self.revenir(l);
    }
}
