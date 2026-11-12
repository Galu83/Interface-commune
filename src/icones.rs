//! Les icônes communes : des caractères de Segoe Fluent Icons (Windows 11), qui existent
//! aussi dans Segoe MDL2 Assets (Windows 10, le repli). Un test vérifie chacune dans les deux
//! polices. Les icônes de « Paramètres », « Mise à jour » et « À propos » sont les mêmes dans
//! tous les logiciels (règles : interface Windows 11, points 7 et 10).
//!
//! Un logiciel ajoute les siennes dans son propre fichier d'icônes, et les fait vérifier par
//! [`crate::verification`].

/// « Paramètres » (petit menu, roue dentée du volet) et la page « Général » : la roue dentée.
pub const REGLAGES: char = '\u{E713}';
/// « Rechercher une mise à jour » et la page « Mise à jour » : les deux flèches en cercle
/// de Windows Update. Aussi l'état « en cours ».
pub const MISE_A_JOUR: char = '\u{E895}';
/// La page « À propos », une information.
pub const A_PROPOS: char = '\u{E946}';
/// « Quitter ».
pub const QUITTER: char = '\u{E7E8}';
/// La page « Accueil ».
pub const ACCUEIL: char = '\u{E80F}';
/// État « prêt », et le choix actif d'une liste.
pub const COCHE: char = '\u{E73E}';
/// État « à regarder » (orange).
pub const ATTENTION: char = '\u{E7BA}';
/// État « problème » (rouge).
pub const PROBLEME: char = '\u{E783}';
/// « Ouvrir le journal ».
pub const JOURNAL: char = '\u{E8A5}';
/// Le dossier des installateurs.
pub const DOSSIER: char = '\u{E8B7}';
/// « Installer maintenant ».
pub const INSTALLER: char = '\u{E896}';
/// « Démarrer avec Windows ».
pub const DEMARRAGE: char = '\u{E768}';
/// « Langue ».
pub const LANGUE: char = '\u{E774}';
/// « Notifications ».
pub const NOTIFICATIONS: char = '\u{EA8F}';
/// « Rétablir les réglages par défaut », « Redémarrer maintenant ».
pub const RETABLIR: char = '\u{E72C}';
/// « Annuler » (la bulle après un changement).
pub const ANNULER: char = '\u{E7A7}';
/// Fermer une bulle ou un bandeau.
pub const FERMER: char = '\u{E711}';
/// « Rechercher un paramètre ».
pub const RECHERCHER: char = '\u{E721}';
/// Le chevron « › » d'une carte qui ouvre une sous-page.
pub const CHEVRON: char = '\u{E76C}';
/// Le retour d'une sous-page à sa page.
pub const RETOUR: char = '\u{E72B}';
/// La licence et les crédits.
pub const LICENCE: char = '\u{E8D7}';

/// Toutes les icônes communes, pour le contrôle des polices.
pub const TOUTES: [char; 21] = [
    REGLAGES,
    MISE_A_JOUR,
    A_PROPOS,
    QUITTER,
    ACCUEIL,
    COCHE,
    ATTENTION,
    PROBLEME,
    JOURNAL,
    DOSSIER,
    INSTALLER,
    DEMARRAGE,
    LANGUE,
    NOTIFICATIONS,
    RETABLIR,
    ANNULER,
    FERMER,
    RECHERCHER,
    CHEVRON,
    RETOUR,
    LICENCE,
];
