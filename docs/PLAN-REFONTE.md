# Interface commune — le plan de la refonte à part (A)

| Projet | Date | Version du plan | Décidé par |
|---|---|---|---|
| Interface commune | 2026-11-25 | 1 | le propriétaire a choisi A le 24 novembre ; ce plan est à valider par l'Atelier Master |

**Ce que dit ce plan** : dans quel ordre écrire les quinze blocs de `docs/ORGANIGRAMME.md`, combien
de jours chacun coûte, ce qui avance en parallèle, ce qui change pour les quatre logiciels et
comment ils passent à la nouvelle version, et à quelles conditions on bascule.

**Ce qu'il ne dit pas** : la conception (c'est `ORGANIGRAMME.md`), les comportements à garder
(c'est `COMPORTEMENTS.md`), les changements déjà faits sur `main` (c'est `MIGRATION.md`).

---

## 1. Les règles de la refonte

1. **Tout se fait sur la branche `refonte`.** `main` ne reçoit que des corrections pendant la
   refonte (méthode, §5) ; l'ancienne bibliothèque reste utilisable et installable jusqu'à la
   version suivante. PowerPeek et KVMSoft épinglent leur `rev` : rien ne bouge pour eux tant
   qu'ils ne changent pas eux-mêmes leur `rev`.
2. **Chaque bloc est une session de travail à part**, sur sa propre branche partie de `refonte`,
   intégrée par l'Atelier Master, qui tient ce dépôt. Deux sessions en même temps au plus (les deux
   voies du §3), jamais sur les mêmes fichiers.
3. **Avant le premier bloc**, l'Atelier fusionne la branche des filets (`claude/filets-interface`)
   dans `main`, puis `main` dans `refonte` : les essais de compatibilité, les images de référence
   et `tests/comportements.rs` sont alors sur `refonte` et doivent passer à chaque commit.
4. **Même rendu** : chaque bloc de dessin garde les images de référence telles qu'elles sont. Un
   pixel ne bouge que par une décision écrite au §5 ; les images touchées se refont alors dans le
   même commit (`REFERENCES=refaire`), et le changement s'écrit dans `MIGRATION.md`.
5. **Même API** : `tests/compatibilite.rs` compile à chaque commit. Seuls les changements du §4
   sont permis.
6. **Le numéro de version ne bouge pas** pendant la refonte (0.1.0). L'Atelier choisit celui de la
   bascule.
7. **Sur le PC du propriétaire** : `CARGO_BUILD_JOBS=2`, `--test-threads=2`, aucune fenêtre
   pendant les essais (les essais en vraies fenêtres tournent sur GitHub), `controle-prive
   --dossier` à 0 avant chaque envoi.
8. **La fin de chaque bloc** : ses tests passent, les comportements de son bloc (ORGANIGRAMME
   §13) passent, clippy et fmt sont propres, aucun fichier de plus de 800 lignes, aucune fonction
   de plus de 80 lignes, `docs/ETAT.md` mis à jour.

## 2. Les blocs dans l'ordre, et leur coût

Un jour = un jour de travail d'une session. Les coûts viennent de la taille du code d'aujourd'hui
(`ORGANIGRAMME.md` §10.1) et de son risque (`ANALYSE.md`).

| Étape | Bloc | Contenu | Demande d'abord | Coût (jours) |
|---|---|---|---|---|
| 0 | Préparation | espace de travail Cargo, `crates/ic-coeur` vide, `tests/etages.rs`, CI des deux crates | — | 1 |
| 1 | A Contrat | types partagés dans `ic-coeur`, repris à la racine | Préparation | 1 |
| 2 | B Textes | catalogue, `lisible`, espace insécable (C-98) | A | 1 |
| 2 | C Thème, mesures | palette, pastilles, animation, relecture à 2 s | A | 1,5 |
| 3 | D Gestes, placement, secours | aiguilleur unique, placement, pertes, info-bulle | B, C | 3 |
| 3 | E Modèle des Paramètres | pages, recherche, « Annuler » (C-70), « Conserver ? », Mise à jour | B, C | 2,5 |
| 4 | F Contrôles, traits | contrôles communs, briques, traits `Reglages` et `Logiciel` | B, C | 5 |
| 5 | G Volet, menu | dessin du volet et du petit menu | F, D | 3 |
| 5 | H Paramètres, toutes faites | squelette des Paramètres, fenêtres toutes faites | F, E | 5 |
| 6 | I Captures | captures sans fenêtre, image de la fenêtre de lecture | G, H | 1 |
| 3 | L Système | lectures de Windows, polices, DWM, fenêtre cachée unique | C | 2 |
| 2 | K Guichet | `Poste` et ses messages | A | 1 |
| 3 | N Instance unique | verrou, arguments, `Reveil` | K | 1 |
| 4 | O Sonde, journal | sonde, délais, journal, vérification | D | 1,5 |
| 5 | M Icône | `Zone` seule, `Horloge` en façade, secours, notifications | D, K, L | 5 |
| 6 | J Boucles des fenêtres | les deux boucles, faux moteur, paniques par fenêtre | K, G, H | 5 |
| — | Transversal | `verrou` commun, états globaux rangés, README, CARTE | M | 1 |
| 7 | Bascule | §6 et §7 | tous | 4 |
| | **Total** | | | **44,5** |

Le détail de la bascule : vérification complète 1,5 ; essai du retour en arrière 0,5 ; KVMSoft 1 ;
PowerPeek 1 ; Vocalis 1 ; Chat PDC 0,5.

Les blocs les plus risqués sont **J** (les boucles, les paniques, la recréation) et **M** (deux
systèmes d'icône à fondre en un) : ce sont eux qui portent la marge du §3.

## 3. Ce qui avance en parallèle, et les dates

Deux voies, une session par voie. Jours ouvrés ; pause du 24 décembre au 1er janvier.

| Voie 1 : cœur, puis dessin | Dates | Voie 2 : cœur, puis bords | Dates |
|---|---|---|---|
| Préparation | 26 nov. | | |
| A Contrat | 27 nov. | | |
| B Textes | 30 nov. | C Thème, mesures | 30 nov. – 1er déc. |
| D Gestes, placement, secours | 1er – 3 déc. | E Modèle des Paramètres | 1er – 3 déc. |
| F Contrôles, traits | 4 – 10 déc. | L Système | 4 – 7 déc. |
| | | K Guichet | 8 déc. |
| | | N Instance unique | 9 déc. |
| G Volet, menu | 11 – 15 déc. | O Sonde, journal | 10 – 11 déc. |
| H Paramètres, toutes faites | 16 – 22 déc. | M Icône | 11 – 18 déc. |
| | | Transversal | 21 déc. |
| I Captures | 23 déc. | J Boucles des fenêtres | 22 déc. – 6 janv. |
| | | | |
| **Bascule** | | | |
| vérification complète, retour en arrière | 7 – 8 janv. | | |
| migration des quatre logiciels | 11 – 14 janv. | | |
| essai du propriétaire | semaine du 18 janv. | | |

- **J** commence le 22 décembre par ce qui ne demande pas H (le faux moteur, la boucle de la
  bibliothèque, la supervision) ; la façon 2 (Paramètres dans la boucle du logiciel) attend H,
  fini le 22.
- Les deux voies ne touchent pas les mêmes fichiers : la voie 1 écrit `crates/ic-coeur/src/{contrat,
  textes, gestes, placement}` puis `src/dessin/` ; la voie 2 écrit `crates/ic-coeur/src/{theme,
  parametres}` puis `src/bords/`. Le fichier `crates/ic-coeur/src/lib.rs` (la liste des modules)
  est écrit par la Préparation, une fois pour toutes.
- **Fin du travail : vers le 22 janvier 2027.** Avec 20 % de marge pour le risque de J et M :
  **fin estimée vers le 5 février 2027.**
- **Si le forfait dépasse 80 %** (fiche « Forfait ») : une seule voie, dans l'ordre de la voie 1
  puis de la voie 2 ; la fin glisse vers la fin février 2027.

> **Décision de l'Atelier Master, 25 nov. 2026** : plan et organigramme (version 2) validés.
> **Une seule voie à la fois** pour commencer : le forfait de la semaine est déjà à 38 % après
> un jour, et Outils communs se refait en même temps. On suit l'ordre de la voie 1, puis
> celui de la voie 2 ; la fin glisse vers la fin février 2027. Une deuxième voie s'ouvre quand
> le forfait le permet (fiche « Forfait »). La Préparation commence le 26 nov.

## 4. Les changements d'API, et la migration de chaque logiciel

Ce que prend chaque logiciel aujourd'hui (`ANALYSE.md` §6) :
- **KVMSoft** épingle `rev = "e077572"` ;
- **PowerPeek** : sa refonte épingle `rev = "e077572"` (dans `crates/bords-windows`) ; son
  `main`, gelé, épingle encore `rev = "119c500"` (relevé sur le disque le 25 nov.) ;
  il garde sa propre boucle eframe et y branche les fenêtres de la bibliothèque
  (`AutresFenetres::lier`, `montrer`) : façon 2 du bloc J ;
- **Vocalis** épingle `rev = "119c500"` ;
- **Chat PDC** épingle `rev = "77bad2b"` et ne prend que `horloge::Horloge` et
  `parametres::modele`.

**Première étape, la même pour tous** : rattraper `main` d'après `MIGRATION.md` (de son `rev` au
dernier commit de `main`), puis passer au commit de la bascule. Les deux marches sont séparées :
un logiciel qui a rattrapé `main` sait que ce qui casse ensuite vient de la refonte.

**Les changements de la refonte** (tous les autres chemins et signatures restent, essai de
compatibilité à l'appui) :

| # | Changement | Pourquoi | KVMSoft | PowerPeek | Vocalis | Chat PDC |
|---|---|---|---|---|---|---|
| 1 | Les éléments publics qu'aucun des quatre ne prend (213, `ANALYSE.md` §1.2) deviennent internes (`pub(crate)`) ou disparaissent | une API plus petite à garder ; le relevé est **refait au moment de la bascule** sur le code des quatre | rien | rien | rien | rien |
| 2 | Les méthodes `Logiciel::dessiner_rapides` et `Logiciel::largeur_rapides` sont retirées | aucun logiciel ne les écrit (relevé du 25 nov. dans le code des quatre) ; leur place dans le volet est prise par les briques | rien (ne les écrit pas) | rien (ne les écrit pas) | rien (ne les écrit pas) | sans objet (pas de `Logiciel`) |
| 3 | La constante `theme::HAUTEUR_LIGNE` est retirée | aucun logiciel ne la prend ; une seule mesure de ligne reste (`mesures`) | rien | rien | rien | rien |
| 4 | Le texte `conserver_retour` (français et anglais) prend l'espace insécable (C-98), ainsi que « 24 h » dans `journal_detaille_explication` (trouvé au bloc B) | règle de l'unité, sans exception à la bascule | rien à écrire ; le texte affiché change d'une espace | idem | idem | idem |
| 5 | `briques::pourcent` écrit `42 %` avec l'espace insécable (C-98) | idem | rien à écrire ; si un test du logiciel compare ce texte, y mettre `\u{A0}` | idem | idem | sans objet |

Le doublon interne de `effets_d_animation` (dans `parametres`) disparaît au profit de
`systeme::effets_d_animation` : il n'était pas public, aucun logiciel n'a rien à changer.

Ce qui **ne change pas**, dit parce qu'on aurait pu le changer :
- les types d'erreur de `lancer` (`Result<(), String>`) et d'`instance_unique`
  (`Result<Unique, bool>`) ;
- les traits `Reglages` et `Logiciel`, leurs méthodes de dessin comprises, à leur chemin ;
- `Brique` et ses variantes ;
- les signatures de `menu_choix::placer` et de `pastilles::plan_des_cases` (rectangles d'egui) ;
- `Horloge`, `Reveil`, `Poste`, `Niveau`, `MessageWindows`, `FinDeSession`, `FenetresCommunes`,
  `guid_stable` ;
- **le GUID de l'icône et son identifiant** : Windows garde la place que l'utilisateur a choisie
  pour l'icône (visible ou dans le menu caché) ;
- `Place` et ses champs : les fichiers des logiciels n'ont rien à migrer ;
- **les rangs 0 à 9 des couleurs** (`couleurs::palette`) : KVMSoft envoie le rang à un autre PC,
  c'est un format (essai `les_rangs_des_couleurs_restent`) ;
- les constantes `theme::ARRONDI_*` et `theme::TAILLE_*`, avec leur type et leur valeur ;
- `systeme::disque_local`, parade de sécurité du sélecteur de dossier (C-111) ;
- `Horloge::new` et `sonde::regler` sans attente : les tests de KVMSoft en dépendent ;
- **la version d'egui (0.37)** : PowerPeek la prend en direct, elle fait partie de l'API ; un
  changement d'egui se traite comme un changement d'API, avec sa ligne dans ce tableau et la
  migration de chaque logiciel (essai `la_version_d_egui_reste`) ;
- la ligne `interface-commune = { git = …, rev = … }` du `Cargo.toml` : seul `rev` change.

Après la bascule, toute suppression d'un élément public qui sert passe par une étape dépréciée
(`ORGANIGRAMME.md`, principe 1).

## 5. Les décisions de rendu

La règle : **même rendu, mêmes images de référence.** Les seuls écarts permis sont ceux-ci, chacun
avec la liste des images qu'il touche :

| # | Écart | Décidé par | Images touchées |
|---|---|---|---|
| R1 | C-98 : l'espace insécable dans `conserver_retour` | `ORGANIGRAMME.md`, bloc B (règle sans exception) | la carte « Conserver ces modifications ? » (`parametres-conserver-clair`), si l'espace change la coupure ; « 24 h » : la carte du journal détaillé (page Général, si le logiciel la montre). Fait au bloc B (25 nov.) : `references` passe, aucune image ne change (même coupure, écart sous la tolérance) |
| R2 | C-98 : l'espace insécable dans `briques::pourcent` | idem | `volet-*` (la ligne de la souris inventée, à 42 %), si un pixel bouge |

Tout autre écart, même d'un pixel au-delà de la tolérance (`ECART_PERMIS` = 6 par canal, au plus
1 pixel sur 5 000), fait échouer l'essai. La session qui le trouve ne refait pas l'image : elle
corrige son dessin, ou elle écrit une ligne de plus dans ce tableau et la fait valider par
l'Atelier Master.

## 6. Les conditions de bascule (méthode, §5, étape 4)

On bascule les logiciels sur la nouvelle version seulement quand **toutes** ces conditions sont
remplies, chacune vérifiée et notée dans `docs/ETAT.md` :

1. **Chaque comportement passe** dans la nouvelle version : les 116 lignes de `COMPORTEMENTS.md`,
   chacune par son essai (110 automatiques sur le PC sans fenêtre, 4 en vérification GitHub, 2 à
   la main par le propriétaire) ; les images de référence passent, aux seuls écarts du §5 près ;
   `tests/compatibilite.rs` compile.
2. **Les quatre logiciels compilent et passent leurs propres tests** sur une branche à eux qui
   pointe sur la refonte, avant tout changement de `rev` sur leur `main`. Pour **PowerPeek**, cela
   comprend **ses images de référence** : il compare ses pages au pixel avant chaque dépôt, et
   elles passent sans être refaites (aux seuls écarts du §5 près, chacun montré à son Atelier).
3. **La reprise des réglages et des données** : sans objet pour la bibliothèque, qui n'écrit
   aucun fichier (principe 4) ; vérifié pour chaque logiciel par ses propres tests sur une copie
   inventée de chacun de ses formats (place retenue, réglages), comme sa fiche le demande.
4. **Le retour à l'ancienne version est essayé** : remettre l'ancien `rev` dans un logiciel, le
   compiler, le lancer ; ses fichiers écrits par la nouvelle version se relisent (aucun format
   n'a changé).
5. **L'essai du propriétaire** : il utilise chaque logiciel une semaine sur la nouvelle version
   (volet, petit menu, Paramètres, notifications, fermeture de session), et rien ne manque.
6. **L'ancienne version** n'a reçu que des corrections pendant la refonte, chacune reportée sur
   `refonte` ; elle reste installable jusqu'à la version suivante de chaque logiciel.

L'ordre de bascule des logiciels : **Chat PDC** d'abord (il prend le moins), puis **Vocalis**, puis
**KVMSoft**, puis **PowerPeek** (façon 2, la plus exposée). Un logiciel qui échoue garde l'ancien
`rev` ; les autres continuent.

## 7. Le retour en arrière

- Pendant la refonte : `main` est intacte, rien à faire.
- À la bascule : chaque logiciel peut remettre son ancien `rev` à tout moment (essai du §6,
  point 4) ; aucune donnée n'est à reconvertir.
- Après la bascule : `refonte` est fusionnée dans `main` par l'Atelier, l'ancien code reste
  lisible dans l'histoire de Git ; le dernier commit de l'ancienne bibliothèque est noté dans
  `MIGRATION.md`.

## 8. Les sessions à lancer

Chaque ligne du §2 devient une session, briefée par l'Atelier Master avec : son
bloc (ORGANIGRAMME §3), ses comportements (§13), ses fichiers (§10.1), ses limites (§1 de ce
plan), et le rappel « aucune fenêtre sur le PC ». La première est la **Préparation**, le
26 novembre ; ensuite deux à la fois au plus, selon le §3.
