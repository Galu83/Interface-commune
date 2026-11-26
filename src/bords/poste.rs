//! Le guichet (bloc K de l'organigramme) : la porte entre les fils du logiciel et la
//! bibliothèque. Chaque demande devient un **message**, déposé dans la boîte du bon fil, puis ce
//! fil est **réveillé** ; le guichet ne traite jamais la demande lui-même.
//!
//! Deux destinataires :
//! - **le fil de l'icône** (la fenêtre cachée, bloc M) : notifier, changer l'infobulle ou
//!   l'icône, faire du code sur son fil, quitter. Les messages attendent dans une boîte bornée
//!   ([`CAPACITE`]) ; le réveil est un message Windows posté à la fenêtre cachée
//!   ([`ReveilParMessage`]), qui vide alors la boîte ;
//! - **la boucle des fenêtres de la bibliothèque** (bloc J) : redessiner, ouvrir les
//!   Paramètres, tout fermer. Elles passent par [`Liens`] et le canal [`Demandes`] de la boucle,
//!   qui la réveillent eux-mêmes.
//!
//! Un logiciel qui garde sa propre boucle (bloc J, façon 2) parle à ses fenêtres communes
//! directement, sans le guichet ; la sonde (bloc O) s'appelle de tout fil, sans lui non plus.

use std::collections::VecDeque;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use crate::bords::systeme::fenetre::hwnd;
use crate::bords::verrou::verrou;
use crate::fenetres::{Demandes, Liens};
use crate::horloge::Niveau;

/// Le nombre de messages qui peuvent attendre le fil de l'icône : au-delà, les nouveaux sont
/// refusés (un logiciel qui enverrait sans fin ne remplit pas la mémoire).
pub(crate) const CAPACITE: usize = 64;

/// Ce que fait un clic sur une notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuClic {
    /// Ouvre les Paramètres (à cette page).
    Parametres(Option<String>),
    /// Rend cette action au logiciel (`Reglages::action`) : par exemple, montrer sa propre
    /// fenêtre.
    Action(String),
}

impl Default for AuClic {
    fn default() -> Self {
        AuClic::Parametres(None)
    }
}

/// Un message du guichet au fil de l'icône (traité par la fenêtre cachée, bloc M).
pub(crate) enum Commande {
    Notifier {
        texte: String,
        niveau: Niveau,
        au_clic: AuClic,
    },
    Infobulle(String),
    Icone,
    Quitter,
    /// Du code du logiciel, à faire sur le fil de la fenêtre cachée (son adresse en
    /// paramètre) : `RegisterHotKey`…
    Executer(Box<dyn FnOnce(isize) + Send>),
}

/// La boîte bornée d'un fil : on y dépose de tout fil, le fil destinataire la vide d'un coup.
pub(crate) struct Boite<T> {
    file: Mutex<VecDeque<T>>,
}

impl<T> Default for Boite<T> {
    fn default() -> Self {
        Boite {
            file: Mutex::new(VecDeque::new()),
        }
    }
}

impl<T> Boite<T> {
    /// Dépose un message ; faux s'il est refusé (boîte pleine). Un verrou empoisonné est repris
    /// ([`verrou`]) : la file reste utilisable.
    pub(crate) fn deposer(&self, m: T) -> bool {
        let mut f = verrou(&self.file);
        if f.len() < CAPACITE {
            f.push_back(m);
            true
        } else {
            false
        }
    }

    /// Tous les messages en attente, dans l'ordre d'arrivée ; la boîte est vide après.
    pub(crate) fn vider(&self) -> Vec<T> {
        verrou(&self.file).drain(..).collect()
    }
}

/// Ce que le guichet appelle après chaque dépôt : tirer le fil destinataire de son attente.
/// Appelé de n'importe quel fil, il ne doit jamais attendre.
pub(crate) trait Reveiller: Send + Sync {
    fn reveiller(&self);
}

/// Le réveil du fil de l'icône : un message Windows posté à la fenêtre cachée (son adresse est
/// dans [`Liens::fenetre`] ; rien n'est posté tant qu'elle n'existe pas). Le numéro du message
/// et son jeton sont ceux de la fenêtre cachée, qui seule les connaît (bloc M).
pub(crate) struct ReveilParMessage {
    pub(crate) liens: Arc<Liens>,
    pub(crate) message: u32,
    pub(crate) jeton: usize,
}

impl Reveiller for ReveilParMessage {
    fn reveiller(&self) {
        let h = self.liens.fenetre.load(Ordering::Acquire);
        if h != 0 {
            // SAFETY: la fenêtre est celle du logiciel ; PostMessageW n'attend rien et
            // accepte une fenêtre déjà détruite (il échoue simplement).
            let _ = unsafe { PostMessageW(Some(hwnd(h)), self.message, WPARAM(self.jeton), LPARAM(0)) };
        }
    }
}

/// De quoi parler à l'icône depuis n'importe quel fil (à cloner et garder par le logiciel).
#[derive(Clone)]
pub struct Poste {
    liens: Arc<Liens>,
    demandes: Demandes,
    boite: Arc<Boite<Commande>>,
    reveil: Arc<dyn Reveiller>,
}

impl Poste {
    /// Le guichet : `liens` et `demandes` mènent à la boucle des fenêtres, `reveil` au fil de
    /// l'icône.
    pub(crate) fn nouveau(liens: Arc<Liens>, demandes: Demandes, reveil: Arc<dyn Reveiller>) -> Poste {
        Poste {
            liens,
            demandes,
            boite: Arc::default(),
            reveil,
        }
    }

    /// Le canal des demandes à la boucle des fenêtres.
    pub(crate) fn demandes(&self) -> &Demandes {
        &self.demandes
    }

    /// Les messages qui attendent le fil de l'icône (à appeler sur ce fil, à son réveil).
    pub(crate) fn commandes(&self) -> Vec<Commande> {
        self.boite.vider()
    }

    /// Dépose le message dans la boîte du fil de l'icône, puis le réveille (même si la boîte
    /// est pleine : il la videra).
    fn envoyer(&self, c: Commande) {
        let _ = self.boite.deposer(c);
        self.reveil.reveiller();
    }

    /// Une notification près de l'horloge ; un clic dessus ouvre cette page des Paramètres.
    pub fn notifier(&self, texte: &str, probleme: bool, page: Option<&str>) {
        self.envoyer(Commande::Notifier {
            texte: texte.into(),
            niveau: niveau(probleme),
            au_clic: AuClic::Parametres(page.map(str::to_string)),
        });
    }

    /// Une notification près de l'horloge, à ce niveau ; un clic dessus fait `au_clic`.
    pub fn notifier_avec(&self, texte: &str, niveau: Niveau, au_clic: AuClic) {
        self.envoyer(Commande::Notifier {
            texte: texte.into(),
            niveau,
            au_clic,
        });
    }

    /// Fait ce code du logiciel sur le fil de la fenêtre cachée de l'icône, qui le reçoit en
    /// paramètre (son adresse) : par exemple `RegisterHotKey`, qui exige le fil de la fenêtre.
    /// Les messages arrivent ensuite au récepteur
    /// ([`Horloge::recevoir_messages`](crate::horloge::Horloge::recevoir_messages)).
    pub fn sur_le_fil_principal(&self, f: impl FnOnce(isize) + Send + 'static) {
        self.envoyer(Commande::Executer(Box::new(f)));
    }

    /// Une notification près de l'horloge ; un clic dessus rend cette action au logiciel
    /// (`Reglages::action`), pour revenir à sa propre fenêtre par exemple.
    pub fn notifier_action(&self, texte: &str, probleme: bool, action: &str) {
        self.envoyer(Commande::Notifier {
            texte: texte.into(),
            niveau: niveau(probleme),
            au_clic: AuClic::Action(action.into()),
        });
    }

    /// Le texte qui s'affiche au survol de l'icône.
    pub fn infobulle(&self, texte: &str) {
        self.envoyer(Commande::Infobulle(texte.into()));
    }

    /// L'icône doit être redessinée (son état a changé).
    pub fn redessiner_icone(&self) {
        self.envoyer(Commande::Icone);
    }

    /// L'état a changé : le volet ou les Paramètres ouverts se redessinent.
    pub fn redessiner(&self) {
        self.liens.redessiner();
    }

    /// Ouvre les Paramètres à cette page (une notification, une demande du logiciel).
    pub fn ouvrir_parametres(&self, page: Option<&str>) {
        self.liens.montrer_page(&self.demandes, page.map(str::to_string));
    }

    /// Ferme tout : les fenêtres, l'icône, puis rend la main à `Horloge::lancer`.
    pub fn quitter(&self) {
        self.liens.tout_fermer();
        self.envoyer(Commande::Quitter);
    }

    pub fn liens(&self) -> &Arc<Liens> {
        &self.liens
    }

    /// Les autres fenêtres du logiciel (une fenêtre simple, une fenêtre de lecture…),
    /// ouvertes et fermées dans la boucle de la bibliothèque, depuis n'importe quel fil (voir
    /// [`crate::autres_fenetres`]).
    pub fn fenetres(&self) -> crate::autres_fenetres::AutresFenetres {
        self.liens.autres.clone()
    }
}

/// Le niveau d'une notification simple : un problème, ou une information.
fn niveau(probleme: bool) -> Niveau {
    if probleme {
        Niveau::Erreur
    } else {
        Niveau::Information
    }
}

#[cfg(test)]
mod essais;
