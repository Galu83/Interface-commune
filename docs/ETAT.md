# État

- **Refonte, bloc N « Instance unique »** (26 nov. 06:20, branche `refonte-bloc-n-instance`,
  partie de `refonte` à 61c66d4, à intégrer par l'Atelier Master ; rien n'est commité) :
  - **`src/bords/unique.rs`** (nouveau ; ORGANIGRAMME §10.1) : le verrou nommé
    `Local\<nom>-une-seule` (`Unique`, fermé à sa fin), le second lancement qui cherche la
    fenêtre cachée `<nom>-principal` et lui demande de s'ouvrir (`WM_APP + 3`) ou lui transmet
    ses arguments (`WM_COPYDATA`, 3 s au plus), et `lire_arguments` pour la copie qui reçoit ;
    chaque `unsafe` commenté ;
  - **sa partie pure** (`src/bords/unique/codage.rs`, aucun appel à Windows) : les noms du
    rendez-vous, le codage et la lecture des arguments (16 au plus, 4 096 octets en tout,
    marque « ICAR », vérifiés à l'arrivée), la décision d'un second lancement selon son
    `Reveil` (`Rien` : s'arrêter sans rien chercher ; `Parametres` : demander l'ouverture ;
    `Arguments` : transmettre, ou refuser hors des bornes), des **erreurs nommées**
    (`ErreurUnique`, cinq cas, en français) et leur traduction pour l'API ;
  - **repris aux mêmes chemins** : `interface_commune::horloge::{instance_unique,
    instance_unique_avec, Unique, ARGUMENTS_MAX, ARGUMENTS_OCTETS_MAX, Reveil}` (`pub use`) ;
    aucun changement d'API ; `tests/compatibilite.rs` et `examples/demo.rs` inchangés ;
    `horloge.rs` passe de 1382 à 1162 lignes (il garde la fenêtre cachée et sa réaction aux
    deux messages : bloc M) ;
  - **essais sans fenêtre** (`src/bords/unique/essais.rs`, 8) : aller-retour et refus du codage,
    bornes exactes (16 arguments, 4 096 octets, arguments vides), noms du rendez-vous, décisions
    du second lancement, traduction et ligne du journal, lecture prudente d'un `WM_COPYDATA`
    fabriqué à la main ; avec le vrai verrou et des noms inventés propres au processus : un
    second lancement sans rien dire (et ses demandes perdues faute de fenêtre), le verrou
    présent dans `Local\`, absent de `Global\`, libéré à la fin de la première copie. Les 3
    essais de `horloge` (`second_lancement_sans_rien_dire`, `arguments_du_second_lancement`,
    `copie_de_donnees_lue_avec_prudence`) y sont venus avec toutes leurs vérifications ;
    `COMPORTEMENTS.md` (C-15, C-16, C-18) et `CARTE.md` suivent ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, cible hors de la copie,
    aucune fenêtre vue) : `ic-coeur` 159 essais ; `interface-commune` 134 essais internes (129 − 3
    déplacés + 8 de l'instance unique) ; les 17 fichiers de `tests/` verts (compatibilite 8,
    comportements 12, etages 6, references aux images identiques ; `fenetres_en_rafale` et
    `recreation_apres_panique` passés, leurs fenêtres voilées et hors de l'écran) ; exemples de
    documentation 1 + 8 ; `fmt`, `clippy -D warnings`, `cargo deny` propres ;
    contrôle de la règle n° 1 : seulement `.git:1` (le fichier du worktree) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - N-1 : la partie pure reste dans `src/bords/unique/codage.rs`, pas dans `ic-coeur` : la
      liste des modules du cœur est fixée au bloc 0 et l'ORGANIGRAMME (§3.1, §10.1) range N
      dans les bords ; elle est pure et essayée là ;
    - N-2 : erreurs nommées dedans (`ErreurUnique`, `Issue`), traduites à la frontière en
      `Result<Unique, bool>` (§9) ; une demande perdue s'écrit au journal
      (`sonde::noter`, appelé, pas modifié) ; à ce moment le journal n'est souvent pas encore
      branché : la ligne se perd alors, sans dommage ;
    - N-3 : le numéro du message « ouvre-toi » (`WM_OUVRIR_PARAMETRES`) et le nom de la classe
      (`classe`, le point de rendez-vous des deux copies) passent dans `unique.rs` ;
      `horloge.rs` les reprend pour sa fenêtre cachée ;
    - N-4 : le canal qui ouvre une page demandée reste `Liens::montrer_page` (bloc J) ; la
      réaction de la première copie (`WM_OUVRIR_PARAMETRES` → ouvrir le logiciel,
      `WM_COPYDATA` → `lire_arguments` → le récepteur, réponse 1 ou 0) reste dans la procédure
      de la fenêtre cachée de `horloge.rs` (bloc M) ;
    - N-5 : comportement inchangé : noms, marque, bornes et délai identiques ; un verrou que
      Windows refuse compte comme « déjà là » ; seul changement : des arguments hors des
      bornes sont refusés avant de chercher la fenêtre (même résultat pour l'API :
      `Err(false)`) ; le nom du logiciel (sans espace ni barre oblique) n'est toujours pas
      vérifié ;
    - N-6 : `Unique` reste tenu par le logiciel (aucun état global), comme avant ;
  - **fichiers partagés touchés** : `src/bords/mod.rs` (+1 ligne : `pub(crate) mod unique;`) ;
    `src/horloge.rs` (seulement la partie instance unique et ses imports) ; `docs/CARTE.md`
    (2 lignes), `docs/COMPORTEMENTS.md` (3 lignes) ; ni `src/lib.rs`, ni `Cargo.toml`, ni
    `crates/ic-coeur/src/lib.rs` ;
  - **Pas vérifié sur le PC** : le vrai échange entre deux processus (il faudrait une fenêtre
    cachée vivante) ; il reste couvert par `tests/comportements.rs` (C-17, C-18) et par
    GitHub ;
  - **sous-agents** : aucun.

## Bloc Transversal — verrou, états globaux, documentation, Chat PDC retiré

(26 nov., branche `refonte-transversal`, partie de `refonte` à 00d42bf, à intégrer par
l'Atelier Master ; rien n'est commité, le message est dans `MESSAGE-COMMIT.txt`.)

- **Un seul verrou** (ORGANIGRAMME §6) : `src/bords/verrou.rs` (nouveau) tient `verrou`
  (`Mutex`), `lecture` et `ecriture` (`RwLock`) et `sans_poison` (une attente sur une
  `Condvar`) ; un verrou empoisonné est repris tel quel. La fonction de `fenetres.rs` et les
  reprises à la main de la sonde (bloc O), du guichet, du canal et des polices passent toutes
  par lui ; plus aucun doublon. 3 essais.
- **Changement de conduite, voulu** : `poste.rs` (`deposer`) **refusait** une demande quand
  son verrou était empoisonné ; il la dépose maintenant comme les autres. Aucune panique ne
  doit faire perdre une demande au guichet.
- **`reprise` rangée dans les bords** (`src/reprise.rs` → `src/bords/reprise.rs`) : module
  privé, aucun changement d'API.
- **États globaux** (ORGANIGRAMME §7) : chacun des `static` qui change, `OnceLock` et
  `thread_local` porte `// ÉTAT GLOBAL : <pourquoi>` (27), tous dans les bords ou dans les
  essais. Un a disparu : `OUVERTURES` du dessin des Paramètres, remplacé par un numéro tiré au
  hasard (`RandomState`) ; le dessin n'a donc plus d'état global.
- **Documentation** : `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace` est vert.
  Les `<logiciel>` des commentaires sont entre guillemets de code, les liens cassés réparés ;
  dans `crates/ic-coeur/src/lib.rs`, les commentaires `///` des modules deviennent des `//`
  (ils se mêlaient à la documentation `//!` de chaque module et cassaient ses liens). Une
  seule tolérance, `#[allow(rustdoc::invalid_html_tags)]` sur `textes` : la ligne fautive est
  dans `textes.rs`, que ce bloc ne touche pas ; à retirer quand le bloc des textes l'aura
  corrigée.
- **README et CARTE** : la nouvelle arborescence par étage (cœur, bords, dessin, façades) ;
  la commande de documentation entre dans les vérifications.
- **Chat PDC retiré** (arrêté par le propriétaire) : `mod chat_pdc` de
  `tests/compatibilite.rs`, et `IDEAL`, `ANALYSE`, `MIGRATION`, `ORGANIGRAMME`,
  `PLAN-REFONTE` (§4 et §6) et `COMPORTEMENTS` (C-82) ne nomment plus que KVMSoft, PowerPeek
  et Vocalis. Une seule note d'histoire, dans `ANALYSE.md` : ses usages étaient compris dans
  ceux des autres, les chiffres ne changent pas. **L'ordre de bascule** devient Vocalis (il
  prend le moins), KVMSoft, PowerPeek.
- **Plan, §4 et §6** (ajouté par l'Atelier Master) : KVMSoft épingle `d9d56da`, pas `e077572`
  (relevé dans son `Cargo.toml` et son `Cargo.lock`). Condition 2 de la bascule, essais du
  26 nov. sur le rev `e750cb1` : KVMSoft passe sans aucun changement de code (620 tests, clippy,
  deny) ; PowerPeek passe aussi (849 tests ; les écarts d'images annoncés sont acceptés).
- **Trois défauts trouvés par l'essai de compatibilité de PowerPeek** (ajoutés au bloc par
  l'Atelier Master) :
  - **carte « Journal », journal effaçable** : la place réservée à droite se mesure sur les
    boutons montrés (`place_des_boutons`, interne, dans `dessin/controles/cartes.rs` ;
    `largeur_bouton` reprend le calcul de `bouton`) au lieu des 200 points d'un contrôle
    simple ; « Effacer le journal » ne recouvre plus la fin de l'explication, qui passe à la
    ligne. Essai : `boutons_du_journal_ne_recouvrent_pas_l_explication` ;
  - **titre seul** (`carte_avec`) : sans explication, le titre se centre sur la hauteur de la
    carte (48 points), comme l'icône et l'interrupteur. Essai : `titre_seul_centre_dans_sa_carte` ;
  - les deux essais **échouent** sans les corrections (vérifié en les retirant un instant) ;
  - **`MIGRATION.md`** : la redite retirée est décrite de façon générale (la phrase qui redit
    le chemin des installateurs), avec la forme de KVMSoft et celle de PowerPeek ; les deux
    corrections du dessin y sont notées ;
  - **12 images de référence refaites** (`REFERENCES=refaire`, puis comparaison verte ;
    empreinte des polices inchangée) : `parametres-general-clair`, `parametres-general-sombre`,
    `parametres-general-clair-150`, `parametres-conserver-clair`, `parametres-mise-a-jour-clair`,
    `parametres-mise-a-jour-sombre`, `mise-a-jour-coupee-clair`, `mise-a-jour-erreur-clair`,
    `mise-a-jour-inconnu-clair`, `mise-a-jour-information-clair`, `mise-a-jour-prete-clair`,
    `mise-a-jour-recherche-clair`. Chaque écart a été regardé : rien d'autre ne bouge.
- **API publique** : aucun changement (les deux fonctions nouvelles sont internes) ;
  `MIGRATION.md` change son exemple (« Mon logiciel ») et la note ci-dessus.
- **Résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, cible hors de la copie) :
  `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`, `cargo deny check`,
  `cargo doc --no-deps --workspace` avec `-D warnings` et `cargo build --examples` à 0 ;
  `cargo test --workspace` : 420 essais passés, 0 échec, 1 ignoré (comme avant), images de
  référence comprises. Contrôle de la règle n° 1 sur l'index exporté : rien trouvé.
- **Fichiers partagés touchés** : `src/lib.rs` (`mod reprise` retiré), `src/bords/mod.rs`,
  `crates/ic-coeur/src/lib.rs` (commentaires seulement), `README.md`, tous les documents de
  `docs/`.
- **Sous-agents** : aucun.

## Correctif lisible — symboles séparés par une espace

(26 nov., branche `refonte-correctif-lisible`, partie de `refonte` à e750cb1, à intégrer par
l'Atelier Master ; rien n'est commité, le message est dans `MESSAGE-COMMIT.txt`.)

- **Le défaut** (trouvé au hasard par `lisible_ne_change_pas_deux_fois`, entrée
  `["⌃", " ", "⌃"]`, séparateur vide, donc « ⌃ ⌃ ») : `lisible` détachait les symboles collés
  mais s'arrêtait à l'espace ; le second « ⌃ » devenait une touche. Premier appel :
  « Ctrl+⌃ » ; relu : « Ctrl ». La faute est donc au premier appel, pas au second.
- **Le correctif** (`crates/ic-coeur/src/textes/raccourcis.rs`, une ligne) : après chaque
  symbole, les espaces qui suivent sont sautées ; « ⌃ ⌃ » donne « Ctrl », « ⌃ ⇧ K » donne
  « Ctrl+Maj+K ». Rien d'autre ne change (les écritures déjà comprises donnent le même
  résultat).
- **Essais** : un essai nommé, `textes::essais::lisible_symboles_separes_d_une_espace` (5
  vérifications) ; le dossier `crates/ic-coeur/proptest-regressions/` entre à l'index : le cas
  trouvé est rejoué à chaque passage. `COMPORTEMENTS.md` (C-46) suit.
- **Résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, cible hors de la copie) :
  tous les essais du dépôt verts ; `fmt`, `clippy -D warnings`, `cargo deny` propres ;
  contrôle de la règle n° 1 sur la copie de l'index : rien.
- **Fichiers partagés touchés** : aucun (seulement `raccourcis.rs`, `textes/essais.rs`, le
  fichier de `proptest`, `COMPORTEMENTS.md` et ce fichier).
- **Sous-agents** : aucun.

## Bloc I — les captures

(26 nov., branche `refonte-bloc-i-captures`, partie de `refonte` à c4fd163, à intégrer par
l'Atelier Master ; rien n'est commité, le message est dans `MESSAGE-COMMIT.txt`.)

- **Les captures passent dans le dessin** (ORGANIGRAMME §3.2, §10.1 ligne `capture`) :
  `src/dessin/capture.rs` (`Options`, `Image`, `Erreur`, `dessin`, `volet`, `volet_borne`,
  `petit_menu`, `parametres`, `parametres_de`, `toutes`, et la mise en page par egui en
  quatre passes) et `src/dessin/capture/toile.rs` (le rendu logiciel : textures, triangles,
  règle « haut-gauche », découpe ; ses 2 essais). Le code du rendu est déplacé tel quel :
  aucun calcul de pixel n'a changé.
- **Réutilisation du dessin rangé** : le volet et le petit menu par `dessin::surface`
  (bloc G), les Paramètres par `dessin::parametres::Parametres` (bloc H), les fenêtres toutes
  faites et les contrôles par `capture::dessin` (blocs F et H) ; rien n'est recopié.
- **Chemin public inchangé** : `src/capture.rs` ne fait plus que reprendre
  `crate::dessin::capture::*` (`pub use`), avec la documentation pour les logiciels (son
  exemple compris) ; `interface_commune::capture::{self, Options, Image, …}` reste valable ;
  `tests/compatibilite.rs` inchangé ; aucune ligne de `src/lib.rs` ne change.
- **L'image de la fenêtre de lecture** (ORGANIGRAMME §3.2 I, « à ajouter ») : deux images de
  référence de plus, `fenetre-lecture-clair.png` et `fenetre-lecture-sombre.png` (un texte
  inventé de deux paragraphes, le mot « voix » surligné), faites par `capture::dessin` et
  `toutes_faites::dessiner_lecture` dans `tests/references.rs` ; C-115 compare 36 images.
- **Résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, cible hors de la copie,
  aucune fenêtre ouverte par le bloc) : `ic-coeur` 184 essais ; `interface-commune` 136 essais
  internes (les 3 du rendu, déplacés : 2 dans `toile.rs`, 1 dans `capture.rs`) ; les 16
  fichiers de `tests/` verts (compatibilite 8, comportements 13, etages 6, sans_fenetre 23,
  references 2 avec les 36 images ; `fenetres_en_rafale`, `premiere_creation` et
  `recreation_apres_panique` passés, leurs fenêtres voilées et hors de l'écran comme avant) ;
  exemples de documentation 1 + 8 (1 ignoré, comme avant) ; `fmt`, `clippy -D warnings`,
  `cargo deny` propres ; contrôle de la règle n° 1 : seulement `.git:1` (le fichier du
  worktree).
- **Les 34 images d'avant ne changent pas d'un pixel** : refaites cinq fois de suite
  (`REFERENCES=refaire`), 33 sont identiques octet pour octet à chaque fois ;
  `parametres-conserver-clair.png` l'est deux fois sur cinq, et les autres fois diffère
  (10 pixels les deux fois où l'écart a été mesuré, une zone de 3 × 4 pixels, au bout de la
  barre du compte à rebours). Ce n'est
  pas le déplacement : la carte « Conserver ces modifications ? » lit l'heure réelle
  (`Instant::now`, `src/dessin/parametres/bulles.rs`), et la barre avance de quelques
  millisecondes entre deux rendus. L'image de référence est gardée telle quelle (remise par
  `git checkout`) ; l'écart reste bien sous la tolérance de l'essai (C-115 vert). Les deux
  images de la fenêtre de lecture, refaites deux fois, sont identiques octet pour octet.
- **Décisions prises seul** (à relire par l'Atelier Master) :
  - **I-1** : `src/capture.rs` devient une façade (comme `sonde`, `verification` au bloc O) au
    lieu d'un module en ligne dans `src/lib.rs` : aucune ligne de `src/lib.rs` ne bouge ;
  - **I-2** : le rendu logiciel va dans son propre fichier (`capture/toile.rs`, 247 lignes) :
    `capture.rs` dit ce qu'on capture, `toile.rs` comment on pose les pixels ; l'essai du PNG
    reste avec `Image` dans `capture.rs` ;
  - **I-3** : pas de fonction publique `capture::lecture` (aucun changement d'API, plan §1) :
    l'image de la fenêtre de lecture passe par `capture::dessin`, comme la fenêtre simple ;
  - **I-4** : la carte « Conserver » qui dépend de l'heure n'est pas corrigée ici : il faudrait
    donner un instant aux Paramètres (blocs E et H) ; point ouvert pour l'Atelier Master.
- **Fichiers partagés touchés** (des lignes ajoutées seulement) : `src/dessin.rs` (+2 :
  `pub(crate) mod capture;` et une ligne de son en-tête) ; `docs/CARTE.md` (+1 ligne) ;
  `docs/COMPORTEMENTS.md` (+2 lignes, sous le tableau des promesses : la ligne C-115 elle-même
  dit encore « 34 images », à mettre à 36 à l'intégration) ; `docs/ETAT.md` (cette section).
  Ni `src/lib.rs`, ni `Cargo.toml`.
- **Pas vérifié** : les captures sur un PC dont les polices diffèrent (la comparaison s'y
  saute, comme avant) ; la ligne « la fenêtre de lecture est à ajouter » de l'ORGANIGRAMME
  (§3.2 I) n'est pas modifiée (fichier hors du bloc).
- **Tailles** : `capture.rs` 258 lignes, `toile.rs` 247, `src/capture.rs` 35 ; aucune
  fonction de plus de 60 lignes.
## Bloc J — les boucles des fenêtres

(26 nov., branche `refonte-bloc-j-boucles`, partie de `refonte` à c4fd163, à intégrer par
l'Atelier Master ; rien n'est commité, le message est dans `MESSAGE-COMMIT.txt`.)

- **Ce qui a déménagé dans `src/bords/fenetres/`** (les anciens fichiers restent en modules minces : Git montre les nouveaux comme ajoutés) :
  `src/fenetres.rs` (1 261 lignes) → `boucle.rs` (la boucle de la bibliothèque,
  `boucle_des_fenetres`, `Ouvreur`), `liens.rs` (demandes, `envoyer_le_clic`, `Liens`),
  `canal.rs` (envoi, réception, réveil), `supervision.rs` (façon 2 : `superviser_les_fenetres`,
  `Supervise`) ; `src/autres_fenetres.rs` (1 507) → `autres.rs`, `autres/ouverture.rs`
  (`Ouverture`, `Contenu`, `Cadre`), `autres/delais.rs` (attentes avant une création) ;
  `src/volet.rs` (1 205) → `volet.rs`, `volet/veille.rs` (`Veilleur`), `volet/fenetre.rs` (la
  fenêtre et ses étapes), `volet/app.rs` (son `eframe::App`) ; `src/parametres/fenetre.rs` →
  `parametres.rs` (`AutresFenetres::lier/montrer` et `ouvrir_dans_boucle` y passent) ;
  `src/toutes_faites.rs` → `toutes_faites.rs`. La racine `src/bords/fenetres.rs` tient
  `Pertes` et `verrou`.
- **Le faux moteur** (`faux_moteur.rs`) : une fausse fenêtre du volet pour la boucle (elle compte
  préparations et ouvertures, se perd ou panique sur commande) et le faux egui des autres
  fenêtres (egui croit avoir de vraies fenêtres, rien ne s'ouvre). Les paniques sont attrapées
  fenêtre par fenêtre ; la recréation est essayée sur le faux moteur.
- **Chemins publics inchangés** : `src/fenetres.rs`, `src/autres_fenetres.rs`, `src/volet.rs`,
  `src/parametres/fenetre.rs` et `src/toutes_faites.rs` ne font plus que reprendre
  (`pub use crate::bords::fenetres::…::*`) avec leur documentation ; aucune ligne de
  `src/lib.rs` ne change ; `tests/compatibilite.rs` inchangé ; aucune image de référence
  changée.
- **Ce qui est réutilisé, pas copié** : `bords::poste` (K), `bords::systeme` (L), `src/dessin/`
  (F, G, H) ; la décision du clic et d'une perte reste dans le cœur (`ic_coeur::gestes`, bloc D).
- **Essais** : ceux des fenêtres dans `bords/fenetres/essais.rs` (faux moteur : surface
  préparée une fois, perte recréée et clic refait, rafales, supervision), ceux des autres
  fenêtres dans `bords/fenetres/autres/essais.rs`, ceux du volet et des fenêtres toutes faites
  à côté de leur code ; `COMPORTEMENTS.md` donne leurs nouveaux noms complets (C-31 à C-34,
  C-47, C-50, C-58 à C-63…) ; C-32, C-35, C-60, C-62, la recréation après panique et la
  façon 2 gardés.
- **Résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, cible à part, aucune
  fenêtre vue) : `interface-commune` 136 essais internes verts ;
  les 17 fichiers de `tests/` verts (compatibilite 8, comportements 13, sans_fenetre 23,
  references aux images identiques ; `fenetres_en_rafale` et `recreation_apres_panique`
  passés, fenêtres voilées) ; exemples de documentation 1 + 8 ; `fmt`, `clippy -D warnings`,
  `cargo deny` propres ; contrôle de la règle n° 1 sur le contenu de l'index et le
  message : 0 trouvé (le dossier de compilation, dans la copie mais ignoré par Git, est écarté) ; `ic-coeur` 183
  essais sur 184 : l'essai au hasard `textes::essais::lisible_ne_change_pas_deux_fois`
  (bloc B, aucune ligne de `crates/` touchée par J) échoue sur `["⌃", " ", "⌃"]` avec un
  séparateur vide (« Ctrl » au lieu de « Ctrl+⌃ ») : un vrai défaut, voir « À suivre »
- **Décisions prises seul** (à relire par l'Atelier Master) :
  - **J-1** : les anciens chemins deviennent des modules minces `pub use …::*` (comme L et O) ;
  - **J-2** : un seul `verrou` commun (`bords::fenetres::verrou`) pour tous les `Mutex` du
    bloc : un verrou empoisonné est repris, jamais `unwrap`. Petit changement : avant, une
    action était sautée sur un verrou empoisonné (`if let Ok`) ; elle se fait maintenant ;
  - **J-3** : les statiques gardées sont justifiées par `// ÉTAT GLOBAL :` :
    `OBSERVATEUR` et `DISCRET` (`observer()` est une fonction publique sans poignée),
    `GARDER_CACHEES` (fonction publique des essais), `AFFICHAGE_CHANGE` (signal du fil de
    l'icône), `PRISES` (les fenêtres de Windows sont à tout le processus), `DEJA_OUVERTE` et
    `DEJA_PREPAREE` (« depuis le lancement » : doivent survivre à la recréation) ;
    `parametres::OUVERTURES` (bloc H) n'est pas touché ;
  - **J-4** : les délais avant création restent dans les bords (`autres/delais.rs`) ; ils sont
    candidats pour le cœur plus tard ;
  - **J-5** : tous les faux moteurs (boucle et egui) sont réunis dans `faux_moteur.rs` ;
  - **J-6** : une cible de compilation à part (`target-bloc-j`) pour ne pas gêner les autres
    blocs ;
  - **J-7** : `boucle_des_fenetres` est découpée en un état et des pas ; le texte du journal
    est mot pour mot le même ;
  - **J-8** : `autres` et `volet` sont découpés en sous-modules, leurs longues fonctions
    coupées sous 80 lignes (plus long fichier : 634 lignes, `autres.rs`).
- **Fichiers partagés touchés** : `src/bords/mod.rs` (+3 lignes : `pub(crate) mod fenetres;` et
  sa documentation) ; `docs/CARTE.md` (une ligne ajoutée, quatre complétées),
  `docs/COMPORTEMENTS.md` (les noms complets des essais déplacés) ; ni `src/lib.rs`, ni
  `Cargo.toml`, ni `crates/ic-coeur/src/lib.rs` ; ni `src/horloge.rs`, ni `src/zone.rs`, ni
  `src/capture*`.
- **Pas vérifié sur le PC** : les vraies fenêtres à l'écran (interdit pendant les essais) ;
  elles restent couvertes par `tests/fenetres_en_rafale`, `tests/recreation_apres_panique`
  (fenêtres voilées) et par GitHub.
- **À suivre** (hors du bloc) : la sonde (bloc O) a ses propres `PoisonError` (pourrait
  prendre `verrou`) ; `cargo doc` avec `-D warnings` signale des liens déjà cassés avant J
  (`dessin::fond`, `crate::reprise`, `NOUVEL_ESSAI` privé, `VOLET_TUILE`, balises `<logiciel>`) ;
  l'essai au hasard du bloc B (`textes::lisible`, entrée `["⌃", " ", "⌃"]`, séparateur vide)
  trouve un vrai défaut, rejoué à chaque passage depuis que `proptest` l'a gardé ; son dossier
  `crates/ic-coeur/proptest-regressions/` reste hors de l'index (à corriger dans le bloc B).
- **Sous-agents** : aucun.

## Bloc O — la sonde, le journal et la vérification

(26 nov. 06:30, branche `refonte-bloc-o-sonde`, partie de `refonte`, à intégrer par l'Atelier
Master ; rien n'est commité, le message est dans `MESSAGE-COMMIT.txt`.)

- **La sonde passe dans les bords** : `src/bords/sonde.rs` (les fils, leurs compteurs, la
  dernière étape, le journal branché, les délais), `src/bords/sonde/chien.rs` (le fil de
  surveillance et le chien des créations de fenêtre), `src/bords/sonde/panique.rs` (le lieu
  de la dernière panique), `src/bords/sonde/essais.rs`.
- **La décision « bloqué » est dans le cœur** : `crates/ic-coeur/src/gestes/sonde.rs`
  (`Surveillance<K, T>`, `Releve`, `Constat`, `Delais`, `est_une_veille`, le chien des
  créations, le texte de chaque ligne du journal, `lieu_court`), essais dans
  `gestes/sonde/essais.rs`.
- **La vérification passe dans les bords** : `src/bords/verification.rs` (déplacé tel quel).
- **Chemins publics inchangés** : `src/sonde.rs` et `src/verification.rs` ne font plus que
  reprendre (`pub use`) ce qui est dans `crate::bords::…`, avec la documentation pour les
  logiciels ; `interface_commune::sonde::…` et `interface_commune::verification::…` restent
  valables.
- **Décisions prises seul** (à relire par l'Atelier Master) :
  - **O-1** : la décision « bloqué » manquait dans le cœur ; elle est ajoutée au module voie 1
    `gestes` (`ic_coeur::gestes::sonde`), avec la veille du PC, le chien des créations, les
    phrases du journal (mot pour mot comme avant) et le lieu court d'une panique. Le cœur
    reçoit l'heure UTC en secondes (`u64`) : `SystemTime` y est interdit ;
  - **O-2** : `src/sonde.rs` et `src/verification.rs` deviennent des façades ; aucune ligne de
    `src/lib.rs` ne change. Comme les bords, elles ne valent que sous Windows (c'était déjà le
    cas de fait : la sonde et la vérification appellent Windows) ;
  - **O-3** : `sonde::Surveillance` garde son nom, `new` et `avec_pas` : une enveloppe autour
    de `coeur::Surveillance<usize, Instant>`, la clé d'un fil étant l'adresse de son `Arc`.
    `Delais`, `est_une_veille` et `SEUIL_DE_CREATION` viennent du cœur, repris au même chemin ;
  - **O-4** : les essais de la sonde sont partagés : 15 dans le cœur (fausse horloge
    `Duration`, fils simulés `Releve<u8, Duration>`), 8 dans les bords (les vrais fils, le
    journal branché — C-110 —, la vraie boucle à délais courts de 200 ms, le chien des
    créations, les paniques). Aucune attente de 15 s, aucune fenêtre. Le lieu de panique
    attendu par l'essai devient `sonde/essais.rs:` (le fichier de l'essai a changé).
- **Fichiers partagés touchés** (une ligne ajoutée chacun) : `src/bords/mod.rs`
  (`pub(crate) mod sonde;` et `pub(crate) mod verification;`, une ligne par module) ;
  `crates/ic-coeur/src/gestes.rs` (`pub mod sonde;` et une ligne de son en-tête). Ni
  `src/lib.rs`, ni `Cargo.toml`, ni `crates/ic-coeur/src/lib.rs`.
- **Avec le bloc N** (entré dans `refonte`, c9c0394) : `crate::sonde::noter(ligne: String)`,
  qu'appelle `src/bords/unique.rs`, reste valable (repris `pub(crate)` par `src/sonde.rs`) ;
  dans `src/bords/mod.rs`, garder les deux côtés (`unique` et `sonde`, `verification`).
- **Docs** : `COMPORTEMENTS.md` (C-92, C-93, C-107 à C-110 : les nouveaux noms des essais),
  `CARTE.md` (les nouveaux fichiers).
- **Tailles** : le plus long fichier du bloc fait 335 lignes (`gestes/sonde.rs`), aucune
  fonction de plus de 80 lignes ; `src/sonde.rs` passe de plus de 1 000 lignes à 46.

## Bloc M — l'icône seule devient une zone à une icône

(26 nov. 08:59, branche `refonte-bloc-m-icone`, partie de `refonte`, à intégrer par l'Atelier
Master ; rien n'est commité, le message est dans `MESSAGE-COMMIT.txt`.)

- **Un seul moteur des icônes** : `src/bords/icone/zone.rs` (`Zone` : la fenêtre cachée de
  `bords::systeme::cachee`, les appels à `Shell_NotifyIconW`, la remise après le redémarrage
  de l'Explorateur, le menu gris de secours, la fin de session, le changement d'affichage).
  `Horloge` (`src/bords/icone/horloge.rs`) n'est plus qu'une zone à **une seule icône** : une
  seule fenêtre cachée, un seul menu de secours ; elle y ajoute sa procédure (commandes,
  veille, thème, instance unique de `bords::unique`, guichet de `bords::poste`) et sa boucle.
  `FenetresCommunes` (`communes.rs`) reste pour le logiciel qui garde sa boucle.
- **Les autres fichiers** : `decodage.rs` (le décodage pur : messages et rappels, fin de
  session, destinataire d'un message, lignes et textes du menu, alimentation), `image.rs`
  (l'image depuis des pixels), `fil.rs` (le fil des fenêtres et son secours), `icone.rs` (la
  liste des modules).
- **Chemins publics inchangés** : `src/horloge.rs` (39 lignes) et `src/zone.rs` (63 lignes)
  gardent leur documentation et reprennent tout par `pub use`, sans `unsafe` ;
  `tests/compatibilite.rs` et `examples/` inchangés ; aucune image de référence ne change.
- **Décisions prises seul** (à relire par l'Atelier Master) :
  - **M-1** : l'icône seule garde son numéro 1 sans GUID et son message `WM_ICONE` (Windows
    retient déjà l'icône des logiciels sous ce numéro) ; les icônes de la zone gardent leur
    GUID tiré du nom. `Ident::{Guid, Numero}` dans la zone ;
  - **M-2** : le menu de secours est celui de la zone pour les deux : mêmes lignes, même ordre
    (C-45) ; les « & » y sont doublés, la ligne par défaut est mise par `SetMenuDefaultItem`
    et un `WM_NULL` est posté après le menu (conseil de Windows), pour l'icône seule aussi ;
  - **M-3** : la procédure de l'icône seule passe **avant** celle de la zone (« complément ») ;
    pendant son appel, il est sorti de la zone et remis ensuite si la fenêtre vit encore
    (`GWLP_USERDATA` relu) ; un message envoyé pendant ce temps va à la zone seule. Le
    changement d'affichage (C-13) est signalé avant ; la fin de session et « TaskbarCreated »
    restent à la zone ;
  - **M-4** : une modification relance l'ajout si l'icône n'était pas dans la barre ; une
    remise manquée pendant le menu de secours est rattrapée au message suivant ; les gestes
    reçus pendant le menu sont ignorés ; la place de l'icône est relue à chaque geste ;
  - **M-5** : `Aiguilleur` (bloc D) remplace les clics et l'ordre propres à l'icône seule :
    la même suite des gestes pour les deux ;
  - **M-6** : la fin de la zone (`Drop`) retire ses icônes et détruit sa fenêtre ; `Horloge`
    ferme par `detruire` puis `PostQuitMessage`. `Horloge::new` reste sans effet sur Windows ;
  - **M-7** : dans les essais (`cfg(test)`), `Shell_NotifyIconW` est remplacé par une note de
    chaque appel (opération, numéro, GUID, message) qui réussit : **aucune icône n'est
    montrée** et la remise après le redémarrage de l'Explorateur s'essaie (C-06). En
    contrepartie, le vrai appel n'est fait que par les logiciels ;
  - **M-8** : le décodage pur reste dans `bords/icone/decodage.rs` (essayé là, sans Windows) ;
    le passer dans `ic-coeur` est une suite possible. Les `#[allow(unsafe_code)]` de
    `src/lib.rs` pour `horloge` et `zone` restent (lib.rs n'est pas touché) : les retirer est
    une suite possible, les façades n'ayant plus d'`unsafe` ;
  - **M-9** : les essais de la zone sont dans `zone/essais.rs`, un module enfant (il lit les
    champs privés de `Zone`) ; ceux de l'icône seule dans `essais_horloge.rs`. L'ancien essai
    `gestes_par_les_mecanismes_communs` (93 lignes) est coupé en deux
    (`gestes_vers_le_secours`) pour la limite de 80 lignes ; aucune vérification perdue ;
  - **M-10** : aucune sonde de plus pour `FenetresCommunes` ; les étapes de sonde autour de
    `Shell_NotifyIcon` valent maintenant aussi pour les icônes de la zone.
- **Essais nouveaux** (sans fenêtre visible, aucune icône montrée) :
  `remise_apres_le_redemarrage_de_l_explorateur` (l'icône seule et une icône de zone, ajout
  puis version 4, « TaskbarCreated » reçu, remise une seule fois, modification, retrait),
  `rappel_de_l_icone_seule` (le rappel au numéro 1, les messages qui ne sont pas des gestes),
  `procedure_de_plus_avant_la_zone` (réponse du complément, réentrée, fin de session gardée
  par la zone).
- **Fichiers partagés touchés** : `src/bords/mod.rs` (+2 lignes : la doc et
  `pub(crate) mod icone;`, sous `cfg(windows)`) ; `docs/CARTE.md` (+1 ligne) ;
  `docs/COMPORTEMENTS.md` (9 lignes **modifiées**, pas seulement ajoutées : les essais ont
  changé de chemin — C-03, C-04, C-06, C-08 à C-12, C-45) ; `tests/etages.rs` (la liste
  `UNSAFE_EN_ATTENDANT` est vide : `horloge` et `zone` n'ont plus d'`unsafe`). Ni
  `src/lib.rs`, ni `Cargo.toml`, ni `crates/ic-coeur/src/lib.rs`.
- **Tailles** : `src/horloge.rs` passe de 1 129 à 39 lignes, `src/zone.rs` de 1 556 à 63 ; le
  plus long fichier du bloc fait 708 lignes (`bords/icone/zone.rs`), aucune fonction de plus
  de 80 lignes.
- **Résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, cible hors de la copie,
  aucune fenêtre vue, aucune icône montrée) : `ic-coeur` 184 essais ; `interface-commune` 140
  essais internes ; les 18 fichiers de `tests/` verts (compatibilite 8, comportements 13,
  etages 6, zone_affichage 1, references aux images identiques, `fenetres_en_rafale` et
  `recreation_apres_panique` passés) ; exemples de documentation 1 + 8 ; `fmt`,
  `clippy -D warnings`, `cargo deny` propres ; contrôle de la règle n° 1 : seulement `.git:1`
  (le fichier du worktree).
- **Pas vérifié sur le PC** : le vrai `Shell_NotifyIconW` (remplacé dans les essais, M-7), un
  vrai redémarrage de l'Explorateur, le menu de secours ouvert à l'écran et la vraie boucle de
  `Horloge::lancer` : ils demandent une icône visible ; à essayer à la main ou par un
  logiciel (C-06 reste aussi « manuel »).
- **Sous-agents** : aucun.

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
- **Fait sur `correctif/case-debranchee`** (25 nov., pour KVMSoft) : un écran débranché dans
  un choix illustré (`Case::debranche`) : case pâle (opacité 0,4), bord en tirets, lettre
  gardée ; l'emporte sur la veille (`docs/MIGRATION.md`, C-117). Deux images de référence
  nouvelles, aucune autre changée. Reporté dans `refonte` (fusion de `main`, 25 nov.) : la
  décision (débranché l'emporte sur la veille) et l'opacité dans le cœur
  (`ic_coeur::theme::{etat_case, OPACITE_DEBRANCHE}`), le dessin dans `src/dessin/pastilles.rs`.
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
- **Refonte, bloc G « Volet et petit menu (dessin) »** (25 nov. 23:05, branche
  `refonte-bloc-g-volet`, partie de `refonte` à c6a354d, à intégrer par l'Atelier Master ;
  **rien n'est commité** : tout est indexé, le message est dans `docs/MESSAGE-COMMIT.txt`) :
  - **le dessin dans `src/dessin/`** : `src/surface.rs` déplacé (`git mv`) en
    `dessin/surface.rs` (largeur, `dessiner`, `dessiner_borne`, petit menu, séparateur) ; le
    dessin du volet sorti de `src/volet.rs` en `dessin/volet.rs` (haut du volet, briques qui
    défilent, pied), `dessin/volet/entete.rs` (titre, version, roue) et
    `dessin/volet/clavier.rs` (les flèches changées en Tab, C-33) ;
  - **dans le cœur** (calcul pur) : `ic_coeur::placement::surface` (marges, largeur entre la
    base et `LARGEUR_MAX`, besoin du volet et du menu, hauteur permise et rendue, place des
    briques, places de l'en-tête : roue, titre, version ; 8 essais) ;
    `ic_coeur::gestes::clavier` (flèche haut/bas → Maj+Tab/Tab, rien avec un modificateur ;
    1 essai) ; `GenreEtat::icone_et_couleur` dans `contrat` (l'icône et la couleur d'un état ;
    1 essai) ; `menu::Ligne::en_gras` (seule « Ouvrir » est en gras ; 1 essai) ;
  - **repris aux mêmes chemins** : `interface_commune::surface` est un module mince qui reprend
    `dessin::surface` ; `volet::fleches_en_tab` reprend `dessin::volet::clavier` ;
    `tests/compatibilite.rs` inchangé ; **aucun changement d'API** (le PLAN, §4, n'en prévoit
    pas pour G) ; **aucune image de référence ne change** (`references` vert) ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 127 essais (116 + 11) ; `interface-commune` 145 essais internes, tous les
    fichiers de `tests/` verts (references, compatibilite, comportements, polices, etages,
    sans_fenetre…), exemples de documentation ; `fmt`, `clippy -D warnings`, `cargo deny`
    propres ; contrôle de la règle n° 1 : seulement `.git:1` (le fichier du worktree) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - `icone_et_couleur` vit dans `contrat` (à côté de `GenreEtat`) et lit la palette sans la
      changer : `theme`, `parametres` et `src/bords/` ne sont pas touchés ;
    - `Choisi` et la hauteur du pied gardée d'une image à l'autre (mémoire d'egui) restent
      dans le dessin : ils sont liés à egui, sans décision dedans ;
    - les opérations sur les nombres gardent leur ordre d'avant (même rendu au bit près) ;
    - `tests/polices.rs` lit les nouveaux fichiers (`dessin/surface.rs`, `dessin/volet.rs`,
      `dessin/volet/entete.rs`, `dessin/volet/clavier.rs`) ; `CARTE.md` et `COMPORTEMENTS.md`
      (C-25, C-33) suivent ;
    - un dossier de compilation propre au worktree (`CARGO_TARGET_DIR`), comme aux blocs D, F ;
  - **Reste pour H et J** :
    - J : `src/volet.rs` fait encore 1205 lignes (la fenêtre du volet : sa boucle, son voile,
      son veilleur C-34, C-31) ; il part dans `src/bords/fenetres/volet.rs` et se découpe là ;
    - H : les Paramètres se dessinent de la même façon (mise en page dans le cœur, dessin dans
      `src/dessin/`) ; `separateur` et `version` de `dessin::surface` peuvent servir ;
  - **À savoir pour l'intégration** : conflits possibles avec `refonte` si elle a bougé dans
    `src/surface.rs`, `src/volet.rs`, `src/lib.rs`, `src/dessin.rs`, `tests/polices.rs`,
    `docs/CARTE.md`, `crates/ic-coeur/src/{contrat,gestes,placement}.rs`.
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
- **Refonte, bloc L « Système Windows »** (26 nov. 06:30, branche `refonte-bloc-l-systeme`,
  partie de `refonte` à 61c66d4, à intégrer par l'Atelier Master ; **rien n'est commité** :
  tout est indexé, le message est dans `docs/MESSAGE-COMMIT.txt`, hors de l'index) :
  - **dans les bords** : `src/systeme.rs` (802 lignes) découpé en `src/bords/systeme/` :
    `fenetre` (habillage DWM, place, opacité, fenêtre du fil), `ecran`, `reveil`, `dossier`,
    `registre` (lecture du thème), `echelle` (changement d'échelle, `reglage_dpi` venu de
    `horloge`), `apparence` (le thème suivi), `polices` (lecture des polices) ;
    `src/verification.rs` déplacé (`git mv`) en `src/bords/verification.rs` ;
  - **une seule fabrique de fenêtre cachée** (`bords/systeme/cachee.rs`) : l'icône unique
    (`horloge`), les icônes d'une zone (`zone`) et les essais en font leurs fenêtres ;
  - **dans le cœur** (calcul pur, avec essais) : `theme::choix_du_registre` (ce que veulent dire
    les valeurs du registre ; 4 essais), `theme::polices` (le plan des polices : quel fichier,
    ses replis, graisse et taille optique ; 5 essais), `gestes::clics::temps_double_clic` (la
    borne du double-clic ; 1 essai) ;
  - **dans le dessin** : `src/dessin/apparence.rs`, la palette, les styles de texte et les
    espacements donnés à egui (2 essais) ;
  - **essais** : ceux de `systeme` sortent du code dans `bords/systeme/essais.rs` (fenêtres
    toujours cachées, détruites à la fin) ; nouveaux : la fabrique refuse ou reprend une classe
    déjà là ; le changement d'échelle suit Windows quoi que winit demande, et une fenêtre non
    suivie va où winit la pose (C-113) ; `Apparence` (3), les polices lues (3) ;
  - **repris aux mêmes chemins** : `systeme`, `apparence`, `polices`, `echelle` et
    `verification` sont des modules minces (`pub use`) ; **aucun changement d'API**
    (`tests/compatibilite.rs` inchangé) ; **aucune image de référence ne change** ;
    `COMPORTEMENTS.md` (C-57, C-90, C-92, C-93, C-111 à C-113) et `CARTE.md` suivent ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre vue) :
    `ic-coeur` 169 essais (avant : 159) ; `interface-commune` 139 essais internes, les 18
    fichiers de `tests/` verts (references aux images identiques, compatibilite, comportements,
    polices, etages, fenetres_en_rafale, recreation_apres_panique, sans_fenetre…), exemples de
    documentation (1 sauté, comme avant) : 398 essais en tout ; `fmt`, `clippy -D warnings`,
    `cargo deny` propres ; `clippy -D warnings` de `ic-coeur` propre aussi pour Linux ;
    contrôle de la règle n° 1 : 0 trouvé dans une copie de l'index avec
    `docs/MESSAGE-COMMIT.txt` ; sur le worktree, seulement `.git:1` (le fichier du worktree, hors
    de l'index) ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - L-1 : la fabrique a un choix `ClasseDejaLa` : `Refuser` pour l'icône unique (une deuxième
      classe serait une erreur, comme avant), `Reprendre` pour une deuxième zone et les essais ;
      la fenêtre reste de style simple, demandée de taille nulle (pas `HWND_MESSAGE`, qui ne
      reçoit pas « TaskbarCreated ») ;
    - L-2 : `Apparence::appliquer` passe la palette au dessin (`dessin::apparence::appliquer`) :
      les réglages d'egui sont du dessin pur, la lecture de Windows reste dans les bords ;
    - L-3 : `bords` est compilé sur tous les systèmes (une deuxième déclaration
      `#[cfg(not(windows))] mod bords;` ajoutée dans `src/lib.rs`, sans toucher à la première) ;
      ses parties Windows seulement sous Windows (`apparence` et `polices` valent partout, comme
      avant) ; `echelle` et `systeme` n'ont plus de code `unsafe` (leur `allow(unsafe_code)` de
      `src/lib.rs` est laissé tel quel, pour n'y faire que des ajouts) ; `tests/etages.rs` ne
      tolère plus que `horloge.rs` et `zone.rs` ;
    - L-4 : `verification` va dans les bords (il lit les fichiers de polices), sans `unsafe` ;
    - L-5 : l'essai de la fabrique vérifie que la fenêtre n'est pas visible, pas sa taille :
      Windows donne sa taille minimale (136 × 39 ici) à une fenêtre de style simple ;
  - **reste** : `horloge.rs` et `zone.rs` gardent leur `unsafe` (blocs M et J) ;
  - **à savoir pour l'intégration** : conflits possibles si `refonte` a bougé dans `src/lib.rs`,
    `src/horloge.rs`, `src/zone.rs`, `src/dessin.rs`, `tests/etages.rs`, `docs/CARTE.md`,
    `docs/COMPORTEMENTS.md` ou `crates/ic-coeur/src/{theme,gestes/clics}.rs` ; à part, et déjà
    avant ce bloc : la crate principale ne se compile pas hors de Windows (`eframe` n'y est
    pas, `src/toutes_faites.rs`) ; la CI ne la compile que sous Windows ;
  - **lignes ajoutées aux fichiers partagés avec N et O** (seulement des ajouts) :
    `src/lib.rs` : le commentaire et `#[cfg(not(windows))] #[allow(unsafe_code)] mod bords;`,
    après le `mod bords;` de Windows ; `src/bords/mod.rs` : trois lignes de doc,
    `#[cfg(windows)]` devant `pub(crate) mod poste;`, puis `pub(crate) mod systeme;` et
    `pub(crate) mod verification;` ; `Cargo.toml` et `crates/ic-coeur/src/lib.rs` : rien ;
    `src/horloge.rs` : seulement `reglage_dpi` (repris des bords), la fenêtre faite par la
    fabrique dans `boucle`, et les `use` que cela retire (`GetModuleHandleW`,
    `SetProcessDpiAwarenessContext`, `CreateWindowExW`, `RegisterClassW`, `WNDCLASSW`,
    `WS_OVERLAPPED`…) : rustfmt a reformé le bloc `use …WindowsAndMessaging::{…}`, un conflit
    possible avec N s'il y retire aussi des noms (garder les deux retraits) ;
  - **`src/bords/verification.rs`** : ce bloc y a déplacé `src/verification.rs` (`git mv`,
    tête de doc raccourcie, `use crate::bords::systeme::polices`) et laissé à
    `src/verification.rs` un module mince ; si O le déplace aussi, garder une seule copie, avec
    les changements de O et l'appel `crate::bords::systeme::polices` ;
  - **sous-agents** : aucun.
- **Prochaine étape de la refonte** : blocs G et E intégrés (26 nov.) ; ensuite le bloc H, puis le bloc
- **Refonte, bloc H « Paramètres (dessin) et fenêtres toutes faites »** (26 nov. 07:35, branche
  `refonte-bloc-h-parametres`, partie de `refonte` à 61c66d4, à intégrer par l'Atelier Master ;
  **rien n'est commité** : tout est indexé, le message est dans `MESSAGE-COMMIT.txt`, à la
  racine, hors de l'index) :
  - **le dessin des Paramètres dans `src/dessin/`** : `src/parametres/mod.rs` déplacé
    (`git mv`) en `dessin/parametres.rs` (l'état gardé, `new`, `aller_a`, `suivre`, la mise en
    page navigation et page), puis découpé en un fichier par partie : `navigation` (en-tête,
    logo, recherche, liste des pages), `contexte` (le `Contexte` donné aux pages du logiciel),
    `gestes` (ce qui est rendu au logiciel), `bulles` (la bulle « Annuler », la carte
    « Conserver ? », le bandeau « Redémarrer »), `general`, `mise_a_jour`, `a_propos` ; le
    squelette dessine d'après l'état que tient le cœur (bloc E) et ne décide rien ;
  - **les fenêtres toutes faites dans `src/dessin/toutes_faites/`** : `simple.rs` (la fenêtre
    simple, son dessin découpé en en-tête, nouveautés, état, champ, choix, boutons),
    `prete.rs` (« <Logiciel> x.y.z est prête »), `lecture.rs` (la fenêtre de lecture : mots,
    surlignage, clic) ; elles se servent des contrôles du bloc F ; `src/toutes_faites.rs`
    (1 299 → 357 lignes) ne garde que les fenêtres de la boucle (poignée, rappel, ouverture) ;
  - **repris aux mêmes chemins** : `src/parametres/mod.rs` est un module mince (70 lignes) qui
    reprend `Parametres` et `Contexte` du dessin ; `Parametres::dessiner` garde sa signature, un
    logiciel peut l'appeler dans sa propre boucle ; `toutes_faites` reprend tout le dessin
    (`pub use`) ; `tests/compatibilite.rs` inchangé ; **aucun changement d'API**, version
    0.1.0 ; **aucune image de référence ne change** (`references` vert : une image par page
    commune et par état de Mise à jour, la fenêtre simple et « est prête », clair et sombre) ;
    fichiers partagés avec N et O (`src/lib.rs`, `Cargo.toml`, `crates/ic-coeur/src/lib.rs`) : aucune ligne changée ; `src/bords/` non plus ;
  - **essais** : les essais des fenêtres toutes faites passent avec leur dessin (sans fenêtre) ;
    nouveau : `tests/comportements::c68_la_recherche_ouvre_la_section` (le dessin ouvre la
    section trouvée par la recherche, pas `aller_a` seul, et une nouvelle ouverture repart
    repliée) ; `tests/polices.rs` lit les 13 nouveaux fichiers ; `COMPORTEMENTS.md` (C-68,
    C-84 à C-89) et `CARTE.md` suivent ;
  - **résultats sur le PC** (`CARGO_BUILD_JOBS=2`, `--test-threads=2`) : `ic-coeur` 159 essais
    et 1 exemple ; `interface-commune` 129 essais internes, `carte_info`, `compatibilite`,
    `comportements` (13), `conserver`, `couleurs`, `entrees_fabriquees`, `etages`, `licences`,
    `polices`, `polices_windows10`, `references`, `sans_fenetre`, `zone_affichage` verts,
    8 exemples de documentation ; `fmt`, `clippy -D warnings`, `cargo deny` propres ; aucun
    fichier de plus de 800 lignes ni fonction de plus de 80 lignes dans ce qui a changé
    (la plus longue : `dessiner_lecture`, 74) ; contrôle de la règle n° 1 (`--dossier .`) :
    seulement `.git:1` (le fichier du worktree, jamais envoyé) ;
  - **une erreur de ma part** : une fois, au milieu du bloc, j'ai lancé tous les essais de
    `interface-commune`, donc aussi `premiere_creation`, `recreation_apres_panique` et
    `fenetres_en_rafale`, qui créent de vraies fenêtres (masquées ou hors de l'écran, jamais
    montrées) ; ensuite, je les ai laissés de côté. Ils restent à faire tourner sur GitHub ;
  - **Décisions prises seul** (à relire par l'Atelier Master) :
    - H-1 : le dessin ne lit pas Windows. Le module mince `crate::parametres` lit les « effets
      d'animation » (`systeme::effets_d_animation`) une fois par image et appelle
      `dessiner_avec` ; avant, ils étaient lus une fois par section repliable : même
      comportement. Au bloc J, la boucle pourra donner la valeur elle-même ;
    - H-2 : les aides des essais des fenêtres toutes faites sont partagées dans
      `dessin/toutes_faites/essais.rs` (une image sans fenêtre, un clic, Entrée, une fenêtre
      simple inventée) ; l'essai de la boucle (C-89) reste dans `src/toutes_faites.rs` ;
    - H-3 : un dossier de compilation à part sous `%LOCALAPPDATA%`, hors de la copie ;
  - **Reste pour J** : la fenêtre des Paramètres (`src/parametres/fenetre.rs`) et les fenêtres
    de la boucle (`src/toutes_faites.rs` : `ouvrir_simple`, `ouvrir_lecture`) partent dans les
    bords ;
  - **À savoir pour l'intégration** : `refonte` a bougé depuis le départ (bloc N, c9c0394) dans
    `docs/ETAT.md`, `docs/CARTE.md`, `docs/COMPORTEMENTS.md`, `src/bords/` et `src/horloge.rs` :
    conflits possibles seulement dans les trois documents ;
  - **sous-agents** : aucun.
- **Prochaine étape de la refonte** : blocs G et E intégrés (26 nov.) ; bloc H prêt à intégrer (26 nov.) ; ensuite le bloc
  suivant du `docs/PLAN-REFONTE.md`, §2 (une seule voie à la fois).
- **Prochaine étape sur `main`** : fusionner `manques-projets`, puis passer KVMSoft, Vocalis et PowerPeek
  (`docs/MIGRATION.md`).
