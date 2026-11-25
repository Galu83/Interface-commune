//! Ce que les gestes dans les Paramètres rendent au logiciel, dans l'ordre : un changement
//! (`Reglages::changer`), un brouillon entier remis (`Reglages::remettre`). La bulle
//! « Annuler » et « Conserver ? » se tiennent ensemble : un changement à confirmer ferme la
//! bulle ; « Revenir », le délai écoulé ou la fermeture remettent le brouillon d'abord, puis
//! chaque valeur d'avant. Et le bandeau « Redémarrer maintenant » (C-78).
//!
//! Chaque fonction décide et rend la liste [`Envoi`] ; le dessin (bloc H) la passe au logiciel
//! telle quelle.

use super::annuler::{Annulation, Changement};
use super::conserver::{Confirmation, Retour};
use crate::contrat::{Instantane, Valeur};
use crate::gestes::Horodatage;

/// Une chose à rendre au logiciel.
#[derive(Debug, Clone, PartialEq)]
pub enum Envoi {
    /// `Reglages::changer(clé, valeur)` : effet immédiat, ou retour à la valeur d'avant.
    Changer(String, Valeur),
    /// `Reglages::remettre(instantané)` : le brouillon entier revient comme avant.
    Remettre(Instantane),
}

impl Retour {
    /// Ce que ce retour rend au logiciel : le brouillon d'avant d'abord, puis chaque valeur
    /// d'avant, par le même chemin qu'« Annuler ».
    pub fn envois(self) -> Vec<Envoi> {
        let mut v: Vec<Envoi> = self.brouillon.into_iter().map(Envoi::Remettre).collect();
        v.extend(self.valeurs.into_iter().map(|(c, a)| Envoi::Changer(c, a)));
        v
    }
}

/// Un changement : rendu au logiciel tout de suite, et la bulle « Annuler ».
pub fn changer<T: Horodatage>(
    annulation: &mut Annulation<T>,
    cle: &str,
    titre: &str,
    avant: Valeur,
    apres: Valeur,
    maintenant: T,
) -> Vec<Envoi> {
    annulation.changer(cle, titre, avant, &apres, maintenant);
    vec![Envoi::Changer(cle.to_string(), apres)]
}

/// Les changements d'un groupe qui changent quelque chose, à rendre au logiciel.
fn effets(changements: &[Changement]) -> Vec<Envoi> {
    changements
        .iter()
        .filter(|(_, avant, apres)| avant != apres)
        .map(|(cle, _, apres)| Envoi::Changer(cle.clone(), apres.clone()))
        .collect()
}

/// Un groupe de changements : tous rendus au logiciel, une seule bulle « Annuler ».
pub fn changer_groupe<T: Horodatage>(
    annulation: &mut Annulation<T>,
    titre: &str,
    changements: Vec<Changement>,
    maintenant: T,
) -> Vec<Envoi> {
    let v = effets(&changements);
    annulation.changer_groupe(titre, changements, maintenant);
    v
}

/// « Annuler » est cliqué : le brouillon d'avant, ou les valeurs d'avant (rien si la bulle a
/// déjà disparu).
pub fn annuler<T: Horodatage>(annulation: &mut Annulation<T>, maintenant: T) -> Vec<Envoi> {
    let mut v: Vec<Envoi> = annulation
        .annuler_brouillon(maintenant)
        .into_iter()
        .map(Envoi::Remettre)
        .collect();
    v.extend(
        annulation
            .annuler(maintenant)
            .into_iter()
            .map(|(c, a)| Envoi::Changer(c, a)),
    );
    v
}

/// À chaque image : le délai de « Conserver » est-il écoulé ? Alors tout revient comme avant,
/// et la bulle se ferme. `None` si rien ne vient d'arriver.
pub fn echu<T: Horodatage>(
    annulation: &mut Annulation<T>,
    confirmation: &mut Confirmation<T>,
    maintenant: T,
) -> Option<Vec<Envoi>> {
    let retour = confirmation.echu(maintenant)?;
    annulation.oublier();
    Some(retour.envois())
}

/// « Revenir » (ou la fermeture des Paramètres) : tout revient comme avant le premier
/// changement à confirmer, et la bulle se ferme.
pub fn revenir<T: Horodatage>(
    annulation: &mut Annulation<T>,
    confirmation: &mut Confirmation<T>,
) -> Vec<Envoi> {
    let v = confirmation.revenir().envois();
    annulation.oublier();
    v
}

/// Un changement **à confirmer** : un délai écoulé d'abord ([`echu`]) ; puis, s'il change
/// quelque chose, la bulle se ferme, la carte « Conserver ces modifications ? » s'ouvre (ou
/// continue), et le changement est rendu au logiciel tout de suite.
pub fn changer_a_confirmer<T: Horodatage>(
    annulation: &mut Annulation<T>,
    confirmation: &mut Confirmation<T>,
    cle: &str,
    titre: &str,
    avant: Valeur,
    apres: Valeur,
    maintenant: T,
) -> Vec<Envoi> {
    let mut v = echu(annulation, confirmation, maintenant).unwrap_or_default();
    if avant == apres {
        return v;
    }
    annulation.oublier();
    confirmation.changer(cle, titre, avant, &apres, maintenant);
    v.push(Envoi::Changer(cle.to_string(), apres));
    v
}

/// Un groupe de changements **à confirmer**, d'un seul coup ([`changer_a_confirmer`]).
pub fn changer_groupe_a_confirmer<T: Horodatage>(
    annulation: &mut Annulation<T>,
    confirmation: &mut Confirmation<T>,
    titre: &str,
    changements: Vec<Changement>,
    maintenant: T,
) -> Vec<Envoi> {
    let mut v = echu(annulation, confirmation, maintenant).unwrap_or_default();
    if changements.iter().all(|(_, avant, apres)| avant == apres) {
        return v;
    }
    annulation.oublier();
    v.extend(effets(&changements));
    confirmation.changer_groupe(titre, changements, maintenant);
    v
}

/// Un brouillon entier **à confirmer** : `avant` est son état avant ce changement (le
/// logiciel l'a déjà modifié ; rien de plus à lui rendre, sauf un délai écoulé).
pub fn brouillon_a_confirmer<T: Horodatage>(
    annulation: &mut Annulation<T>,
    confirmation: &mut Confirmation<T>,
    titre: &str,
    avant: Instantane,
    maintenant: T,
) -> Vec<Envoi> {
    let v = echu(annulation, confirmation, maintenant).unwrap_or_default();
    annulation.oublier();
    confirmation.changer_brouillon(titre, avant, maintenant);
    v
}

/// C-78 : une page demande un redémarrage (`raison`) ; le bandeau « Redémarrer maintenant » la
/// montre en haut de chaque page jusqu'au redémarrage. La dernière raison demandée l'emporte ;
/// une image sans demande ne l'efface pas. Le bouton du bandeau envoie l'action
/// [`crate::contrat::cles::REDEMARRER`].
pub fn noter_redemarrage(bandeau: &mut Option<String>, demande: Option<String>) {
    if let Some(r) = demande {
        *bandeau = Some(r);
    }
}
