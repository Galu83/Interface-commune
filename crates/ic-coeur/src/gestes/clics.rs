//! Les gestes sur l'icône près de l'horloge, en calcul pur (règles : interface Windows 11,
//! point 10) : clic gauche, le volet ; clic droit, touche menu ou Maj+F10, le petit menu ;
//! double-clic, ouvrir le logiciel (sa fenêtre principale s'il en a une, sinon les
//! Paramètres) ; Entrée ou Espace, le volet ; le clic du milieu ne fait rien ; chaque action
//! part au relâchement du bouton. Avec `FenetrePrincipale::clic_gauche`, le clic gauche (et
//! Entrée ou Espace) ouvre la fenêtre principale au lieu du volet ([`suite`]).
//!
//! L'instant arrive en paramètre ([`Horodatage`]) : `Instant` dans la bibliothèque
//! (`interface_commune::clics::Clics`), une fausse horloge dans les essais.

use std::time::Duration;

use super::temps::Horodatage;
use crate::contrat::FenetrePrincipale;

/// Ce que Windows dit d'un geste sur l'icône (`NOTIFYICON_VERSION_4`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evenement {
    /// Clic gauche relâché (`NIN_SELECT`).
    Selection,
    /// Entrée ou Espace sur l'icône (`NIN_KEYSELECT`).
    Clavier,
    /// Clic droit relâché, touche menu ou Maj+F10 (`WM_CONTEXTMENU`).
    MenuContextuel,
    /// Deuxième appui d'un double-clic (`WM_LBUTTONDBLCLK`).
    DoubleClic,
    /// Bouton gauche relâché (`WM_LBUTTONUP`).
    Relache,
}

impl Evenement {
    /// Le nom du geste pour la sonde des fils et le journal.
    pub fn nom(self) -> &'static str {
        match self {
            Evenement::Selection => "gauche",
            Evenement::Clavier => "clavier",
            Evenement::MenuContextuel => "droit",
            Evenement::DoubleClic => "double",
            Evenement::Relache => "relâché",
        }
    }
}

/// Le geste reconnu, avant de savoir ce que le logiciel a ([`suite`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geste {
    Rien,
    Volet,
    Menu,
    /// Double-clic : ouvrir le logiciel.
    Ouvrir,
}

/// Ce que le logiciel fait du geste, selon qu'il a une fenêtre principale ou non.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    Rien,
    Volet,
    Menu,
    Parametres,
    FenetrePrincipale,
}

/// Ce que fait le geste `g` : le double-clic ouvre la fenêtre principale s'il y en a une,
/// sinon les Paramètres ; le clic gauche ouvre le volet, ou la fenêtre principale avec
/// `clic_gauche`.
pub fn suite(g: Geste, principale: Option<FenetrePrincipale>) -> Suite {
    match g {
        Geste::Rien => Suite::Rien,
        Geste::Menu => Suite::Menu,
        Geste::Ouvrir if principale.is_some() => Suite::FenetrePrincipale,
        Geste::Ouvrir => Suite::Parametres,
        Geste::Volet if principale.is_some_and(|p| p.clic_gauche) => Suite::FenetrePrincipale,
        Geste::Volet => Suite::Volet,
    }
}

/// Le comportement de l'icône, en calcul pur (testé) : clic gauche, le volet ; clic droit,
/// le petit menu ; double-clic, ouvrir le logiciel, au relâchement du deuxième clic ; Entrée ou
/// Espace, le volet. Le clic du milieu n'arrive jamais ici : il ne fait rien. `T` : l'instant
/// ([`Horodatage`]).
#[derive(Debug)]
pub struct Clics<T> {
    /// Un double-clic attend son relâchement.
    double_en_attente: bool,
    /// Le dernier double-clic fait : le `NIN_SELECT` qui le suit ne rouvre pas le volet.
    double_fait: Option<T>,
    /// La dernière touche Entrée ou Espace : Windows peut envoyer deux fois `NIN_KEYSELECT`
    /// pour un seul appui sur Entrée ; le second ne doit pas refermer le volet.
    clavier: Option<T>,
}

// À la main : `derive(Default)` demanderait `T: Default`, qu'`Instant` n'a pas.
impl<T> Default for Clics<T> {
    fn default() -> Self {
        Clics {
            double_en_attente: false,
            double_fait: None,
            clavier: None,
        }
    }
}

/// Deux `NIN_KEYSELECT` plus proches que ce temps sont un seul appui.
const MEME_TOUCHE: Duration = Duration::from_millis(300);

impl<T: Horodatage> Clics<T> {
    pub fn recevoir(&mut self, e: Evenement, maintenant: T, double: Duration) -> Geste {
        let recent = |t: Option<T>, d: Duration| t.is_some_and(|t| maintenant.depuis_ou_zero(t) < d);
        match e {
            Evenement::DoubleClic => {
                self.double_en_attente = true;
                Geste::Rien
            }
            Evenement::Relache | Evenement::Selection if self.double_en_attente => {
                self.double_en_attente = false;
                self.double_fait = Some(maintenant);
                Geste::Ouvrir
            }
            Evenement::Relache => Geste::Rien,
            Evenement::Selection if recent(self.double_fait, double) => Geste::Rien,
            Evenement::Selection => Geste::Volet,
            Evenement::Clavier if recent(self.clavier, MEME_TOUCHE) => Geste::Rien,
            Evenement::Clavier => {
                self.clavier = Some(maintenant);
                Geste::Volet
            }
            Evenement::MenuContextuel => Geste::Menu,
        }
    }
}

/// Le temps du double-clic le plus court et le plus long que la bibliothèque prend de Windows
/// (millisecondes) : un réglage hors de ces bornes (registre abîmé) ne rend jamais le
/// double-clic impossible, ni chaque clic double.
pub const DOUBLE_CLIC_MIN_MS: u32 = 100;
pub const DOUBLE_CLIC_MAX_MS: u32 = 5_000;

/// Le temps du double-clic, d'après celui que Windows donne (bloc L, C-112), ramené entre
/// [`DOUBLE_CLIC_MIN_MS`] et [`DOUBLE_CLIC_MAX_MS`].
pub fn temps_double_clic(windows_ms: u32) -> Duration {
    Duration::from_millis(u64::from(
        windows_ms.clamp(DOUBLE_CLIC_MIN_MS, DOUBLE_CLIC_MAX_MS),
    ))
}

/// Les numéros que Windows donne aux gestes sur l'icône (`NOTIFYICON_VERSION_4`).
const NIN_SELECT: u32 = 0x400;
const NIN_KEYSELECT: u32 = 0x401;
const WM_CONTEXTMENU: u32 = 0x007B;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;

/// L'événement de l'icône (le mot bas du `lparam` du message de l'icône), s'il en est un
/// qui compte. Le clic du milieu et le survol n'en sont pas.
pub fn evenement(lp: isize) -> Option<Evenement> {
    match (lp as u32) & 0xFFFF {
        NIN_SELECT => Some(Evenement::Selection),
        NIN_KEYSELECT => Some(Evenement::Clavier),
        WM_CONTEXTMENU => Some(Evenement::MenuContextuel),
        WM_LBUTTONDBLCLK => Some(Evenement::DoubleClic),
        WM_LBUTTONUP => Some(Evenement::Relache),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// La fausse horloge des essais : le temps depuis une origine inventée.
    type Faux = Clics<Duration>;

    /// Règles (interface Windows 11, point 10) : clic gauche, le volet ; clic droit ou touche
    /// menu, le petit menu ; Entrée ou Espace, le volet.
    #[test]
    fn clic_gauche_droit_et_clavier() {
        let mut c = Faux::default();
        let t = Duration::from_secs(1_000);
        let d = Duration::from_millis(500);
        assert_eq!(c.recevoir(Evenement::Relache, t, d), Geste::Rien);
        assert_eq!(c.recevoir(Evenement::Selection, t, d), Geste::Volet);
        assert_eq!(c.recevoir(Evenement::MenuContextuel, t, d), Geste::Menu);
        let t2 = t + Duration::from_secs(2);
        assert_eq!(c.recevoir(Evenement::Selection, t2, d), Geste::Volet);
        // Entrée : Windows peut envoyer deux NIN_KEYSELECT ; un seul volet.
        let t3 = t + Duration::from_secs(4);
        assert_eq!(c.recevoir(Evenement::Clavier, t3, d), Geste::Volet);
        assert_eq!(
            c.recevoir(Evenement::Clavier, t3 + Duration::from_millis(20), d),
            Geste::Rien
        );
        assert_eq!(
            c.recevoir(Evenement::Clavier, t3 + Duration::from_secs(1), d),
            Geste::Volet
        );
        assert_eq!(evenement(0x400), Some(Evenement::Selection));
        assert_eq!(evenement(0x401), Some(Evenement::Clavier));
        assert_eq!(
            evenement(WM_CONTEXTMENU as isize),
            Some(Evenement::MenuContextuel)
        );
        assert_eq!(evenement(WM_LBUTTONDBLCLK as isize), Some(Evenement::DoubleClic));
        // Le clic du milieu ne fait rien.
        assert_eq!(evenement(0x0208), None);
    }

    /// Double-clic : ouvrir le logiciel, au relâchement du deuxième clic ; le `NIN_SELECT` qui
    /// suit ne rouvre pas le volet. Que Windows envoie ou non `WM_LBUTTONUP` d'abord.
    #[test]
    fn double_clic_les_parametres() {
        let d = Duration::from_millis(500);
        for avec_relache in [true, false] {
            let mut c = Faux::default();
            let t = Duration::from_secs(1_000);
            let ms = |n| t + Duration::from_millis(n);
            assert_eq!(c.recevoir(Evenement::Selection, t, d), Geste::Volet);
            assert_eq!(c.recevoir(Evenement::DoubleClic, ms(150), d), Geste::Rien);
            if avec_relache {
                assert_eq!(c.recevoir(Evenement::Relache, ms(200), d), Geste::Ouvrir);
                assert_eq!(c.recevoir(Evenement::Selection, ms(201), d), Geste::Rien);
            } else {
                assert_eq!(c.recevoir(Evenement::Selection, ms(201), d), Geste::Ouvrir);
            }
            // Plus tard, un clic simple ouvre de nouveau le volet.
            assert_eq!(c.recevoir(Evenement::Selection, ms(2_000), d), Geste::Volet);
        }
    }

    /// Sans fenêtre principale, rien ne change : double-clic, les Paramètres ; clic gauche,
    /// le volet. Avec : le double-clic l'ouvre ; avec `clic_gauche`, le clic gauche aussi.
    #[test]
    fn suite_selon_la_fenetre_principale() {
        assert_eq!(suite(Geste::Ouvrir, None), Suite::Parametres);
        assert_eq!(suite(Geste::Volet, None), Suite::Volet);
        assert_eq!(suite(Geste::Menu, None), Suite::Menu);
        assert_eq!(suite(Geste::Rien, None), Suite::Rien);
        let p = Some(FenetrePrincipale::default());
        assert_eq!(suite(Geste::Ouvrir, p), Suite::FenetrePrincipale);
        assert_eq!(suite(Geste::Volet, p), Suite::Volet);
        assert_eq!(suite(Geste::Menu, p), Suite::Menu);
        let g = Some(FenetrePrincipale { clic_gauche: true });
        assert_eq!(suite(Geste::Volet, g), Suite::FenetrePrincipale);
        assert_eq!(suite(Geste::Ouvrir, g), Suite::FenetrePrincipale);
        assert_eq!(suite(Geste::Menu, g), Suite::Menu);
    }

    /// Le temps du double-clic de Windows est pris tel quel entre 100 ms et 5 s, ramené
    /// aux bornes au-delà.
    #[test]
    fn temps_double_clic_borne() {
        assert_eq!(temps_double_clic(500), Duration::from_millis(500));
        assert_eq!(temps_double_clic(100), Duration::from_millis(100));
        assert_eq!(temps_double_clic(0), Duration::from_millis(100));
        assert_eq!(temps_double_clic(5_000), Duration::from_secs(5));
        assert_eq!(temps_double_clic(u32::MAX), Duration::from_secs(5));
    }

    /// Le nom de chaque geste pour la sonde (les mots qu'écrivait l'horloge).
    #[test]
    fn nom_des_gestes_pour_la_sonde() {
        let noms: Vec<_> = [
            Evenement::Selection,
            Evenement::Clavier,
            Evenement::MenuContextuel,
            Evenement::DoubleClic,
            Evenement::Relache,
        ]
        .map(Evenement::nom)
        .into();
        assert_eq!(noms, ["gauche", "clavier", "droit", "double", "relâché"]);
    }
}
