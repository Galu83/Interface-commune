//! Les attentes avant de créer une autre fenêtre : après une création ratée, après un
//! changement d'affichage de Windows ([`affichage_change`]), après la destruction d'une
//! fenêtre ([`Destructions`]).

use std::sync::Mutex;

use super::super::verrou;

/// Les créations ratées de suite après lesquelles une fenêtre est abandonnée (fermée).
pub const CREATIONS_MAX: u32 = 3;

/// L'attente avant un nouvel essai de création.
pub(super) const NOUVEL_ESSAI: std::time::Duration = std::time::Duration::from_millis(500);

/// Après un changement d'affichage de Windows (`WM_DISPLAYCHANGE` : écran retiré, rendu, autre
/// résolution), une fenêtre qui n'existe pas encore n'est créée qu'au bout de ce temps : le
/// pilote graphique refait alors son travail, et créer une fenêtre OpenGL au même moment a
/// fait échouer la création, ou bloqué le fil dans SwapBuffers pendant des dizaines de
/// secondes. Une fenêtre déjà créée n'attend jamais (egui détruit celle qu'on ne dessine pas).
pub const REPRISE: std::time::Duration = std::time::Duration::from_secs(1);

/// Un premier dessin après un changement d'affichage est mesuré au journal pendant ce temps.
pub(super) const SUIVI_DE_LA_REPRISE: std::time::Duration = std::time::Duration::from_secs(10);

/// Une fenêtre retirée n'est détruite par egui qu'à la fin de l'image qui ne la dessine plus.
/// Créer une autre fenêtre aussitôt après a fait bloquer sans fin `show_viewport_immediate`
/// (un processeur très chargé, plusieurs fenêtres fermées puis ouvertes de suite). Aucune
/// fenêtre n'est donc créée tant qu'une fenêtre retirée existe encore, ni pendant
/// [`APRES_DESTRUCTION`] après la dernière destruction. Au plus [`DESTRUCTION_MAX`] d'attente
/// pour une fenêtre qui ne disparaît pas.
pub const DESTRUCTION_MAX: std::time::Duration = std::time::Duration::from_secs(2);

/// Le temps laissé à Windows et au pilote graphique après la destruction d'une fenêtre,
/// avant d'en créer une autre.
pub const APRES_DESTRUCTION: std::time::Duration = std::time::Duration::from_millis(100);

/// Le temps entre deux vérifications qu'une fenêtre retirée a disparu.
pub(super) const VERIFICATION: std::time::Duration = std::time::Duration::from_millis(10);

/// Les fenêtres retirées (leur HWND et l'heure du retrait) qui existent encore, et l'heure
/// de la dernière destruction constatée.
#[derive(Default)]
pub(super) struct Destructions {
    en_attente: Vec<(isize, std::time::Instant)>,
    derniere: Option<std::time::Instant>,
}

impl Destructions {
    /// Une fenêtre vient d'être retirée : egui va la détruire.
    pub(super) fn retiree(&mut self, h: isize, maintenant: std::time::Instant) {
        self.en_attente.push((h, maintenant));
        self.derniere = Some(maintenant);
    }

    /// Constate les destructions (`existe` : la fenêtre existe-t-elle encore ?) et rend
    /// l'attente avant de pouvoir créer une fenêtre (`None` : maintenant).
    pub(super) fn attente(
        &mut self,
        maintenant: std::time::Instant,
        existe: impl Fn(isize) -> bool,
    ) -> Option<std::time::Duration> {
        let mut derniere = self.derniere;
        self.en_attente.retain(|&(h, retiree)| {
            let depuis = maintenant.saturating_duration_since(retiree);
            if depuis >= DESTRUCTION_MAX {
                // Elle ne disparaît pas : on ne bloque pas pour toujours.
                return false;
            }
            if existe(h) {
                return true;
            }
            // Détruite. Constatée longtemps après son retrait (personne ne regardait) : la
            // destruction date du retrait, pas d'aujourd'hui.
            let quand = if depuis > APRES_DESTRUCTION {
                retiree
            } else {
                maintenant
            };
            derniere = Some(derniere.map_or(quand, |d| d.max(quand)));
            false
        });
        self.derniere = derniere;
        attente_apres_destruction(
            !self.en_attente.is_empty(),
            derniere.map(|d| maintenant.saturating_duration_since(d)),
        )
    }
}

/// L'attente avant de créer une fenêtre après des destructions : tant qu'une fenêtre retirée
/// existe (`en_attente`), le temps de la prochaine vérification ; sinon, ce qui reste de
/// [`APRES_DESTRUCTION`] depuis la dernière destruction (`depuis`) ; `None` : créer.
pub fn attente_apres_destruction(
    en_attente: bool,
    depuis: Option<std::time::Duration>,
) -> Option<std::time::Duration> {
    if en_attente {
        return Some(VERIFICATION);
    }
    depuis
        .and_then(|d| APRES_DESTRUCTION.checked_sub(d))
        .filter(|d| !d.is_zero())
}

/// L'heure du dernier changement d'affichage de Windows.
// ÉTAT GLOBAL : signal posé par le fil de l'icône (message de Windows), qui n'a aucune poignée
// vers les fenêtres de la boucle.
static AFFICHAGE_CHANGE: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// Windows vient de changer l'affichage (`WM_DISPLAYCHANGE`) : les fenêtres pas encore créées
/// attendent [`REPRISE`]. À appeler depuis le fil de l'icône.
pub fn affichage_change() {
    *verrou(&AFFICHAGE_CHANGE) = Some(std::time::Instant::now());
}

/// Le temps écoulé depuis le dernier changement d'affichage signalé par
/// [`affichage_change`] (`None` : aucun depuis le lancement).
pub fn depuis_le_changement() -> Option<std::time::Duration> {
    (*verrou(&AFFICHAGE_CHANGE)).map(|t| t.elapsed())
}

/// Combien de temps une fenêtre pas encore créée attend avant d'être dessinée (`None` :
/// maintenant). `depuis` : le temps écoulé depuis le dernier changement d'affichage ;
/// `essai` : ce qui reste avant le nouvel essai d'une création ratée ; `destruction` :
/// l'attente due aux fenêtres qui viennent d'être détruites ([`attente_apres_destruction`]).
/// Elle attend le plus long des trois.
pub fn attente_avant_creation(
    depuis: Option<std::time::Duration>,
    essai: Option<std::time::Duration>,
    destruction: Option<std::time::Duration>,
) -> Option<std::time::Duration> {
    let reprise = depuis.and_then(|d| REPRISE.checked_sub(d));
    [reprise, essai, destruction]
        .into_iter()
        .flatten()
        .filter(|d| !d.is_zero())
        .max()
}
