# Relecture de `IDEAL.md` par un sous-agent neuf

| Projet | Date | Relu par | Résultat |
|---|---|---|---|
| Interface commune | 2026-11-24 | un sous-agent neuf (Opus), sans lire le code | **3 points bloquants** et 12 à améliorer : `IDEAL.md` est à corriger avant de servir d'`ORGANIGRAMME.md` |

Les numéros sont les lignes de `IDEAL.md` au commit 262c3c0. Une cause commune : l'idéal ne cite
pas `COMPORTEMENTS.md` parmi ses sources (l. 7-9).

**Décision de l'Atelier Master** : la recommandation B ne change pas. B ne sépare pas les crates,
ne sort pas les méthodes de dessin des traits et ne change pas l'API (ANALYSE §4, « Ce que B ne
fait pas »). Il échappe donc déjà au bloquant 2. La correction de `IDEAL.md` devient la
**première tâche de l'étape 0** :
- citer `COMPORTEMENTS.md` ;
- ajouter la table « module public d'aujourd'hui → bloc » ;
- reprendre les bloquants 1 et 3.

Le doute de la l. 405 (quatre crates ou une seule) est tranché pour B : **une seule crate**. La
séparation reste une décision à part, plus tard, avec une période de dépréciation.

## Bloquants

1. **§3.3 J l. 181 et §6 l. 320** : « la seule boucle eframe » et « les fils ne passent que par
   `Poste` » oublient trois usages documentés :
   - le logiciel qui garde sa boucle (`Zone`, `FenetresCommunes`, `Aiguilleur`,
     `superviser_les_fenetres`) ;
   - les Paramètres dessinés dans la boucle du logiciel (`ouvrir_dans_boucle`,
     `autres.lier/montrer`, `Parametres::dessiner`, `echelle::suivre_windows`) ;
   - la sonde appelée par les fils du logiciel (`sonde::inscrire`, `battement`, `etape`).

   PowerPeek s'en sert, et C-82 en dépend.
2. **§10 l. 372-374** : « mêmes chemins, seul `rev` change » est faux si l'on suit §3 et les
   crates :
   - les méthodes de dessin sortent des traits ;
   - un cœur sans egui change des types publics (`plan_des_cases`, `menu_choix::placer`, la
     palette, `Brique`, `briques_avec_contexte`) ;
   - Rust interdit d'ajouter des méthodes à un type d'une autre crate ;
   - des constantes publiques bougent sans renvoi.

   Il manque aussi la table des 27 modules publics vers les blocs.
3. **§9 l. 363-364** : « le seul `catch_unwind`, à un seul endroit » casse C-62 (une panique dans
   une fenêtre ne tue pas les autres) et va contre C-60.

## À améliorer

4. Le journal passe par `Logiciel::noter`. Rien n'est prévu pour un logiciel qui ne prend que les
   Paramètres, ni pour `brancher_le_journal` (C-110).
5. Les traits sont appelés depuis plusieurs fils (chien de garde, fil de l'icône) : il faut dire
   `Send + Sync`.
6. Le schéma de §5 et son tableau ne disent pas les mêmes flèches (F, B→F/G, C→F, L→C, I→F,
   E→H, D→G, K→M, seconde copie vers D/K). La case cochée de la l. 416 est fausse.
7. « 3 pertes en 60 s → menu de secours » est une décision, posée dans un bord : elle va au cœur.
   Le « faux moteur » n'est pas dessiné.
8. Le test des polices et les captures ont besoin des polices de Windows. Ni « sur Linux aussi »,
   ni la source des images de référence ne sont dits.
9. Il faut dire quel état reste à egui (focus, défilement, section ouverte) et à `MenuChoix`, qui
   vit chez le logiciel.
10. Un `ErreurInterface` unique changerait le type d'erreur de `lancer` et de `instance_unique`.
11. « déjà la pratique » (§2, l. 40-43) est contredit par l'histoire de MIGRATION : `glyphe`
    retiré, constantes de `theme`, variantes de `Brique`, champs ajoutés, `[Rgb; 10]`. De plus,
    « au moins une version » ne veut rien dire tant que le numéro reste 0.1.0.
12. Comportements sans bloc nommé : C-05, 07, 09, 10, 11, 12, 13, 28, 32, 33, 34, 35, 51, 79, 98,
    113, `Reveil::Rien`, et `sonde::regler`, alors que O écrit 15 s en dur.
13. « quand Windows change » contre C-90 (thème relu toutes les 2 s) : il faut choisir l'un des
    deux.
14. « un geste = une bulle » : écrire la règle de C-70 telle quelle.
15. Images de référence :
    - un nom qui porte la version les renomme toutes à chaque version ;
    - il manque un mode « refaire » distinct de « comparer » ;
    - `capture` ne sait pas encore faire les fenêtres toutes faites.
