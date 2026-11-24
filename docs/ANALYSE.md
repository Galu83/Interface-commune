# Interface commune — l'analyse du code (étape 2 de la refonte « code propre »)

24 nov. 2026. Ce document compare le code d'aujourd'hui à l'organisation idéale (`IDEAL.md`,
écrite avant de lire le code), bloc par bloc. Il s'appuie sur la liste des comportements à
garder (`COMPORTEMENTS.md`). Il finit par une recommandation : **A**, une refonte à part, ou
**B**, un nettoyage sur place.

**Réponse courte : B, un nettoyage sur place, par étapes. Fin estimée vers le 15 décembre
2026.** Les raisons sont au § 5.

## 0. Comment lire ce document

- **Verdict** : *tel quel* (on n'y touche pas, ou presque), *adapter* (on range, on découpe,
  sans changer ce que ça fait), *réécrire* (on refait l'intérieur en gardant ce qui se voit du
  dehors).
- **Coût** : en jours de travail d'une session (une journée de développeur, à peu près).
- **Risque** : la chance de casser un comportement de `COMPORTEMENTS.md`, et la gravité si ça
  arrive.
- **Rendu ?** : est-ce que ce qu'on voit à l'écran change ? **API ?** : est-ce qu'un logiciel
  doit changer son code ? Pour chaque « oui », on dit comment migrer.
- **Les logiciels qui dépendent de la bibliothèque** : KVMSoft, PowerPeek (refonte en cours),
  Vocalis et Chat PDC. Chacun épingle une version précise (`rev`). Rien ne change pour eux tant
  qu'ils ne montent pas de version.

## 1. Les chiffres

Mesurés sur le code de `main` au 24 nov. 2026 : 54 fichiers Rust, 27 679 lignes : 22 194 dans
`src/` (33 fichiers, dont environ 5 000 lignes de modules de tests), 4 031 dans `tests/`
(12 fichiers) et 1 454 dans `examples/` (9 fichiers).

### 1.1 Ce qui va bien

| Mesure | Résultat |
|---|---|
| `cargo fmt --check` | propre |
| `cargo clippy --all-targets` | **0 avertissement** |
| `unwrap`, `expect`, `panic!`, `todo!` hors tests | **0** (interdits par `Cargo.toml`) |
| `unsafe` | 110 blocs, environ 105 avec leur commentaire `SAFETY`. Le `#[allow(unsafe_code)]` est limité à 4 modules : `echelle`, `horloge`, `systeme`, `zone` |
| Tests sans fenêtre | **254 tests, 0 échec** (`--lib` et 8 essais de `tests/`) |
| Code mort | **2 éléments** : `surface::dessiner_volet` et `theme::HAUTEUR_LIGNE`. Aucun usage, ni dans la bibliothèque, ni dans ses tests, ni dans les quatre logiciels |
| Cœur déjà sans Windows | `clics`, `menu`, `place`, `parametres::modele`, `textes`, `touches`, `theme`, `mesures` et la partie « décision » de `fenetres` ne touchent pas Windows, et sont testés |

### 1.2 Les défauts

| Défaut | Combien | Où |
|---|---|---|
| **Fichiers au-dessus de 800 lignes** (seuil) | **13** dans `src/`, plus 1 dans `tests/` | voir § 1.3 |
| … dont le code seul (hors tests) dépasse 800 | 6 | `horloge` 1 306, `parametres/mod` 1 317, `zone` 1 155, `volet` 1 135, `fenetres` 988, `toutes_faites` 965 |
| **Fichier de tests au-dessus de 1 200 lignes** | 1 | `tests/sans_fenetre.rs` : 1 701 |
| **Fonctions de plus de 80 lignes**, hors tests | **17** | `briques::brique` 173, `parametres::page_general` 161, `toutes_faites::dessiner_simple` 158, `seuils::barre_seuils` 158, `briques::appareil` 156, `fenetres::boucle_des_fenetres` 155, `horloge::procedure` 126, `parametres::page_mise_a_jour` 123, `volet::montree_image` 122, `pastilles::dessiner_choix` 121, `surface::volet` 105, `menu_choix::lignes_du_menu` 105, `menu_choix::afficher` 97, `parametres::navigation` 90, `horloge::boucle` 90, `volet::mettre_en_place` 83, `pastilles::pastille` 83 |
| … dans les tests et les exemples | 5 | `entrees_fabriquees` 190, `zone::essais` 93, `textes::essais` 87, exemple `volet_pastilles` 98, exemple `mesure_ouverture` 93 |
| **Blocs `impl` géants** | 3 | `impl Parametres` 857 lignes (mod.rs:274-1131), `impl Volet` 500 lignes (volet.rs:391-894), `impl Principal` 424 lignes (horloge.rs:792-1216) |
| **État global** sans le commentaire `// ÉTAT GLOBAL : <pourquoi>` | **environ 30** `static` / `thread_local`, **aucun** commenté | `sonde` 11, `autres_fenetres` 3 (+ 4 en tests), `volet` 4, `polices` 2, `horloge` 1, `parametres` 1, `echelle` 1 |
| **`#[allow]` sans commentaire** | 7 | 6 `too_many_arguments` (`briques.rs:288`, `couleurs.rs:139`, `dessin.rs:95` et `:519`, `pastilles.rs:216` et `:550`), 1 `result_unit_err` (`fenetres.rs:362`) |
| **Doublons** | **6** | voir § 1.4 |
| **Comportements sans essai** | **15** sur 116 | 9 sans aucun essai, 6 en partie (`COMPORTEMENTS.md`, bilan) |
| **Fichiers sans aucun test interne** | 9 | `apparence`, `couleurs`, `echelle`, `icones`, `lib`, `mesures`, `polices`, `surface`, `parametres/fenetre`. Plusieurs sont couverts par les essais de `tests/` |
| **Surface publique plus large que l'usage** | 213 sur 680 éléments publics ne servent à aucun des quatre logiciels | surtout `mesures` (45), `fenetres` (20), `parametres` (32), `place` (12). **Ce n'est pas du code mort** : la bibliothèque s'en sert elle-même |
| Fichier `.seuils-acquis` | absent | il faut le créer pour les gros fichiers gardés en attendant leur découpe |

### 1.3 La taille des fichiers

| Fichier | Total | Code | Tests dans le fichier |
|---|---|---|---|
| `fenetres.rs` | 1 630 | 988 | 642 |
| `zone.rs` | 1 618 | 1 155 | 463 |
| `horloge.rs` | 1 509 | 1 306 | 203 |
| `autres_fenetres.rs` | 1 501 | 610 | 891 |
| `parametres/mod.rs` | 1 391 | 1 317 | 74 |
| `toutes_faites.rs` | 1 300 | 965 | 335 |
| `volet.rs` | 1 269 | 1 135 | 134 |
| `sonde.rs` | 1 028 | 470 | 558 |
| `pastilles.rs` | 1 006 | 672 | 334 |
| `menu_choix.rs` | 923 | 612 | 311 |
| `parametres/modele.rs` | 880 | 543 | 337 |
| `dessin.rs` | 860 | 686 | 174 |
| `systeme.rs` | 803 | 641 | 162 |

Pour 7 de ces 13 fichiers, il suffit de **sortir les tests** dans un fichier à part
(`src/<module>/essais.rs` ou `tests/`) pour passer sous 800 lignes. Le code ne change pas.
Restent 6 fichiers où il faut vraiment découper le code.

### 1.4 Les doublons

| Doublon | Où | Remède |
|---|---|---|
| **Le routage d'un geste vers sa suite**, écrit deux fois : clic reçu → `clics::suite` → Paramètres, fenêtre principale, volet ou menu → `envoyer_le_clic` → rien, envoyé ou secours | `horloge::Principal::geste`, `ouvrir`, `ouvrir_fenetre_principale` (horloge.rs:903-970) et `zone::Aiguilleur::recevoir` (zone.rs:973-1045) | un seul aiguilleur, que `horloge` emprunte à `zone` |
| **Deux systèmes d'icône**, chacun avec sa fenêtre cachée (`RegisterClassW` et `CreateWindowExW` écrits trois fois, avec `systeme.rs:674`) et son menu de secours (`horloge::menu_de_secours` contre `zone::menu_windows`) | `horloge.rs`, `zone.rs`, `systeme.rs` | `Horloge` devient une `Zone` à une icône (bloc M). `horloge` reprend déjà `zone::icone_windows`, `copier` et `ajouter_avec_repli` : la moitié du chemin est faite |
| `effets_d_animation` | `parametres/mod.rs:222` (enveloppe) et `systeme.rs:311` | garder celle de `systeme` |
| `fn verrou` (lire un `Mutex` empoisonné) | `autres_fenetres.rs:615`, `toutes_faites.rs:43`, plus 18 traitements écrits à la main | une seule fonction partagée |
| `sur_une_ligne` et `coupe` (presque les mêmes : `max(0.0)` contre `max(1.0)`) | `briques.rs:281`, `pastilles.rs:542` | une seule, avec la borne en paramètre |
| `titre_section` | `dessin.rs:434`, `pastilles.rs:204` | garder celle de `dessin` |

## 2. Bloc par bloc

Les lettres renvoient aux blocs de `IDEAL.md` (§ 3).

| Bloc | Ce qui existe | Écart avec l'idéal | Verdict | Coût | Risque | Rendu ? | API ? |
|---|---|---|---|---|---|---|---|
| **A. Contrat** | `lib.rs` (447) : `Reglages`, `Logiciel`, `Identite`, `Etat`, `Communs`, `cles` | Les traits prennent `egui::Ui` (`dessiner_page`…) : l'idéal les met dans un prolongement de trait côté dessin. `dessiner_rapides` et `largeur_rapides` sont dits « passage seulement », mais pas marqués `#[deprecated]` | **adapter** : marquer `#[deprecated]` ce qui doit partir ; **ne pas** sortir les méthodes de dessin des traits maintenant | 0,5 | faible | non | non (un avertissement de dépréciation seulement ; aucun logiciel n'utilise `dessiner_rapides`) |
| **B. Textes** | `textes` (423), `touches` (302) | aucun | **tel quel** | 0 | — | non | non |
| **C. Thème et mesures** | `theme` (415), `mesures` (101), `couleurs` (180), palette des pastilles | `effets_d_animation` en double ; `HAUTEUR_LIGNE` mort ; `mesures` sans test | **tel quel** (retirer le doublon et le mort) | 0,25 | faible | non | `HAUTEUR_LIGNE` disparaît : personne ne s'en sert |
| **D. Gestes et placement** | `clics` (202), `menu` (202), `place` (790), le calcul du plan dans `pastilles`, l'anti-rebond dans `fenetres` | Le plan des écrans (pur) est mêlé au dessin dans `pastilles.rs` ; l'anti-rebond et `decider_le_clic` vivent dans `fenetres.rs` | **adapter** : sortir le plan des pastilles dans `pastilles/plan.rs` ; laisser la décision du clic dans `fenetres` (elle y est pure et testée) | 0,5 | faible | non | non (`pub use` aux anciens chemins) |
| **E. Modèle des Paramètres** | `parametres/modele.rs` (543 de code, pur) | `presentation_mise_a_jour`, `TonMiseAJour`, `ligne_grise_mise_a_jour` et `texte_mise_a_jour` (des décisions pures) sont dans `parametres/mod.rs` | **adapter** : les déplacer dans le modèle ; sortir les tests du modèle | 0,5 | faible | non | non (`pub use` dans `parametres`) |
| **F. Contrôles communs** | `dessin` (860), `briques` (781), `pastilles` (1 006), `menu_choix` (923), `seuils` (527), `nuancier` (228), `icones` (92) | 7 fonctions trop longues (`brique` 173, `appareil` 156, `barre_seuils` 158, `dessiner_choix` 121, `lignes_du_menu` 105, `afficher` 97, `pastille` 83) ; 6 `too_many_arguments` sans commentaire ; 2 doublons | **adapter** : découper les fonctions en étapes (mesurer, placer, peindre, répondre) ; commenter les `#[allow]` ; sortir les tests | 3 | **moyen** : un pixel qui bouge se voit dans PowerPeek et KVMSoft | **ne doit pas changer** : vérifié par les images de référence (§ 4, étape 0) | non : les fonctions publiques gardent leur signature ; les découpes restent internes |
| **G. Volet et petit menu (dessin)** | `surface` (418), `volet` (1 269) | `volet.rs` mêle **le dessin** (`impl Volet`, 500 lignes), **la fenêtre** (`impl eframe::App`, `voilee`, `veiller`, l'animation) et **4 `static`** (`OBSERVATEUR`, `DISCRET`, `DEJA_OUVERTE`, `DEJA_PREPAREE`). `montree_image` 122 et `mettre_en_place` 83 lignes | **adapter** : couper en `volet/dessin.rs`, `volet/fenetre.rs`, `volet/veille.rs` ; commenter l'état global | 2 | **moyen** : le minutage de l'animation et de la première image (C-30, C-31) est délicat | non | non (`volet::…` garde ses noms par `pub use`) |
| **H. Paramètres (dessin) et fenêtres toutes faites** | `parametres/mod.rs` (1 391), `parametres/fenetre.rs` (341), `toutes_faites.rs` (1 300) | `impl Parametres` fait 857 lignes : toutes les pages communes, la navigation, l'en-tête, la bulle et la carte « Conserver ». `page_general` 161, `page_mise_a_jour` 123, `navigation` 90. `toutes_faites` réunit deux fenêtres sans lien (simple et lecture) | **adapter** : un fichier par page (`parametres/general.rs`, `mise_a_jour.rs`, `a_propos.rs`, `navigation.rs`, `entete.rs`), avec le même `struct Parametres` et plusieurs `impl` ; `toutes_faites/simple.rs` et `toutes_faites/lecture.rs` | 3 | **moyen** : ce sont les écrans les plus vus | **ne doit pas changer** (images de référence de chaque page) | non : `Parametres`, `Contexte`, `TAILLE`, `fenetre` restent où ils sont |
| **I. Captures** | `capture` (495) | aucun | **tel quel** ; elle sert de base aux images de référence | 0 | — | non | non |
| **J. Boucle des fenêtres** | `fenetres` (1 630), `autres_fenetres` (1 501), une partie de `horloge` (`Fenetres`, `fil_des_fenetres`, secours) | Le code de `fenetres` est déjà pur et bien testé (19 tests), mais `boucle_des_fenetres` fait 155 lignes ; les tests (642 et 891 lignes) gonflent les fichiers ; le lancement du fil vit dans `horloge.rs` | **adapter** : sortir les tests ; couper `boucle_des_fenetres` en étapes ; déplacer `fil_des_fenetres` et `Fenetres` de `horloge.rs` vers `fenetres/fil.rs` | 2 | **élevé** : c'est la partie la plus fragile (pannes passées : rafales, recréation, fil bloqué). Elle est couverte surtout par des tests GitHub en vraies fenêtres | non | non (`fenetres::envoyer_le_clic`, `Demande` et `autres_fenetres::Ouverture` gardés pour PowerPeek et KVMSoft) |
| **K. Guichet (`Poste`)** | dans `horloge.rs` (378-480) | il n'a pas de fichier à lui | **adapter** : `poste.rs`, `pub use` dans `horloge` | 0,5 | faible | non | non |
| **L. Système Windows** | `systeme` (803), `apparence` (157), `polices` (152), `echelle` (91), `verification` (165) | une troisième fenêtre cachée (`systeme.rs:674`) ; `apparence`, `echelle` et `polices` sans test interne | **adapter** : une seule fabrique de fenêtre cachée ; sortir les tests de `systeme` | 1 | moyen | non | non |
| **M. Icône et fenêtre cachée** | `horloge` (1 509), `zone` (1 618) | **Deux systèmes pour la même chose** (§ 1.4) : deux fenêtres cachées, deux procédures de messages (`horloge::procedure` 126 lignes), deux menus de secours, deux aiguilleurs. Les deux fichiers dépassent 1 100 lignes de code | **réécrire l'intérieur** : `Horloge` devient une façade sur une `Zone` à une seule icône ; un seul aiguilleur, une seule fenêtre cachée, un seul menu de secours. Les types publics (`Horloge`, `Reveil`, `Poste`, `Niveau`, `MessageWindows`, `FinDeSession`, `FenetresCommunes`, `guid_stable`) restent | 4 | **élevé** : messages de Windows, redémarrage de l'Explorateur (C-06, vérifié seulement à la main), notifications | **non**, à une condition : le menu de secours garde les lignes et l'ordre d'aujourd'hui (C-45) | **non** pour KVMSoft, Vocalis et Chat PDC (`Horloge` garde ses méthodes). Le GUID de l'icône seule doit rester le même, sinon Windows oublie la place de l'icône choisie par l'utilisateur : garder l'identifiant actuel (ID 1) ou le migrer une seule fois, en l'annonçant dans `MIGRATION.md` |
| **N. Instance unique** | dans `horloge.rs` (121-287) | elle n'a pas de fichier à elle | **adapter** : `unique.rs`, avec `pub use` de `instance_unique_avec` et `Unique` | 0,5 | faible | non | non |
| **O. Sonde et vérification** | `sonde` (1 028, dont 470 de code), `verification` (165) | 11 `static` sans commentaire ; 558 lignes de tests dans le fichier | **tel quel**, sauf : commenter l'état global, sortir les tests | 0,5 | faible | non | non |

**Total des blocs : 18,25 jours.**

## 3. Le travail transversal

| Travail | Pourquoi | Coût | Rendu ? | API ? |
|---|---|---|---|---|
| **Images de référence** (C-115) : chaque capture de `captures_pour_la_documentation`, plus une par page commune et par état de la page Mise à jour, comparée pixel par pixel (avec une petite tolérance) à une image gardée dans le dépôt | sans elles, personne ne voit qu'une découpe a déplacé un pixel. C'est la promesse faite à PowerPeek et KVMSoft | 2 | non (c'est un filet) | non |
| **Essais de compatibilité** (C-114) : un fichier `tests/compatibilite_<logiciel>.rs` par logiciel, qui écrit chaque chemin importé par ce logiciel (relevé au § 6). Si un chemin disparaît, la compilation échoue | garantir qu'un logiciel n'a qu'à changer `rev` | 1 | non | non |
| Découper `tests/sans_fenetre.rs` (1 701) en `sans_fenetre_volet.rs`, `sans_fenetre_parametres.rs`, `sans_fenetre_captures.rs` | seuil de 1 200 lignes | 0,5 | non | non |
| Commenter l'état global (`// ÉTAT GLOBAL : <pourquoi>`), environ 30 lieux | règle de la méthode ; `IDEAL.md` § 7 n'en permet que 4 familles (polices, compteurs de la sonde, lieu de la dernière panique, verrou d'instance). Les autres (`volet`, `autres_fenetres`, `parametres::OUVERTURES`) sont à justifier ou à ranger dans un objet | 0,5 | non | non |
| Commenter les 7 `#[allow]` | règle de la méthode | 0,25 | non | non |
| Une seule fonction `verrou` pour les 20 verrous empoisonnés | doublon | 0,25 | non | non |
| Créer `.seuils-acquis` pour les fichiers gardés au-dessus de 800 lignes pendant le chantier, puis le vider | règle de la méthode | 0,25 | non | non |

**Total transversal : 4,75 jours.**

**Total B : environ 23 jours de travail.**

## 4. Le plan de B, dans l'ordre

Chaque étape finit par un commit sur `main` qui garde tout `COMPORTEMENTS.md`. Les tests sans
fenêtre et les vérifications GitHub doivent passer, et les images de référence ne doivent pas
bouger. Un logiciel peut monter sa version (`rev`) à n'importe quelle étape sans changer son
code.

| Étape | Contenu | Coût | Peut aller en parallèle avec |
|---|---|---|---|
| **0. Les filets** | images de référence, essais de compatibilité, découpe de `sans_fenetre.rs`, `.seuils-acquis` | 3,75 | — (tout le reste en dépend) |
| **1. Le rangement sans risque** | sortir les tests des 7 fichiers (§ 1.3) ; commenter l'état global et les `#[allow]` ; retirer les 2 morts et les petits doublons ; blocs C, E, K, N, O | 3,5 | — |
| **2. Le dessin** | blocs F, G, H (découpe des fonctions et des fichiers) | 8 | étape 3 (pas les mêmes fichiers) |
| **3. Les bords** | blocs J, L, puis M (une seule icône, un seul aiguilleur) | 7 | étape 2 |
| **4. La fin** | bloc A (dépréciations) ; vider `.seuils-acquis` ; mise à jour de `MIGRATION.md`, `CARTE.md` et du README | 0,75 | — |

Avec deux sessions en parallèle aux étapes 2 et 3 (elles ne touchent pas les mêmes fichiers),
il faut environ **15 jours de calendrier**. En partant le 25 nov. 2026, ça donne :

- étape 0 : 25 au 28 nov. ;
- étape 1 : 28 nov. au 2 déc. ;
- étapes 2 et 3 : 2 au 12 déc. ;
- étape 4 : vers le 15 déc.

**Date estimée de fin : vers le 15 décembre 2026.** Il faut compter quelques jours de plus si
les vérifications GitHub en vraies fenêtres trouvent un défaut à l'étape 3.

**Pendant la refonte de PowerPeek.** PowerPeek épingle son `rev`. Il peut monter de version à
la fin de chaque étape, sans rien changer, puisque les essais de compatibilité gardent ses
chemins. Les étapes 2 et 3 ne changent ni son rendu ni son API. Si PowerPeek a besoin d'une
nouveauté pendant le chantier, elle s'ajoute sur `main` entre deux étapes, avec ses tests.

### Ce que B ne fait pas (et pourquoi)

- **B ne sépare pas le code en trois crates** (`ic-coeur`, `ic-dessin`, `ic-windows`). Après B,
  les modules sont rangés comme les blocs de l'idéal, et la séparation devient un simple
  déménagement de dossiers (environ 3 jours). Elle reste possible plus tard, si un logiciel en a
  besoin, par exemple pour compiler le cœur sans Windows.
- **B ne retire pas les méthodes de dessin des traits** (`Reglages::dessiner_page`…). Les quatre
  logiciels les implémentent : il faudrait changer leur code. Ce serait un changement d'API à
  annoncer, à faire plus tard, avec une période de dépréciation.
- **B ne réduit pas la surface publique.** 213 éléments publics ne servent à aucun logiciel. Les
  rendre privés casserait un logiciel futur sans rien gagner aujourd'hui. On les range ; on ne
  les cache pas.

## 5. A ou B : la recommandation

| | **A. Refonte à part** (nouveau dépôt ou nouvelle branche longue, d'après l'idéal) | **B. Nettoyage sur place** (par étapes, sur `main`) |
|---|---|---|
| Coût | environ 40 à 45 jours : réécrire environ 20 000 lignes de code et de tests, plus la migration de 4 logiciels | **environ 23 jours** |
| Fin | février 2027 au plus tôt | **vers le 15 décembre 2026** |
| Risque | **élevé** : tous les comportements à refaire et à revérifier (dont les 15 sans essai et les 6 vérifiés seulement sur GitHub ou à la main) ; deux bibliothèques à tenir pendant des semaines ; PowerPeek en refonte au même moment | **moyen** : chaque étape est petite, gardée par les filets de l'étape 0, et peut être annulée seule |
| Rendu | à refaire et à comparer en entier | ne bouge pas (images de référence) |
| API | nouvelle : les 4 logiciels à migrer | la même : les logiciels ne changent que `rev` |
| Ce qu'on gagne de plus | les trois crates d'un coup | rien de moins sur le fond : les trois crates restent possibles plus tard, en 3 jours |

**Recommandation : B.** Le code n'est pas en mauvais état : clippy est propre, il n'y a ni
`unwrap` ni code mort, et la moitié du cœur est déjà pure et testée. Ses défauts sont des
défauts de **rangement** : des fichiers et des fonctions trop longs, deux systèmes d'icône,
l'état global non commenté, deux promesses (API et rendu) sans essai. Tout cela se corrige sur
place, sans toucher aux logiciels qui en dépendent. Une refonte à part coûterait deux fois plus
cher et ferait courir un risque bien plus grand, pour arriver au même endroit.

**Le seul vrai chantier est le bloc M** (fusionner `horloge` et `zone`). Il est classé
« réécrire l'intérieur », mais il reste dans B : ses types publics ne changent pas, et la
moitié du chemin est déjà faite.

## 6. Annexe : ce que les logiciels importent

Le relevé a été fait avec `git grep interface_commune::` dans chaque dépôt. C'est la liste que
garderont les essais de compatibilité (§ 3).

- **KVMSoft** (rev `e077572`) : `capture` ; `dessin::{focus, icone, bouton_icone,
  interrupteur_ecrit, insecables, Genre}` ; `toutes_faites` ; `parametres::{Parametres,
  Contexte, modele, fenetre}` ; `horloge::{Horloge, Reveil, Poste, reglage_dpi,
  instance_unique_avec}` ; `polices::{icone_disponible, semibold}` ; `autres_fenetres::Ouverture`
  ; `apparence::choix_windows` ; `systeme::disque_local` ; `sonde::{imbriquee,
  brancher_le_journal, Inscription}` ; `pastilles::couleur_lettre` ; `couleurs::palette` ;
  `zone::FinDeSession` ; `verification::logos_absents` ; `textes::FRANCAIS` ; `place::Place` ;
  `mesures::PARAMETRES_TAILLE` ; `menu_choix` ; `menu::Action`.
- **PowerPeek** (rev `119c500`) : `theme::{Surface, ChoixWindows}` ; `zone::{FenetresCommunes,
  guid_stable}` ; `polices::{polices, semibold, polices_ou_vides}` ; `place::{Place,
  PlacementVolet}` ; `apparence::{couleur, Apparence}` ; `volet` ; `textes` ;
  `surface::dessiner_borne` ; `seuils::ReponseSeuils` ; `parametres::{fenetre, TAILLE}` ;
  `nuancier::bouton_nuancier` ; `menu::{lignes, Action}` ; `icones::INSTALLER` ;
  `horloge::Niveau` ; `fenetres::{envoyer_le_clic, Demande}` ; `capture` ; `briques::Brique`.
- **Vocalis** (rev `119c500`) : `apparence::{couleur, Apparence}` ; `parametres::{modele,
  Parametres, Contexte}` ; `systeme::{ecran_autour, position_souris, choix_windows}` ;
  `place::Place` ; `horloge::{Niveau, MessageWindows}` ; `textes::FRANCAIS` ; `briques::Brique` ;
  `autres_fenetres::AutresFenetres` ; `zone::FinDeSession` ; `theme::Surface` ; `surface` ;
  `polices::polices_ou_vides` ; `capture`.
- **Chat PDC** (rev `77bad2b`) : `horloge::Horloge` ; `parametres::modele`.

Les quatre logiciels implémentent `Reglages` et `Logiciel`. Aucun n'utilise
`dessiner_rapides` ni `largeur_rapides`.

## 7. Comment ces mesures ont été prises

- Les tailles et les fonctions longues : un petit script qui compte les lignes de chaque
  fichier, en séparant le module `#[cfg(test)]`, et qui suit les accolades de chaque `fn`.
- Le code mort et la surface publique : un script qui cherche chaque nom public (`pub fn`,
  `struct`, `enum`, `trait`, `const`, `type`, `static`), mot entier, dans la bibliothèque, ses
  tests et ses exemples, puis dans les fichiers Rust suivis par Git des quatre logiciels. Une
  méthode appelée par un trait, ou un nom très courant, peut échapper à ce relevé : les chiffres
  sont des ordres de grandeur.
- Les tests : `cargo test --lib` et les essais sans fenêtre, avec `CARGO_BUILD_JOBS=2` et
  `--test-threads=2`. Les essais en vraies fenêtres n'ont pas été lancés sur le PC.
- Aucun sous-agent n'a été utilisé.
