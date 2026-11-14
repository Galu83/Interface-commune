# Passer chaque logiciel à l'interface commune

Pour chaque projet : ajouter la dépendance (voir le README), écrire le trait `Logiciel`, retirer
le code devenu commun, garder ses tests propres, puis faire essayer au propriétaire. Une session
de travail par projet, sur sa branche.

## Ce qui change depuis la version 0.1.0 (pour tous)

- **Le trait `Logiciel` est coupé en deux.** `Reglages` porte tout ce que demandent les
  Paramètres (`identite`, `textes`, `action`, `rechercher_mise_a_jour`, `accueil`, `pages`,
  `dessiner_page`, `dessiner_general`, `alerte`, `communs`, `changer`, `place_parametres`,
  `retenir_place`) ; `Logiciel: Reglages` ajoute l'icône, le volet et le petit menu (`etat`,
  `dessiner_rapides`, `largeur_rapides`, `actions_menu`, `lignes_en_plus`, `quitter`,
  `noter`). À faire : déplacer ces méthodes dans un `impl Reglages for …`. Un logiciel qui ne
  veut que les Paramètres n'écrit que `Reglages`.
- **`Identite::glyphe` disparaît**, remplacé par `logo: Option<FaireLogo>` : le vrai logo,
  en RVBA, à la taille demandée (l'en-tête des Paramètres le montre en 32 points, la page À
  propos et la fenêtre simple aussi). `None` : l'icône du logiciel sert de logo.
- **Français ou anglais.** Tous les mots écrits par la bibliothèque viennent de `Textes`
  (`textes::FRANCAIS`, `textes::ANGLAIS`) ; `Reglages::textes` rend ceux de la langue du
  logiciel (par défaut : le français), et peut changer en direct. Prennent désormais
  `&Textes` : `menu::lignes`, `menu::Action::ouvrir_le_journal`, `dessin::interrupteur_ecrit`,
  `modele::pages_communes`, `modele::page_accueil`, `modele::navigation`. Dans une page,
  `Contexte::textes`.
- **Une seule boucle, plusieurs fenêtres.** `Poste::fenetres()` rend les `AutresFenetres` du
  logiciel : `ouvrir(cle, Ouverture, contenu)` (le contenu écrit le trait
  `autres_fenetres::Contenu`), `fermer`, `devant`, `redessiner`, depuis n'importe quel fil.
  Les Paramètres s'ouvrent aussi dans cette boucle (plus de deuxième boucle, plus d'erreur
  « EventLoop can't be recreated »). `Liens::devant` disparaît ; `Liens::page_voulue` est
  partagé.
- **Les Paramètres sans l'icône.** `parametres::fenetre::ouvrir` devient `ouvrir_seuls`
  (fenêtre principale de sa boucle) ; dans la boucle d'un logiciel qui a la sienne :
  `parametres::fenetre::ouvrir_dans_boucle(&autres, &reglages, &page_voulue, page)`, avec
  `autres.lier(Some(ctx))` une fois et `autres.montrer(ctx, &palette)` à chaque image de sa
  fenêtre principale ; ou `Parametres::dessiner(ui, &palette, &reglages)` dans sa propre
  fenêtre.
- **Fenêtres toutes faites** (`toutes_faites`) : `ouvrir_simple` (titre, nouveautés, état,
  barre d'avancement, champ, choix, boutons avec confirmation ; la hauteur suit le contenu)
  et `ouvrir_lecture` (texte lu, mot surligné et suivi, clic sur un mot). Elles se changent
  par leur poignée (`changer`, `fermer`) et rendent les clics par un rappel. Leurs fonctions
  de dessin servent aussi seules.
- **Nouveau, facultatif** : `Reglages::dessiner_general` (cartes propres dans Général) ;
  `Reglages::alerte` (point d'alerte dans la navigation ; par défaut sur Mise à jour quand une
  version est prête ; rendre `false` pour n'en avoir aucun) ; `Communs::dossier_fixe` (cache
  « Choisir… » ; un `Communs { … }` écrit en entier doit l'ajouter) ;
  `Poste::notifier_action` (un clic sur la notification rend une action au logiciel) ;
  `dessin::champ_raccourci` (saisie d'un raccourci clavier).
- **Fenêtre vide au démarrage corrigée** : chaque fenêtre préparée d'avance (volet et petit
  menu, Paramètres, autres fenêtres) reste voilée (`DWMWA_CLOAK`) jusqu'à son ouverture pour de
  bon ; eframe 0.37 la montrait d'office à sa première image. Vocalis peut retirer son
  contournement `voile_volet.rs`.
- **Licences** : copier `docs/LICENCES.md` (avis MIT d'egui et des crates sans fichier de
  licence) dans l'avis de licence livré avec le logiciel.

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
- Depuis 0.1.0 : couper son `impl Logiciel` en `Reglages` + `Logiciel` ; `logo` à la place de
  `glyphe` ; la saisie de ses raccourcis dans ses pages peut prendre `dessin::champ_raccourci`.

## Vocalis

- La branche `claude/interface-rust-987b2a` a déjà une interface Rust : garder ce qui est propre
  à Vocalis (état de l'écoute, choix du micro et de la voix) et le passer par `etat`,
  `dessiner_rapides` et ses pages ; jeter ce qui double la bibliothèque (icône, volet, menu,
  thème, squelette des Paramètres).
- L'état de l'écoute va dans `Etat` (Prêt / En cours / Attention / Problème) ; un changement
  d'état appelle `Poste::redessiner_icone` et `Poste::redessiner`.
- Les textes affichés restent en français ; ajouter à ses tests `verification::caracteres_absents`
  sur ses propres fichiers.
- Depuis 0.1.0, ce qui remplace son code à lui :
  - ses fenêtres simples et sa fenêtre de lecture (`simples.rs`, `fenetres.rs`, et leur repli
    quand une deuxième boucle était refusée) → `toutes_faites::ouvrir_simple` et
    `ouvrir_lecture` sur `Poste::fenetres()` ; sa barre de lecture (langue, pause, vitesse,
    voix) devient la `BarreLecture` qu'il dessine ; ses événements deviennent le rappel ;
  - sa carte propre de la page Général → `Reglages::dessiner_general` ;
  - son dossier des installateurs non réglable → `Communs::dossier_fixe = true` ;
  - le retour à Vocalis au clic sur une notification → `Poste::notifier_action(texte,
    probleme, "<son action>")` ;
  - le champ de saisie d'un raccourci → `dessin::champ_raccourci` (`Saisie::Commencee` : il
    suspend ses raccourcis ; `Changee` ou `Annulee` : il les reprend) ;
  - le point d'alerte de sa navigation → `Reglages::alerte`.

## PowerPeek

- PowerPeek est sous licence MIT depuis sa version 0.20.0. Ses Paramètres servent de modèle
  à ceux de la bibliothèque (règles de l'interface, point 7) : en-tête avec le vrai logo (32
  points), nom en 16 points, version en 12, lignes de navigation de 38 points, colonne de 250
  à 340 points selon le plus long nom de page, point d'alerte à 16 points du bord droit,
  fenêtre de 920 × 680 points (au moins 700 × 500), page à 18 points du bord gauche et 22 du
  haut.
- Il peut ne prendre que les Paramètres : écrire `Reglages` seul (ses pages propres,
  `textes()` en anglais quand sa langue est l'anglais, `logo` = son logo), puis, dans sa
  propre boucle, `parametres::fenetre::ouvrir_dans_boucle` avec ses `AutresFenetres`
  (dessinées par sa fenêtre principale), ou `Parametres::dessiner` dans une fenêtre à lui.
  Le titre de la fenêtre est « <logiciel> Settings » en anglais.
- Retirer ensuite son ancien code de Paramètres ; l'icône, le volet et le petit menu de la
  bibliothèque (`Logiciel`) restent un choix pour plus tard.

## Pour tous

- `deny.toml` : `allow-git` pour ce dépôt seul.
- Vérifications GitHub : la clé de déploiement (README, « Sur GitHub »).
- Monter de version de la bibliothèque = changer `rev`, relancer les tests, faire essayer.
