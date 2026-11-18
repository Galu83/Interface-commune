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
- **Plusieurs icônes, la boucle du logiciel** (`zone`, pour PowerPeek). Un logiciel qui
  garde sa boucle de messages crée `Zone::creer(nom, graine)` sur son fil : `poser(id,
  &Image, infobulle)` ajoute ou change une icône (GUID stable tiré de `graine`, du chemin du
  programme et de `id`), `retirer(id)`, `notifier(id, titre, texte, Niveau)` pose la
  notification sur cette icône, `rect(id)`. Après chaque `DispatchMessageW`, `evenements()`
  rend les gestes : l'icône, le `Message` (appui, relâche, double-clic, sélection, clavier,
  menu contextuel, clic sur la notification), le point du geste et la place de l'icône ;
  `Message::geste()` le passe à `clics::Clics`. L'Explorateur redémarré : les icônes sont
  remises seules. `menu_windows(point, &[LigneMenu])` : le menu gris de Windows de secours,
  avec cases cochées, lignes grisées et sous-menus. `horloge::Horloge` ne change pas.
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
- **Nouveau dans les pages communes (point 7)**, tout facultatif :
  - cartes propres dans Mise à jour et À propos : `Reglages::dessiner_mise_a_jour`,
    `Reglages::dessiner_a_propos` ;
  - `Communs::demarrage_bloque` : « Démarrer avec Windows » grisé, la raison à la place de
    l'explication ;
  - `Communs::recherches` (interrupteur « Chercher les mises à jour », clé
    `cles::RECHERCHES`) et `Communs::dossier_automatique` (bouton « Automatique », action
    `cles::DOSSIER_AUTO`) ; les états `EtatMiseAJour::Coupee` et `Information(texte)` se
    montrent comme des informations (dossier introuvable, refusé…), pas comme des erreurs ;
  - `Reglages::retablir_explication` : la phrase de la confirmation de « Rétablir » ;
  - `Communs::fichier_licences` : bouton « Voir les licences » (action `cles::LICENCES`) ;
    les crédits passent à la ligne ;
  - `Contexte::changer_groupe` : plusieurs réglages changés d'un geste, une seule bulle
    « Annuler » qui les remet tous (`Annulation::annuler` rend maintenant une liste) ;
  - `Simple::version_prete` : la fenêtre « <Logiciel> x.y.z est prête » (« Plus tard »,
    « Installer maintenant ») à ouvrir avec `ouvrir_simple` ;
  - déjà dans la bibliothèque : l'anti-rebond du clic sur l'icône (`fenetres::ANTI_REBOND`,
    400 ms), le menu de secours sans OpenGL, le journal des temps d'ouverture (première
    ouverture et ouverture lente, au-delà de 150 ms), « Ouvrir le journal » dans le petit
    menu par défaut.
  Un `Communs { … }` écrit en entier doit ajouter les nouveaux champs (ou finir par
  `..Default::default()`).
- **Même volet et même petit menu pour tous** (point 6). Le volet se décrit par des briques
  (`Logiciel::briques`, module `briques`) que la bibliothèque dessine : interrupteur, choix
  coché avec raccourci à droite, choix segmenté, appareil (icône, nom, %, barre, sous-ligne),
  titre de section, liste de raccourcis, action. Un interrupteur ou un choix segmenté appelle
  `Reglages::changer` sans fermer le volet ; les autres rendent leur clé à
  `Reglages::action`. `dessiner_rapides` ne sert plus qu'au passage, puis disparaîtra.
- **Les mesures, écrites une fois** dans `mesures` : petit menu comme Windows 11 (lignes de
  32, écart de 2, marge de 4, séparateur de 9, icône 16 centrée à 20, texte à 42) ; volet
  (briques de 36, choix segmenté de 32, écart de 2, section de 28, raccourci de 24, marges 6
  et 14) ; Paramètres (celles de PowerPeek). Les lignes du petit menu passent de 36 + 8 à
  32 + 2 : plus serrées, comme KVMSoft et PowerPeek.
 : `Horloge::recevoir_messages(f)`
  reçoit les messages de la fenêtre cachée qui ne sont pas à la bibliothèque (`WM_HOTKEY`, ses
  messages internes, ses minuteries…) et voit la fin de session ; `Poste::sur_le_fil_principal(f)`
  fait du code sur le fil de cette fenêtre (`RegisterHotKey` sur l'adresse reçue). Plus besoin
  d'une boucle de messages à soi.
- **Notifications à trois niveaux** : `Poste::notifier_avec(texte, Niveau, AuClic)`
  (information, avertissement, erreur). **Icône selon l'état** : `Logiciel::icone_etat`, au
  plus un changement par seconde.
- **Fenêtre vide au démarrage corrigée** : chaque fenêtre préparée d'avance (volet et petit
  menu, Paramètres, autres fenêtres) reste voilée (`DWMWA_CLOAK`) jusqu'à son ouverture pour de
  bon ; eframe 0.37 la montrait d'office à sa première image. Vocalis peut retirer son
  contournement `voile_volet.rs`.
- **Fenêtres flottantes** (Vocalis : carte de réponse, pastille, panneau de traduction) :
  `Ouverture::sans_focus` pose `WS_EX_NOACTIVATE` (et `WS_EX_TOOLWINDOW` sans barre des tâches) et
  montre la fenêtre par `SW_SHOWNOACTIVATE` (`systeme::sans_focus`, `systeme::montrer_sans_activer`) :
  elle ne prend jamais le clavier. `Contenu::prete` (vrai par défaut) garde la fenêtre cachée tant
  que le contenu n'est pas prêt (taille connue, place attendue) ; `Ouverture::pose_par_le_contenu`
  : la bibliothèque ne centre pas la fenêtre, le contenu la pose lui-même (près d'une zone).
  `AutresFenetres::ouvrir`, `fermer` et `redessiner`, appelés depuis un autre fil quand la boucle
  est au repos (volet caché), réveillent maintenant la fenêtre principale (`systeme::reveiller`) :
  sans cela, la fenêtre demandée ne se dessinait pas. Une fenêtre `sans_cadre` ou `sans_focus`
  est reposée à chaque image cachée (`systeme::sans_cadre`, `systeme::sans_focus`) : winit réécrit
  les styles de la fenêtre à chaque changement de ses réglages (visibilité, boutons) et efface ceux
  qu'il ne connaît pas. `systeme::fenetre_du_fil(titre, prises)` ne rend qu'une fenêtre de la classe de
  winit et pas déjà attribuée à une autre : une fenêtre cachée étrangère du même titre (celles que
  le pilote OpenGL de NVIDIA crée dans chaque programme) n'est jamais prise pour une fenêtre du
  logiciel, donc jamais montrée.
- **Licences** : copier `docs/LICENCES.md` (avis MIT d'egui et des crates sans fichier de
  licence) dans l'avis de licence livré avec le logiciel.

## Les manques de KVMSoft, couverts (branche `manques-kvmsoft`)

Tout est facultatif et a une valeur par défaut qui ne change rien. KVMSoft peut retirer ses
contournements :

- **Instance unique avec arguments** : `horloge::instance_unique_avec(nom, &Reveil)` avec
  `Reveil::Arguments(vec!["--choisir=2"…])` (transmis à l'instance en cours), `Reveil::Rien`
  (le démarrage automatique en double : rien du tout, sans chercher la fenêtre) ou
  `Reveil::Parametres` (l'ancien comportement). L'instance en cours les reçoit par
  `Horloge::recevoir_arguments(|args| …)` (sur le fil de l'icône, à rendre vite : le second
  lancement attend 3 s au plus). **À retirer de `systeme.rs`** : `instance_deja_la`,
  `instance_unique` (et son `FindWindowW`/`PostMessageW`), `WM_CHOISIR` et son traitement dans
  `Horloge::recevoir_messages` ; `--choisir=N` devient un argument transmis (`--choisir=N`
  relu par `recevoir_arguments`). Les arguments sont bornés (16, 4 Ko), marqués et vérifiés.
- **Fermeture des Paramètres** : `Reglages::peut_fermer(imposee)`, toujours appelé (croix,
  Alt+F4, barre des tâches, fenêtre réduite, `fermer`, `tout_fermer`) ; `false` garde la fenêtre
  ouverte (pas pour `imposee`, quand le logiciel se ferme : la réponse est alors ignorée).
  **À retirer** : l'enregistrement accroché à `retenir_place`, le « dernier signe de vie »
  (`SILENCE`) qui devinait la fermeture ; `peut_fermer` appelle
  `enregistrer_maintenant()` et rend son résultat (`true` si le service est arrêté : rien ne
  partira plus). `retenir_place` ne garde que la place.
- **Brouillon entier et pied de page** : `Reglages::instantane()` (le brouillon, cloné au début
  de chaque image), `Reglages::remettre(&Instantane)` (« Annuler »), `Contexte::brouillon_modifie
  (titre)` (une page a changé le brouillon : une seule bulle « Annuler » qui le remet d'un
  coup) et `Reglages::dessiner_pied` (sous **toutes** les pages, communes comprises : refus en
  rouge, avis du service, « Annuler mes changements »). **À retirer** : `Annulation` de KVMSoft
  (`interface/annulation.rs`), `avant_image`, `bulle_annuler`, le `pied` dessiné par les pages
  (il passe dans `dessiner_pied`).
- **Sélecteur de dossier** : `parametres::fenetre::choisir_dossier(&reglages, depart, titre)`
  (propriétaire : la fenêtre des Paramètres), ou `systeme::choisir_dossier(Some(hwnd), …)` ;
  `systeme::dossier_de_depart` et `systeme::disque_local` (disque du PC seulement, jamais un
  partage réseau ni un lien) y sont aussi. **À retirer de `systeme.rs`** : `choisir_dossier`,
  `dossier_de_depart`, `disque_local`, `sans_redirection`, `racine_de_lecteur` et leurs essais
  (repris tels quels dans la bibliothèque).
- **Fenêtres qui ne « gèlent » pas** : `Horloge::sans_fantome()` (ou `systeme::sans_fantome()`
  avant toute fenêtre) ; et une fenêtre bloquée ne retient pas la fermeture du logiciel plus
  de 5 s (`horloge::ATTENTE_FERMETURE`, le logiciel le note). **À retirer** :
  `systeme::sans_fantome` et son appel dans `main.rs` (devenu `.sans_fantome()` sur l'icône).
- **Captures pour la documentation** : le module `capture` (`volet`, `petit_menu`,
  `parametres`, `toutes`) fait des PNG sans ouvrir aucune fenêtre, avec un logiciel de données
  inventées ; il remplace l'ancien `--capture=<dossier>` de KVMSoft (qui avait déjà disparu
  avec `interface/capture.rs`). Pour le rétablir : `--capture=<dossier>` appelle
  `capture::toutes(&kvm_simule, dossier, 1.0)`. Il ne fait ni la fenêtre « est prête » ni
  `toutes_faites` (à dessiner avec `capture::` si besoin plus tard).

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
- Ses manques sont couverts (point 7) : sa carte « Chercher les mises à jour » et son bouton
  « Automatique » deviennent `Communs::recherches` et `Communs::dossier_automatique` ; ses états
  « coupée » et « introuvable » deviennent `EtatMiseAJour::Coupee` et `Information(…)` (plus
  `Erreur`) ; son volet devient des briques (`Logiciel::briques`) ; sa fenêtre de version
  prête devient `Simple::version_prete` ; ses messages Windows (`WM_HOTKEY`…) passent par
  `Horloge::recevoir_messages`.

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
- Branche `powerpeek-appareil` : la brique `Appareil` prend `plus: PlusAppareil` (image de
  l'appareil dans une tuile, couleur de la barre, texte du niveau, éclair de charge, appareil
  éteint, bouton « Oublier », aide sur le niveau) ; `PlusAppareil::default()` garde l'appareil
  simple. Nouvelles mesures `VOLET_TUILE` (44) et `VOLET_BOUTON` (22), icône `ECLAIR`.
- Retirer ensuite son ancien code de Paramètres ; l'icône, le volet et le petit menu de la
  bibliothèque (`Logiciel`) restent un choix pour plus tard.

## La sonde des fils (branche `kvmsoft-sonde`, pour tous)

Une icône qui ne réagit plus alors que la fenêtre cachée répond encore, c'est un fil pris. La
bibliothèque a maintenant `sonde`, active **par défaut**, sans rien à changer dans un projet :
- une ligne au journal (`Logiciel::noter`) à chaque clic reçu par l'icône (gauche, droit,
  double), avec l'heure UTC ;
- une ligne au début et une à la fin (la durée) de chaque appel qui fait tourner sa propre
  boucle de messages : menu de secours de Windows, menu de la zone, sélecteur de dossier,
  fenêtre des Paramètres seule ;
- un chien de garde, un seul fil pour tous : les fils « icône » (sa boucle de messages) et
  « fenêtres » (le volet) ont chacun un compteur de battements ; toutes les 5 s il est lu, et un
  compteur immobile depuis 15 s donne une ligne (une seule par blocage) avec **la dernière
  étape** du fil (changement de contexte OpenGL, dessin, `Shell_NotifyIcon`…) et, si le fil est
  dans une boucle imbriquée, laquelle et depuis combien de temps ; une autre ligne dit quand il
  repart, avec la durée totale.
Les lignes ne portent jamais rien de l'utilisateur : seulement des noms écrits dans le code.
Coût : un compteur atomique et un petit verrou sans attente par étape.

Pour un projet : rien d'obligatoire. `sonde::regler(Delais { pas, blocage })` change les délais
(avant `Horloge::lancer`) ; un fil à soi qui a sa boucle s'inscrit avec `sonde::inscrire`,
bat avec `sonde::battement()`, pose `sonde::etape("…")` avant un appel à risque et entoure un
appel à boucle propre de `sonde::imbriquee("…")`.

## La fenêtre du volet perdue est recréée (branche `kvmsoft-volet-recree`, pour tous)

Le défaut : au bout de quelques heures (une nuit d'écrans en veille, carte NVIDIA, OpenGL), plus
rien ne s'ouvrait au clic sur l'icône. Une panique dans la boucle des fenêtres (eframe garde des
`unwrap()` sur son contexte OpenGL, par exemple `make_current`) remontait de `run_native` et
**tuait le fil des fenêtres en silence** : ses fenêtres détruites avec lui, aucune ligne au
journal (le crochet de panique par défaut écrit sur une console absente), le chien de garde muet
(le fil mort sort de la surveillance), et les clics envoyés à un canal sans personne au bout.

Maintenant, sans rien à changer dans un projet :
- `boucle_des_fenetres` attrape la panique, note « Fenêtre du volet et du petit menu perdue
  (panique : …, à fichier:ligne, étape « … ») : recréée », recrée la fenêtre tout de suite et
  refait le clic qui était en train de s'ouvrir (`Liens::poser_en_cours`) ;
- plus de 3 pertes en 60 s : le menu de secours pour ce clic, la fenêtre recréée au clic
  suivant (jamais de boucle folle) ; un échec de recréation après une première préparation
  est passager (secours pour ce clic, nouvel essai au suivant), plus jamais définitif ;
- une fermeture de la fenêtre par Windows est notée aussi ;
- `GardeDuFil` : si le fil s'arrête quand même, l'icône passe au menu de secours et le journal
  le dit ; un clic dont la demande ne peut plus partir ouvre aussi le menu de secours ;
- `sonde::retenir_les_paniques()` (posé par le fil des fenêtres) garde le lieu de la panique
  (deux derniers éléments du chemin seulement), puis passe la main au crochet déjà en place.

## Pour tous

- `deny.toml` : `allow-git` pour ce dépôt seul.
- Vérifications GitHub : la clé de déploiement (README, « Sur GitHub »).
- Monter de version de la bibliothèque = changer `rev`, relancer les tests, faire essayer.
