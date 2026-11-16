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
- **Pas vérifié** : l'affichage dans de vraies fenêtres (interdit pendant les essais
  automatiques). À essayer à la main avec l'exemple (`cargo run --example demo`, petit menu :
  « Ouvrir une fenêtre », « Fenêtre simple », « Fenêtre de lecture », Paramètres).
- **En attente** : les vérifications GitHub ne démarrent pas (limite de dépense des Actions du
  compte atteinte) ; à relancer quand c'est réglé.
- **Prochaine étape** : fusionner `manques-projets`, puis passer KVMSoft, Vocalis et PowerPeek
  (`docs/MIGRATION.md`).
