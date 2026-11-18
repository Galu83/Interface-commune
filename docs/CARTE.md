# Carte

| Où | Quoi |
|---|---|
| `src/lib.rs` | les traits `Reglages` (les Paramètres) et `Logiciel` (l'icône, le volet, le menu), `Identite`, `Etat`, `Communs`, les clés communes |
| `src/textes.rs` | les mots écrits par la bibliothèque, en français et en anglais |
| `src/theme.rs`, `apparence.rs`, `polices.rs` | couleurs Windows 11, clair/sombre, accent, durées d'animation, polices Segoe |
| `src/horloge.rs` | l'icône près de l'horloge, une seule copie, `Horloge::lancer`, `Poste` |
| `src/zone.rs` | plusieurs icônes à la fois pour un logiciel qui garde sa boucle de messages : `Zone` (poser, retirer, notifier sur une icône, clic avec l'icône et sa place, menu gris de Windows avec cases et sous-menus) |
| `src/sonde.rs` | la sonde des fils : clics de l'icône, boucles imbriquées, chien de garde (un compteur de battements et une « dernière étape » par fil), tout au journal du logiciel |
| `src/fenetres.rs`, `volet.rs`, `surface.rs` | le fil des fenêtres, la fenêtre unique du volet et du petit menu (la fenêtre principale de la boucle), leur contenu ; la supervision d'une boucle de fenêtres tenue par le logiciel (`superviser_les_fenetres`) |
| `src/autres_fenetres.rs` | les autres fenêtres de la boucle (Paramètres, fenêtres du logiciel) |
| `src/briques.rs`, `mesures.rs` | les briques du volet ; les mesures communes du volet, du menu et des Paramètres |
| `src/toutes_faites.rs` | la fenêtre simple et la fenêtre de lecture |
| `src/menu.rs` | l'ordre du petit menu |
| `src/parametres/` | les Paramètres : modèle (pages, Annuler), dessin, fenêtre |
| `src/nuancier.rs` | le nuancier : pastille de couleur, grille, bouton qui ouvre la grille (choisir une couleur sans taper de code) |
| `src/seuils.rs` | la barre à seuils : une barre de 0 à 100 coupée en zones colorées, dont on règle les limites en glissant les poignées (ou au clavier) |
| `src/dessin.rs`, `icones.rs` | cartes, interrupteurs, bandeau, boutons, champ de raccourci ; les icônes communes |
| `src/place.rs`, `clics.rs`, `systeme.rs` | placement près de la barre des tâches, clics de l'icône, appels à Windows (dont le sélecteur de dossier et les fenêtres sans fantôme) |
| `src/capture.rs` | captures PNG du volet, du petit menu et des Paramètres pour la documentation, sans fenêtre |
| `src/verification.rs` | contrôles des polices, pour les tests des logiciels |
| `tests/` | volet, menu et Paramètres sans fenêtre ; polices ; avis de licence à jour |
| `docs/LICENCES.md` | avis MIT des crates publiées sans fichier de licence (egui…) |
| `docs/MIGRATION.md`, `ETAT.md` | ce que chaque projet change ; où en est la bibliothèque |
| `examples/demo.rs` | un logiciel d'exemple |
| `examples/mesure_ouverture.rs` | mesure l'ouverture du volet et du petit menu, image par image, fenêtre voilée (rien à l'écran) |

Tests : `cargo test -q` ; avant un envoi : `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo deny check`.
