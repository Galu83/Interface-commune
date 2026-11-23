//! Tous les mots que la bibliothèque écrit elle-même (petit menu, volet, Paramètres), en
//! français et en anglais. Aucun texte en dur : un logiciel choisit une langue
//! ([`Reglages::textes`](crate::Reglages::textes)), ou donne les siens en partant d'une des
//! deux (`Textes { quitter: "…", ..textes::FRANCAIS }`).
//!
//! Les modèles qui contiennent `{}` reçoivent leurs valeurs dans l'ordre ([`remplir`]).

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

pub const FRANCAIS: Textes = Textes {
    parametres: "Paramètres",
    parametres_de: "Paramètres de {}",
    rechercher_mise_a_jour: "Rechercher une mise à jour",
    quitter: "Quitter",
    ouvrir_le_journal: "Ouvrir le journal",
    ouvrir_le_logiciel: "Ouvrir {}",
    version: "Version {}",
    accueil: "Accueil",
    general: "Général",
    mise_a_jour: "Mise à jour",
    a_propos: "À propos",
    rechercher_un_parametre: "Rechercher un paramètre",
    active: "Activé",
    desactive: "Désactivé",
    modifie: "« {} » modifié.",
    annuler: "Annuler",
    fermer: "Fermer",
    confirmer: "Confirmer",
    redemarrer_maintenant: "Redémarrer maintenant",
    retour: "Retour",
    conserver_question: "Conserver ces modifications ?",
    conserver_retour: "Retour au réglage d'avant dans {} s",
    conserver: "Conserver",
    revenir: "Revenir",
    demarrer_avec_windows: "Démarrer avec Windows",
    demarrer_explication: "Le logiciel s'ouvre tout seul à l'ouverture de la session.",
    langue: "Langue",
    langue_explication: "La langue de l'interface.",
    notifications: "Notifications",
    notifications_explication: "Une bulle près de l'horloge quand quelque chose demande ton attention.",
    essayer: "Essayer",
    journal: "Journal",
    journal_explication: "Ce que le logiciel a fait, pour comprendre un souci.",
    journal_detaille: "Journal détaillé (dépannage)",
    journal_detaille_explication: "Plus de lignes dans le journal, pour chercher un souci. Se coupe tout seul après 24 h.",
    effacer_le_journal: "Effacer le journal",
    effacer_journal_question: "Effacer le journal ?",
    effacer_journal_explication: "Le journal et l'ancien journal sont supprimés. Impossible de revenir en arrière.",
    oui_effacer: "Oui, effacer",
    remise_a_zero: "Remise à zéro",
    retablir: "Rétablir les réglages par défaut",
    retablir_explication: "Une confirmation est demandée avant.",
    retablir_bouton: "Rétablir…",
    retablir_question: "Rétablir les réglages par défaut ?",
    retablir_question_explication: "Tous les réglages reviennent à ceux du premier lancement.",
    non: "Non",
    oui_retablir: "Oui, rétablir",
    maj_pas_cherche: "Pas encore cherché",
    maj_a_jour: "Vous êtes à jour",
    maj_recherche: "Recherche en cours…",
    maj_prete: "Version {} disponible",
    maj_erreur: "La recherche a échoué",
    derniere_recherche: "Dernière recherche : {}",
    version_actuelle: "Version actuelle : {}",
    installer_maintenant: "Installer maintenant",
    nouveautes: "Nouveautés",
    reglages: "Réglages",
    dossier_installateurs: "Dossier des installateurs",
    choisir: "Choisir…",
    changer_raccourci: "Changer…",
    appuyer_touches: "Appuyez sur les touches… (Échap : annuler)",
    licence: "Licence",
    sous_licence: "{} est sous licence {}.",
    voir_licences: "Voir les licences",
    chercher_mises_a_jour: "Chercher les mises à jour",
    recherches_coupees: "Coupé : le logiciel ne cherche pas de nouvelle version.",
    recherches_automatique: "Dans le dossier « Installateurs », trouvé tout seul.",
    recherches_dossier: "Dans le dossier des installateurs choisi plus bas.",
    automatique: "Automatique",
    maj_coupee: "Recherche des mises à jour coupée",
    prete_titre: "{} {} est prête",
    prete_explication: "L'installation prend quelques secondes ; le logiciel se rouvre tout seul.",
    plus_tard: "Plus tard",
    credits: "Crédits",
};

pub const ANGLAIS: Textes = Textes {
    parametres: "Settings",
    parametres_de: "{} Settings",
    rechercher_mise_a_jour: "Check for updates",
    quitter: "Quit",
    ouvrir_le_journal: "Open the log",
    ouvrir_le_logiciel: "Open {}",
    version: "Version {}",
    accueil: "Home",
    general: "General",
    mise_a_jour: "Update",
    a_propos: "About",
    rechercher_un_parametre: "Find a setting",
    active: "On",
    desactive: "Off",
    modifie: "“{}” changed.",
    annuler: "Undo",
    fermer: "Close",
    confirmer: "Confirm",
    redemarrer_maintenant: "Restart now",
    retour: "Back",
    conserver_question: "Keep these changes?",
    conserver_retour: "Going back to the previous setting in {} s",
    conserver: "Keep",
    revenir: "Revert",
    demarrer_avec_windows: "Start with Windows",
    demarrer_explication: "The app opens by itself when you sign in.",
    langue: "Language",
    langue_explication: "The language of the interface.",
    notifications: "Notifications",
    notifications_explication: "A bubble near the clock when something needs your attention.",
    essayer: "Try",
    journal: "Log",
    journal_explication: "What the app has done, to understand a problem.",
    journal_detaille: "Detailed log (troubleshooting)",
    journal_detaille_explication: "More lines in the log, to track down a problem. Turns itself off after 24 h.",
    effacer_le_journal: "Clear the log",
    effacer_journal_question: "Clear the log?",
    effacer_journal_explication: "The log and the previous log are deleted. This cannot be undone.",
    oui_effacer: "Yes, clear",
    remise_a_zero: "Reset",
    retablir: "Restore the default settings",
    retablir_explication: "You will be asked to confirm.",
    retablir_bouton: "Restore…",
    retablir_question: "Restore the default settings?",
    retablir_question_explication: "Every setting goes back to how it was at first launch.",
    non: "No",
    oui_retablir: "Yes, restore",
    maj_pas_cherche: "Not checked yet",
    maj_a_jour: "You're up to date",
    maj_recherche: "Checking…",
    maj_prete: "Version {} is available",
    maj_erreur: "The check failed",
    derniere_recherche: "Last checked: {}",
    version_actuelle: "Current version: {}",
    installer_maintenant: "Install now",
    nouveautes: "What's new",
    reglages: "Settings",
    dossier_installateurs: "Folder of installers",
    choisir: "Choose…",
    changer_raccourci: "Change…",
    appuyer_touches: "Press the keys… (Esc: cancel)",
    licence: "Licence",
    sous_licence: "{} is under the {} licence.",
    voir_licences: "View licences",
    chercher_mises_a_jour: "Check for updates",
    recherches_coupees: "Off: the app does not look for a new version.",
    recherches_automatique: "In the “Installateurs” folder, found automatically.",
    recherches_dossier: "In the folder of installers chosen below.",
    automatique: "Automatic",
    maj_coupee: "Update checks are off",
    prete_titre: "{} {} is ready",
    prete_explication: "Installing takes a few seconds; the app opens again by itself.",
    plus_tard: "Later",
    credits: "Credits",
};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remplir_dans_l_ordre() {
        assert_eq!(remplir("Version {}", &["1.2.3"]), "Version 1.2.3");
        assert_eq!(
            remplir(ANGLAIS.sous_licence, &["Démo", "MIT"]),
            "Démo is under the MIT licence."
        );
        assert_eq!(remplir("{} et {}", &["a"]), "a et ");
        assert_eq!(remplir("rien", &["a"]), "rien");
    }

    /// Chaque modèle a le même nombre de `{}` dans les deux langues, et aucun texte n'est
    /// vide.
    #[test]
    fn deux_langues_completes() {
        let champs = |t: &Textes| -> Vec<&'static str> {
            vec![
                t.parametres,
                t.parametres_de,
                t.rechercher_mise_a_jour,
                t.quitter,
                t.ouvrir_le_journal,
                t.ouvrir_le_logiciel,
                t.version,
                t.accueil,
                t.general,
                t.mise_a_jour,
                t.a_propos,
                t.rechercher_un_parametre,
                t.active,
                t.desactive,
                t.modifie,
                t.annuler,
                t.fermer,
                t.confirmer,
                t.redemarrer_maintenant,
                t.retour,
                t.conserver_question,
                t.conserver_retour,
                t.conserver,
                t.revenir,
                t.demarrer_avec_windows,
                t.demarrer_explication,
                t.langue,
                t.langue_explication,
                t.notifications,
                t.notifications_explication,
                t.essayer,
                t.journal,
                t.journal_explication,
                t.journal_detaille,
                t.journal_detaille_explication,
                t.effacer_le_journal,
                t.effacer_journal_question,
                t.effacer_journal_explication,
                t.oui_effacer,
                t.remise_a_zero,
                t.retablir,
                t.retablir_explication,
                t.retablir_bouton,
                t.retablir_question,
                t.retablir_question_explication,
                t.non,
                t.oui_retablir,
                t.maj_pas_cherche,
                t.maj_a_jour,
                t.maj_recherche,
                t.maj_prete,
                t.maj_erreur,
                t.derniere_recherche,
                t.version_actuelle,
                t.installer_maintenant,
                t.nouveautes,
                t.reglages,
                t.dossier_installateurs,
                t.choisir,
                t.changer_raccourci,
                t.appuyer_touches,
                t.licence,
                t.sous_licence,
                t.credits,
                t.voir_licences,
                t.chercher_mises_a_jour,
                t.recherches_coupees,
                t.recherches_automatique,
                t.recherches_dossier,
                t.automatique,
                t.maj_coupee,
                t.prete_titre,
                t.prete_explication,
                t.plus_tard,
            ]
        };
        let (fr, en) = (champs(&FRANCAIS), champs(&ANGLAIS));
        assert_eq!(fr.len(), en.len());
        for (f, e) in fr.iter().zip(&en) {
            assert!(!f.is_empty() && !e.is_empty());
            assert_eq!(f.matches("{}").count(), e.matches("{}").count(), "{f} / {e}");
        }
        assert_eq!(Textes::de(Langue::Anglais).quitter, "Quit");
    }
}
