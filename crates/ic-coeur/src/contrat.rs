//! Bloc A, le contrat (`docs/ORGANIGRAMME.md`, §3.1) : les types que le logiciel fournit à
//! l'interface commune ou reçoit d'elle, et qui ne dessinent pas.
//!
//! La crate `interface-commune` les reprend aux chemins d'aujourd'hui (`interface_commune::Identite`,
//! `interface_commune::cles`, `interface_commune::Valeur`, `interface_commune::horloge::Reveil`…) :
//! les logiciels ne voient pas ce module.
//!
//! Ce qui n'est pas ici, et pourquoi :
//! - les traits `Reglages` et `Logiciel` : leurs méthodes dessinent (egui), ils restent dans
//!   `interface-commune` (bloc F) ;
//! - `menu::Action` : sa fonction `Action::ouvrir_le_journal` prend les textes (`Textes`,
//!   bloc B) et une icône commune (`icones`, bloc C). Elle vit avec le petit menu, dans
//!   [`crate::gestes::menu`] (bloc D), et non ici.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

#[cfg(test)]
mod essais;

/// Fait le logo à une taille (pixels) : RVBA, ou `None`.
pub type FaireLogo = fn(taille: u32) -> Option<Vec<u8>>;

/// Qui est le logiciel.
#[derive(Debug, Clone)]
pub struct Identite {
    /// Son nom (« KVMSoft ») : aussi l'éditeur, jamais un nom de personne.
    pub nom: String,
    /// Sa version, « x.y.z ».
    pub version: String,
    /// Sa licence, pour la page « À propos » (« MIT », « GPL-3.0 »…).
    pub licence: String,
    /// Une phrase qui dit ce qu'il fait (page « À propos »).
    pub description: String,
    /// Son icône près de l'horloge, dessinée : `taille` × `taille` pixels en BGRA (alpha non
    /// prémultiplié), lignes de haut en bas ; `barre_sombre` : la barre des tâches est
    /// sombre. Aussi l'icône des Paramètres (barre des tâches, Alt+Tab).
    pub icone: fn(taille: u32, barre_sombre: bool) -> Vec<u8>,
    /// Son vrai logo, en haut de la navigation des Paramètres et dans « À propos » :
    /// `taille` × `taille` pixels en RVBA (alpha non prémultiplié), ou `None` si le logo ne
    /// peut pas être fait à cette taille. Sans logo, l'icône sert (sur fond sombre si le
    /// thème l'est).
    pub logo: Option<FaireLogo>,
}

impl Identite {
    /// Le logo à cette taille (pixels), en RVBA ; à défaut, l'icône. `None` si ni l'un ni
    /// l'autre n'a la bonne taille.
    pub fn logo_rvba(&self, taille: u32, sombre: bool) -> Option<Vec<u8>> {
        let n = (taille as usize) * (taille as usize) * 4;
        if let Some(logo) = self.logo {
            return logo(taille).filter(|px| px.len() == n);
        }
        let mut px = (self.icone)(taille, sombre);
        if px.len() != n {
            return None;
        }
        for p in px.chunks_mut(4) {
            p.swap(0, 2);
        }
        Some(px)
    }
}

/// L'état montré en haut du volet, en gros et en couleur (règles : point 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenreEtat {
    /// Vert.
    Pret,
    /// Orange : en cours.
    EnCours,
    /// Orange : à regarder.
    Attention,
    /// Rouge.
    Probleme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Etat {
    pub genre: GenreEtat,
    /// Le gros titre du volet : le nom du logiciel (la version s'écrit petite, à sa droite).
    /// Un état (« Prêt », la disposition active, « Changement en cours… ») ne va pas ici mais
    /// dans la ligne d'état.
    pub titre: String,
    /// La ligne d'état, en couleur, sous le titre : « Prêt · <disposition> », « Changement en
    /// cours… ».
    pub ligne: String,
    /// Une explication plus longue, en texte secondaire, s'il y en a une.
    pub detail: Option<String>,
}

/// L'état de la mise à jour (page « Mise à jour »).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EtatMiseAJour {
    #[default]
    Inconnu,
    AJour,
    Recherche,
    /// Une version plus récente est prête : son numéro.
    Prete(String),
    /// La recherche a échoué : pourquoi, en mots simples (montré en rouge).
    Erreur(String),
    /// Les recherches sont coupées par l'utilisateur (montré comme une information).
    Coupee,
    /// Rien de grave, à dire simplement : dossier introuvable, dossier refusé… (montré comme
    /// une information, pas comme une erreur).
    Information(String),
}

/// Une pièce empruntée, pour les crédits de la page « À propos ».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credit {
    pub nom: String,
    pub licence: String,
}

/// Ce que montrent les trois pages communes. Un réglage à `None` n'est pas montré (par
/// exemple « Notifications » pour un logiciel qui n'en envoie pas).
#[derive(Debug, Clone, Default)]
pub struct Communs {
    /// « Démarrer avec Windows » (clé `general.demarrage`).
    pub demarrage: Option<bool>,
    /// « Démarrer avec Windows » ne peut pas changer : pourquoi (l'interrupteur est grisé et
    /// la raison remplace l'explication).
    pub demarrage_bloque: Option<String>,
    /// « Langue » : la langue choisie et celles proposées (clé `general.langue`).
    pub langue: Option<(String, Vec<String>)>,
    /// « Notifications » (clé `general.notifications`).
    pub notifications: Option<bool>,
    /// Un bouton « Essayer » à côté de l'interrupteur « Notifications » (action
    /// `general.notifications_essayer`) : le logiciel envoie alors une bulle d'essai. Sans
    /// effet quand `notifications` est `None`.
    pub notifications_essai: bool,
    /// « Journal détaillé (dépannage) » (clé `general.journal_detaille`) ; `None` : pas montré.
    /// Le logiciel le coupe seul après 24 h (point 7 des règles de sécurité).
    pub journal_detaille: Option<bool>,
    /// « Effacer le journal » (action `journal.effacer`, après confirmation) est montré.
    pub journal_effacable: bool,
    pub mise_a_jour: EtatMiseAJour,
    /// Quand le logiciel a cherché une mise à jour pour la dernière fois, écrit par lui dans
    /// la langue de l'utilisateur (« 20 nov. 2026, 14:05 ») ; `None` : pas encore cherché, rien
    /// n'est montré. Sous le titre de la page « Mise à jour », en gris, avec la version.
    pub derniere_recherche: Option<String>,
    /// Le dossier des installateurs, tel qu'il s'affiche (raccourci, sans le profil).
    pub dossier_installateurs: String,
    /// Le dossier des installateurs ne se choisit pas (le bouton « Choisir… » est caché).
    pub dossier_fixe: bool,
    /// Le dossier peut être trouvé tout seul : `Some(true)` s'il l'est déjà, `Some(false)`
    /// pour montrer le bouton « Automatique » (action `mise_a_jour.dossier_auto`).
    pub dossier_automatique: Option<bool>,
    /// « Chercher les mises à jour » : l'interrupteur qui coupe les recherches (clé
    /// `mise_a_jour.recherches`) ; `None` : pas montré.
    pub recherches: Option<bool>,
    /// Le fichier complet des licences existe : bouton « Voir les licences » dans À propos
    /// (action `licences`).
    pub fichier_licences: bool,
    /// Les nouveautés de la version prête, une par ligne.
    pub nouveautes: Vec<String>,
    pub credits: Vec<Credit>,
}

/// Les clés des réglages et des actions communs, rendues au logiciel.
pub mod cles {
    pub const DEMARRAGE: &str = "general.demarrage";
    pub const LANGUE: &str = "general.langue";
    pub const NOTIFICATIONS: &str = "general.notifications";
    /// « Essayer » : le logiciel envoie une bulle d'essai (carte « Notifications »).
    pub const NOTIFICATIONS_ESSAYER: &str = "general.notifications_essayer";
    /// « Journal détaillé (dépannage) » (interrupteur, page Général).
    pub const JOURNAL_DETAILLE: &str = "general.journal_detaille";
    /// « Effacer le journal », après confirmation.
    pub const JOURNAL_EFFACER: &str = "journal.effacer";
    /// « Ouvrir le journal » (page Général et petit menu).
    pub const JOURNAL: &str = "journal";
    /// « Rétablir les réglages par défaut », après confirmation.
    pub const RETABLIR: &str = "retablir";
    /// « Choisir… » le dossier des installateurs.
    pub const DOSSIER: &str = "mise_a_jour.dossier";
    /// « Installer maintenant ».
    pub const INSTALLER: &str = "mise_a_jour.installer";
    /// « Redémarrer maintenant » (bandeau de redémarrage).
    pub const REDEMARRER: &str = "redemarrer";
    /// « Chercher les mises à jour » (interrupteur, page Mise à jour).
    pub const RECHERCHES: &str = "mise_a_jour.recherches";
    /// « Automatique » : le dossier des installateurs trouvé tout seul.
    pub const DOSSIER_AUTO: &str = "mise_a_jour.dossier_auto";
    /// « Voir les licences » : ouvrir le fichier complet des licences.
    pub const LICENCES: &str = "licences";
}

/// La fenêtre principale d'un logiciel (`Logiciel::fenetre_principale`) et ses réglages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FenetrePrincipale {
    /// Le clic gauche sur l'icône (et Entrée ou Espace) ouvre la fenêtre principale au lieu
    /// du volet. Le volet reste alors sans geste : à ne choisir que pour un logiciel dont la
    /// fenêtre principale est tout l'usage.
    pub clic_gauche: bool,
}

/// Une page de la navigation des Paramètres.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// La clé de la page (`--parametres <clé>`) : minuscules, sans espace.
    pub cle: String,
    pub titre: String,
    pub icone: char,
    /// Des mots de plus pour « Rechercher un paramètre » (les titres de ses cartes…).
    pub mots: Vec<String>,
}

impl Page {
    pub fn new(cle: &str, titre: &str, icone: char) -> Page {
        Page {
            cle: cle.into(),
            titre: titre.into(),
            icone,
            mots: Vec::new(),
        }
    }

    /// Ajoute des mots pour la recherche.
    pub fn avec_mots(mut self, mots: &[&str]) -> Page {
        self.mots.extend(mots.iter().map(|m| (*m).to_string()));
        self
    }
}

/// La valeur d'un réglage, telle que la bulle « Annuler » la rend au logiciel.
#[derive(Debug, Clone, PartialEq)]
pub enum Valeur {
    Booleen(bool),
    Nombre(f64),
    Texte(String),
}

/// L'état d'un **brouillon entier** de réglages, pris avant un changement et rendu au logiciel
/// par « Annuler » (`Reglages::instantane`, `Reglages::remettre`). Le logiciel y met ce qu'il
/// veut (souvent une copie de ses réglages) et le relit par [`Instantane::get`].
///
/// Deux instantanés sont égaux s'ils sont **le même** (deux copies d'un même instantané), pas
/// s'ils contiennent des valeurs égales : la bibliothèque ne sait rien de ce qu'il y a dedans.
#[derive(Clone)]
pub struct Instantane(Arc<dyn Any + Send + Sync>);

impl Instantane {
    pub fn new<T: Any + Send + Sync>(valeur: T) -> Instantane {
        Instantane(Arc::new(valeur))
    }

    /// Ce que le logiciel y a mis, s'il est de ce type.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

impl fmt::Debug for Instantane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Instantane(..)")
    }
}

impl PartialEq for Instantane {
    fn eq(&self, autre: &Instantane) -> bool {
        Arc::ptr_eq(&self.0, &autre.0)
    }
}

/// Ce que fait un second lancement, quand le logiciel tourne déjà : il ne fait rien d'autre
/// que le dire à l'instance en cours (et s'arrête). Le verrou et la transmission sont au
/// bloc N (`horloge::instance_unique_avec`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Reveil {
    /// Ouvre l'instance en cours (le comportement de `horloge::instance_unique`) : sa fenêtre
    /// principale s'il en a une (`Logiciel::fenetre_principale`), sinon ses Paramètres.
    #[default]
    Parametres,
    /// Rien du tout : un démarrage automatique en double, par exemple. La seconde copie
    /// s'arrête sans rien demander à la première.
    Rien,
    /// Transmet ces arguments (`--choisir=2`…) à l'instance en cours, qui les reçoit par
    /// `Horloge::recevoir_arguments`. Au plus `horloge::ARGUMENTS_MAX` arguments, de
    /// `horloge::ARGUMENTS_OCTETS_MAX` octets en tout.
    Arguments(Vec<String>),
}

/// Le niveau d'une notification : son icône dans la bulle de Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Niveau {
    #[default]
    Information,
    Avertissement,
    Erreur,
}
