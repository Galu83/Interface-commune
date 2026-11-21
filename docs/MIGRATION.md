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

### Passer au volet commun (branche `volet-powerpeek`)

Ce qui manquait au volet commun pour remplacer celui de PowerPeek est ajouté, sans rien
changer pour KVMSoft ni Vocalis (tout est facultatif, avec la valeur d'avant par défaut) :

- **Le contexte egui pour les briques** : `Logiciel::briques_avec_contexte(ctx)` (par défaut,
  elle rend `briques()`). PowerPeek y charge les images de ses appareils
  (`ImageAppareil::texture`) et les garde comme aujourd'hui. Elle est appelée une fois par
  image du volet (avant : deux fois, pour la largeur et pour le dessin).
- **Le placement** : `Logiciel::placement_volet()` rend un `place::PlacementVolet` :
  `contre_l_icone` (en face de l'icône, du côté opposé à la barre, comme `lbt::popup::place`),
  `ecart` en points (converti en pixels réels à l'échelle de l'écran : 8 points pour PowerPeek)
  et `hauteur_max` en points (600 pour PowerPeek ; toujours bornée aussi par la zone de travail
  de l'écran). Au-delà, l'en-tête (titre, roue) et l'état restent en place et **les briques
  défilent**. Hors fenêtre : `surface::dessiner_borne(ui, p, s, l, Some(hauteur))`.
  Le volet borné **grandit avec son contenu** jusqu'à cette hauteur (corrigé : il restait à la
  taille de la fenêtre du moment, donc ne grandissait plus quand des appareils apparaissaient
  après l'ouverture). Capture sans fenêtre : `capture::volet_borne`.
  **Le pied fixe** : `Logiciel::briques_du_pied()` (vide par défaut) rend des briques toujours
  visibles en bas, sous un séparateur (« Rechercher les appareils » pour PowerPeek : à sortir de
  `briques()` pour l'y mettre). Avec une hauteur maximale, seule la liste défile au-dessus du
  pied, avec une barre de défilement visible dès qu'elle dépasse.
- **L'icône par les mécanismes communs** : `zone::FenetresCommunes::lancer(&zone, logiciel,
  parametres, secours)` lance le fil des fenêtres de la bibliothèque (le même que celui
  d'`Horloge`, aussi public : `horloge::lancer_le_fil_des_fenetres`). Chaque événement de
  `zone.evenements()` passe par `communes.geste(&e)` (clics par icône, `Demandes`, `Liens`,
  second clic qui referme, anti-rebond, double-clic vers les Paramètres) ; il rend
  `SuiteDuGeste::Secours { icone, point }` (menu gris : `zone.menu_windows`) ou
  `Notification(icone)`. `secours` est appelé sur le fil des fenêtres : poster un message à
  `zone.fenetre()`, le menu s'ouvre sur le fil de la zone. À « Quitter » :
  `communes.fermer()` (5 s au plus). La logique seule, sans thread : `zone::Aiguilleur` ;
  sans zone : `fenetres::envoyer_le_clic`.
- **La place des Paramètres retenue 1 s après un déplacement**, et jamais celle d'une fenêtre
  réduite ou agrandie : dans les Paramètres communs (`place::SuiviDePlace`), pour tous les
  logiciels. PowerPeek peut retirer `FenetreParametres` (son `REMEMBER_AFTER`) et ouvrir les
  Paramètres communs (`parametres::fenetre::ouvrir_dans`, ou `FenetresCommunes`).

Ce que PowerPeek retire ou change de son côté : sa fenêtre du volet (`gui::host::WindowHost`,
`MenuWindow`, `MenuUi::show`, `lbt::tray::motion` pour le volet), son placement
(`lbt::popup::place`, `MenuRequest`) et son propre fil des fenêtres ; `volet_commun::Volet`
devient son `Logiciel` complet (état, `briques_avec_contexte` avec ses textures,
`placement_volet`, `actions_menu`, `quitter`, `noter`, ses `Reglages`) ; sa boucle de messages
donne les gestes de sa `Zone` à `FenetresCommunes::geste` et appelle
`FenetresCommunes::redessiner` quand un appareil change. L'animation du volet devient celle de
la bibliothèque (fondu de 83 ms, sans glissement).

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

## Le volet et le petit menu s'ouvrent sous le doigt (branche `latence-volet`, pour tous)

Le défaut : une impression d'attente à chaque ouverture. Mesuré image par image
(`examples/mesure_ouverture.rs`, fenêtre voilée, rien à l'écran) : le retard avant
l'animation était déjà minime (2 à 7 ms du clic à la première image) ; c'est l'animation qui
faisait attendre. Sa première image montrée était transparente, le fondu partait de 0 sur
167 ms et le volet montait de 50 px en 250 ms : lisible vers 150 ms, à sa place à 260 ms.

Maintenant, sans rien à changer dans un projet : le volet et le petit menu apparaissent à leur
place, sans glisser, en un fondu de 83 ms parti de 50 % d'opacité ; lisibles dès ~20 ms,
animation finie vers 90 ms. « Effets d'animation » coupés : opaques tout de suite. Dans
`theme` : `VOLET_GLISSEMENT_MS`, `VOLET_FONDU_MS`, `MENU_FONDU_MS`, `VOLET_DECALAGE` et
`Image::decalage` disparaissent ; `OUVERTURE_FONDU_MS`, `OUVERTURE_OPACITE_DE_DEPART` ;
`ouverture(ms, effets)` ne prend plus la surface. `volet::observer` sert aux mesures seulement.

Le symptôme « des fois, un moment sans rien, puis le menu d'un coup, sans animation » : quand
l'échange de la première image (SwapBuffers) fait attendre, l'horloge de l'animation avançait
quand même, et l'image suivante était déjà la dernière. Maintenant l'animation avance image
par image, d'au plus 20 ms par image : elle se joue toujours en entier. Le journal note le
délai du clic à la première image présentée (au-delà de 150 ms) et toute image de l'ouverture
bloquée plus de 100 ms : de quoi voir quand et combien l'écran fait attendre.

## Les Paramètres glissent d'un écran à l'autre (branche `parametres-ecrans-dpi`, pour tous)

Le défaut : sur deux écrans d'échelles différentes (un portable à 150 % sous un écran à
100 %…), une fenêtre glissée d'un écran à l'autre sautait dans tous les sens et revenait sur le
premier écran. La cause est dans winit 0.30 : à chaque changement d'échelle (`WM_DPICHANGED`),
il « repousse » la fenêtre vers l'écran qu'elle quitte, ce qui relance un changement d'échelle.
Le correctif de winit (#4341 : prendre le rectangle que Windows suggère) n'est que dans sa
future version 0.31 ; la bibliothèque le reprend.

Maintenant, sans rien à changer dans un projet : les Paramètres et toutes les fenêtres de
`AutresFenetres` suivent la souris d'un écran à l'autre (`echelle::suivre_windows`), et les
Paramètres seuls sont posés à leur place retenue à chaque image cachée (taille exacte même
rouverts sur un écran d'une autre échelle). Un logiciel qui dessine les Paramètres dans **sa
propre** fenêtre appelle `interface_commune::echelle::suivre_windows(hwnd)` une fois, sur le fil
de cette fenêtre. Dans `place` : `place_au_changement_d_echelle`, `ecran_de`, `centre_de`.

## La version à droite du gros titre du volet (pour tous)

Le volet montre la version (« x.y.z », `Identite::version`) petite et grise, à droite du gros
titre et sur sa ligne de base. Rien à changer dans le code d'un projet. **Le gros titre
(`Etat::titre`) est le nom du logiciel** : un état (disposition, mode…) va dans la ligne d'état
(`Etat::ligne`), pas dans le titre. Un test qui cherchait le texte « Nom x.y.z » dans le volet
cherche maintenant « x.y.z », à droite du titre.

## La roue des Paramètres en haut à droite du volet (pour tous)

La roue dentée n'est plus en bas du volet : elle est **en haut à droite, sur la ligne du
titre** (comme le volet de OneDrive), toujours au même endroit. Il n'y a plus de ligne du
bas (ni trait ni marge) : le volet est plus court d'environ 60 points. Rien à changer dans le
code d'un projet ; même bulle « Paramètres », même clic (`Choisi::Parametres`). Le titre ne
passe jamais sous la roue : il garde au moins 8 points avec elle et se coupe par « … » ; la
version reste entière. Avec une hauteur maximale (`PlacementVolet::hauteur_max`), l'en-tête
(titre, roue) et l'état restent en place et les briques défilent dessous. Au clavier, la roue
est le premier élément du volet (Tab) : elle est la première dans l'ordre de lecture, de haut
en bas, comme dans les volets de Windows 11. Un test qui cherchait la roue en bas du volet la
cherche en haut à droite.

**Note signalée par PowerPeek : un cache de textures et `capture`.** Le contexte egui d'une
capture (`capture::volet`, `capture::toutes`…) est **neuf à chaque passe**. Un logiciel qui
garde d'une passe à l'autre un cache de textures (les images de ses appareils, via
`briques_avec_contexte(ctx)`) rend des **carrés noirs** : le cache pointe vers des textures
d'un ancien contexte. Dans un contexte de capture, recharger les textures à chaque passe (ne
pas réutiliser le cache), ou le vider dès que le contexte change.

## Les raccourcis clavier en toutes lettres (pour tous)

Un raccourci s'affiche comme dans Windows en français : « Ctrl+Maj+K », « Alt+Flèche droite »,
« Win+Échap », jamais en symboles (⇧, ⌃, ⌘, →). Une seule fonction,
`interface_commune::touches::lisible(&str) -> String`, met en forme n'importe quelle écriture
(« ctrl+alt+shift+k », « Ctrl+Alt+→ », « ⌃⇧K », les noms d'egui comme « Down » ou « PageUp ») :
modificateurs d'abord (Win, Ctrl, Alt, Maj), puis la touche ; elle ne change pas deux fois
(`lisible(lisible(x)) == lisible(x)`). Le volet (`Brique::Choix::raccourci`,
`Brique::Raccourcis`) et le champ `dessin::champ_raccourci` l'appliquent seuls, y compris pour
mesurer la largeur : **la valeur gardée par le logiciel ne change pas**, seul l'affichage passe
par elle. Un logiciel qui écrit ses propres raccourcis ailleurs (une info-bulle, un menu qu'il
dessine lui-même) appelle `touches::lisible`. « Commande » (⌘) vaut Ctrl, comme dans egui sous
Windows ; la touche Windows s'écrit « Win » (ou « Super »).

## « Conserver ces modifications ? » (pour tous)

Règle commune des Paramètres : **effet immédiat partout** (la bulle « Annuler » reste), jamais
de bouton Appliquer. Mais un réglage qui **peut couper l'écran, la souris, le clavier ou la
liaison** est appliqué tout de suite, puis une carte demande « Conserver ces modifications ? »,
comme Windows après un changement de résolution d'écran : deux boutons « Conserver » et
« Revenir », et le compte à rebours « Retour au réglage d'avant dans 15 s ». Sans réponse au
bout de 15 s, tout revient comme avant.

Dans la page du réglage, **une ligne de plus** : `changer_a_confirmer` à la place de `changer`.

```rust
fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
    // … la carte, le choix de l'utilisateur : `avant` et `apres` …
    if apres != avant {
        // Au lieu de `page.changer(…)` : appliqué tout de suite, puis « Conserver ? ».
        page.changer_a_confirmer("affichage.mode", "Mode d'affichage",
                                 Valeur::Texte(avant), Valeur::Texte(apres));
    }
}
```

Les mêmes trois formes qu'« Annuler » : `Contexte::changer_a_confirmer(cle, titre, avant,
apres)`, `Contexte::changer_groupe_a_confirmer(titre, changements)` (plusieurs réglages d'un
geste) et `Contexte::brouillon_a_confirmer(titre)` (un brouillon entier, pour les logiciels qui
ont `Reglages::instantane` et `Reglages::remettre`).

**Ce que fait la bibliothèque, sans rien d'autre à écrire :**
- le changement part au logiciel tout de suite (`Reglages::changer`), **sans** bulle
  « Annuler » (la carte la remplace) ;
- « Revenir », **Entrée** (le bouton par défaut, qui a le focus), **Échap**, le délai écoulé et
  la **fermeture des Paramètres** rendent la valeur d'avant **par le même chemin qu'« Annuler »** :
  `Reglages::changer(cle, avant)` (ou `Reglages::remettre(&brouillon)`). Le logiciel n'a donc
  rien de nouveau à gérer : il applique et enregistre comme pour tout changement ;
- « Conserver » : les changements restent, rien n'est rendu ;
- **une seule confirmation à la fois** : un nouveau changement à confirmer pendant le compte à
  rebours **garde l'état d'avant le premier**, et le compte à rebours repart de 15 s ; revenu à la
  valeur de départ, il n'y a plus rien à confirmer ;
- la carte est devant les Paramètres (le reste ne répond plus tant qu'elle est là) ; sombre et
  clair comme le reste ; textes dans `Textes` (`conserver_question`, `conserver_retour` avec le
  nombre de secondes, `conserver`, `revenir`), en français et en anglais.

Le délai est la constante publique `parametres::modele::DUREE_CONSERVER` (15 s). Le calcul pur
(compte à rebours, valeurs d'avant, retour) est `parametres::modele::Confirmation`, testé sans
fenêtre. Pour un logiciel qui dessine les Paramètres dans sa propre boucle : `Parametres::dessiner`
fait tout (il vérifie le délai à chaque image, et demande lui-même la prochaine image à
chaque seconde) ; à la fermeture, `Parametres::fermer(l)` doit être appelé **avant** l'enregistrement
(la fenêtre des Paramètres de la bibliothèque le fait seule). Limite : le délai n'est vérifié
qu'à l'image suivante ; une fenêtre réduite et jamais redessinée ne reviendrait qu'à sa
réouverture ou à sa fermeture.

À quoi s'en servir : un mode d'affichage ou de disposition des écrans, la vitesse ou le bouton
principal de la souris, un raccourci clavier global, l'adresse ou le port de la liaison entre
deux PC. Pas pour un simple thème ou une langue : « Annuler » suffit.

Un exemple sans fenêtre, qui écrit deux captures (clair et sombre) : `cargo run --example
conserver -- <dossier>`.

## Deux briques de plus au volet : `Pastilles` et `ChoixIllustre` (branche `volet-pastilles`, pour tous)

Deux **nouvelles variantes** de `Brique` (aucune brique existante ne change ; un `match` complet sur
`Brique` doit en tenir compte). `Brique::Pastilles { titre, pastilles }` : une rangée d'appareils
(`Pastille` : clé facultative, lettre, couleur, nom, mention « ce PC », glyphe, absent hachuré) sur
2 colonnes, retour à la ligne dès 3 ; avec une clé, un clic ferme le volet et la rend.
`apparence::couleurs_pastilles()` donne 6 couleurs lisibles en clair et en sombre.
`Brique::ChoixIllustre` : un choix à deux lignes avec une illustration de 1 à 4 cases d'écran
(`Case` : lettre, couleur, veille), le raccourci en toutes lettres, et, actif, « ✓ » et
`etiquette_active` en accent ; un clic ferme le volet et rend la clé. `Choix` reste inchangé.
Exemple et captures sans fenêtre : `cargo run --example volet_pastilles -- <dossier>`.

### Le vrai plan des écrans dans un choix illustré (branche `plan-des-ecrans`, pour tous ceux qui utilisent `Case`)

- **`Case` reçoit `place: Option<[i32; 4]>`** : `[x, y, largeur, hauteur]` en pixels du bureau, valeurs
  quelconques (négatives permises). **À faire dans chaque `Case { … }` existante : ajouter `place: None`
  ou `..Default::default()`** (`Case` est désormais `Default`, et reste `Copy` et `PartialEq`).
- Quand **toutes** les cases d'un choix ont une place valide (largeur et hauteur > 0), l'illustration
  n'est plus une rangée de cases égales : c'est le plan des écrans, à l'échelle, centré dans la même
  zone qu'avant (même largeur : le texte ne bouge pas ; toute la hauteur de la ligne). Sinon, la
  rangée d'aujourd'hui. Lettre, couleur et veille ne changent pas ; 4 cases au plus. Dans le plan,
  pas de pied d'écran, et une case de moins de 12 points de haut n'est que sa couleur.
- `pastilles::plan_des_cases(places, zone) -> Vec<Rect>` : le calcul pur, public (échelle = le plus
  petit des deux rapports, proportions gardées, centré, chaque case réduite de 1 point de chaque côté).
- L'exemple `volet_pastilles` montre un plan de 3 écrans (« Trois écrans »).

## Les couleurs dans les Paramètres : `couleurs` (branche `couleurs-parametres`, pour tous, en option)

Nouveau module `couleurs`, **rien d'existant ne change** (`nuancier::pastille` garde son
comportement ; elle partage seulement son dessin avec `nuancier::dessiner_pastille`). Les couleurs
sont celles du volet : `couleurs::palette()` = `apparence::couleurs_pastilles()`, aucune autre liste.
- `couleurs::pastille_de_carte(ui, palette, &Pastille)` : la pastille d'un appareil (carré, lettre,
  hachuré si absent), à poser dans le contrôle d'une `dessin::carte`.
- `couleurs::choisir_couleur(ui, palette, &couleurs, choisie, nom) -> Option<usize>` : une rangée de
  pastilles rondes, comme l'accent de Windows 11 : contour sur la choisie, flèches gauche/droite pour
  passer de l'une à l'autre, Entrée ou Espace pour choisir, contour de focus. Rend le rang choisi.
- `couleurs::carte_couleur(ui, page, glyphe, texte, explication, &couleurs, choisie, annuler)` : la
  carte avec la rangée. `annuler: Some((cle, titre))` : le choix part aussi au logiciel par
  `Reglages::changer(cle, Valeur::Nombre(rang))` et la bulle « Annuler » remet l'ancien rang ;
  `None` : rien n'est signalé, tout reste au logiciel. À toi d'enregistrer le rang (et de colorer
  le volet avec `palette()[rang]`).
Exemple et captures sans fenêtre : `cargo run --example couleurs -- <dossier>`.

## Les logos dans les pastilles et le menu flottant de choix : `menu_choix` (branche `glyphes-menu-choix`, pour tous, en option)

**À faire dans chaque `Pastille { … }` écrite en entier : ajouter `logo: None`** (ou `..` si tu
as un `Default` à toi). `Pastille` n'a pas de `Default` ; `Case` en a un, donc `Case { …, ..Default::default() }`
ne change pas. Le champ s'appelle `logo` (et non `glyphe` : `Pastille::glyphe` existe déjà, c'est le
petit glyphe après la mention « ce PC »).
- **`Pastille::logo` et `Case::logo`** : `Option<char>`, un glyphe de Segoe Fluent Icons dessiné **à la
  place de la lettre**, `None` par défaut = la lettre, comme avant. `couleurs::pastille_de_carte` le
  prend aussi. Le glyphe est blanc ou noir selon la couleur du carré, avec la même règle que la lettre
  (le plus fort contraste des deux, au moins 3:1), en clair et en sombre ; hachuré/absent : grisé.
- **Jamais un carré vide** : si la police d'icônes chargée n'a pas ce glyphe (Windows 10 n'a que Segoe
  MDL2 Assets), la **lettre** s'affiche. Aucune police de plus n'est lue : on interroge celle que
  `polices::polices()` a déjà chargée (`polices::icone_disponible(c)`, réponses retenues). La décision
  est pure : `pastilles::marque(logo, lettre, disponible) -> Option<Marque>`. Pour tes tests :
  `verification::logos_absents(&[…])` rend les logos qui retomberaient sur la lettre.
- **Nouveau module `menu_choix`** : un menu flottant au style des menus de Windows 11, des lignes
  (`Ligne` : clé, logo, lettre, couleur, texte, sous-texte, coche, grisée). Il s'ouvre **dans la
  fenêtre** (une `egui::Area`, jamais une fenêtre de plus), sous la case ou au-dessus si la place
  manque, sans sortir de la fenêtre (défile si même alors il est trop haut).
  - Dans l'état du logiciel : un `MenuChoix` (`ouvrir`, `fermer`, `basculer`, `est_ouvert`, `active`).
    Dans la page : `menu_choix::afficher(ui, palette, &mut menu, id, case, &lignes, "coché")` rend
    `Some(Resultat::Choisi(rang))` ou `Some(Resultat::Ferme)` ; le logiciel appelle `menu.basculer(..)`
    au clic sur la case (un clic sur la case ne ferme pas le menu tout seul).
  - Clavier : flèches (font le tour, sautent les lignes grisées), Début, Fin, Entrée, Échap ; contour
    de focus sur la ligne active (au clavier seulement) ; Échap ou un clic à côté ferme. Souris : le
    survol rend la ligne active, un clic la choisit (une ligne grisée ne se choisit pas).
  - **Lecteur d'écran** : chaque ligne a un nom `texte, sous-texte, coché` (`nom_accessible`, le mot
    « coché » vient du logiciel : français ou anglais), le rôle `MenuItemRadio` quand une ligne est
    cochée. Le sens ne repose pas sur la couleur seule : texte + coche dessinée.
  - **La logique est pure, sans fenêtre** : `suivre(grisees, active, touche) -> Effet`,
    `ligne_initiale`, `placer(ancre, taille, fenetre) -> Rect`, `hauteur_totale`, `largeur_menu`.
- `capture::dessin(taille, options, |ui, palette| …)` : une capture sans fenêtre de ce que dessine le
  rappel (pour documenter un élément hors des Paramètres).
Exemple et captures sans fenêtre, en clair, en sombre, à 150 % et menu au-dessus de la case :
`cargo run --example menu_choix -- <dossier>`.

## Les sections repliables dans les Paramètres (branche `section-repliable`, pour tous, en option)

Rien d'existant ne change. Dans une page, `Contexte` sait dessiner une section repliable, comme
l'« Expander » des Paramètres de Windows 11 :
```ignore
page.section_repliable(ui, "Réglages avancés", "Pour les cas rares.", &["Délai de veille"],
    |ui, page| { /* les cartes de la section ; `page` signale les changements */ });
```
(`section_repliable_avec_icone` ajoute une icône à gauche du titre.) L'en-tête est une carte : titre,
explication, chevron « ∨ » à droite qui tourne de 180° en 83 ms (rien si Windows a coupé les effets
d'animation). **Repliée à chaque ouverture des Paramètres** ; l'état est gardé par page et par titre :
deux sections d'une même page ne doivent pas porter le même titre. Au clavier, l'en-tête prend le
focus ; Entrée et Espace l'ouvrent et la ferment ; pour les lecteurs d'écran : un bouton « ouvert » ou
« fermé ». Le contenu reçoit `page` en second argument (la page ne pourrait pas servir deux fois).
- **La recherche** : quand on arrive sur la page en la choisissant dans « Rechercher un paramètre »,
  `page.recherche()` rend ce qui était tapé (vide sinon, et oublié dès qu'on change de page à la
  main). Si tous les mots tapés sont dans le titre ou dans les `mots` de la section
  (`modele::section_trouvee`, accents et majuscules sans importance), elle s'ouvre une fois ; on peut la
  refermer. `Parametres::aller_a_par_recherche(cle, tape)` fait la même chose depuis le code (captures).
- **À faire dans chaque logiciel** : mettre aussi le titre et les `mots` de la section dans
  `Page::avec_mots` de la page ; sinon la recherche ne trouve pas la page.
- `dessin::carte_chevron` reste pour ouvrir une **sous-page**.

Exemple et captures sans fenêtre (repliée, ouverte) : `cargo run --example section_repliable -- <dossier>`.

## La page « Mise à jour » à la façon de Windows Update

- **Une seule icône selon l'état** (coche verte = à jour, flèche = version disponible, triangle rouge
  = la recherche a échoué), un titre qui le dit (« Vous êtes à jour » / « Version x.y.z disponible »,
  « You're up to date » / « Version x.y.z is available »), une ligne grise dessous (la raison d'une
  erreur, la version installée, la dernière recherche) et **un seul bouton principal** (bleu :
  « Installer maintenant » ou « Rechercher une mise à jour »).
- **Nouveau champ `Communs::derniere_recherche: Option<String>`** : la date de la dernière recherche,
  écrite par le logiciel dans la langue de l'utilisateur. `None` : rien n'est montré. Comme `Communs`
  se remplit avec `..Default::default()`, rien ne casse ; à renseigner pour avoir la ligne complète.
- **Nouveaux champs de `Textes`** (`maj_erreur`, `derniere_recherche`) : seul un logiciel qui écrit
  son propre `Textes` en entier doit les ajouter. Les textes `maj_a_jour` et `maj_prete` changent
  (« Vous êtes à jour », « Version {} disponible »).
- Nouveau : `parametres::presentation_mise_a_jour` (état → icône, ton, titre) et
  `parametres::ligne_grise_mise_a_jour`, testés ; `texte_mise_a_jour` garde sa signature.

Exemple et captures sans fenêtre : `cargo run --example mise_a_jour -- <dossier>`.

## Les pages envoyées après l'ouverture des Paramètres (branche `parametres-pages-tardives`, pour tous)

- **La navigation suit les pages** : à chaque image, si `Reglages::accueil()` ou les clés de
  `Reglages::pages()` ont changé (des pages arrivées après l'ouverture, d'un autre processus par
  exemple), la liste des pages est refaite. La page montrée reste si elle existe encore, sinon
  Général. Seules les clés sont comparées : un titre changé sous la même clé ne refait rien.
- `pages()` est donc appelé à chaque image des Paramètres : il doit rester bon marché (pas de
  lecture de disque).
- Nouveau, public : `Parametres::suivre(&reglages)` (déjà appelé par `Parametres::dessiner`).
  Rien à changer dans les logiciels ; Vocalis peut retirer son contournement (pages envoyées
  d'avance).

## Pour tous

- `deny.toml` : `allow-git` pour ce dépôt seul.
- Vérifications GitHub : la clé de déploiement (README, « Sur GitHub »).
- Monter de version de la bibliothèque = changer `rev`, relancer les tests, faire essayer.
