//! Les deux traits que le logiciel écrit : [`Reglages`] (les Paramètres seuls) et [`Logiciel`]
//! (en plus, l'icône, le volet et le petit menu). Repris à la racine de la bibliothèque.

use crate::{
    Communs, Etat, EtatMiseAJour, FenetrePrincipale, Identite, Instantane, Page, Textes, Valeur, briques,
    menu, parametres, place, textes,
};

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

    /// La **fenêtre principale** du logiciel, s'il en a une à lui (règles : interface
    /// Windows 11, point 10, « double-clic = ouvrir le logiciel »). Avec elle :
    /// - le double-clic sur l'icône l'ouvre (sans elle : les Paramètres) ;
    /// - le petit menu commence par « Ouvrir <logiciel> », en gras ;
    /// - relancer le logiciel déjà ouvert l'ouvre (sans elle : les Paramètres) ;
    /// - avec [`FenetrePrincipale::clic_gauche`], le clic gauche l'ouvre aussi, au lieu du
    ///   volet.
    ///
    /// `None` par défaut : rien ne change.
    fn fenetre_principale(&self) -> Option<FenetrePrincipale> {
        None
    }

    /// Ouvre la fenêtre principale (ou la met devant si elle l'est déjà). Appelée seulement
    /// si [`Logiciel::fenetre_principale`] en déclare une, depuis le fil de l'icône : elle
    /// **ne doit pas attendre** (le logiciel reçoit l'ordre et rend la main).
    fn ouvrir_fenetre_principale(&self) {}

    /// « Quitter » : le logiciel se ferme (il appelle [`crate::fenetres::Liens::tout_fermer`]).
    fn quitter(&self);

    /// Une ligne pour le journal du logiciel. Appelée depuis un fil à part : le fil des
    /// fenêtres ne touche jamais au disque pendant une ouverture.
    fn noter(&self, ligne: String);
}
