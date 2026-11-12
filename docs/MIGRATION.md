# Passer chaque logiciel à l'interface commune

Pour chaque projet : ajouter la dépendance (voir le README), écrire le trait `Logiciel`, retirer
le code devenu commun, garder ses tests propres, puis faire essayer au propriétaire. Une session
de travail par projet, sur sa branche.

## KVMSoft (le plus simple : la bibliothèque en vient)

- Remplacer par la bibliothèque : `theme.rs` (sauf le dessin de l'icône, qui reste dans
  KVMSoft et devient `Identite::icone`), le placement, les clics de l'icône, la fenêtre unique du
  menu (`menu.rs`), la boucle de l'icône et le menu de secours.
- Le volet de KVMSoft (écran choisi, dispositions) devient `dessiner_rapides` ; ses actions
  propres (« Relire les écrans »…) deviennent `actions_menu`.
- Ses pages de Paramètres deviennent `pages` + `dessiner_page` ; Général, Mise à jour et À
  propos viennent de la bibliothèque (`communs` donne les valeurs, `changer` les applique).
- À vérifier : les raccourcis clavier globaux restent dans KVMSoft ; les tests de thème et de
  placement de KVMSoft peuvent partir (ils sont dans la bibliothèque).

## Vocalis

- La branche `claude/interface-rust-987b2a` a déjà une interface Rust : garder ce qui est propre
  à Vocalis (état de l'écoute, choix du micro et de la voix) et le passer par `etat`,
  `dessiner_rapides` et ses pages ; jeter ce qui double la bibliothèque (icône, volet, menu,
  thème, squelette des Paramètres).
- L'état de l'écoute va dans `Etat` (Prêt / En cours / Attention / Problème) ; un changement
  d'état appelle `Poste::redessiner_icone` et `Poste::redessiner`.
- Les textes affichés restent en français ; ajouter à ses tests `verification::caracteres_absents`
  sur ses propres fichiers.

## PowerPeek

- PowerPeek est sous GPL : la bibliothèque (MIT) peut être utilisée par lui, l'inverse non.
  Aucun code de PowerPeek n'est entré dans la bibliothèque.
- Son interface actuelle est remplacée en entier : écrire `Logiciel` à partir de ce qu'il montre
  (mesures dans `etat`, réglages rapides, pages propres), puis retirer l'ancien code d'interface.
- Si PowerPeek n'est pas encore en Rust pour son interface, prévoir d'abord ce passage : c'est
  le plus gros des trois chantiers.

## Pour tous

- `deny.toml` : `allow-git` pour ce dépôt seul.
- Vérifications GitHub : la clé de déploiement (README, « Sur GitHub »).
- Monter de version de la bibliothèque = changer `rev`, relancer les tests, faire essayer.
