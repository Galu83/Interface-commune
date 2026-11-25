# État

- **Version** : 0.1.0 publiée sur `main` (dépôt privé). La branche `manques-projets` ajoute ce
  qui manquait aux projets (pas encore fusionnée, numéro de version inchangé : c'est à
  l'Atelier de le choisir).
- **Fait sur `main`** : thème Windows 11, icône près de l'horloge, volet préparé au repos,
  petit menu, squelette des Paramètres, contrôles des polices, exemple.
- **Fait sur `manques-projets`** :
  - plusieurs fenêtres dans la seule boucle de la bibliothèque (`autres_fenetres`), les
    Paramètres compris ;
  - les Paramètres au dessin de PowerPeek, en français ou en anglais (`textes`), et seuls
    (trait `Reglages`) ;
  - les fenêtres toutes faites (`toutes_faites`) : fenêtre simple et fenêtre de lecture ;
  - les petits manques de Vocalis : cartes propres dans Général, « Choisir… » masquable,
    point d'alerte, champ de raccourci, notification qui rend une action, icône sans variable
    globale ;
  - les avis MIT des crates sans fichier de licence (`docs/LICENCES.md`) ;
  - le même volet et le même petit menu pour tous : briques toutes faites, mesures communes
    (`mesures`) ;
  - les manques de KVMSoft : messages Windows sur la boucle commune, notifications à trois
    niveaux, icône selon l'état, pages communes plus complètes (cartes propres partout,
    démarrage grisé, recherches coupées, « Automatique », licences, « Annuler » en groupe),
    fenêtre « est prête » toute faite.
  - tests, clippy, cargo deny verts sur le PC, sans aucune fenêtre ouverte.
- **Fait sur `manques-kvmsoft`** (les manques vus par KVMSoft, `docs/MIGRATION.md`) : instance
  unique qui transmet ses arguments (`Reveil`, `recevoir_arguments`), fermeture des Paramètres
  refusable (`peut_fermer`), brouillon entier avec « Annuler » (`instantane`, `remettre`,
  `brouillon_modifie`) et pied de page (`dessiner_pied`), sélecteur de dossier au-dessus des
  Paramètres, `sans_fantome`, fermeture bornée à 5 s, captures PNG pour la documentation
  (`capture`). **Pas vérifié** : le refus de fermeture d'une fenêtre réduite et la
  transmission des arguments entre deux vrais processus (interdit sans fenêtre visible) ; les
  essais couvrent le crochet (faux moteur de fenêtres), le codage des arguments et la lecture
  d'un `WM_COPYDATA`.
- **Fait sur `kvmsoft-sonde`** : la sonde des fils (`src/sonde.rs`, `docs/MIGRATION.md`) pour
  trouver une icône muette. **Pas vérifié** : un vrai blocage de l'icône ou du volet (la logique
  est essayée avec des fils simulés ; les points d'appel ne l'ont pas été en fenêtres réelles).
- **Fait sur `claude/ic-double-clic`** (23 nov.) : fenêtre principale facultative
  (double-clic, « Ouvrir <logiciel> » en gras en tête du petit menu, relance, `clic_gauche`),
  roue dentée des Paramètres avec Ctrl+virgule, interrupteur écrit, explications des cartes en
  14 px, redite du dossier retirée (`docs/MIGRATION.md`). **Pas vérifié** : en vraies fenêtres
  (la relance entre deux vrais processus, le gras du menu à l'écran) ; les essais couvrent le
  calcul des gestes, le menu, l'aiguillage d'une icône, la roue et l'interrupteur sans fenêtre.
- **Pas vérifié** : l'affichage dans de vraies fenêtres (interdit pendant les essais
  automatiques). À essayer à la main avec l'exemple (`cargo run --example demo`, petit menu :
  « Ouvrir une fenêtre », « Fenêtre simple », « Fenêtre de lecture », Paramètres).
- **En attente** : les vérifications GitHub ne démarrent pas (limite de dépense des Actions du
  compte atteinte) ; à relancer quand c'est réglé.
- **Refonte à part (A, choisie par le propriétaire le 24 nov.)**, sur la branche `refonte` :
  la conception est dans `docs/ORGANIGRAMME.md` (version 2, 25 nov. : deux crates, API gardée) ;
  l'ordre des blocs, les dates, la migration et les conditions de bascule sont dans
  `docs/PLAN-REFONTE.md` (validés par l'Atelier Master le 25 nov. ; une seule voie à la fois,
  fin vers la fin février 2027). Les filets de l'ancienne bibliothèque sont fusionnés dans `main`
  et dans `refonte`.
- **Refonte, bloc 0 « Préparation »** (25 nov., branche `claude/bloc-0-preparation`, partie de
  `refonte`, à intégrer par l'Atelier Master) :
  - un **espace de travail Cargo** à la racine : `interface-commune` (inchangée pour les
    logiciels, même ligne dans leur `Cargo.toml`) et le cœur **`crates/ic-coeur`**, vide, sans
    aucune dépendance, `#![forbid(unsafe_code)]` ; `interface-commune` le prend déjà (le sens
    des étages est en place) ;
  - la liste des modules du cœur, écrite une fois pour toutes (plan, §3) : `contrat` (A),
    `textes` (B), `theme` (C), `gestes` et `placement` (D), `parametres` (E), chacun vide ;
  - **`tests/etages.rs`** : le cœur ne déclare aucune dépendance et `Cargo.lock` le confirme ;
    `interface-commune` prend le cœur ; le code du cœur n'appelle ni egui, ni Windows, ni le
    disque, le réseau, l'horloge ou le système, et n'a aucun `unsafe` ; le dessin
    (`src/dessin.rs`, puis `src/dessin/`) n'appelle ni `windows::`, ni eframe, ni winit, ni
    `unsafe` ; `unsafe` seulement dans `src/bords/`, plus, en attendant leur bloc, `echelle`,
    `horloge`, `systeme` et `zone` (liste à vider aux blocs L, M, J). Chaque essai a été vu
    échouer sur une faute posée exprès, puis la faute retirée ;
  - **CI** : `fmt --all`, `clippy --workspace`, `test --workspace` sous Windows ; un petit
    travail Linux pour le cœur seul (clippy et tests de `ic-coeur`) ; la vérification part aussi
    sur un envoi vers `refonte` ;
  - `CARTE.md` et `README.md` : les commandes de l'espace de travail.
  - **Décisions prises seul** (à relire par l'Atelier Master) : le cœur peut avoir des
    dépendances **d'essai** (`[dev-dependencies]`, par exemple `proptest`), jamais egui, eframe,
    winit, Windows, skrifa ni `interface-commune` ; aucune autre section de dépendances ;
    le travail Linux du cœur (quelques minutes Linux par envoi) ; `refonte` ajoutée aux
    branches vérifiées à l'envoi.
  - **Pas vérifié** : la CI elle-même (rien n'est envoyé sur GitHub par cette session ; les
    Actions étaient bloquées par la limite de dépense). Les fichiers de plus de 800 lignes
    d'aujourd'hui (`volet`, `fenetres`, `horloge`…) restent : ils se découpent dans leur bloc.
- **Refonte, bloc A « Contrat »** (25 nov. 06:30, branche `claude/bloc-a-contrat`, partie de
  `refonte` après le bloc 0, à intégrer par l'Atelier Master) :
  - dans **`ic_coeur::contrat`** : `Identite` (et `logo_rvba`), `FaireLogo`, `Etat`,
    `GenreEtat`, `EtatMiseAJour`, `Credit`, `Communs`, `cles`, `FenetrePrincipale`, `Page`,
    `Valeur`, `Instantane`, `Reveil`, `Niveau` : mêmes champs, mêmes valeurs, même code ;
  - **repris aux mêmes chemins** par `pub use` : `interface_commune::{Identite, …, cles}`,
    `parametres::modele::{Page, Valeur, Instantane}` (et `interface_commune::Page`…),
    `horloge::{Reveil, Niveau}` ; aucun changement d'API ; `tests/compatibilite.rs` inchangé ;
  - **essais dans le cœur** (`crates/ic-coeur/src/contrat/essais.rs`, 11 essais, logiciel
    inventé « Fictif ») : logo et icône en RVBA, image de la mauvaise taille refusée, défauts
    des pages communes (C-82, avec F et H), valeurs des clés figées, `Page`, `Instantane`,
    `Valeur`, `Reveil::Rien` et le défaut `Reveil::Parametres` (le verrou et la transmission
    restent au bloc N) ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 11/11 ; `interface-commune` : 202 essais internes et tous les fichiers de
    `tests/` verts (compatibilite, comportements, references aux images identiques, polices,
    polices_windows10, etages, sans_fenetre…) ; `fmt`, `clippy -D warnings`, `cargo deny`
    propres ; contrôle de la règle n° 1 : seulement `.git:1` (le fichier du worktree) ;
  - **Décision prise seule** (à relire par l'Atelier Master) : **`menu::Action` reste dans
    `interface-commune`** jusqu'au bloc D. Sa méthode publique `Action::ouvrir_le_journal(&Textes)`
    (documentée dans `MIGRATION.md`) demande `Textes` (bloc B) et `icones::JOURNAL` (bloc C) ;
    Rust interdit d'ajouter une méthode à un type d'une autre crate, la déplacer seule casserait
    cette signature. Elle part dans le cœur avec `menu` (bloc D, après B et C).
  - **Pas lancés sur le PC** : `fenetres_en_rafale` et `recreation_apres_panique` (vraies
    fenêtres, prévus pour GitHub ; le bloc A ne touche pas aux fenêtres). Les traits `Reglages`
    et `Logiciel` restent à la racine (bloc F).
- **Refonte, bloc B « Textes »** (25 nov. 07:35, branche `claude/bloc-b-textes`, partie de
  `refonte` après le bloc A, à intégrer par l'Atelier Master) :
  - dans **`ic_coeur::textes`** : le catalogue (`Langue`, `Textes`, `FRANCAIS`, `ANGLAIS`,
    `Textes::de`, `remplir`, fichier `textes/catalogue.rs`), `lisible` (`textes/raccourcis.rs`),
    l'espace insécable (C-98 : `INSECABLE`, `avec_unite`, servie aux seuils) et `insecables`
    (la typographie française du dessin) ;
  - **repris aux mêmes chemins** par `pub use` : `interface_commune::textes::…`,
    `interface_commune::{Langue, Textes}`, `touches::lisible`, `dessin::insecables` ; aucun
    changement d'API ; `tests/compatibilite.rs` inchangé ; l'essai de `touches` qui passe par
    egui reste dans `src/touches.rs` ;
  - **R1 fait** : `conserver_retour` prend l'espace insécable, en français et en anglais ;
    **aucune image de référence ne change** (`parametres-conserver-clair` montre la carte :
    même coupure, `references` vert) ; `tests/conserver.rs` attend `15\u{a0}s` ;
  - **essais dans le cœur** (`crates/ic-coeur/src/textes/essais.rs`, logiciel inventé
    « Fictif ») : les deux catalogues ont les mêmes 74 champs (lus sans liste à tenir), aucun
    vide ; les modèles se remplissent ; C-98 sans aucune espace simple avant une unité (le
    détecteur est lui-même essayé sur des fautes posées exprès) ; `avec_unite`, `insecables` ;
    les essais de `lisible`, et `lisible` ne change pas deux fois (au hasard, proptest) ;
  - `tests/comportements.rs` (C-98) : plus d'écart connu dans les textes ; `COMPORTEMENTS.md`
    (C-46, C-97, C-98) suit ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 27 essais et 1 exemple de documentation ; `interface-commune` : 193 essais
    internes (202 moins les 9 partis dans le cœur), tous les fichiers de `tests/` verts
    (references aux images identiques, conserver, comportements, compatibilite, polices,
    polices_windows10, etages, sans_fenetre…), exemples de documentation ; `fmt`,
    `clippy -D warnings`, `cargo deny` propres ; contrôle de la règle n° 1 : seulement `.git:1`
    (le fichier du worktree) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - `dessin::insecables` part aussi dans le cœur (du texte pur ; l'organigramme la mettait
      au bloc F), reprise à son chemin ;
    - « 24 h » de `journal_detaille_explication` prend aussi l'espace insécable (même règle,
      trouvé par le nouvel essai ; ajouté au PLAN §4, ligne 4) ; aucune image ne change ;
    - `proptest` en dépendance **d'essai** du cœur (permis par le bloc 0) ;
    - `INSECABLE` et `avec_unite` sont nouveaux dans le cœur mais **pas repris à la racine** :
      l'API de `interface-commune` ne bouge pas ;
    - R2 (`briques::pourcent`, « 42 % ») reste au bloc F, comme prévu ;
  - **Pas lancés sur le PC** : `fenetres_en_rafale` et `recreation_apres_panique` (vraies
    fenêtres, prévus pour GitHub ; le bloc B ne touche pas aux fenêtres).
- **Refonte, bloc C « Thème, mesures »** (25 nov. 09:36, branche `claude/bloc-c-theme`,
  partie de `refonte` après les blocs 0, A et B, à intégrer par l'Atelier Master) :
  - dans **`ic_coeur::theme`** : `Rgb`, `contraste`, `ChoixWindows`, `Palette` (clair, sombre,
    accent ; `theme/couleurs.rs`), les 10 couleurs des pastilles et `couleur_lettre`, les tailles
    et arrondis (`theme/tailles.rs`), l'animation (`theme/animation.rs` : fondus, courbe,
    `Surface`, `Image`, et `PAS_MAX`/`avancer` venus du volet), `couverture`
    (`theme/formes.rs`), les modules `theme::mesures` et `theme::icones` ;
  - **suivre Windows (C-90)** dans le cœur (`theme/relecture.rs`) : `RELECTURE` (2 s),
    `faut_relire(depuis)`, `impose(choix, force)`, `relu(actuel, lu, force)` ; le temps écoulé
    arrive en paramètre, l'horloge n'est jamais lue dans le cœur. `Apparence` reste dans la
    bibliothèque (ses méthodes prennent egui ; Rust interdit d'en ajouter à un type d'une autre
    crate) : elle lit Windows (`choix_windows`, bloc L) et l'instant, puis laisse le cœur
    décider ;
  - **repris aux mêmes chemins** par `pub use` : `theme::…`, `mesures::*`, `icones::*`,
    `apparence::{RELECTURE, couleurs_pastilles}`, `pastilles::{couleur_lettre,
    couleurs_pastilles}` (`couleurs::palette` les prend toujours) ; aucun changement d'API ;
    `tests/compatibilite.rs` inchangé ; **aucune image de référence ne change** (`references`
    vert) ;
  - **essais dans le cœur** (`theme/essais.rs`, horloge inventée, logiciel « Fictif ») : les 6
    essais de `theme`, `lettre_selon_le_fond` (venu de `pastilles`),
    `un_retard_ne_fait_pas_sauter_l_animation` (venu de `volet`), et 5 nouveaux : accent par
    défaut et accent seul, relecture toutes les 2 s, thème de Windows suivi, thème imposé
    (la barre des tâches, elle, reste suivie), couverture d'une forme ;
    `tests/palette_pastilles.rs` devient `theme/essais_pastilles.rs` (4 essais, tournent aussi
    sous Linux) ; `COMPORTEMENTS.md` (C-30, C-40, C-90, C-91, C-94) et `CARTE.md` suivent ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 44 essais (27 + 17) et 1 exemple de documentation ; `interface-commune` : 185
    essais internes (193 moins les 8 partis dans le cœur), tous les fichiers de `tests/` verts
    (references aux images identiques, compatibilite, comportements, couleurs, polices,
    polices_windows10, etages, sans_fenetre, premiere_creation…), exemples de documentation ;
    `fmt`, `clippy -D warnings`, `cargo deny` propres ; contrôle de la règle n° 1 : seulement
    `.git:1` (le fichier du worktree) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - `HAUTEUR_LIGNE` **gardée** (dans le cœur, reprise à la racine) : le bloc C ne change pas
      l'API ; son retrait reste le changement n° 3 du PLAN, §4, à faire avec les autres ;
    - `PAS_MAX`, `avancer`, `RELECTURE`, `faut_relire`, `impose`, `relu` sont publics dans le
      cœur ; seul `RELECTURE` est repris à la racine (il l'était déjà) ;
    - l'essai des couleurs des pastilles passe de `tests/` au cœur (l'ORGANIGRAMME, §3, le met
      dans les essais du bloc C) ;
    - `choix_windows` et `couleur` (egui) restent dans `apparence` (blocs L et F) ;
  - **Pas lancés sur le PC** : `fenetres_en_rafale` et `recreation_apres_panique` (vraies
    fenêtres, prévus pour GitHub ; le bloc C ne touche pas aux fenêtres).
- **Refonte, bloc K « Guichet »** (25 nov. 14:10, branche `claude/bloc-k-guichet`, partie de
  `refonte` après le bloc C, à intégrer par l'Atelier Master) :
  - **`src/bords/poste.rs`** (nouveau dossier `src/bords/`, module privé ; ORGANIGRAMME §10 :
    K est dans les bords, rien ne part dans le cœur) : `Poste`, `AuClic`, les messages au fil
    de l'icône (`Commande`), leur **boîte bornée** (`Boite`, 64 messages, comme avant) et le
    **réveil** (trait `Reveiller` ; le vrai, `ReveilParMessage`, poste le message Windows à la
    fenêtre cachée). Le guichet dépose puis réveille ; il ne traite rien ;
  - **repris au même chemin** : `interface_commune::horloge::{Poste, AuClic}` (`pub use`) ;
    aucun changement d'API ; `tests/compatibilite.rs` inchangé ; aucune image ne change ;
    `horloge.rs` passe de 1486 à 1289 lignes (`Principal::commandes` vide la boîte du guichet) ;
  - **essais sans fenêtre ni matériel** (`src/bords/poste/essais.rs`, un faux réveil qui
    compte) : rafales de 8 fils à la fois (rien ne se perd, un réveil par demande, l'ordre de
    chaque fil gardé) ; rafale plus grande que la boîte (64 gardés, chaque demande réveille
    quand même, la boîte reprend après lecture) ; un faux fil de l'icône qui vide la boîte à
    chaque réveil pendant que 4 fils déposent (il reçoit tout ce qui est accepté, une fois,
    dans l'ordre) ; chaque demande va au bon fil (icône ou boucle des fenêtres) ; `Poste` est
    `Send + Sync` ; sans fenêtre cachée, rien n'est posté. Les 3 essais de `horloge`
    (`notification_a_trois_niveaux`, `code_pour_le_fil_principal`,
    `clic_sur_une_notification`) y sont venus avec toutes leurs vérifications ;
    `COMPORTEMENTS.md` (C-14, C-19, C-20) et `CARTE.md` suivent ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 44 essais ; `interface-commune` : 191 essais internes (185 − 3 déplacés + 9 du
    guichet ; ceux du guichet relancés 5 fois, toujours verts), tous les autres fichiers de
    `tests/` verts (references aux images identiques, compatibilite, comportements, etages…),
    exemples de documentation ; `fmt`, `clippy -D warnings`, `cargo deny` propres ; contrôle de
    la règle n° 1 : seulement `.git:1` (le fichier du worktree) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - le canal `Demandes`/`Reception` de la boucle des fenêtres reste dans `fenetres.rs`
      (bloc J) : le guichet s'en sert, il ne le possède pas ;
    - le numéro du message de réveil (`WM_COMMANDE`) et son jeton restent dans `horloge.rs`
      (le protocole de la fenêtre cachée, bloc M) ; ils sont donnés au guichet à sa création ;
    - `horloge.rs` reste au-dessus de 800 lignes et garde l'`unsafe` (liste d'attente de
      `tests/etages.rs`) : il se découpe au bloc M ;
    - comportement inchangé : capacité de 64, réveil à chaque demande même boîte pleine ;
  - **Pas lancés sur le PC** : `fenetres_en_rafale` et `recreation_apres_panique` (vraies
    fenêtres, prévus pour GitHub).
- **Refonte, bloc D « Gestes, placement, secours »** (25 nov. 15:40, branche
  `claude/bloc-d-gestes`, partie de `refonte` à 10b19d3, à intégrer par l'Atelier Master) :
  - dans **`ic_coeur::gestes`** (calcul pur, l'instant arrive par le trait `Horodatage`, jamais
    lu dans le cœur) : `clics` (le geste reconnu et sa suite), `aiguilleur` (**l'aiguilleur
    unique** : `SuiviDesIcones` et `ordre`, pris par l'icône seule, `horloge`, et par la zone,
    `zone::Aiguilleur`), `ouverture` (décision d'un clic, anti-rebond, garde, clic en dehors,
    `juste_fermee`), `secours` (pertes de fenêtres, rafale, `decider_la_reprise`), `icone`
    (info-bulle, copie dans le tampon de Windows, délai contre le clignotement), `menu` (lignes
    du petit menu et **`Action`, venue de la bibliothèque comme le bloc A l'avait décidé**) ;
  - dans **`ic_coeur::placement`** : tout le placement de `place.rs` (volet, petit menu,
    Paramètres, écrans, changement d'échelle ; essais dans `placement/essais.rs`) et
    `placement/plan.rs` : `Rectangle` (même calcul qu'egui, essai `rectangle_comme_egui`), le
    plan des écrans des pastilles et la place du menu de choix ;
  - **repris aux mêmes chemins** par `pub use` (`clics`, `menu`, `place`, `fenetres`, `zone`,
    `horloge`, `pastilles`, `menu_choix`) ; les appels à Windows et les fils restent dans la
    bibliothèque ; aucun changement d'API ; `tests/compatibilite.rs` inchangé ; aucune image de
    référence ne change ; version 0.1.0 ;
  - **essais** : 46 essais venus de la bibliothèque avec toutes leurs vérifications (horloges
    inventées en `Duration`), 11 nouveaux dans le cœur (aiguilleur ×4, nom des gestes pour la
    sonde, copie dans le tampon, `juste_fermee`, secours ×3, `rectangle_comme_egui`) et 1 dans la
    bibliothèque (`place::rectangles_egui_et_coeur`, l'aller-retour egui ↔ cœur) ;
    `COMPORTEMENTS.md` et `CARTE.md` suivent ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 101 essais (44 + 57) ; `interface-commune` : 146 essais internes (191 − 46 + 1),
    tous les fichiers de `tests/` verts (references aux images identiques, compatibilite,
    comportements, etages…), exemples de documentation ; `fmt`, `clippy -D warnings`,
    `cargo deny` propres ; contrôle de la règle n° 1 : seulement `.git:1` (le fichier du
    worktree) ; les fichiers touchés de plus de 800 lignes l'étaient déjà et ont tous
    rapetissé (`fenetres` 1633 → 1258, `zone` 1617 → 1584, `horloge` 1404 → 1268, `pastilles`
    967 → 852, `menu_choix` 922 → 860) ; aucune fonction nouvelle de plus de 80 lignes ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - **C-07 corrigé** : `copier` ne coupe plus un emoji (deux unités UTF-16) en deux dans le
      tampon de Windows (essai `copie_dans_le_tampon_de_windows`) ;
    - `decider_la_reprise` reçoit l'horloge à lire seulement si besoin (même comportement) ;
      `juste_fermee` lit maintenant l'instant à chaque appel (différence négligeable) ;
    - `ATTENTE_FERMETURE` et `attendre_la_fin` restent dans `fenetres.rs` (des fils) ; la
      décision « bloqué » de la sonde reste au bloc O ;
    - le plan des écrans et la place du menu de choix partent dans le cœur avec un
      `Rectangle` à lui, converti au bord (`place::vers_le_coeur`, `vers_egui`) ;
    - un dossier de compilation propre au worktree (`CARGO_TARGET_DIR`) : le dossier partagé
      avec les autres worktrees faisait prendre un vieux `ic-coeur` à Cargo ;
  - **À savoir pour l'intégration** : `refonte` a avancé depuis 10b19d3 (d1b48ad, reprise
    après la veille : `src/reprise.rs`, `horloge.rs`, `fenetres.rs`) ; des conflits sont
    probables dans `horloge.rs` et `fenetres.rs` ;
  - **Pas lancés sur le PC** : `fenetres_en_rafale` et `recreation_apres_panique` (vraies
    fenêtres, prévus pour GitHub).
- **Refonte, bloc F « Contrôles et traits »** (25 nov. 16:20, branche
  `claude/bloc-f-controles`, partie de `refonte` à 03f937a, à intégrer par l'Atelier Master ;
  **rien n'est commité** : tout est indexé, le message est dans `docs/MESSAGE-COMMIT.txt`) :
  - **le dessin dans `src/dessin/`** (ni Windows, ni eframe, ni `unsafe` : `tests/etages.rs`) :
    `dessin.rs` (843 lignes) coupé en `dessin/controles/` (`base`, `interrupteur`, `raccourci`,
    `cartes`) ; `briques`, `pastilles`, `menu_choix`, `seuils`, `nuancier`, `couleurs`, `touches`
    déplacés dans `src/dessin/` (`git mv`) ; `couleur` (de `Rgb` à egui) dans
    `dessin/couleur.rs` ; les traits **`Reglages` et `Logiciel`** dans `dessin/traits.rs` ;
  - **dans le cœur** (calcul pur) : `ic_coeur::gestes::choix` (les touches du menu de choix, sa
    ligne initiale, le nom pour le lecteur d'écran), `ic_coeur::gestes::seuils` (bornes des
    poignées, déplacement, pas du clavier : `voulu_au_clavier`), `ic_coeur::theme::marque` (le
    logo d'une pastille si la police l'a, sinon sa lettre ; le texte lisible sur une zone) ;
  - **repris aux mêmes chemins** : `briques`, `seuils`, `menu_choix`, `pastilles`, `nuancier`,
    `couleurs`, `touches` restent des modules publics de la racine qui reprennent `dessin::…`
    (`pub use`) ; `dessin::*` reprend les contrôles ; `interface_commune::{Logiciel, Reglages}`
    et `apparence::couleur` aussi. `tests/compatibilite.rs` inchangé ;
  - **changements d'API du PLAN, §4** : n° 2, `Logiciel::dessiner_rapides` et `largeur_rapides`
    retirés (la Demo de `tests/sans_fenetre.rs` passe ce contenu en une `Brique::Choix`) ;
    n° 5, R2 : `briques::pourcent` écrit « 42 % » avec l'espace insécable (`avec_unite`,
    nouvel essai `le_pourcentage_a_l_espace_insecable`) ; **aucune image de référence ne
    change** (`references` vert) ; `MIGRATION.md` suit ;
  - **code propre** (méthode, §1.8) : plus aucune fonction de plus de 80 lignes dans
    `src/dessin/` (`barre_seuils`, `afficher` du menu de choix, `pastille`, `dessiner_choix`,
    `brique` et l'appareil découpés) ; aucun fichier du dessin au-dessus de 800 lignes
    (`menu_choix` 860 → 764, `pastilles` 852 → 636) ; l'appareil dans `briques/appareil.rs` ;
    les essais des pastilles dans `pastilles/essais.rs` ; chaque fichier a son en-tête `//!` ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 116 essais (101 + 15 : `choix` 9, `seuils` 4, `marque` 2) ; `interface-commune` :
    145 essais internes, tous les fichiers de `tests/` verts (references aux images identiques,
    compatibilite, comportements, polices, etages, sans_fenetre…), exemples de documentation ;
    `fmt`, `clippy -D warnings`, `cargo deny` propres ; contrôle de la règle n° 1 : seulement
    `.git:1` (le fichier du worktree) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - les anciens modules publics restent à la racine comme modules **minces** qui reprennent
      `dessin::…` : les logiciels gardent leurs chemins ;
    - `texte_sur` (cœur, noir à contraste égal) et `couleur_lettre` (blanc à contraste égal)
      sont gardés tous les deux : chacun sert un dessin qui ne doit pas bouger ;
    - `voulu_au_clavier` prend la valeur de départ : une glissée puis une flèche dans la même
      image partent de la valeur glissée (comme avant) ;
    - `places_valides` et `genre_du_plan` restent dans `pastilles` (petits, liés à egui ; leur
      passage au cœur n'apporte rien aujourd'hui) ;
    - l'`assert_eq!` d'entrée de `barre_seuils` (une limite entre deux zones) est gardé tel quel :
      une faute du logiciel, pas une entrée de l'utilisateur ;
    - `tests/polices.rs` lit les nouveaux fichiers (`dessin/controles/*`, `dessin/briques/
      appareil.rs`…) : le contrôle des glyphes couvre toujours tout le dessin ;
    - les fonctions longues hors du bloc F (`fenetres`, `horloge`, `surface::volet`,
      `toutes_faites`, `volet`, `parametres`) restent : elles se découpent dans leur bloc ;
    - un dossier de compilation propre au worktree (`CARGO_TARGET_DIR`), comme au bloc D ;
  - **À savoir pour l'intégration** : conflits possibles avec `refonte` si elle a bougé dans
    `src/lib.rs`, `src/surface.rs`, `src/apparence.rs`, `tests/polices.rs`,
    `tests/sans_fenetre.rs`, `crates/ic-coeur/src/gestes.rs` et `theme.rs`, ou dans un des
    fichiers déplacés (`briques`, `pastilles`, `menu_choix`, `seuils`…) ;
  - `fenetres_en_rafale` et `recreation_apres_panique` ont tourné avec la suite complète
    (fenêtres voilées et hors de l'écran, jamais montrées) : verts.
- **Refonte, bloc E « Modèle des Paramètres »** (25 nov. au soir, deuxième voie, branche
  `refonte-bloc-e-parametres`, partie de `refonte` à c6a354d, à intégrer par l'Atelier Master ;
  **rien n'est commité** : tout est indexé, le message est dans `MESSAGE-COMMIT.txt`, à la
  racine, hors de l'index) :
  - **dans le cœur** (calcul pur, sans egui, sans Windows, sans horloge) :
    `ic_coeur::parametres` en sept modules : `pages` (ordre de la navigation, page demandée sur
    la ligne de commande, page gardée quand les pages changent : `Suivi`, `page_suivie`),
    `recherche` (sans accents, section ouverte), `colonne` (lignes montrées et espace avant
    Général, champ de recherche dès sept pages, largeur de la colonne C-80), `annuler` (la bulle
    « Annuler »), `conserver` (« Conserver ces modifications ? »), `envois` (ce que chaque geste
    rend au logiciel, dans l'ordre : la liste `Envoi` ; le bandeau « Redémarrer », C-78),
    `mise_a_jour` (icône, titre, couleurs, ligne grise, explication des recherches, bouton de la
    carte, carte du dossier) ;
  - **repris aux mêmes chemins** : `parametres::modele` (816 → 18 lignes) reprend tout du cœur
    (`pub use`) ; `parametres::{TonMiseAJour, presentation_mise_a_jour, texte_mise_a_jour,
    ligne_grise_mise_a_jour}` aussi. `Parametres` ne fait plus que dessiner et passer les
    envois au logiciel (`envoyer`) ; `src/parametres/mod.rs` : 1 390 → 1 204 lignes.
    `tests/compatibilite.rs` et les images de référence inchangés ;
  - **essais** : ceux du modèle et de la page Mise à jour passent dans le cœur, sur une fausse
    horloge (`Duration`) : `essais.rs` (8), `essais_gestes.rs` (15), `essais_mise_a_jour.rs`
    (8) ; nouveaux : largeur de la colonne, lignes et espace, recherche montrée, page gardée,
    bulle qui disparaît à 6 s, envois d'un changement et d'« Annuler », brouillon remis d'abord,
    changement et groupe à confirmer, bandeau, bouton et carte du dossier. `COMPORTEMENTS.md`
    (C-66 à C-80) et `CARTE.md` suivent ; la ligne du 25 nov. au soir est au plan, §3 ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 147 essais ; `interface-commune` : 128 essais internes (les essais du modèle et de
    la page Mise à jour sont partis dans le cœur), tous les fichiers de `tests/` verts
    (references aux images identiques, compatibilite, conserver, comportements, etages,
    sans_fenetre…), exemples de documentation ; `fmt`, `clippy -D warnings`, `cargo deny`
    propres ; contrôle de la règle n° 1 : 0 trouvé dans les fichiers suivis et dans une copie de
    l'index avec `MESSAGE-COMMIT.txt` (`--dossier .` lit aussi `target-bloc-e/`, hors du dépôt,
    plein des chemins du PC : on contrôle donc la copie de l'index) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - E-1 : les types qui gardent un instant sont génériques (`Annulation<T = Instant>`,
      `Confirmation<T = Instant>`…) sur le trait `Horodatage` du bloc D : le code des logiciels
      ne change pas, les essais passent une fausse horloge ;
    - E-2 : le calcul de la colonne de navigation s'appelle `colonne` (et non `navigation`, déjà
      le nom de la fonction qui fait la liste des pages) ;
    - E-3 : chaque geste rend une liste `Envoi` (changer, remettre) dans l'ordre d'avant (le délai
      écoulé d'abord, le brouillon avant les valeurs) ; les fonctions de `envois` ne sont pas
      reprises dans `ic_coeur::parametres` (leurs noms croiseraient ceux des méthodes) : on les
      appelle `envois::changer`… ;
    - E-4 : le bandeau « Redémarrer » est `envois::noter_redemarrage` : la dernière raison
      l'emporte, une image sans demande ne l'efface pas (comme avant) ;
    - E-5 : toute la présentation de la page Mise à jour passe dans le cœur, avec deux décisions
      de plus (le bouton de la carte, ce que montre la carte du dossier) ; le dessin les pose ;
      l'explication des recherches ne cite plus un autre projet (« pas de redite ») ;
    - E-6 : la page gardée quand la navigation est refaite est `Suivi` (les clés d'avant) et
      `page_suivie` (`Gardee` ou `Perdue`, qui renvoie à Général) ;
  - **reste** : `src/parametres/mod.rs` garde plus de 800 lignes et des fonctions longues : il se
    découpe au bloc H (le dessin des Paramètres, un fichier par page) ;
  - **à savoir pour l'intégration** : conflits possibles si `refonte` a bougé dans
    `src/parametres/`, `crates/ic-coeur/src/parametres.rs`, `docs/COMPORTEMENTS.md` ou
    `docs/CARTE.md` ; le dossier de compilation `target-bloc-e/` reste hors de l'index ;
  - **sous-agents** : aucun.
- **Prochaine étape de la refonte** : intégrer le bloc F dans `refonte`, puis le bloc
  suivant du `docs/PLAN-REFONTE.md`, §2 (une seule voie à la fois).
- **Prochaine étape sur `main`** : fusionner `manques-projets`, puis passer KVMSoft, Vocalis et PowerPeek
  (`docs/MIGRATION.md`).
