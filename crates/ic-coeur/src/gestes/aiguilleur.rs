//! L'aiguillage unique des gestes sur une icône, en calcul pur : le suivi des clics par
//! icône ([`SuiviDesIcones`] : double-clic, Entrée en double…), puis l'ordre qui en découle
//! ([`ordre`]). L'icône unique (`interface_commune::horloge`) et les icônes d'une zone
//! (`interface_commune::zone::Aiguilleur`) passent toutes deux par ici ; elles n'exécutent
//! que l'ordre (envoyer la demande, ouvrir la fenêtre principale, le menu de secours).

use std::collections::BTreeMap;
use std::time::Duration;

use super::clics::{Clics, Evenement, Geste, Suite, suite};
use super::ouverture::Ouvert;
use super::temps::Horodatage;
use crate::contrat::FenetrePrincipale;
use crate::placement::Rect;
use crate::theme::Surface;

/// Le nombre d'icônes suivies à la fois (largement assez : la zone de notification n'en
/// montre pas autant). Au-delà, le suivi des icônes retirées est oublié.
pub const ICONES_MAX: usize = 256;

/// Le suivi des clics de chaque icône, par son nom : un double-clic sur une icône ne se mêle
/// jamais à un clic sur une autre. `T` : l'instant ([`Horodatage`]).
#[derive(Debug)]
pub struct SuiviDesIcones<T> {
    clics: BTreeMap<String, Clics<T>>,
}

// À la main : `derive(Default)` demanderait `T: Default`, qu'`Instant` n'a pas.
impl<T> Default for SuiviDesIcones<T> {
    fn default() -> Self {
        SuiviDesIcones {
            clics: BTreeMap::new(),
        }
    }
}

impl<T: Horodatage> SuiviDesIcones<T> {
    /// Le geste reconnu pour l'événement `e` sur l'icône `icone`, reçu à `maintenant` ;
    /// `double` : le temps d'un double-clic de Windows.
    pub fn recevoir(&mut self, icone: &str, e: Evenement, maintenant: T, double: Duration) -> Geste {
        if self.clics.len() >= ICONES_MAX && !self.clics.contains_key(icone) {
            // Des icônes retirées depuis longtemps : leur suivi est oublié.
            self.clics.clear();
        }
        self.clics
            .entry(icone.to_string())
            .or_default()
            .recevoir(e, maintenant, double)
    }

    /// Le nombre d'icônes suivies.
    pub fn nombre(&self) -> usize {
        self.clics.len()
    }
}

/// Ce que l'icône doit faire d'un geste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordre {
    Rien,
    /// Les Paramètres, à leur première page (double-clic sans fenêtre principale) : le volet
    /// ouvert par le premier clic lit la demande, se cache, puis leur laisse la place.
    Parametres,
    /// La fenêtre principale du logiciel ; le volet ou le petit menu ouvert par le premier
    /// clic d'un double-clic se ferme d'abord ([`fermer_avant_la_fenetre_principale`]).
    FenetrePrincipale,
    /// Ouvrir (ou refermer) le volet, ou le petit menu à son point `ancre` (pixels).
    Surface {
        surface: Surface,
        ancre: Option<[i32; 2]>,
    },
}

/// L'ordre que donne le geste `g`, selon que le logiciel a une fenêtre principale ou non
/// ([`suite`]) ; `ancre` : le point du geste, où s'ouvre le petit menu.
pub fn ordre(g: Geste, principale: Option<FenetrePrincipale>, ancre: [i32; 2]) -> Ordre {
    match suite(g, principale) {
        Suite::Rien => Ordre::Rien,
        Suite::Parametres => Ordre::Parametres,
        Suite::FenetrePrincipale => Ordre::FenetrePrincipale,
        Suite::Volet => Ordre::Surface {
            surface: Surface::Volet,
            ancre: None,
        },
        Suite::Menu => Ordre::Surface {
            surface: Surface::Menu,
            ancre: Some(ancre),
        },
    }
}

/// Avant d'ouvrir la fenêtre principale : faut-il fermer ce qui est ouvert ? Le volet ou le
/// petit menu, oui (il lui laisse la place) ; les Paramètres, non.
pub fn fermer_avant_la_fenetre_principale(ouvert: Ouvert) -> bool {
    matches!(ouvert, Ouvert::Surface(_))
}

/// Le point du menu de secours de Windows : le point du petit menu, ou le coin haut gauche de
/// l'icône pour le volet.
pub fn point_de_secours(ancre: Option<[i32; 2]>, icone: Rect) -> [i32; 2] {
    ancre.unwrap_or([icone[0], icone[1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_secs(1_000) + Duration::from_millis(n)
    }

    const DOUBLE: Duration = Duration::from_millis(500);

    /// Un double-clic se reconnaît icône par icône : deux clics sur deux icônes ne font pas
    /// un double-clic.
    #[test]
    fn double_clic_par_icone() {
        let mut s = SuiviDesIcones::<Duration>::default();
        assert_eq!(s.recevoir("a", Evenement::Selection, ms(0), DOUBLE), Geste::Volet);
        assert_eq!(
            s.recevoir("b", Evenement::DoubleClic, ms(100), DOUBLE),
            Geste::Rien
        );
        assert_eq!(s.recevoir("a", Evenement::Relache, ms(150), DOUBLE), Geste::Rien);
        assert_eq!(
            s.recevoir("b", Evenement::Relache, ms(160), DOUBLE),
            Geste::Ouvrir
        );
        assert_eq!(
            s.recevoir("b", Evenement::Selection, ms(170), DOUBLE),
            Geste::Rien
        );
        assert_eq!(
            s.recevoir("a", Evenement::Selection, ms(180), DOUBLE),
            Geste::Volet
        );
        assert_eq!(s.nombre(), 2);
    }

    /// Au-delà d'[`ICONES_MAX`] icônes, le suivi est oublié avant d'en ajouter une ; une icône
    /// déjà suivie ne l'efface pas.
    #[test]
    fn suivi_borne() {
        let mut s = SuiviDesIcones::<Duration>::default();
        for i in 0..ICONES_MAX {
            s.recevoir(&format!("i{i}"), Evenement::Relache, ms(0), DOUBLE);
        }
        assert_eq!(s.nombre(), ICONES_MAX);
        s.recevoir("i0", Evenement::Relache, ms(1), DOUBLE);
        assert_eq!(s.nombre(), ICONES_MAX);
        s.recevoir("nouvelle", Evenement::Relache, ms(2), DOUBLE);
        assert_eq!(s.nombre(), 1);
    }

    /// Le geste devient un ordre : le volet sans point, le petit menu au point du geste ; le
    /// double-clic, la fenêtre principale ou les Paramètres.
    #[test]
    fn ordres_des_gestes() {
        let a = [40, 50];
        let volet = Ordre::Surface {
            surface: Surface::Volet,
            ancre: None,
        };
        let menu = Ordre::Surface {
            surface: Surface::Menu,
            ancre: Some(a),
        };
        assert_eq!(ordre(Geste::Rien, None, a), Ordre::Rien);
        assert_eq!(ordre(Geste::Volet, None, a), volet);
        assert_eq!(ordre(Geste::Menu, None, a), menu);
        assert_eq!(ordre(Geste::Ouvrir, None, a), Ordre::Parametres);
        let p = Some(FenetrePrincipale::default());
        assert_eq!(ordre(Geste::Ouvrir, p, a), Ordre::FenetrePrincipale);
        assert_eq!(ordre(Geste::Volet, p, a), volet);
        let g = Some(FenetrePrincipale { clic_gauche: true });
        assert_eq!(ordre(Geste::Volet, g, a), Ordre::FenetrePrincipale);
        assert_eq!(ordre(Geste::Menu, g, a), menu);
    }

    #[test]
    fn fermer_puis_secours() {
        assert!(fermer_avant_la_fenetre_principale(Ouvert::Surface(
            Surface::Volet
        )));
        assert!(fermer_avant_la_fenetre_principale(Ouvert::Surface(Surface::Menu)));
        assert!(!fermer_avant_la_fenetre_principale(Ouvert::Parametres));
        assert!(!fermer_avant_la_fenetre_principale(Ouvert::Rien));
        let icone = [10, 20, 34, 44];
        assert_eq!(point_de_secours(None, icone), [10, 20]);
        assert_eq!(point_de_secours(Some([7, 8]), icone), [7, 8]);
    }
}
