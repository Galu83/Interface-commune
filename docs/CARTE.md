# Carte

| Où | Quoi |
|---|---|
| `src/lib.rs` | le trait `Logiciel` (ce qui est propre à chaque logiciel), `Identite`, `Etat`, `Communs`, les clés communes |
| `src/theme.rs`, `apparence.rs`, `polices.rs` | couleurs Windows 11, clair/sombre, accent, durées d'animation, polices Segoe |
| `src/horloge.rs` | l'icône près de l'horloge, une seule copie, `Horloge::lancer`, `Poste` |
| `src/fenetres.rs`, `volet.rs`, `surface.rs` | le fil des fenêtres, la fenêtre unique du volet et du petit menu, leur contenu |
| `src/menu.rs` | l'ordre du petit menu |
| `src/parametres/` | les Paramètres : modèle (pages, Annuler), dessin, fenêtre |
| `src/dessin.rs` | cartes, interrupteurs, bandeau, boutons |
| `src/place.rs`, `clics.rs`, `systeme.rs` | placement près de la barre des tâches, clics de l'icône, appels à Windows |
| `src/verification.rs` | contrôles des polices, pour les tests des logiciels |
| `tests/` | volet, menu et Paramètres sans fenêtre ; polices |
| `examples/demo.rs` | un logiciel d'exemple |

Tests : `cargo test -q` ; avant un envoi : `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo deny check`.
