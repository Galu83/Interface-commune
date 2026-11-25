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
- **Prochaine étape de la refonte** : intégrer le bloc 0 dans `refonte`, puis le bloc A
  « Contrat » (`docs/PLAN-REFONTE.md`, §2).
- **Prochaine étape sur `main`** : fusionner `manques-projets`, puis passer KVMSoft, Vocalis et PowerPeek
  (`docs/MIGRATION.md`).
