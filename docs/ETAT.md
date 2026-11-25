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
- **Prochaine étape de la refonte** : intégrer le bloc C dans `refonte`, puis le bloc suivant
  du `docs/PLAN-REFONTE.md`, §2 (une seule voie à la fois).
- **Prochaine étape sur `main`** : fusionner `manques-projets`, puis passer KVMSoft, Vocalis et PowerPeek
  (`docs/MIGRATION.md`).
