# Interface commune

Une bibliothèque Rust, privée, partagée par les logiciels du propriétaire qui vivent près de
l'horloge de Windows : la même allure Windows 11, le même comportement, écrits une seule fois.

## Ce qu'elle contient

- **Le thème Windows 11** (`theme`, `apparence`, `polices`) : clair ou sombre comme Windows, la
  couleur d'accentuation suivie en direct, la palette Fluent, Segoe UI Variable (repli : Segoe UI)
  et Segoe Fluent Icons (repli : Segoe MDL2 Assets), lues sur le PC, jamais livrées. Les durées
  d'animation communes (règles de l'interface, point 10) : volet et petit menu, à leur place,
  fondu de 83 ms parti de 50 % ; fermeture, fondu de 83 ms ; aucune animation si Windows les a
  coupées.
- **L'icône près de l'horloge** (`horloge`) : clic gauche = le volet, clic droit = le petit menu,
  double-clic = les Paramètres, clavier (Entrée, Espace, touche menu) ; notifications ; une seule
  copie du logiciel à la fois ; l'icône revient si l'Explorateur redémarre.
- **Le volet**, préparé au repos (une seule fenêtre, cachée puis montrée : deux ouvertures ne
  reconstruisent rien), avec ses animations ; **le petit menu** au style Windows 11, toujours
  dans le même ordre : actions du logiciel, Paramètres (E713), Rechercher une mise à jour (E895),
  lignes en plus (Ouvrir le journal…), Quitter. Sans OpenGL : le menu de secours de Windows.
- **Les Paramètres** (`parametres`), squelette du point 7, au dessin de PowerPeek : en-tête
  avec le vrai logo, le nom et « Version x.y.z », navigation à gauche (lignes de 38 points,
  colonne qui s'élargit pour un long nom, recherche dès 7 pages, point d'alerte), pages
  Général, Mise à jour et À propos juste après celles du logiciel, cartes, interrupteurs,
  effet immédiat avec la bulle « Annuler » (6 s), bandeau « Redémarrer maintenant », taille et
  place de la fenêtre retenues, `--parametres <page>`. Utilisables seuls (trait `Reglages`),
  sans l'icône ni le volet.
- **Français ou anglais** (`textes`) : chaque mot écrit par la bibliothèque, dans la langue
  du logiciel, changeable en direct.
- **Plusieurs fenêtres dans une seule boucle** (`autres_fenetres`) : les Paramètres et les
  fenêtres du logiciel s'ouvrent et se ferment à côté du volet, depuis n'importe quel fil.
- **Des fenêtres toutes faites** (`toutes_faites`), reprises de Vocalis : la fenêtre simple
  (texte, barre d'avancement, champ, choix, boutons ; hauteur qui suit le contenu) et la
  fenêtre de lecture (mot lu surligné et suivi).
- **Des contrôles pour les tests** (`verification`) : chaque icône et chaque caractère affiché
  existe dans les polices de Windows.

Rien n'y est propre à un logiciel : le nom, la version, l'icône, les pages, les actions du menu,
les réglages rapides et les textes viennent du logiciel, par les traits `Reglages` (les
Paramètres) et `Logiciel` (tout le reste).

## L'utiliser dans un projet

1. Dans le `Cargo.toml` du projet, épinglée sur un commit (jamais une branche : une mise à jour
   de l'interface est un choix, fait en changeant ce numéro) :

   ```toml
   [dependencies]
   interface-commune = { git = "https://github.com/Galu83/Interface-commune", rev = "<numéro du commit>" }
   ```

2. Le dépôt étant privé, Cargo passe par le `git` du PC (qui connaît déjà le compte GitHub). Dans
   le `.cargo/config.toml` **suivi par Git** du projet :

   ```toml
   [net]
   git-fetch-with-cli = true
   ```

3. Dans le `deny.toml` du projet, autoriser cette source et elle seule :

   ```toml
   [sources]
   allow-git = ["https://github.com/Galu83/Interface-commune"]
   ```

4. Le logiciel décrit ce qui lui est propre en écrivant les traits `Reglages` et `Logiciel`,
   puis :

   ```rust,ignore
   let Ok(_verrou) = horloge::instance_unique("MonLogiciel") else { return };
   let h = horloge::Horloge::new("MonLogiciel");
   let poste = h.poste(); // notifier, redessiner, ouvrir les Paramètres, quitter
   h.lancer(Arc::new(mon_logiciel), parametres::modele::page_demandee(&args))?;
   ```

   L'exemple complet : `examples/demo.rs` (`cargo run --example demo`).

### Sur GitHub (vérifications du projet)

Les vérifications GitHub d'un projet doivent pouvoir lire ce dépôt privé. Choix retenu : **une clé
de déploiement**, en lecture seule, valable pour ce seul dépôt, sans date de fin.

- Une fois par projet (fait par l'Atelier du projet, depuis le PC) :

  ```bash
  ssh-keygen -t ed25519 -N "" -C "verifications" -f cle
  gh repo deploy-key add cle.pub --repo Galu83/Interface-commune --title "Vérifications de <projet>"
  gh secret set INTERFACE_COMMUNE_CLE --repo Galu83/<projet> < cle
  ```

  puis effacer `cle` et `cle.pub`.
- Dans le fichier de vérifications du projet, avant toute commande `cargo` :

  ```yaml
      - name: Accès à l'interface commune (clé de déploiement, lecture seule)
        shell: bash
        env:
          CLE: ${{ secrets.INTERFACE_COMMUNE_CLE }}
        run: |
          mkdir -p ~/.ssh
          printf '%s\n' "$CLE" > ~/.ssh/interface_commune
          chmod 600 ~/.ssh/interface_commune
          ssh-keyscan github.com >> ~/.ssh/known_hosts
          git config --global core.sshCommand "ssh -i ~/.ssh/interface_commune -o IdentitiesOnly=yes"
          git config --global url."git@github.com:Galu83/Interface-commune".insteadOf "https://github.com/Galu83/Interface-commune"
  ```

  Une pull request venue d'un autre dépôt ne voit pas les secrets : sans objet, tout est privé.

## Les choix faits seuls (et comment revenir en arrière)

- **Une seule bibliothèque**, licence MIT (pour pouvoir servir aux logiciels sous toute licence).
- **La base vient de KVMSoft** (MIT) : thème, placement, clics, fenêtre unique du volet. Les
  fenêtres toutes faites et le champ de raccourci sont repris de Vocalis (MIT). Les mesures
  des Paramètres sont celles de PowerPeek (MIT depuis sa version 0.20.0).
- **Une seule boucle de fenêtres** : la fenêtre du volet est la fenêtre principale, jamais
  refaite ; les autres fenêtres sont dessinées par elle (`show_viewport_immediate` d'egui),
  même quand elle est cachée. Revenir : une boucle par fenêtre est impossible (winit n'en
  permet qu'une par programme).
- **Le logiciel qui ne veut que les Paramètres** écrit `Reglages` seul ; `Logiciel` le
  prolonge. Revenir : réunir les deux traits.
- **Segoe UI Variable** est lue une fois et réglée par ses axes (graisse 400/600, taille optique
  texte/titre) ; sans elle, Segoe UI et Segoe UI Semibold. Revenir : `polices.rs`.
- **Les pages se nomment par une clé** (`general`, `mise_a_jour`, `a_propos`, celles du
  logiciel) : la même clé sert à `--parametres <page>`, aux notifications et à la recherche.
- **« Annuler » reste 6 secondes** (`DUREE_ANNULER`), puis le changement est gardé.
- **Une deuxième copie du logiciel** ne s'ouvre pas : la première montre ses Paramètres.
- **Vérifications GitHub sous Windows** (là où tout le code se compile et où les polices sont) :
  deux fois plus de minutes que Linux, mais la bibliothèque change rarement.

## Les vérifications

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, `cargo build --examples`, `cargo deny check` (les deux crates de
  l'espace de travail : `interface-commune` et le cœur `crates/ic-coeur`) : aucune fenêtre ne s'ouvre pendant les tests
  (le volet, le menu et les Paramètres sont mis en page par egui seul).
- Le crochet `pre-push` commun contrôle chaque envoi (règle n° 1).

## Licences des dépendances

| Dépendance | Licence | Pour |
|---|---|---|
| egui, eframe | MIT OR Apache-2.0 | dessin de l'interface, fenêtres |
| winit | Apache-2.0 | fenêtres |
| windows | MIT OR Apache-2.0 | appels à Windows |
| raw-window-handle | MIT OR Apache-2.0 OR Zlib | lien fenêtre ↔ Windows |
| skrifa | MIT OR Apache-2.0 | contrôle des polices (tests) |
| proptest (essais seulement) | MIT OR Apache-2.0 | essais au hasard |

Toutes les licences des dépendances indirectes sont vérifiées par `cargo deny` (`deny.toml`,
le même que dans les outils communs). Les polices de Windows ne sont jamais copiées. Les crates
d'egui (et accesskit, profiling) sont publiées sans fichier de licence : leur avis MIT est dans
`docs/LICENCES.md`, à livrer avec chaque logiciel.

Les notes de passage de chaque projet : `docs/MIGRATION.md`.
