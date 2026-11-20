//! L'interface commune des logiciels du même propriétaire, à la façon de Windows 11
//! (règles générales : `interface-windows11.md`, points 1 à 10) :
//!
//! - le thème de Windows suivi en direct (clair ou sombre, accent), la palette Fluent, les
//!   polices Segoe UI Variable et Segoe Fluent Icons, les durées d'animation communes
//!   ([`theme`], [`apparence`], [`polices`]) ;
//! - l'icône près de l'horloge : clic gauche, le volet ; clic droit, le petit menu ;
//!   double-clic, les Paramètres ; clavier ([`clics`], et `horloge` sous Windows) ;
//!   plusieurs icônes à la fois pour un logiciel qui garde sa boucle de messages (`zone`
//!   sous Windows) ;
//! - le volet et le petit menu, dans une seule fenêtre préparée au repos et animée
//!   ([`fenetres`], [`surface`], [`menu`], et `volet` sous Windows) ;
//! - le squelette des Paramètres ([`parametres`]) : en-tête, navigation, pages Général,
//!   Mise à jour et À propos, cartes, interrupteurs, effet immédiat avec « Annuler »,
//!   bandeau de redémarrage, taille retenue, `--parametres <page>`.
//!
//! Rien n'y est propre à un logiciel : le nom, la version, le logo, les pages propres, les
//! actions du petit menu, les réglages rapides du volet viennent du logiciel ; les mots
//! communs, en français ou en anglais, de [`textes`].
//!
//! Deux traits, pour prendre ce dont on a besoin :
//! - [`Reglages`] : ce qu'il faut aux **Paramètres seuls** (un logiciel qui garde son icône,
//!   son menu et sa boucle de fenêtres peut ne prendre qu'eux : [`parametres::Parametres`]
//!   dans sa fenêtre, ou [`autres_fenetres`] et `parametres::fenetre::ouvrir_seuls`) ;
//! - [`Logiciel`] (qui comprend [`Reglages`]) : en plus, l'icône près de l'horloge, le volet
//!   et le petit menu (`horloge::Horloge`).

pub mod apparence;
pub mod autres_fenetres;
pub mod briques;
pub mod capture;
pub mod clics;
pub mod couleurs;
pub mod dessin;
pub mod fenetres;
pub mod icones;
pub mod menu;
pub mod mesures;
pub mod nuancier;
pub mod parametres;
pub mod pastilles;
pub mod place;
pub mod polices;
pub mod seuils;
pub mod sonde;
pub mod surface;
pub mod textes;
pub mod theme;
pub mod touches;
pub mod toutes_faites;
pub mod verification;

#[cfg(windows)]
#[allow(unsafe_code)]
pub mod echelle;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod horloge;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod systeme;
#[cfg(windows)]
pub mod volet;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod zone;

pub use parametres::modele::{Instantane, Page, Valeur};
pub use textes::{Langue, Textes};
pub use theme::{Palette, Rgb};

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

/// Fait le logo à une taille (pixels) : RVBA, ou `None`.
pub type FaireLogo = fn(taille: u32) -> Option<Vec<u8>>;

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

/// Ce qu'un logiciel donne aux Paramètres. Toutes les méthodes sont appelées sur le fil des
/// fenêtres, à chaque image pour celles qui décrivent : elles rendent ce que le logiciel a
/// déjà en mémoire, sans disque, réseau ni attente. Celles qui agissent (`action`,
/// `changer`…) envoient l'ordre au logiciel et rendent la main tout de suite.
pub trait Reglages: Send + Sync + 'static {
    fn identite(&self) -> &Identite;

    /// Les mots communs écrits par la bibliothèque : le français par défaut ;
    /// `Textes::de(Langue::Anglais)` pour l'anglais, ou ses propres textes.
    fn textes(&self) -> &Textes {
        &textes::FRANCAIS
    }

    /// Une action commune choisie ([`cles`]), ou une action du logiciel.
    fn action(&self, cle: &str);

    /// « Rechercher une mise à jour » : la recherche part sur un fil à part ; sa réponse
    /// arrive dans [`Communs::mise_a_jour`].
    fn rechercher_mise_a_jour(&self);

    /// La page « Accueil » (le résumé de l'état) est-elle voulue ?
    fn accueil(&self) -> bool {
        false
    }

    /// Les pages propres au logiciel, dans l'ordre de la navigation.
    fn pages(&self) -> Vec<Page> {
        Vec::new()
    }

    /// Dessine une page propre (ou l'Accueil), sous son titre.
    fn dessiner_page(&self, _cle: &str, _ui: &mut egui::Ui, _page: &mut parametres::Contexte) {}

    /// Les cartes propres au logiciel dans la page Général, après les réglages communs et
    /// avant « Journal » (`dessin::carte`…).
    fn dessiner_general(&self, _ui: &mut egui::Ui, _page: &mut parametres::Contexte) {}

    /// Les cartes propres au logiciel dans la page Mise à jour, après le dossier.
    fn dessiner_mise_a_jour(&self, _ui: &mut egui::Ui, _page: &mut parametres::Contexte) {}

    /// Les cartes propres au logiciel dans la page À propos, après la licence et avant les
    /// crédits.
    fn dessiner_a_propos(&self, _ui: &mut egui::Ui, _page: &mut parametres::Contexte) {}

    /// Ce que « Rétablir les réglages par défaut » remet à zéro, en une phrase (montrée dans
    /// la confirmation) ; `None` : la phrase commune.
    fn retablir_explication(&self) -> Option<String> {
        None
    }

    /// Un point d'alerte sur cette page dans la navigation (quelque chose attend
    /// l'utilisateur). Par défaut : la page Mise à jour, quand une version est prête.
    fn alerte(&self, cle: &str) -> bool {
        cle == parametres::modele::MISE_A_JOUR
            && matches!(self.communs().mise_a_jour, EtatMiseAJour::Prete(_))
    }

    /// Ce que montrent les pages communes.
    fn communs(&self) -> Communs;

    /// Un réglage changé : à appliquer et enregistrer tout de suite (aussi quand
    /// « Annuler » remet la valeur d'avant).
    fn changer(&self, cle: &str, valeur: Valeur);

    /// Le bas de la fenêtre, sous **toutes** les pages (communes comprises) : les messages du
    /// logiciel qui valent pour tout (un enregistrement refusé, un avis du service…), avec
    /// leurs boutons (`dessin::carte`). Rien par défaut.
    fn dessiner_pied(&self, _ui: &mut egui::Ui, _page: &mut parametres::Contexte) {}

    /// **Brouillon entier** : l'état des réglages que le logiciel modifie d'un bloc, pris au
    /// début de chaque image des Paramètres (il doit être rapide : un clone). `None` : le
    /// logiciel change ses réglages un par un ([`Reglages::changer`]) et n'a pas de brouillon.
    /// Voir [`parametres::Contexte::brouillon_modifie`].
    fn instantane(&self) -> Option<Instantane> {
        None
    }

    /// « Annuler » après un [`parametres::Contexte::brouillon_modifie`] : le logiciel reprend
    /// ce brouillon (et l'enregistre comme n'importe quel changement).
    fn remettre(&self, _instantane: &Instantane) {}

    /// Les Paramètres vont se fermer : le logiciel enregistre ce qui attend. `false` refuse la
    /// fermeture (l'enregistrement a échoué : la fenêtre reste ouverte, et
    /// [`Reglages::dessiner_pied`] dit pourquoi) ; sans effet quand `imposee` (le logiciel se
    /// ferme). Toujours appelé, même fenêtre réduite. À ne pas refuser pour toujours : si rien
    /// ne peut plus être enregistré (le logiciel s'arrête), rendre `true`.
    fn peut_fermer(&self, _imposee: bool) -> bool {
        true
    }

    /// La place des Paramètres retenue (gauche, haut, largeur, hauteur, en pixels).
    fn place_parametres(&self) -> Option<place::Place> {
        None
    }

    /// La place des Paramètres à retenir, à leur fermeture.
    fn retenir_place(&self, _place: place::Place) {}
}

/// Ce qu'un logiciel donne à l'interface commune entière : les Paramètres ([`Reglages`]),
/// plus l'icône près de l'horloge (une seule, `horloge::Horloge`, ou plusieurs,
/// `zone::FenetresCommunes`), le volet et le petit menu. Mêmes règles d'appel que
/// [`Reglages`].
pub trait Logiciel: Reglages {
    /// L'état, en haut du volet.
    fn etat(&self) -> Etat;

    /// Les réglages rapides du volet, sous l'état : des briques toutes faites, dessinées par
    /// la bibliothèque (interrupteur, choix, choix segmenté, appareil, section, raccourcis,
    /// action). Un interrupteur ou un choix segmenté change le réglage (`Reglages::changer`)
    /// sans fermer le volet ; les autres rendent leur clé à `Reglages::action`, volet fermé.
    fn briques(&self) -> Vec<briques::Brique> {
        Vec::new()
    }

    /// Les mêmes briques, avec le contexte egui du volet : pour un logiciel qui prépare des
    /// images pour ses briques (textures de [`briques::ImageAppareil`], chargées et gardées
    /// dans ce contexte). C'est elle que la bibliothèque appelle, une fois par image du volet ;
    /// par défaut, elle rend [`Logiciel::briques`]. Un logiciel n'écrit que l'une des deux.
    fn briques_avec_contexte(&self, _ctx: &egui::Context) -> Vec<briques::Brique> {
        self.briques()
    }

    /// Le pied du volet : des briques **fixes, toujours visibles en bas** (sous un séparateur),
    /// que la liste des [`Logiciel::briques`] ne fait jamais défiler (par exemple « Rechercher
    /// les appareils »). Avec une hauteur maximale ([`place::PlacementVolet::hauteur_max`]),
    /// seule la liste défile, dans la place qui reste au-dessus du pied. Mêmes briques et mêmes
    /// gestes que [`Logiciel::briques`] ; vide par défaut (pas de pied).
    fn briques_du_pied(&self) -> Vec<briques::Brique> {
        Vec::new()
    }

    /// Où et comment le volet se place : collé à la barre des tâches comme les volets de
    /// Windows 11 (par défaut), ou contre l'icône ; l'écart en points ; une hauteur maximale,
    /// au-delà de laquelle les briques défilent. Lu à chaque image du volet.
    fn placement_volet(&self) -> place::PlacementVolet {
        place::PlacementVolet::default()
    }

    /// **Passage seulement**, à remplacer par [`Logiciel::briques`] : des réglages rapides
    /// dessinés par le logiciel, sous les briques.
    /// Rend la clé d'une action qui ferme le volet (faite une fois la fenêtre cachée,
    /// par [`Logiciel::action`]) ; un réglage qui ne ferme pas le volet se change ici.
    fn dessiner_rapides(&self, _ui: &mut egui::Ui, _p: &Palette) -> Option<String> {
        None
    }

    /// La largeur qu'il faut aux réglages rapides pour tenir sans couper (points).
    fn largeur_rapides(&self, _ui: &egui::Ui) -> f32 {
        0.0
    }

    /// Les une ou deux actions propres du petit menu.
    fn actions_menu(&self) -> Vec<menu::Action> {
        Vec::new()
    }

    /// L'icône près de l'horloge selon l'état du logiciel (BGRA, comme `Identite::icone`) ;
    /// `None` : l'icône du logiciel. Relue à chaque `Poste::redessiner_icone`, au plus une
    /// fois par seconde (contre le clignotement).
    fn icone_etat(&self, _taille: u32, _barre_sombre: bool) -> Option<Vec<u8>> {
        None
    }

    /// Les lignes communes de plus du petit menu, placées avant « Quitter » (« Ouvrir le
    /// journal » par défaut, dans la langue de [`Reglages::textes`]).
    fn lignes_en_plus(&self) -> Vec<menu::Action> {
        vec![menu::Action::ouvrir_le_journal(self.textes())]
    }

    /// « Quitter » : le logiciel se ferme (il appelle [`fenetres::Liens::tout_fermer`]).
    fn quitter(&self);

    /// Une ligne pour le journal du logiciel. Appelée depuis un fil à part : le fil des
    /// fenêtres ne touche jamais au disque pendant une ouverture.
    fn noter(&self, ligne: String);
}
