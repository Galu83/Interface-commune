//! Le menu flottant de choix (`menu_choix`) et les logos dans les pastilles : un écran inventé
//! à gauche, qui s'ouvre sur six lignes (un glyphe sur sa couleur, un texte, un sous-texte, une
//! coche). Un logiciel inventé, des valeurs inventées.
//!
//! Aucune fenêtre : l'exemple écrit des captures PNG, en clair et en sombre, à 100 % et à
//! 150 %, et une capture où le menu s'ouvre **au-dessus** de la case (elle est en bas) :
//! `cargo run --example menu_choix [-- <dossier>]` (sans dossier : le dossier temporaire).
//!
//! Dans un vrai logiciel, la case et son menu tiennent en quelques lignes :
//!
//! ```ignore
//! let r = ui.interact(case, id.with("case"), Sense::click());
//! if r.clicked() {
//!     self.menu.basculer(&lignes);
//! }
//! match menu_choix::afficher(ui, p, &mut self.menu, id.with("menu"), case, &lignes, "coché") {
//!     Some(Resultat::Choisi(rang)) => { /* lignes[rang].cle : à enregistrer */ }
//!     Some(Resultat::Ferme) | None => {}
//! }
//! ```

use std::path::PathBuf;

use egui::{Align2, CornerRadius, Id, Rect, Stroke, StrokeKind, pos2, vec2};
use interface_commune::apparence::couleur;
use interface_commune::capture::{self, Options};
use interface_commune::dessin::icone;
use interface_commune::menu_choix::{self, Ligne, MenuChoix};
use interface_commune::pastilles::{couleur_lettre, couleurs_pastilles};
use interface_commune::theme::{TAILLE_PETIT, TAILLE_TEXTE};

/// Les logos de Segoe Fluent Icons (la liste de l'exemple) : fixe, portable, tout-en-un,
/// tablette, boîtier, jeu.
const FIXE: char = '\u{E977}';
const PORTABLE: char = '\u{E7F8}';
const TOUT_EN_UN: char = '\u{E7F4}';
const BOITIER: char = '\u{EC59}';
const LUNE: char = '\u{E708}';

fn lignes() -> Vec<Ligne> {
    let c = couleurs_pastilles();
    let mut fixe = Ligne::new("fixe", "Fixe");
    fixe.logo = Some(FIXE);
    fixe.lettre = Some('F');
    fixe.couleur = Some(c[0]);
    fixe.coche = true;
    let mut portable = Ligne::new("portable", "Portable");
    portable.logo = Some(PORTABLE);
    portable.lettre = Some('P');
    portable.couleur = Some(c[1]);
    portable.sous_texte = Some("Dans la pièce d'à côté".into());
    let mut tout = Ligne::new("tout", "Tout-en-un");
    tout.logo = Some(TOUT_EN_UN);
    tout.lettre = Some('T');
    tout.couleur = Some(c[2]);
    // Un logo que la police n'a pas : la lettre revient (jamais un carré vide).
    let mut manque = Ligne::new("manque", "Logo absent de la police");
    manque.logo = Some('\u{F8FE}');
    manque.lettre = Some('M');
    manque.couleur = Some(c[3]);
    let mut veille = Ligne::new("veille", "Veille");
    veille.logo = Some(LUNE);
    veille.sous_texte = Some("L'écran s'éteint".into());
    let mut hors = Ligne::new("boitier", "Boîtier");
    hors.logo = Some(BOITIER);
    hors.lettre = Some('B');
    hors.couleur = Some(c[4]);
    hors.sous_texte = Some("Hors ligne".into());
    hors.grise = true;
    vec![fixe, portable, tout, manque, veille, hors]
}

/// Une scène : la case d'un écran (un carré coloré, son nom), puis le menu ouvert.
fn scene(
    sombre: bool,
    echelle: f32,
    case_en_bas: bool,
    dossier: &std::path::Path,
    nom: &str,
) -> Result<(), String> {
    let taille = vec2(360.0, 330.0);
    let lignes = lignes();
    let mut menu = MenuChoix::default();
    menu.ouvrir(&lignes);
    let options = Options { sombre, echelle };
    let image = capture::dessin(taille, options, |ui, p| {
        let y = if case_en_bas { taille.y - 52.0 } else { 16.0 };
        let case = Rect::from_min_size(pos2(16.0, y), vec2(190.0, 36.0));
        ui.painter()
            .rect_filled(case, CornerRadius::same(4), couleur(p.controle));
        ui.painter().rect_stroke(
            case,
            CornerRadius::same(4),
            Stroke::new(1.0, couleur(p.bord_controle)),
            StrokeKind::Inside,
        );
        let carre = Rect::from_center_size(pos2(case.left() + 21.0, case.center().y), vec2(26.0, 20.0));
        let fond = couleurs_pastilles()[0];
        ui.painter()
            .rect_filled(carre, CornerRadius::same(4), couleur(fond));
        icone(ui, carre.center(), FIXE, TAILLE_PETIT + 4.0, couleur_lettre(fond));
        ui.painter().text(
            pos2(case.left() + 42.0, case.center().y),
            Align2::LEFT_CENTER,
            "Fixe",
            egui::FontId::proportional(TAILLE_TEXTE),
            couleur(p.texte),
        );
        icone(
            ui,
            pos2(case.right() - 16.0, case.center().y),
            interface_commune::icones::CHEVRON,
            TAILLE_PETIT,
            p.texte_secondaire,
        );
        menu_choix::afficher(ui, p, &mut menu, Id::NULL.with("menu"), case, &lignes, "coché");
    })?;
    let fichier = dossier.join(format!("{nom}.png"));
    image.enregistrer(&fichier)?;
    println!("{}", fichier.display());
    Ok(())
}

fn principal() -> Result<(), String> {
    let dossier = std::env::args_os()
        .nth(1)
        .map_or_else(std::env::temp_dir, PathBuf::from);
    for (sombre, theme) in [(false, "clair"), (true, "sombre")] {
        scene(sombre, 1.0, false, &dossier, &format!("menu-choix-{theme}"))?;
        scene(sombre, 1.5, false, &dossier, &format!("menu-choix-{theme}-150"))?;
    }
    scene(false, 1.0, true, &dossier, "menu-choix-clair-au-dessus")?;
    Ok(())
}

fn main() {
    if let Err(e) = principal() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
