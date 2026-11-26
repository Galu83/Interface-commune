# Carte

La bibliothèque est rangée en trois étages (`docs/ORGANIGRAMME.md`), contrôlés par
`tests/etages.rs` : **le cœur** (`crates/ic-coeur/`, du calcul pur), **les bords**
(`src/bords/`, Windows, egui et winit : le seul étage où `unsafe` et l'état global sont permis)
et **le dessin** (`src/dessin.rs`, `src/dessin/`, egui seul). Les fichiers à la racine de `src/`
ne sont plus que des **façades** : ils reprennent le travail des étages à leur ancien chemin, pour
que les logiciels n'aient rien à changer.

| Où | Quoi |
|---|---|
| `Cargo.toml` | l'espace de travail : deux crates, `interface-commune` (ici, celle que prennent les logiciels) et `crates/ic-coeur` |

## Le cœur : `crates/ic-coeur/` (sans egui, sans Windows, sans `unsafe`)

| Où | Quoi |
|---|---|
| `src/lib.rs` | les six modules du cœur, un par bloc de la refonte (`docs/PLAN-REFONTE.md`) |
| `src/contrat.rs`, `contrat/` | le contrat (bloc A) : `Identite`, `Etat`, `Communs`, `Valeur`, `Page`, `Instantane`, `Reveil`, `Niveau`, les clés communes (`cles`) ; essais dans `contrat/essais.rs` |
| `src/textes.rs`, `textes/` | les mots écrits par la bibliothèque, en français et en anglais (bloc B) : `catalogue.rs`, `raccourcis.rs` (les raccourcis en toutes lettres, `lisible`), les espaces insécables ; essais dans `textes/essais.rs` |
| `src/theme.rs`, `theme/` | le thème et les mesures (bloc C) : `couleurs`, `tailles`, `formes`, `animation`, `mesures` (volet, menu, Paramètres), `icones` (les icônes communes), `marque.rs` (la marque d'une pastille : son logo si la police l'a, sinon sa lettre ; le texte lisible sur une zone colorée, bloc F), `registre.rs` (ce que veulent dire les valeurs du registre de Windows : clair/sombre, accent, `choix_du_registre`), `polices.rs` (quel fichier pour quelle police, ses replis, sa graisse et sa taille optique, `plan`, `FAMILLES`, bloc L), `relecture` ; essais dans `essais.rs` et `essais_pastilles.rs` (couleurs des pastilles : ΔE2000 d'au moins 12, lisibles, rangs gardés) |
| `src/gestes.rs`, `gestes/` | les gestes et les secours (bloc D) : `clics` (geste reconnu et sa suite), `aiguilleur` (suivi par icône et ordre unique, pour l'icône seule et pour la zone), `ouverture` (décision d'un clic, anti-rebond, garde, clic en dehors), `secours` (pertes de fenêtres, rafale, reprise), `icone` (info-bulle, copie dans le tampon de Windows, délai contre le clignotement), `menu` (lignes et `Action` du petit menu), `choix` (les touches du menu de choix, sa ligne initiale, son nom pour le lecteur d'écran, bloc F), `seuils` (les bornes et le pas des poignées de la barre à seuils, bloc F), `clavier` (les flèches comme Tab, C-33) ; l'instant arrive par le trait `Horodatage` (`temps.rs`) |
| `src/gestes/sonde.rs` | la sonde en calcul pur (bloc O) : la décision « bloqué » et la reprise, la veille du PC, les créations de fenêtre longues, le texte de chaque ligne du journal, le lieu court d'une panique ; essais à fausse horloge dans `gestes/sonde/essais.rs` |
| `src/placement.rs`, `placement/` | le placement (bloc D) : volet contre la barre des tâches, petit menu, Paramètres, écrans et changement d'échelle (essais dans `placement/essais.rs`) ; `plan.rs` : `Rectangle` (même calcul qu'egui), le plan des écrans des pastilles, la place du menu de choix ; `surface.rs` : la mise en page du volet et du petit menu |
| `src/parametres.rs`, `parametres/` | le modèle des Paramètres (bloc E) : `pages` (ordre de la navigation, page demandée, page gardée quand les pages changent), `recherche` (sans accents, section ouverte), `colonne` (lignes montrées, champ de recherche, largeur de la colonne), `annuler` (la bulle « Annuler »), `conserver` (« Conserver ces modifications ? »), `envois` (ce que chaque geste rend au logiciel, dans l'ordre ; bandeau « Redémarrer »), `mise_a_jour` (icône, titre, couleurs, ligne grise, bouton et carte du dossier) ; essais dans `essais.rs`, `essais_gestes.rs`, `essais_mise_a_jour.rs` |

## Les bords : `src/bords/` (Windows ; `unsafe` permis, chaque appel commenté)

| Où | Quoi |
|---|---|
| `mod.rs` | la liste des bords |
| `verrou.rs` | le seul mécanisme de verrou de la bibliothèque (ORGANIGRAMME §6) : `verrou`, `lecture`, `ecriture`, `sans_poison` ; un verrou empoisonné par un fil qui a paniqué est repris tel quel, jamais par `unwrap`. Les boucles des fenêtres, le guichet, la sonde, les polices et la reprise passent tous par lui |
| `reprise.rs` | le dessin des fenêtres attend la fin d'un réveil de Windows (veille, écran éteint, 2 s après la reprise) ; lu avant toute image OpenGL |
| `poste.rs`, `poste/` | le guichet (bloc K) : `Poste`, `AuClic`, la boîte bornée du fil de l'icône et son réveil ; essais (`poste/essais.rs`) : rafales de plusieurs fils avec un faux réveil, sans fenêtre |
| `unique.rs`, `unique/` | l'instance unique (bloc N) : le verrou `Local\<nom>-une-seule`, le second lancement qui prévient la première copie ; sa partie pure (`unique/codage.rs` : noms, arguments bornés et marqués, décisions, erreurs nommées) et ses essais (`unique/essais.rs`, sans fenêtre) |
| `systeme.rs`, `systeme/` | les appels au système (bloc L) : `fenetre` (habillage DWM, place, opacité, fenêtre du fil), `ecran` (écrans, souris, effets d'animation, double-clic), `reveil`, `dossier` (sélecteur, disque local), `registre` (lecture du thème), `cachee` (la fabrique unique de fenêtre cachée, pour l'icône et la zone), `echelle` (changement d'échelle, `reglage_dpi`), `apparence` (le thème suivi), `polices` (lecture des polices) ; essais Windows (`systeme/essais.rs`), fenêtres toujours cachées |
| `verification.rs` | les contrôles des polices, pour les tests des logiciels |
| `icone.rs`, `icone/` | les icônes près de l'horloge (bloc M) : `zone.rs`, le seul moteur des icônes : `Zone` (une fenêtre cachée de `systeme::cachee`, les appels à `Shell_NotifyIconW`, la remise après le redémarrage de l'Explorateur, le menu gris de secours, une procédure de plus pour l'icône seule) ; `horloge.rs`, `Horloge` : une zone à une seule icône (numéro 1, sans GUID), ses messages (commandes, veille, thème, instance unique) et sa boucle ; `communes.rs`, `FenetresCommunes` et `Aiguilleur` (pour un logiciel qui garde sa boucle) ; `fil.rs`, le fil des fenêtres et son secours ; `image.rs`, l'image depuis des pixels ; `decodage.rs`, le décodage pur (messages, rappels, fin de session, destinataire, lignes du menu) ; essais : `zone/essais.rs` (appels à Windows notés au lieu d'être faits : aucune icône montrée) et `essais_horloge.rs` |
| `sonde.rs`, `sonde/` | la sonde des fils (bloc O) : clics de l'icône, boucles imbriquées, chien de garde (un compteur de battements et une « dernière étape » par fil), tout au journal du logiciel ; `sonde.rs` : fils, journal, délais ; `sonde/chien.rs` : la surveillance et le chien des créations ; `sonde/panique.rs` : le lieu d'une panique ; essais dans `sonde/essais.rs` ; la décision reste dans le cœur (`gestes/sonde.rs`) |
| `fenetres.rs`, `fenetres/` | les boucles des fenêtres (bloc J ; Windows, egui et winit) : `boucle.rs` (la boucle de la bibliothèque, `boucle_des_fenetres`, `Ouvreur`, découpée en état et pas), `liens.rs` (les demandes de l'icône, l'aiguillage d'un clic `envoyer_le_clic`, les `Liens` partagés), `canal.rs` (l'envoi et la réception des demandes, le réveil), `supervision.rs` (façon 2 : `superviser_les_fenetres`, `Supervise`), `volet.rs` et `volet/` (la fenêtre du volet et du petit menu : `veille` le veilleur, `fenetre` la fenêtre et ses étapes, `app` son `eframe::App`), `autres.rs` et `autres/` (les autres fenêtres : `ouverture` pour `Ouverture`, `Contenu`, `Cadre` ; `delais` pour les attentes avant une création ; essais dans `autres/essais.rs`), `parametres.rs` (la fenêtre des Paramètres, ses trois façons de s'ouvrir), `toutes_faites.rs` ; essais dans `essais.rs` sur un faux moteur (`faux_moteur.rs` : fausse boucle, faux egui des autres fenêtres), sans fenêtre ; la décision d'un clic et d'une perte reste dans le cœur (`ic_coeur::gestes`) |

## Le dessin : `src/dessin.rs`, `src/dessin/` (egui seul : ni Windows, ni `unsafe`, ni état global)

| Où | Quoi |
|---|---|
| `dessin.rs` | les contrôles, repris à leur ancien chemin (`dessin::carte`, `interrupteur`…) |
| `controles/` | `base` (contour de focus, icône, lignes du volet et du petit menu, boutons), `interrupteur` (interrupteur, interrupteur écrit, roue dentée), `raccourci` (champ de raccourci), `cartes` (titres, cartes, bandeau et état des Paramètres) |
| `couleur.rs`, `apparence.rs` | de `Rgb` à la couleur d'egui ; la palette, les styles de texte et les espacements donnés à egui (bloc L ; repris par `Apparence::appliquer`) |
| `traits.rs` | les traits `Reglages` (les Paramètres) et `Logiciel` (l'icône, le volet, le menu), repris à la racine |
| `briques.rs`, `briques/` | les briques du volet ; l'appareil dans `briques/appareil.rs` |
| `pastilles.rs`, `pastilles/` | les pastilles et le choix illustré ; essais dans `pastilles/essais.rs` |
| `couleurs.rs` | les couleurs dans les Paramètres, aux couleurs du volet : `pastille_de_carte`, `choisir_couleur` (rangée de pastilles rondes : flèches, Entrée ou Espace, contour de focus ; sur des lignes égales si elle ne tient pas, `colonnes`), `carte_couleur` (« Annuler » en option), `palette()` = `apparence::couleurs_pastilles()` (10 couleurs, les 6 premiers rangs inchangés) |
| `menu_choix.rs` | le menu flottant de choix : le dessin dans une `Area` (les touches et le nom pour le lecteur d'écran sont dans le cœur, `ic_coeur::gestes::choix`) |
| `seuils.rs` | la barre à seuils : une barre de 0 à 100 coupée en zones colorées, dont on règle les limites en glissant les poignées (ou au clavier) ; les bornes et le pas sont calculés dans le cœur |
| `nuancier.rs`, `touches.rs` | pastille de couleur, grille, bouton qui l'ouvre ; la saisie d'un raccourci |
| `surface.rs`, `volet.rs`, `volet/` | le contenu du volet et du petit menu (bloc G) : `surface.rs` (largeur, hauteur bornée `dessiner_borne`, petit menu, séparateur), `volet.rs` (en-tête et état fixes, briques qui défilent, pied), `volet/entete.rs` (titre, version, roue), `volet/clavier.rs` (flèches comme Tab, C-33) ; la mise en page se décide dans le cœur (`ic_coeur::placement::surface`), l'icône et la couleur d'un état aussi (`GenreEtat::icone_et_couleur`), le gras du menu (`Ligne::en_gras`) et les flèches (`ic_coeur::gestes::clavier`) |
| `parametres.rs`, `parametres/` | le squelette des Paramètres (bloc H) : `navigation`, `contexte`, `gestes`, `bulles` (« Annuler », « Conserver ? », « Redémarrer »), puis `general`, `mise_a_jour`, `a_propos` |
| `toutes_faites.rs`, `toutes_faites/` | les fenêtres toutes faites (bloc H) : `simple`, `prete`, `lecture` ; aides des essais dans `essais.rs` |
| `capture.rs`, `capture/toile.rs` | les captures sans fenêtre (bloc I) : `Options`, `Image`, le volet, le petit menu, les Paramètres, tout dessin, la mise en page par egui ; `toile.rs` : le rendu logiciel des triangles et des textures ; elles font aussi les images de référence (`tests/references.rs`) |

## Les façades : la racine de `src/`

| Où | Quoi |
|---|---|
| `lib.rs` | les modules publics et leurs reprises (`briques`, `seuils`, `menu_choix`, `pastilles`, `nuancier`, `couleurs`, `touches`, `surface` reprennent `dessin::…` à leur ancien chemin), `Identite`, `Etat`, `Communs`, les clés communes (`cles`) |
| `textes.rs`, `theme.rs`, `mesures.rs`, `icones.rs`, `menu.rs`, `place.rs`, `clics.rs` | reprennent le cœur (`ic_coeur::textes`, `theme`, `theme::mesures`, `theme::icones`, `gestes::menu`, `placement`, `gestes::clics`) ; `place.rs` convertit aussi les rectangles egui ↔ cœur |
| `apparence.rs`, `polices.rs`, `echelle.rs`, `systeme.rs`, `verification.rs` | reprennent `bords::systeme::…` et `bords::verification` (bloc L) |
| `horloge.rs`, `zone.rs` | l'icône près de l'horloge (`Horloge::lancer` ; `horloge::{Poste, AuClic}` ; l'instance unique : `horloge::{instance_unique, instance_unique_avec, Unique, ARGUMENTS_MAX, ARGUMENTS_OCTETS_MAX}`) et plusieurs icônes pour un logiciel qui garde sa boucle (`Zone`, `FenetresCommunes`, `Aiguilleur`) ; reprennent `bords::icone` (bloc M) |
| `fenetres.rs`, `volet.rs`, `autres_fenetres.rs`, `toutes_faites.rs` | reprennent `bords::fenetres` (bloc J) : l'aiguillage d'un clic d'icône, la fenêtre unique du volet et du petit menu, la supervision d'une boucle tenue par le logiciel (`superviser_les_fenetres`), les autres fenêtres de la boucle, les fenêtres toutes faites (leur dessin vient de `dessin::toutes_faites`) |
| `sonde.rs` | reprend `bords::sonde` à `interface_commune::sonde` (bloc O) |
| `parametres/` | un module mince (`mod.rs` reprend `Parametres` et `Contexte` de `dessin::parametres` et lit les « effets d'animation » avant de dessiner) ; `fenetre.rs` reprend `bords::fenetres::parametres` ; `modele.rs` reprend `ic_coeur::parametres` |
| `capture.rs`, `dessin.rs` | les captures (`dessin::capture`) ; l'entrée du dessin |

## Le reste

| Où | Quoi |
|---|---|
| `tests/` | `etages.rs` (le cœur ne dépend de rien ; le dessin n'appelle pas Windows ; `unsafe` seulement dans les bords) ; volet, menu et Paramètres sans fenêtre ; polices ; avis de licence à jour ; images de référence ; compatibilité avec les logiciels qui prennent la bibliothèque ; `comportements.rs` (`docs/COMPORTEMENTS.md`) |
| `docs/LICENCES.md` | avis MIT des crates publiées sans fichier de licence (egui…) |
| `docs/MIGRATION.md`, `ETAT.md`, `ORGANIGRAMME.md`, `PLAN-REFONTE.md` | ce que chaque projet change ; où en est la bibliothèque ; les règles de rangement ; le plan de la refonte |
| `examples/demo.rs` | un logiciel d'exemple, avec une fenêtre principale (double-clic, « Ouvrir Exemple » en gras, roue dentée, interrupteur écrit) |
| `examples/conserver.rs` | « Conserver ces modifications ? » : un réglage à confirmer, deux captures sans fenêtre |
| `examples/couleurs.rs` | la pastille et le choix de couleur dans une carte, « Annuler » compris : quatre captures sans fenêtre (clair, sombre, avant et après un choix) |
| `examples/menu_choix.rs` | le menu flottant de choix et les logos des pastilles : cinq captures sans fenêtre (clair, sombre, 150 %, menu au-dessus) |
| `examples/section_repliable.rs` | une section repliable dans une page des Paramètres, repliée puis ouverte par la recherche : deux captures sans fenêtre |
| `examples/mesure_ouverture.rs` | mesure l'ouverture du volet et du petit menu, image par image, fenêtre voilée (rien à l'écran) |

Tests (les deux crates) : `cargo test --workspace -q` ; avant un envoi : `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo deny check`, et la documentation
sans avertissement (`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`).
