//! Les couleurs dans les Paramètres, aux couleurs du volet :
//!
//! - [`pastille_de_carte`] : la pastille d'un appareil (le carré coloré et sa lettre ou son logo, la même
//!   que celle de la brique [`crate::briques::Brique::Pastilles`]) posée dans une carte ;
//! - [`choisir_couleur`] : une rangée de pastilles rondes à choisir, comme la couleur d'accent
//!   de Windows 11 : contour sur la couleur choisie, flèches pour passer de l'une à l'autre,
//!   Entrée ou Espace pour choisir, contour de focus visible ; trop étroite, elle passe sur
//!   des lignes égales ([`colonnes`]) ;
//! - [`carte_couleur`] : une carte avec cette rangée, et la bulle « Annuler » en option.
//!
//! Les couleurs viennent d'un seul endroit : [`crate::apparence::couleurs_pastilles`]
//! ([`palette`]). Rien de nouveau n'est écrit ailleurs.

use egui::{EventFilter, Key, Modifiers, Rect, Response, Sense, Ui, pos2, vec2};

use crate::apparence::couleurs_pastilles;
use crate::dessin::carte_large;
use crate::mesures::{PASTILLE_HAUTEUR, PASTILLE_LARGEUR};
use crate::nuancier::{ZONE, dessiner_pastille};
use crate::parametres::{Contexte, modele::Valeur};
use crate::pastilles::{Pastille, carre};
use crate::theme::{Palette, Rgb};

/// Les couleurs proposées : celles des pastilles du volet (une seule liste pour tous).
pub fn palette() -> [Rgb; 10] {
    couleurs_pastilles()
}

/// Ce qu'une carte prend hors de son contrôle : ses marges (2 × 16), l'icône (28), l'espace
/// après l'icône (8), trois écarts entre éléments (3 × 8) et le texte le plus étroit (160).
const CARTE_HORS_CONTROLE: f32 = 2.0 * 16.0 + 28.0 + 8.0 + 3.0 * 8.0 + 160.0;

/// Le nombre de pastilles par ligne pour `n` couleurs dans `largeur` : toutes sur une ligne
/// si elles tiennent, sinon des lignes égales (10 → 2 lignes de 5), jamais plus larges que la
/// place (une pastille au moins par ligne).
pub fn colonnes(n: usize, largeur: f32) -> usize {
    let tiennent = ((largeur / ZONE).floor() as usize).max(1);
    if n <= tiennent {
        return n.max(1);
    }
    n.div_ceil(n.div_ceil(tiennent))
}

/// La pastille d'un appareil dans une carte des Paramètres, à poser dans le contrôle d'une
/// carte (`dessin::carte`). Même carré, même lettre, même couleur que dans le volet ; seuls la
/// couleur, la lettre et `absent` (carré hachuré) comptent ici. Rend sa réponse (survol).
pub fn pastille_de_carte(ui: &mut Ui, p: &Palette, pastille: &Pastille) -> Response {
    let (rect, r) = ui.allocate_exact_size(vec2(PASTILLE_LARGEUR, PASTILLE_HAUTEUR), Sense::hover());
    carre(
        ui,
        p,
        rect,
        Some(pastille.lettre),
        pastille.logo,
        pastille.couleur,
        pastille.absent,
        crate::theme::TAILLE_PETIT,
    );
    r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &pastille.nom));
    r
}

/// Une rangée de pastilles rondes : `choisie` (le rang de la couleur choisie) porte le
/// contour. Trop étroite pour la place, la rangée passe sur des lignes égales ([`colonnes`]),
/// lues de gauche à droite puis de haut en bas. Rend le rang cliqué, ou choisi au clavier
/// (Entrée, Espace). Les flèches gauche et droite passent de l'une à l'autre sans choisir ;
/// Tab sort de la rangée. `nom` : ce que dit un lecteur d'écran (« Couleur du PC », suivi du
/// rang et du code de la couleur).
pub fn choisir_couleur(
    ui: &mut Ui,
    p: &Palette,
    couleurs: &[Rgb],
    choisie: Option<usize>,
    nom: &str,
) -> Option<usize> {
    let mut choix = None;
    // Une clé par rangée (deux cartes de couleur dans une page ne se confondent pas).
    let base = ui.next_auto_id();
    let ids: Vec<egui::Id> = (0..couleurs.len())
        .map(|i| base.with(("choix_couleur", i)))
        .collect();
    // Les flèches sont à nous tant qu'une pastille a le focus : Windows ne s'en sert pas.
    let filtre = EventFilter {
        horizontal_arrows: true,
        ..EventFilter::default()
    };
    let focalise = ids.iter().position(|&id| ui.memory(|m| m.has_focus(id)));
    if let Some(i) = focalise {
        let (gauche, droite) = ui.input_mut(|e| {
            (
                e.consume_key(Modifiers::NONE, Key::ArrowLeft),
                e.consume_key(Modifiers::NONE, Key::ArrowRight),
            )
        });
        let voisine = if gauche && i > 0 {
            Some(i - 1)
        } else if droite && i + 1 < ids.len() {
            Some(i + 1)
        } else {
            None
        };
        if let Some(v) = voisine {
            ui.memory_mut(|m| m.request_focus(ids[v]));
        }
    }
    // Posées une à une à la main : la rangée se lit de gauche à droite même dans une carte,
    // dont le contrôle est aligné à droite.
    let par_ligne = colonnes(couleurs.len(), ui.available_width());
    let lignes = couleurs.len().div_ceil(par_ligne);
    let (zone, _) = ui.allocate_exact_size(
        vec2(ZONE * par_ligne as f32, ZONE * lignes as f32),
        Sense::hover(),
    );
    for (i, &c) in couleurs.iter().enumerate() {
        let coin = pos2(
            zone.left() + ZONE * (i % par_ligne) as f32,
            zone.top() + ZONE * (i / par_ligne) as f32,
        );
        let rect = Rect::from_min_size(coin, vec2(ZONE, ZONE));
        let r = ui.interact(rect, ids[i], Sense::click());
        ui.memory_mut(|m| m.set_focus_lock_filter(ids[i], filtre));
        let libelle = format!("{nom} {}/{}", i + 1, couleurs.len());
        dessiner_pastille(ui, p, &r, c, &libelle, choisie == Some(i));
        if r.clicked() {
            choix = Some(i);
        }
    }
    choix
}

/// Une carte des Paramètres avec la rangée de couleurs à droite : sur une ligne si la carte
/// est assez large, sinon sur deux lignes égales. `choisie` : le rang de la couleur actuelle
/// dans [`palette`] (ou dans `couleurs`). Rend le rang choisi, à enregistrer.
///
/// Avec `annuler: Some(titre)`, le choix est aussi rendu à la bibliothèque comme un réglage
/// (`Contexte::changer`, clé `cle`, valeur [`Valeur::Nombre`] = le rang) : le logiciel le
/// reçoit par `Reglages::changer`, et la bulle « Annuler » (titre `titre`) remet le rang d'avant.
/// Avec `None`, rien n'est signalé : c'est au logiciel de tout faire.
// Fonction publique prise telle quelle par les logiciels : chaque argument est une donnée du
// dessin ; les regrouper changerait leur appel.
#[allow(clippy::too_many_arguments)]
pub fn carte_couleur(
    ui: &mut Ui,
    page: &mut Contexte,
    glyphe: char,
    texte: &str,
    explication: &str,
    couleurs: &[Rgb],
    choisie: usize,
    annuler: Option<(&str, &str)>,
) -> Option<usize> {
    let p = page.palette;
    let mut choix = None;
    let n = couleurs.len();
    let par_ligne = if ui.available_width() - CARTE_HORS_CONTROLE >= n as f32 * ZONE + 16.0 {
        n
    } else {
        n.div_ceil(2)
    };
    carte_large(
        ui,
        p,
        glyphe,
        texte,
        explication,
        par_ligne as f32 * ZONE + 16.0,
        |ui| {
            choix = choisir_couleur(ui, p, couleurs, Some(choisie), texte);
        },
    );
    let nouveau = choix.filter(|&i| i != choisie)?;
    if let Some((cle, titre)) = annuler {
        page.changer(
            cle,
            titre,
            Valeur::Nombre(choisie as f64),
            Valeur::Nombre(nouveau as f64),
        );
    }
    Some(nouveau)
}
