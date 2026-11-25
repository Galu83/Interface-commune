//! Tous les mots que la bibliothèque écrit elle-même (petit menu, volet, Paramètres), en
//! français et en anglais (bloc B, `docs/ORGANIGRAMME.md` §3.1). Aucun texte en dur : un
//! logiciel choisit une langue (`Reglages::textes`, dans `interface-commune`), ou donne les siens
//! en partant d'une des deux (`Textes { quitter: "…", ..textes::FRANCAIS }`).
//!
//! Les modèles qui contiennent `{}` reçoivent leurs valeurs dans l'ordre ([`remplir`]).
//!
//! Ici aussi, parce que c'est du texte et rien d'autre :
//! - les raccourcis en toutes lettres ([`lisible`]) ;
//! - l'espace insécable ([`INSECABLE`]) : avant chaque unité (C-98, [`avec_unite`]), et autour
//!   des guillemets et devant « : ; ! ? » ([`insecables`]).
//!
//! `interface-commune` reprend ces éléments à leurs chemins d'aujourd'hui (`textes`,
//! `touches::lisible`, `dessin::insecables`).

mod catalogue;
#[cfg(test)]
mod essais;
mod raccourcis;

pub use catalogue::{ANGLAIS, FRANCAIS};
pub use raccourcis::lisible;

/// Les langues livrées avec la bibliothèque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Langue {
    #[default]
    Francais,
    Anglais,
}

/// Les textes communs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Textes {
    // Petit menu et volet
    pub parametres: &'static str,
    /// La barre de titre des Paramètres : « Paramètres de {} » (le nom du logiciel).
    pub parametres_de: &'static str,
    pub rechercher_mise_a_jour: &'static str,
    pub quitter: &'static str,
    pub ouvrir_le_journal: &'static str,
    /// La première ligne du petit menu, en gras, quand le logiciel a une fenêtre principale :
    /// « Ouvrir {} » (le nom du logiciel).
    pub ouvrir_le_logiciel: &'static str,
    /// « Version {} » (le numéro).
    pub version: &'static str,

    // Navigation
    pub accueil: &'static str,
    pub general: &'static str,
    pub mise_a_jour: &'static str,
    pub a_propos: &'static str,
    pub rechercher_un_parametre: &'static str,

    // Contrôles
    pub active: &'static str,
    pub desactive: &'static str,
    /// La bulle après un changement : « « {} » modifié. » (le titre du réglage).
    pub modifie: &'static str,
    pub annuler: &'static str,
    pub fermer: &'static str,
    /// Le bouton d'une action qui demande confirmation, au second temps.
    pub confirmer: &'static str,
    pub redemarrer_maintenant: &'static str,
    pub retour: &'static str,
    /// « Conserver ces modifications ? » : la question, la ligne du compte à rebours
    /// (« Retour au réglage d'avant dans {} s », les secondes qui restent), et les deux
    /// boutons.
    pub conserver_question: &'static str,
    pub conserver_retour: &'static str,
    pub conserver: &'static str,
    pub revenir: &'static str,

    // Page Général
    pub demarrer_avec_windows: &'static str,
    pub demarrer_explication: &'static str,
    pub langue: &'static str,
    pub langue_explication: &'static str,
    pub notifications: &'static str,
    pub notifications_explication: &'static str,
    /// Le bouton « Essayer » de la carte « Notifications ».
    pub essayer: &'static str,
    pub journal: &'static str,
    pub journal_explication: &'static str,
    /// « Journal détaillé (dépannage) » : l'interrupteur, et ce qu'il fait.
    pub journal_detaille: &'static str,
    pub journal_detaille_explication: &'static str,
    /// « Effacer le journal » : le bouton, la question de confirmation, sa phrase, et la
    /// réponse qui efface.
    pub effacer_le_journal: &'static str,
    pub effacer_journal_question: &'static str,
    pub effacer_journal_explication: &'static str,
    pub oui_effacer: &'static str,
    pub remise_a_zero: &'static str,
    pub retablir: &'static str,
    pub retablir_explication: &'static str,
    pub retablir_bouton: &'static str,
    pub retablir_question: &'static str,
    pub retablir_question_explication: &'static str,
    pub non: &'static str,
    pub oui_retablir: &'static str,

    // Page Mise à jour
    pub maj_pas_cherche: &'static str,
    pub maj_a_jour: &'static str,
    pub maj_recherche: &'static str,
    /// « Version {} disponible » (le numéro).
    pub maj_prete: &'static str,
    /// Le titre quand la recherche a échoué (la raison est en dessous, en gris).
    pub maj_erreur: &'static str,
    /// « Dernière recherche : {} » (la date, déjà écrite par le logiciel).
    pub derniere_recherche: &'static str,
    /// « Version actuelle : {} ».
    pub version_actuelle: &'static str,
    pub installer_maintenant: &'static str,
    pub nouveautes: &'static str,
    pub reglages: &'static str,
    pub dossier_installateurs: &'static str,
    pub choisir: &'static str,
    /// Le bouton d'un champ de raccourci clavier.
    pub changer_raccourci: &'static str,
    /// Pendant la saisie d'un raccourci.
    pub appuyer_touches: &'static str,

    // Page À propos
    pub licence: &'static str,
    /// « {} est sous licence {}. » (le nom, la licence).
    pub sous_licence: &'static str,
    pub credits: &'static str,
    /// « Voir les licences » (le fichier complet des licences).
    pub voir_licences: &'static str,
    pub chercher_mises_a_jour: &'static str,
    pub recherches_coupees: &'static str,
    /// N'est plus affiché (redite de la carte du dossier) ; gardé pour la compatibilité.
    pub recherches_automatique: &'static str,
    /// N'est plus affiché (redite de la carte du dossier) ; gardé pour la compatibilité.
    pub recherches_dossier: &'static str,
    pub automatique: &'static str,
    pub maj_coupee: &'static str,
    /// La fenêtre « <Logiciel> x.y.z est prête » : son titre (le nom, la version), sa phrase,
    /// et le bouton qui la ferme.
    pub prete_titre: &'static str,
    pub prete_explication: &'static str,
    pub plus_tard: &'static str,
}

impl Textes {
    /// Les textes d'une langue livrée.
    pub fn de(langue: Langue) -> &'static Textes {
        match langue {
            Langue::Francais => &FRANCAIS,
            Langue::Anglais => &ANGLAIS,
        }
    }
}

impl Default for Textes {
    fn default() -> Self {
        FRANCAIS
    }
}

/// Remplit les `{}` d'un modèle, dans l'ordre ; un `{}` de trop reste vide.
pub fn remplir(modele: &str, valeurs: &[&str]) -> String {
    let mut sortie = String::with_capacity(modele.len() + 16);
    let mut v = valeurs.iter();
    let mut reste = modele;
    while let Some(i) = reste.find("{}") {
        sortie.push_str(&reste[..i]);
        if let Some(x) = v.next() {
            sortie.push_str(x);
        }
        reste = &reste[i + 2..];
    }
    sortie.push_str(reste);
    sortie
}

/// L'espace insécable (U+00A0). Entre un nombre et son unité (C-98 : « 15\u{a0}s »,
/// « 42\u{a0}% »), et là où la typographie française l'attend ([`insecables`]) : la ligne ne se
/// coupe jamais à cet endroit.
pub const INSECABLE: char = '\u{a0}';

/// Un nombre et son unité, séparés par l'espace insécable (C-98) : « 42\u{a0}% », « 15\u{a0}s ».
/// Sans unité, le nombre seul.
pub fn avec_unite(nombre: impl std::fmt::Display, unite: &str) -> String {
    if unite.is_empty() {
        nombre.to_string()
    } else {
        format!("{nombre}{INSECABLE}{unite}")
    }
}

/// Pour un texte qui passe à la ligne : les espaces collées aux guillemets et devant « : ; ! ? »
/// deviennent insécables, comme en typographie française (jamais un « » » seul en début de
/// ligne).
pub fn insecables(texte: &str) -> String {
    let mut s = String::with_capacity(texte.len());
    let mut car = texte.chars().peekable();
    let mut avant: Option<char> = None;
    while let Some(c) = car.next() {
        let colle = c == ' '
            && (avant == Some('«')
                || car
                    .peek()
                    .is_some_and(|n| matches!(n, '»' | ':' | ';' | '!' | '?')));
        s.push(if colle { INSECABLE } else { c });
        avant = Some(c);
    }
    s
}
