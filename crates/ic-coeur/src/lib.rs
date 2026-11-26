//! Le cœur pur de l'interface commune (`docs/ORGANIGRAMME.md`, §3.1 et §10).
//!
//! Tout ce qui se **décide** vit ici : quel geste ouvre quoi, où poser le volet, quelle page
//! montrer, que remettre à « Annuler », quand un fil est bloqué, quand passer au secours. Le
//! cœur reçoit des faits (un clic, une taille d'écran, l'instant) et rend des décisions.
//!
//! Ce qu'il n'a pas, et que le compilateur lui interdit :
//! - aucune dépendance : ni egui, ni eframe, ni winit, ni Windows (`Cargo.toml` vide) ;
//! - aucun `unsafe` (`forbid`, ci-dessous) ;
//! - aucun disque, réseau, horloge ni système : l'instant arrive en paramètre.
//!
//! `tests/etages.rs`, à la racine du dépôt, le vérifie aussi en lisant ces fichiers.
//!
//! La crate `interface-commune` reprend ces types et ces fonctions aux chemins d'aujourd'hui
//! (`interface_commune::place::Place`, `interface_commune::textes::FRANCAIS`…) : les logiciels
//! ne prennent jamais `ic-coeur` en direct.
//!
//! La liste des modules est écrite une fois pour toutes par le bloc 0 (`PLAN-REFONTE.md`, §3) :
//! chaque bloc remplit les siens, sans toucher à ce fichier. Un module peut devenir un dossier
//! (`contrat.rs` et `contrat/…`) sans changer cette liste.

#![forbid(unsafe_code)]

// Bloc A, le contrat : les types que le logiciel fournit ou reçoit et qui ne dessinent pas
// (`Identite`, `Etat`, `Communs`, `Valeur`, `Page`, `Instantane`, `Reveil`, `Niveau`,
// `menu::Action`, les clés communes…). Voie 1.
pub mod contrat;

// Bloc B, les textes : chaque mot de la bibliothèque, en français et en anglais ; les
// raccourcis en toutes lettres ; l'espace insécable avant chaque unité (C-98). Voie 1.
// Une ligne de leur documentation écrit « <Logiciel> » sans guillemets de code ; les textes ne
// se touchent pas dans ce bloc (le bloc des textes les tient), d'où cette seule tolérance.
#[allow(rustdoc::invalid_html_tags)]
pub mod textes;

// Bloc C, le thème et les mesures : palette, couleurs d'état et des pastilles, glyphes,
// animation, mesures du volet, du menu et des Paramètres, relecture de Windows toutes les
// 2 s (C-90). Voie 2.
pub mod theme;

// Bloc D, les gestes et les secours : l'aiguilleur des clics, l'anti-rebond, les pertes de
// fenêtres, la décision « bloqué » de la sonde. Voie 1.
pub mod gestes;

// Bloc D, le placement : où poser le volet, le petit menu, un menu de choix, les Paramètres.
// Voie 1.
pub mod placement;

// Bloc E, le modèle des Paramètres : pages, recherche, « Annuler » (C-70), « Conserver ? »,
// présentation de la mise à jour. Voie 2.
pub mod parametres;
