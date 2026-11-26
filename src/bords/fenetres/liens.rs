//! Ce que partagent le fil principal (l'icône), le fil des fenêtres et le logiciel : les
//! demandes que l'icône envoie ([`Demande`]), l'aiguillage d'un clic ([`envoyer_le_clic`]) et
//! les liens ([`Liens`]). La décision du clic est dans le cœur (`ic_coeur::gestes::ouverture`,
//! bloc D).

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ic_coeur::gestes::ouverture::{Clic, Ouvert, decider_le_clic};

use super::autres::AutresFenetres;
use super::canal::{Demandes, Reception};
use crate::bords::verrou::verrou;
use crate::place::Rect;
use crate::theme::Surface;

/// Ce que l'icône (fil principal) demande au fil des fenêtres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Demande {
    /// Ouvrir le volet contre l'icône (rectangle de l'icône, en pixels) : clic gauche,
    /// Entrée ou Espace. `clic` : quand l'icône a été cliquée (le temps d'ouverture).
    Volet {
        icone: Rect,
        clic: Instant,
    },
    /// Ouvrir le petit menu au point `ancre` (pixels) : clic droit, touche menu, Maj+F10.
    Menu {
        icone: Rect,
        ancre: [i32; 2],
        clic: Instant,
    },
    /// Les Paramètres, à cette page (sa clé) ou à la première : une fenêtre de plus de la
    /// boucle, à côté de la fenêtre du volet.
    Parametres(Option<String>),
    Quitter,
}

/// Ce qu'est devenu un clic sur une icône ([`envoyer_le_clic`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aiguillage {
    /// Rien à faire de plus : le clic ferme la surface montrée, ou il est ignoré.
    Rien,
    /// La demande est partie au fil des fenêtres : réveiller la fenêtre prête
    /// ([`Liens::surface_prete`]) pour qu'elle se montre tout de suite.
    Envoye,
    /// Les fenêtres modernes sont impossibles (ou le fil des fenêtres ne répond plus) : le
    /// menu de secours de Windows, au point du clic.
    Secours,
}

/// Le clic sur une icône pour `surface` (le volet, ou le petit menu avec son point `ancre`),
/// par les mécanismes communs : ouvert devant les Paramètres, refermé par un second clic,
/// ignoré juste après une fermeture ([`decider_le_clic`]), envoyé au fil des fenêtres
/// ([`Demandes`]). Si le fil des fenêtres ne répond plus, l'icône passe au menu de secours pour
/// de bon ([`Liens::egui_impossible`]). Le même pour l'icône unique (`horloge`) et pour les
/// icônes d'une `zone`.
pub fn envoyer_le_clic(
    liens: &Liens,
    demandes: &Demandes,
    surface: Surface,
    icone: Rect,
    ancre: Option<[i32; 2]>,
    clic: Instant,
) -> Aiguillage {
    if liens.egui_impossible.load(Ordering::Acquire) {
        return Aiguillage::Secours;
    }
    match decider_le_clic(liens.ouvert(), surface, liens.juste_fermee(surface)) {
        Clic::Fermer => {
            liens.fermer_la_fenetre();
            Aiguillage::Rien
        }
        Clic::Ignorer => Aiguillage::Rien,
        Clic::Ouvrir => {
            let d = match (surface, ancre) {
                (Surface::Menu, Some(ancre)) => Demande::Menu { icone, ancre, clic },
                (Surface::Menu, None) => Demande::Menu {
                    icone,
                    ancre: crate::place::centre_de(icone),
                    clic,
                },
                (Surface::Volet, _) => Demande::Volet { icone, clic },
            };
            if demandes.send(d).is_err() {
                liens.egui_impossible.store(true, Ordering::Release);
                return Aiguillage::Secours;
            }
            Aiguillage::Envoye
        }
    }
}

/// Ce que partagent le fil principal (l'icône), le fil des fenêtres et le logiciel.
#[derive(Default)]
pub struct Liens {
    /// La fenêtre cachée du fil principal (0 tant qu'elle n'existe pas).
    pub fenetre: AtomicIsize,
    /// Le menu moderne n'a pas pu s'ouvrir : le menu de secours prend sa place.
    pub egui_impossible: AtomicBool,
    /// Le contexte egui de la fenêtre ouverte (pour la redessiner).
    pub contexte: Mutex<Option<egui::Context>>,
    ouvert: Mutex<Ouvert>,
    /// Le clic (volet ou petit menu) en train d'être ouvert : refait si la fenêtre est perdue
    /// avant de s'être montrée.
    en_cours: Mutex<Option<Demande>>,
    /// Quand le volet ou le petit menu s'est fermé, et lequel (un clic sur l'icône qui vient
    /// de le fermer, par la perte du focus, ne le rouvre pas).
    pub ferme_a: Mutex<Option<(Instant, Surface)>>,
    /// La fenêtre du volet et du petit menu, prête et cachée (0 : pas prête). Le fil
    /// principal la montre lui-même au clic, encore transparente, pour qu'elle se dessine
    /// tout de suite.
    pub surface_prete: AtomicIsize,
    /// Le logiciel se ferme : le fil des fenêtres ne prépare plus rien.
    pub quitter: AtomicBool,
    /// Le volet ou le petit menu montré doit se fermer.
    pub fermer: AtomicBool,
    /// Les Paramètres ouverts doivent montrer cette page.
    pub page_voulue: Arc<Mutex<Option<String>>>,
    /// Les Paramètres et les autres fenêtres du logiciel, dans la boucle des fenêtres.
    pub autres: AutresFenetres,
}

impl Liens {
    /// Ce qui est ouvert : le volet ou le petit menu s'il est montré ; sinon les Paramètres
    /// s'ils sont ouverts.
    pub fn ouvert(&self) -> Ouvert {
        let o = *verrou(&self.ouvert);
        if o == Ouvert::Rien && self.autres.ouverte(crate::parametres::CLE_FENETRE) {
            Ouvert::Parametres
        } else {
            o
        }
    }

    pub fn mettre(&self, o: Ouvert) {
        *verrou(&self.ouvert) = o;
    }

    /// Le clic en train d'être ouvert (`None` : montré, ou abandonné).
    pub fn poser_en_cours(&self, d: Option<Demande>) {
        *verrou(&self.en_cours) = d;
    }

    /// Le clic qui était en train d'être ouvert (retiré).
    pub fn prendre_en_cours(&self) -> Option<Demande> {
        verrou(&self.en_cours).take()
    }

    /// La fenêtre du volet n'existe plus : plus rien n'est montré ni prêt, aucune fenêtre ne
    /// lit les demandes.
    pub(super) fn oublier_la_surface(&self, demandes: &Reception) {
        self.surface_prete.store(0, Ordering::Release);
        self.mettre(Ouvert::Rien);
        self.fermer.store(false, Ordering::Release);
        *verrou(&self.contexte) = None;
        self.autres.lier(None);
        self.autres.racine(0);
        demandes.racine(0);
        demandes.lecteur(None);
    }

    /// `surface` s'est fermée il y a moins d'[`ANTI_REBOND`](crate::fenetres::ANTI_REBOND) (le clic sur l'icône qui l'a
    /// fermée en lui prenant le focus ne doit pas la rouvrir aussitôt).
    pub fn juste_fermee(&self, surface: Surface) -> bool {
        let fermee = *verrou(&self.ferme_a);
        ic_coeur::gestes::ouverture::juste_fermee(fermee, surface, Instant::now())
    }

    /// Redessine les fenêtres ouvertes : à appeler quand l'état du logiciel change (le volet,
    /// les Paramètres et les autres fenêtres le montrent aussitôt : elles sont dessinées avec
    /// la fenêtre principale).
    pub fn redessiner(&self) {
        if let Some(c) = verrou(&self.contexte).as_ref() {
            c.request_repaint();
        }
    }

    pub fn fermer_la_fenetre(&self) {
        self.fermer.store(true, Ordering::Release);
        self.redessiner();
    }

    /// Les Paramètres ouverts passent devant.
    pub fn mettre_devant(&self) {
        self.autres.devant(crate::parametres::CLE_FENETRE);
    }

    /// Montre les Paramètres sur cette page : ouverts, ils passent devant et changent de
    /// page ; sinon, ils s'ouvrent.
    pub fn montrer_page(&self, demandes: &Demandes, page: Option<String>) {
        if self.ouvert() == Ouvert::Parametres {
            if page.is_some() {
                *verrou(&self.page_voulue) = page;
            }
            self.mettre_devant();
        } else {
            let _ = demandes.send(Demande::Parametres(page));
        }
    }

    /// Fin du programme : fermer la fenêtre ouverte, quelle qu'elle soit, et ne plus rien
    /// préparer.
    pub fn tout_fermer(&self) {
        self.quitter.store(true, Ordering::Release);
        self.surface_prete.store(0, Ordering::Release);
        self.fermer.store(true, Ordering::Release);
        if let Some(c) = verrou(&self.contexte).as_ref() {
            c.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
