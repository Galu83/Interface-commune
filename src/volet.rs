//! La fenêtre du volet (clic gauche) et du petit menu (clic droit), à la façon de Windows 11
//! (règles : interface Windows 11, points 6 et 10).
//!
//! Les deux vivent dans **une seule fenêtre**, préparée au repos et gardée cachée
//! ([`veiller`]) : au clic, elle est mise à jour et montrée, jamais reconstruite (ni fenêtre,
//! ni OpenGL, ni polices). C'est la fenêtre principale de la seule boucle de fenêtres du
//! programme : les Paramètres et les autres fenêtres du logiciel sont dessinés avec elle
//! ([`crate::autres_fenetres`]), même quand elle est cachée. Elle apparaît à sa place, prête
//! et déjà visible dès sa première image, en un fondu court (`theme`), et sans fondu si
//! « Effets d'animation » est coupé. Elle se ferme par Échap, un clic à côté ou la perte du
//! focus, en fondu court. Le clic à côté ne dépend pas du focus ([`Veilleur`]) : même si
//! Windows refuse le focus à la fenêtre, un clic en dehors d'elle la ferme toujours.
//!
//! **Garde** : pendant les 250 premières ms ([`GARDE`](crate::fenetres::GARDE)), un clic ou une touche ne choisit rien.
//! Un clic sur une ligne cache la fenêtre ; ce qu'il demande n'est fait qu'une fois la
//! fenêtre cachée : rien de lent dans le fil des fenêtres pendant qu'elle se montre.
//!
//! Refonte, bloc J : le code vit dans `crate::bords::fenetres::volet`, avec les bords
//! Windows des fenêtres ; il est repris ici, aux mêmes chemins.

pub use crate::bords::fenetres::volet::*;
