//! La boucle des fenêtres de la bibliothèque ([`boucle_des_fenetres`]) : la seule boucle de
//! fenêtres du programme, sur un fil à part. Sa fenêtre principale est celle du volet et du
//! petit menu, préparée une fois au repos, gardée cachée et recréée quand elle est perdue
//! (voir [`crate::fenetres`]). Ce qu'elle ouvre passe par un [`Ouvreur`] : les vraies
//! fenêtres egui, ou le faux moteur des essais (`super::faux_moteur`).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use ic_coeur::gestes::ouverture::Ouvert;
use ic_coeur::gestes::secours::RAFALE;

use super::Pertes;
use super::canal::Reception;
use super::liens::{Demande, Liens};
use super::supervision::cause_de_panique;
use super::verrou;
use crate::theme::Surface;

/// Ce que le fil des fenêtres ouvre : les vraies fenêtres (egui), ou un faux dans les tests.
pub trait Ouvreur {
    /// Prépare au repos la fenêtre du volet et du petit menu, cachée, et la garde jusqu'à la
    /// fin : chaque demande de volet ou de petit menu la montre sans la reconstruire ; une
    /// demande de Paramètres les ouvre dans la même boucle. Rend `Demande::Quitter`, ou
    /// `None` (plus personne n'envoie, le logiciel se ferme, ou la fenêtre a été fermée).
    /// `premiere` : une demande arrivée avant la préparation, faite dès qu'elle est finie. En
    /// erreur, rend aussi la demande de volet ou de menu en cours, s'il y en a une (pour le
    /// menu de secours).
    fn surface(
        &mut self,
        demandes: &Reception,
        premiere: Option<Demande>,
    ) -> Result<Option<Demande>, (String, Option<Demande>)>;
    /// Secours, quand la fenêtre du volet n'a pas pu être préparée : les Paramètres seuls, à
    /// cette page ; rend la main à leur fermeture.
    fn parametres(&mut self, page: Option<String>) -> Result<(), String>;
    /// Le menu gris de Windows, à la place du volet ou du petit menu.
    fn secours(&mut self, d: &Demande);
    /// Une ligne pour le journal du logiciel.
    fn noter(&mut self, ligne: String);
}

fn nom_du_clic(d: &Demande) -> &'static str {
    match d {
        Demande::Volet { .. } => "le volet",
        Demande::Menu { .. } => "le petit menu",
        Demande::Parametres(_) => "les Paramètres",
        Demande::Quitter => "la fermeture",
    }
}

/// La boucle du fil des fenêtres : la surface (volet et petit menu) préparée au repos, qui
/// ouvre aussi les Paramètres et les autres fenêtres, jusqu'à `Demande::Quitter`. Elle est
/// refaite si elle a été fermée (par Windows) ou perdue (une panique, attrapée ici : voir
/// [`crate::fenetres`]).
pub fn boucle_des_fenetres(liens: &Liens, ouvreur: &mut impl Ouvreur, demandes: &Reception) {
    let mut etat = Etat::default();
    loop {
        crate::sonde::battement();
        crate::sonde::etape("boucle des fenêtres");
        if liens.quitter.load(Ordering::Acquire) {
            break;
        }
        let d = match etat.demande(liens, ouvreur, demandes) {
            Pas::Faire(d) => d,
            Pas::Recommencer => continue,
            Pas::Arreter => break,
        };
        if d == Demande::Quitter {
            break;
        }
        sans_surface(liens, ouvreur, d);
    }
}

/// Ce que la boucle garde d'un tour à l'autre.
#[derive(Default)]
struct Etat {
    /// Les pertes récentes de la fenêtre du volet.
    pertes: Pertes,
    /// Le clic à refaire après la recréation de la fenêtre perdue.
    reprise: Option<Demande>,
    /// Pertes en rafale : la fenêtre n'est recréée qu'au prochain clic.
    en_pause: bool,
    /// Une première préparation a été tentée (un échec ensuite est passager).
    deja_preparee: bool,
}

/// La suite d'un tour de la boucle.
enum Pas {
    /// Une demande à faire sans la surface (elle est impossible) : menu de secours ou
    /// Paramètres seuls ; ou `Demande::Quitter`.
    Faire(Demande),
    /// La surface a été perdue ou fermée : un nouveau tour.
    Recommencer,
    /// Plus personne n'envoie, ou le logiciel se ferme.
    Arreter,
}

impl Etat {
    /// La demande du tour : d'abord ce qui attend déjà (premier lancement : les Paramètres) ;
    /// sinon la surface préparée au repos, qui garde la main jusqu'à une autre demande.
    fn demande(&mut self, liens: &Liens, ouvreur: &mut impl Ouvreur, demandes: &Reception) -> Pas {
        let deja = match self.reprise.take() {
            Some(d) => Some(d),
            None => match demandes.prendre() {
                Ok(d) => d,
                Err(()) => return Pas::Arreter,
            },
        };
        let impossible = liens.egui_impossible.load(Ordering::Acquire);
        let deja = match deja {
            None if self.en_pause && !impossible => match demandes.recevoir() {
                Some(d) => Some(d),
                None => return Pas::Arreter,
            },
            autre => autre,
        };
        match deja {
            None if impossible => demandes.recevoir().map_or(Pas::Arreter, Pas::Faire),
            Some(d) if impossible || d == Demande::Quitter => Pas::Faire(d),
            // Rien (ou une demande arrivée avant la préparation, qu'elle fera) : la surface.
            premiere => self.surface(liens, ouvreur, demandes, premiere),
        }
    }

    /// Prépare et garde la surface jusqu'à sa fin, puis décide de la suite.
    fn surface(
        &mut self,
        liens: &Liens,
        ouvreur: &mut impl Ouvreur,
        demandes: &Reception,
        premiere: Option<Demande>,
    ) -> Pas {
        liens.poser_en_cours(premiere.clone());
        let r = catch_unwind(AssertUnwindSafe(|| ouvreur.surface(demandes, premiere)));
        let premiere_preparation = !self.deja_preparee;
        self.deja_preparee = true;
        let en_cours = liens.prendre_en_cours();
        liens.surface_prete.store(0, Ordering::Release);
        liens.mettre(Ouvert::Rien);
        *verrou(&liens.contexte) = None;
        match r {
            Err(p) => self.perdue(liens, ouvreur, demandes, &cause_de_panique(&*p), en_cours),
            Ok(r) => {
                self.en_pause = false;
                self.apres_la_surface(liens, ouvreur, r, premiere_preparation)
            }
        }
    }

    /// La surface a paniqué : recréée tout de suite (le clic en cours refait), ou, en rafale,
    /// le menu de secours pour ce clic et une recréation au clic suivant.
    fn perdue(
        &mut self,
        liens: &Liens,
        ouvreur: &mut impl Ouvreur,
        demandes: &Reception,
        cause: &str,
        en_cours: Option<Demande>,
    ) -> Pas {
        liens.oublier_la_surface(demandes);
        if liens.quitter.load(Ordering::Acquire) {
            return Pas::Arreter;
        }
        let n = self.pertes.compter(Instant::now());
        if !Pertes::en_rafale(n) {
            let refaire = en_cours
                .as_ref()
                .map(|d| format!(" ; {} demandé est rouvert", nom_du_clic(d)))
                .unwrap_or_default();
            ouvreur.noter(format!(
                "Fenêtre du volet et du petit menu perdue ({cause}) : recréée{refaire}."
            ));
            self.reprise = en_cours;
            self.en_pause = false;
        } else {
            ouvreur.noter(format!(
                "Fenêtre du volet et du petit menu perdue ({cause}) : {n} fois en {} s ; menu de secours pour ce clic, recréée au clic suivant.",
                RAFALE.as_secs()
            ));
            if let Some(d) = en_cours {
                ouvreur.secours(&d);
            }
            self.en_pause = true;
        }
        Pas::Recommencer
    }

    /// La surface a rendu la main sans paniquer : une demande, une fermeture par Windows, ou
    /// un échec (pour de bon à la première préparation, passager ensuite).
    fn apres_la_surface(
        &mut self,
        liens: &Liens,
        ouvreur: &mut impl Ouvreur,
        r: Result<Option<Demande>, (String, Option<Demande>)>,
        premiere_preparation: bool,
    ) -> Pas {
        match r {
            Ok(Some(d)) => Pas::Faire(d),
            Ok(None) => {
                if liens.quitter.load(Ordering::Acquire) {
                    return Pas::Arreter;
                }
                let n = self.pertes.compter(Instant::now());
                self.en_pause = Pertes::en_rafale(n);
                ouvreur.noter(if self.en_pause {
                    format!(
                        "Fenêtre du volet et du petit menu fermée par Windows : {n} fois en {} s ; recréée au clic suivant.",
                        RAFALE.as_secs()
                    )
                } else {
                    "Fenêtre du volet et du petit menu fermée par Windows : recréée.".into()
                });
                Pas::Recommencer
            }
            Err((e, en_cours)) if premiere_preparation => {
                // Jamais préparée (ni OpenGL ni fenêtre possibles) : le menu de secours pour
                // de bon.
                ouvreur.noter(format!(
                    "Volet impossible à préparer ({e}) ; menu de secours de Windows."
                ));
                liens.egui_impossible.store(true, Ordering::Release);
                if let Some(d) = en_cours {
                    ouvreur.secours(&d);
                }
                Pas::Recommencer
            }
            Err((e, en_cours)) => {
                // Elle a déjà marché : un échec passager (écrans en veille, carte graphique
                // qui se relance). Le menu de secours pour ce clic, un nouvel essai au clic
                // suivant.
                ouvreur.noter(format!(
                    "Volet impossible à recréer ({e}) ; menu de secours pour ce clic, nouvel essai au clic suivant."
                ));
                if let Some(d) = en_cours {
                    ouvreur.secours(&d);
                }
                self.en_pause = true;
                Pas::Recommencer
            }
        }
    }
}

/// Une demande quand la surface est impossible : le menu de secours (volet, petit menu) ou
/// les Paramètres seuls.
fn sans_surface(liens: &Liens, ouvreur: &mut impl Ouvreur, d: Demande) {
    let resultat = match d {
        Demande::Quitter => return,
        Demande::Volet { .. } | Demande::Menu { .. } => {
            ouvreur.secours(&d);
            Ok(())
        }
        Demande::Parametres(page) => {
            liens.fermer.store(false, Ordering::Release);
            liens.mettre(Ouvert::Parametres);
            catch_unwind(AssertUnwindSafe(|| ouvreur.parametres(page)))
                .unwrap_or_else(|p| Err(cause_de_panique(&*p)))
        }
    };
    liens.mettre(Ouvert::Rien);
    *verrou(&liens.contexte) = None;
    if let Err(e) = resultat {
        ouvreur.noter(format!(
            "Fenêtre impossible à ouvrir ({e}) ; menu de secours de Windows."
        ));
        liens.egui_impossible.store(true, Ordering::Release);
    }
}

/// Le dernier filet du fil des fenêtres : s'il s'arrête alors que le logiciel ne se ferme pas
/// (une panique hors de la boucle), l'icône passe au menu de secours de Windows au lieu de ne
/// plus rien ouvrir, et le journal le dit. À créer au début du fil, gardée jusqu'à sa fin.
pub struct GardeDuFil {
    liens: Arc<Liens>,
    noter: Box<dyn Fn(String) + Send>,
}

impl GardeDuFil {
    pub fn new(liens: Arc<Liens>, noter: impl Fn(String) + Send + 'static) -> GardeDuFil {
        GardeDuFil {
            liens,
            noter: Box::new(noter),
        }
    }
}

impl Drop for GardeDuFil {
    fn drop(&mut self) {
        let l = &self.liens;
        l.surface_prete.store(0, Ordering::Release);
        l.mettre(Ouvert::Rien);
        if l.quitter.load(Ordering::Acquire) {
            return;
        }
        l.egui_impossible.store(true, Ordering::Release);
        let pourquoi = if std::thread::panicking() {
            "panique"
        } else {
            "fin inattendue"
        };
        (self.noter)(format!(
            "Fil des fenêtres arrêté ({pourquoi}) : l'icône ouvre le menu de secours de Windows jusqu'au prochain lancement."
        ));
    }
}

/// Ce que la fenêtre du volet fait d'une demande reçue (sa garde et la mesure de son
/// ouverture : `ic_coeur::gestes::ouverture`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suite {
    /// La montrer avec cette surface (mise à jour, jamais reconstruite).
    Montrer(Surface),
    /// Déjà montrée avec cette surface : rien.
    Ignorer,
    /// Les Paramètres sont demandés : le volet se cache, ils s'ouvrent à côté.
    Parametres,
    /// Le logiciel se ferme : se cacher et rendre la main au fil des fenêtres.
    Sortir,
}

/// La suite d'une demande, quand `montree` est la surface montrée (ou en train de l'être).
pub fn suite(d: &Demande, montree: Option<Surface>) -> Suite {
    let voulue = match d {
        Demande::Volet { .. } => Surface::Volet,
        Demande::Menu { .. } => Surface::Menu,
        Demande::Parametres(_) => return Suite::Parametres,
        Demande::Quitter => return Suite::Sortir,
    };
    if montree == Some(voulue) {
        Suite::Ignorer
    } else {
        Suite::Montrer(voulue)
    }
}
