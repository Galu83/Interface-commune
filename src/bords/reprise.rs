//! Le dessin des fenêtres attend la fin d'un réveil de Windows.
//!
//! Au sortir d'une veille, Windows peut reprendre sans personne devant le PC (réveil
//! automatique : réseau, minuterie) et l'écran reste éteint ; le pilote graphique, lui, se
//! remet en route. Une image OpenGL faite à ce moment (la fenêtre principale réveillée pour
//! ouvrir, fermer ou redessiner une autre fenêtre) a bloqué le fil des fenêtres dans
//! `SwapBuffers` pendant plus d'une minute. Ici :
//! - pendant la veille et tant que l'écran est éteint, aucune image complète n'est demandée ;
//! - à la reprise (ou quand l'écran se rallume), le dessin attend encore [`APRES_REVEIL`].
//!
//! Les demandes ne sont pas perdues : elles sont reprises par un réveil différé. Une ligne au
//! journal dit combien de temps le dessin a attendu.
//!
//! Le fil de l'icône signale la veille et l'écran (`WM_POWERBROADCAST`) ; la sonde signale
//! aussi une reprise qu'elle constate (un long trou dans sa boucle), au cas où Windows n'aurait
//! rien envoyé.

use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::bords::verrou::verrou;

/// Après la reprise ou le retour de l'écran, le temps laissé à Windows et au pilote graphique
/// avant la première image.
pub const APRES_REVEIL: Duration = Duration::from_secs(2);

/// Pendant la veille ou l'écran éteint, une demande de dessin est revue à cet intervalle.
pub const REVUE: Duration = Duration::from_secs(1);

/// Pourquoi le dessin attend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Raison {
    Veille,
    EcranEteint,
    Reprise,
}

impl Raison {
    fn texte(self) -> &'static str {
        match self {
            Raison::Veille => "le PC se met en veille",
            Raison::EcranEteint => "l'écran est éteint",
            Raison::Reprise => "Windows sort de veille",
        }
    }
}

/// L'état du réveil de Windows. Pur : l'heure est toujours donnée par l'appelant.
#[derive(Debug, Default, Clone)]
pub struct Reprise {
    en_veille: bool,
    ecran_eteint: bool,
    /// L'heure de la dernière reprise (ou du dernier retour de l'écran).
    reveil: Option<Instant>,
    /// Le dessin attend depuis cette heure, pour cette raison (la première demande retenue).
    attente: Option<(Instant, Raison)>,
}

impl Reprise {
    /// Windows va se mettre en veille (`PBT_APMSUSPEND`).
    pub fn veille(&mut self) {
        self.en_veille = true;
    }

    /// Windows sort de veille (`PBT_APMRESUMEAUTOMATIC`, `PBT_APMRESUMESUSPEND`), ou la sonde
    /// l'a constaté.
    pub fn reveil(&mut self, maintenant: Instant) {
        self.en_veille = false;
        self.reveil = Some(maintenant);
    }

    /// L'écran s'allume ou s'éteint (`GUID_CONSOLE_DISPLAY_STATE` ; l'écran assombri compte
    /// pour allumé). Un écran qui se rallume compte comme une reprise.
    pub fn ecran(&mut self, allume: bool, maintenant: Instant) {
        if allume && self.ecran_eteint {
            self.reveil = Some(maintenant);
        }
        self.ecran_eteint = !allume;
    }

    /// Ce qui retient le dessin maintenant, et pour combien de temps au plus avant de revoir
    /// (`None` : dessiner).
    pub fn retenue(&self, maintenant: Instant) -> Option<(Duration, Raison)> {
        if self.en_veille {
            return Some((REVUE, Raison::Veille));
        }
        if self.ecran_eteint {
            return Some((REVUE, Raison::EcranEteint));
        }
        self.reveil
            .and_then(|t| APRES_REVEIL.checked_sub(maintenant.saturating_duration_since(t)))
            .filter(|d| !d.is_zero())
            .map(|d| (d, Raison::Reprise))
    }

    /// Une image complète est demandée maintenant : `Ok(())` pour la faire, `Err(d)` pour la
    /// remettre à dans `d`. La ligne de journal (s'il y en a une) va dans `noter` : quand le
    /// dessin repart après avoir attendu, avec la durée de l'attente.
    pub fn demande(&mut self, maintenant: Instant, noter: &mut dyn FnMut(String)) -> Result<(), Duration> {
        match self.retenue(maintenant) {
            Some((d, raison)) => {
                if self.attente.is_none() {
                    self.attente = Some((maintenant, raison));
                }
                Err(d)
            }
            None => {
                if let Some((depuis, raison)) = self.attente.take() {
                    noter(format!(
                        "Fenêtres : dessin remis à la fin du réveil de Windows ({}) : attente de {} ms.",
                        raison.texte(),
                        maintenant.saturating_duration_since(depuis).as_millis()
                    ));
                }
                Ok(())
            }
        }
    }
}

// ÉTAT GLOBAL : la veille et l'écran sont signalés par le fil de l'icône (WM_POWERBROADCAST) et
// par la sonde, et lus par toutes les boucles de fenêtres, sans poignée commune entre eux.
static ETAT: Mutex<Reprise> = Mutex::new(Reprise {
    en_veille: false,
    ecran_eteint: false,
    reveil: None,
    attente: None,
});

fn etat() -> MutexGuard<'static, Reprise> {
    verrou(&ETAT)
}

/// Windows va se mettre en veille. Depuis le fil de l'icône.
pub fn veille() {
    etat().veille();
}

/// Windows sort de veille. Depuis le fil de l'icône, ou la sonde.
pub fn reveil() {
    etat().reveil(Instant::now());
}

/// L'écran s'allume ou s'éteint. Depuis le fil de l'icône.
pub fn ecran(allume: bool) {
    etat().ecran(allume, Instant::now());
}

/// Une image complète (OpenGL, `SwapBuffers`) peut-elle se faire maintenant ? `None` : oui ;
/// `Some(d)` : la remettre à dans `d`.
pub fn attente_avant_dessin() -> Option<Duration> {
    let mut ligne = None;
    let r = etat().demande(Instant::now(), &mut |l| ligne = Some(l));
    if let Some(l) = ligne {
        crate::sonde::noter(l);
    }
    r.err()
}

#[cfg(test)]
mod essais {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn au_repos_on_dessine() {
        let r = Reprise::default();
        assert_eq!(r.retenue(Instant::now()), None);
    }

    /// La veille, puis la reprise : rien pendant la veille, puis encore `APRES_REVEIL`.
    #[test]
    fn veille_puis_reprise() {
        let t = Instant::now();
        let mut r = Reprise::default();
        r.veille();
        assert_eq!(r.retenue(t), Some((REVUE, Raison::Veille)));
        r.reveil(t + ms(10_000));
        assert_eq!(r.retenue(t + ms(10_000)), Some((APRES_REVEIL, Raison::Reprise)));
        assert_eq!(
            r.retenue(t + ms(10_500)),
            Some((APRES_REVEIL - ms(500), Raison::Reprise))
        );
        assert_eq!(r.retenue(t + ms(10_000) + APRES_REVEIL), None);
    }

    /// Réveil automatique sans personne : l'écran reste éteint, le dessin attend qu'il se
    /// rallume, puis `APRES_REVEIL`.
    #[test]
    fn ecran_eteint_au_reveil() {
        let t = Instant::now();
        let mut r = Reprise::default();
        r.ecran(false, t);
        r.veille();
        r.reveil(t + ms(1_000));
        assert_eq!(r.retenue(t + ms(60_000)), Some((REVUE, Raison::EcranEteint)));
        r.ecran(true, t + ms(65_000));
        assert_eq!(r.retenue(t + ms(65_000)), Some((APRES_REVEIL, Raison::Reprise)));
        assert_eq!(r.retenue(t + ms(65_000) + APRES_REVEIL), None);
    }

    /// Un écran qui dit « allumé » sans avoir été éteint (la première nouvelle de Windows à
    /// l'inscription) ne retient rien.
    #[test]
    fn ecran_deja_allume() {
        let t = Instant::now();
        let mut r = Reprise::default();
        r.ecran(true, t);
        assert_eq!(r.retenue(t), None);
    }

    /// Une seule ligne au journal, quand le dessin repart, avec la durée depuis la première
    /// demande retenue.
    #[test]
    fn une_ligne_quand_le_dessin_repart() {
        let t = Instant::now();
        let mut r = Reprise::default();
        let mut lignes = Vec::new();
        r.veille();
        r.reveil(t);
        assert_eq!(
            r.demande(t + ms(100), &mut |l| lignes.push(l)),
            Err(APRES_REVEIL - ms(100))
        );
        assert_eq!(
            r.demande(t + ms(900), &mut |l| lignes.push(l)),
            Err(APRES_REVEIL - ms(900))
        );
        assert!(lignes.is_empty());
        assert_eq!(r.demande(t + APRES_REVEIL, &mut |l| lignes.push(l)), Ok(()));
        assert_eq!(lignes.len(), 1, "{lignes:?}");
        assert!(lignes[0].contains("Windows sort de veille"), "{}", lignes[0]);
        assert!(lignes[0].contains("attente de 1900 ms"), "{}", lignes[0]);
        // Ensuite, plus rien à dire.
        assert_eq!(r.demande(t + ms(5_000), &mut |l| lignes.push(l)), Ok(()));
        assert_eq!(lignes.len(), 1);
    }

    /// Sans attente, pas de ligne.
    #[test]
    fn pas_de_ligne_sans_attente() {
        let mut r = Reprise::default();
        let mut lignes = Vec::new();
        assert_eq!(r.demande(Instant::now(), &mut |l| lignes.push(l)), Ok(()));
        assert!(lignes.is_empty());
    }
}
