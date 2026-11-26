//! Le volet, le petit menu et les Paramètres communs pour les icônes d'une zone (bloc M) :
//! l'aiguillage des gestes vers le fil des fenêtres ([`Aiguilleur`], que l'icône seule
//! d'`Horloge` utilise aussi), et [`FenetresCommunes`] pour le logiciel qui garde sa boucle.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use super::decodage::{Evenement, Message};
use super::fil::{ATTENTE_FERMETURE, Secours, attendre_la_fin, lancer_le_fil_des_fenetres};
use crate::Logiciel;
use crate::fenetres::{self, Aiguillage, Demande, Demandes, Liens, envoyer_le_clic};
use crate::zone::Zone;
use ic_coeur::gestes::aiguilleur::{
    Ordre, SuiviDesIcones, fermer_avant_la_fenetre_principale, ordre, point_de_secours,
};

/// Ce que le logiciel fait d'un geste sur une de ses icônes, une fois passé par les
/// mécanismes communs ([`Aiguilleur::recevoir`], [`FenetresCommunes::geste`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuiteDuGeste {
    /// Rien : la bibliothèque s'en occupe (volet, petit menu, Paramètres), ou le geste ne
    /// compte pas (survol, clic du milieu…).
    Rien,
    /// Les fenêtres modernes sont impossibles : le menu de secours de Windows
    /// ([`Zone::menu_windows`]) au point `point` (pixels réels), pour l'icône `icone`.
    Secours { icone: String, point: [i32; 2] },
    /// Un clic sur la notification de cette icône : au logiciel de faire ce qu'elle annonçait.
    Notification(String),
}

/// L'aiguillage des gestes des icônes d'une zone vers le fil des fenêtres commun, sans
/// Windows ni fenêtre (testé) : le suivi des clics par icône et l'ordre qui en découle, les
/// mêmes que pour l'icône unique (`ic_coeur::gestes::aiguilleur`), puis
/// [`fenetres::envoyer_le_clic`]. [`FenetresCommunes`] en tient un.
pub struct Aiguilleur {
    liens: Arc<Liens>,
    demandes: Demandes,
    clics: SuiviDesIcones<Instant>,
    /// Le logiciel, pour sa fenêtre principale (`Logiciel::fenetre_principale`) ; sans lui,
    /// le double-clic ouvre les Paramètres et le clic gauche le volet.
    logiciel: Option<Arc<dyn Logiciel>>,
}

impl Aiguilleur {
    pub fn new(liens: Arc<Liens>, demandes: Demandes) -> Aiguilleur {
        Aiguilleur {
            liens,
            demandes,
            clics: SuiviDesIcones::default(),
            logiciel: None,
        }
    }

    /// Avec le logiciel : sa fenêtre principale, s'il en a une, s'ouvre au double-clic (et au
    /// clic gauche avec `FenetrePrincipale::clic_gauche`).
    pub fn avec_logiciel(mut self, logiciel: Arc<dyn Logiciel>) -> Aiguilleur {
        self.logiciel = Some(logiciel);
        self
    }

    /// Le geste `e`, reçu à `maintenant` ; `double` : le temps d'un double-clic de Windows.
    pub fn recevoir(&mut self, e: &Evenement, maintenant: Instant, double: Duration) -> SuiteDuGeste {
        if e.message == Message::Notification {
            return SuiteDuGeste::Notification(e.icone.clone());
        }
        let Some(g) = e.message.geste() else {
            return SuiteDuGeste::Rien;
        };
        let g = self.clics.recevoir(&e.icone, g, maintenant, double);
        let principale = self.logiciel.as_ref().and_then(|l| l.fenetre_principale());
        let (surface, ancre) = match ordre(g, principale, e.ancre) {
            Ordre::Rien => return SuiteDuGeste::Rien,
            Ordre::Parametres => {
                // Double-clic : le volet ouvert par le premier clic lit la demande des
                // Paramètres, se cache, puis leur laisse la place.
                self.liens.montrer_page(&self.demandes, None);
                return SuiteDuGeste::Rien;
            }
            Ordre::FenetrePrincipale => {
                // Le volet ou le petit menu ouvert par le premier clic lui laisse la place.
                if fermer_avant_la_fenetre_principale(self.liens.ouvert()) {
                    self.liens.fermer_la_fenetre();
                }
                if let Some(l) = &self.logiciel {
                    l.ouvrir_fenetre_principale();
                }
                return SuiteDuGeste::Rien;
            }
            Ordre::Surface { surface, ancre } => (surface, ancre),
        };
        match envoyer_le_clic(&self.liens, &self.demandes, surface, e.rect, ancre, maintenant) {
            Aiguillage::Rien => SuiteDuGeste::Rien,
            Aiguillage::Envoye => {
                // La fenêtre prête se montre tout de suite, encore invisible.
                crate::systeme::reveiller(self.liens.surface_prete.load(Ordering::Acquire));
                SuiteDuGeste::Rien
            }
            Aiguillage::Secours => SuiteDuGeste::Secours {
                icone: e.icone.clone(),
                point: point_de_secours(ancre, e.rect),
            },
        }
    }
}

/// Le volet, le petit menu et les Paramètres de la bibliothèque pour un logiciel qui garde sa
/// [`Zone`] et sa boucle de messages : le fil des fenêtres commun
/// (`horloge::lancer_le_fil_des_fenetres`), et l'aiguillage des gestes vers lui
/// ([`Aiguilleur`]). Le volet est le même quelle que soit l'icône cliquée ; il se place contre
/// l'icône du geste (`Logiciel::placement_volet`).
///
/// ```no_run
/// # use std::sync::Arc;
/// # fn essai(logiciel: Arc<dyn interface_commune::Logiciel>) -> Result<(), String> {
/// use interface_commune::zone::{FenetresCommunes, SuiteDuGeste, Zone};
/// let mut zone = Zone::creer("MonLogiciel", "MonLogiciel")?;
/// // Le menu de secours est demandé sur le fil des fenêtres : le transmettre au fil de la
/// // zone (un message posté à `zone.fenetre()`), qui l'ouvre avec `Zone::menu_windows`.
/// let mut communes = FenetresCommunes::lancer(&zone, logiciel, None, Box::new(|_demande| {}))?;
/// // Dans la boucle de messages du logiciel, après DispatchMessageW :
/// for e in zone.evenements() {
///     match communes.geste(&e) {
///         SuiteDuGeste::Rien => {}
///         SuiteDuGeste::Secours { icone, point } => { /* zone.menu_windows(point, …) */ }
///         SuiteDuGeste::Notification(icone) => { /* ce que la bulle annonçait */ }
///     }
/// }
/// // « Quitter » : les fenêtres se ferment (5 s au plus), puis la zone.
/// communes.fermer();
/// # Ok(()) }
/// ```
pub struct FenetresCommunes {
    aiguilleur: Aiguilleur,
    liens: Arc<Liens>,
    demandes: Demandes,
    logiciel: Arc<dyn Logiciel>,
    fil: Option<std::thread::JoinHandle<()>>,
}

impl FenetresCommunes {
    /// Lance le fil des fenêtres (le volet préparé au repos, caché). `parametres` : les
    /// Paramètres à ouvrir tout de suite (`--parametres <page>` : `Some(page)`). `secours` :
    /// voir `horloge::lancer_le_fil_des_fenetres` (appelé sur le fil des fenêtres, il transmet
    /// seulement la demande au fil de la zone).
    pub fn lancer(
        zone: &Zone,
        logiciel: Arc<dyn Logiciel>,
        parametres: Option<Option<String>>,
        secours: Secours,
    ) -> Result<FenetresCommunes, String> {
        let (demandes, reception) = fenetres::canal();
        let liens = Arc::new(Liens::default());
        liens.fenetre.store(zone.fenetre(), Ordering::Release);
        if let Some(page) = parametres {
            let _ = demandes.send(Demande::Parametres(page));
        }
        // La sonde des fils écrit dans le journal du logiciel, comme avec `Horloge`.
        let pour_la_sonde = logiciel.clone();
        crate::sonde::brancher_le_journal(move |ligne| pour_la_sonde.noter(ligne));
        let fil = lancer_le_fil_des_fenetres(logiciel.clone(), liens.clone(), reception, secours)?;
        Ok(FenetresCommunes {
            aiguilleur: Aiguilleur::new(liens.clone(), demandes.clone()).avec_logiciel(logiciel.clone()),
            liens,
            demandes,
            logiciel,
            fil: Some(fil),
        })
    }

    /// Un geste lu par [`Zone::evenements`] : le volet, le petit menu ou les Paramètres
    /// s'ouvrent (ou se ferment) par les mécanismes communs ; rend ce qui reste au logiciel.
    pub fn geste(&mut self, e: &Evenement) -> SuiteDuGeste {
        self.aiguilleur
            .recevoir(e, Instant::now(), crate::systeme::temps_double_clic())
    }

    /// Ce que partagent le fil des fenêtres et le logiciel (`Liens::redessiner` quand l'état
    /// change, `Liens::autres` pour ses autres fenêtres…).
    pub fn liens(&self) -> &Arc<Liens> {
        &self.liens
    }

    /// L'envoi des demandes au fil des fenêtres.
    pub fn demandes(&self) -> &Demandes {
        &self.demandes
    }

    /// Ouvre les Paramètres à cette page (une notification, une demande du logiciel) ;
    /// ouverts, ils passent devant et changent de page.
    pub fn ouvrir_parametres(&self, page: Option<String>) {
        self.liens.montrer_page(&self.demandes, page);
    }

    /// Ouvre le logiciel (un second lancement, par exemple) : sa fenêtre principale s'il en a
    /// une (`Logiciel::fenetre_principale`), sinon ses Paramètres.
    pub fn ouvrir_le_logiciel(&self) {
        if self.logiciel.fenetre_principale().is_some() {
            self.logiciel.ouvrir_fenetre_principale();
        } else {
            self.ouvrir_parametres(None);
        }
    }

    /// L'état du logiciel a changé : le volet et les Paramètres ouverts se redessinent.
    pub fn redessiner(&self) {
        self.liens.redessiner();
    }

    /// Ferme les fenêtres et attend la fin de leur fil (`horloge::ATTENTE_FERMETURE` au
    /// plus) ; faux s'il a fallu partir sans l'attendre (c'est noté au journal).
    pub fn fermer(mut self) -> bool {
        self.arreter()
    }

    fn arreter(&mut self) -> bool {
        self.liens.tout_fermer();
        let _ = self.demandes.send(Demande::Quitter);
        let Some(fil) = self.fil.take() else {
            return true;
        };
        let fini = attendre_la_fin(fil, ATTENTE_FERMETURE);
        if !fini {
            self.logiciel
                .noter("Une fenêtre ne répond plus : le logiciel se ferme sans l'attendre.".into());
        }
        fini
    }
}

impl Drop for FenetresCommunes {
    fn drop(&mut self) {
        if self.fil.is_some() {
            self.arreter();
        }
    }
}
