//! Les fenêtres toutes faites, ouvertes dans la boucle du logiciel ([`AutresFenetres`]) :
//! - la fenêtre « simple » ([`Simple`]) : titre, texte, nouveautés, état, barre
//!   d'avancement, champ, choix, boutons (avec confirmation) ; sa hauteur suit son contenu,
//!   jusqu'à un plafond ([`HAUTEUR_MAX`]) au-delà duquel on défile ;
//! - la fenêtre de lecture ([`Lecture`]) : un texte lu à voix haute, le mot lu surligné et
//!   suivi ; un clic sur un mot le rend au logiciel (reprendre la lecture là).
//!
//! Reprises de Vocalis (`interface/src/simples.rs`), sans son canal : le logiciel décrit la
//! fenêtre par une structure, la change par sa [`Poignee`], et reçoit ce que l'utilisateur fait
//! par un rappel. Les fonctions de dessin ([`dessiner_simple`], [`dessiner_lecture`]) servent
//! aussi seules, dans une fenêtre du logiciel.
//!
//! Le rappel est appelé pendant le dessin de la fenêtre : il peut changer la fenêtre par sa
//! poignée ([`Poignee::changer`], [`Poignee::fermer`]) et ouvrir ou fermer d'autres
//! fenêtres, mais ne doit pas appeler `AutresFenetres::fermer` sur sa propre clé (la poignée
//! le fait sans attente).
//!
//! Refonte, bloc H : le dessin de ces fenêtres vit dans `crate::dessin::toutes_faites`
//! (`simple`, `prete`, `lecture`) et il est repris ici, aux mêmes chemins. Ce module garde ce
//! qui touche à la boucle : la poignée, l'ouverture, le rappel, la hauteur demandée à Windows.

use std::sync::{Arc, Mutex, MutexGuard};

use eframe::egui::{self, Ui, ViewportCommand, vec2};

use crate::Textes;
use crate::apparence::couleur;
use crate::autres_fenetres::{AutresFenetres, Cadre, Contenu, Ouverture};
use crate::place::Place;
use crate::theme::Palette;

pub use crate::dessin::toutes_faites::{
    Bouton, Champ, Choix, Clic, EtatLecture, EtatSimple, HAUTEUR_MAX, LARGEUR_SIMPLE, Lecture, PLUS_TARD,
    Simple, Ton, barre_avancement, dessiner_lecture, dessiner_simple, mots,
};

fn verrou<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ======================================================================================
// La poignée commune
// ======================================================================================

struct Partage<T> {
    donnees: T,
    fermer: bool,
}

type Partagee<T> = Arc<Mutex<Partage<T>>>;

/// Ce qui reste au logiciel d'une fenêtre toute faite ouverte : la changer, la fermer. Se
/// clone et s'utilise depuis n'importe quel fil.
pub struct Poignee<T> {
    partage: Partagee<T>,
    autres: AutresFenetres,
    cle: String,
}

impl<T> Clone for Poignee<T> {
    fn clone(&self) -> Self {
        Poignee {
            partage: self.partage.clone(),
            autres: self.autres.clone(),
            cle: self.cle.clone(),
        }
    }
}

impl<T> Poignee<T> {
    /// Change ce que montre la fenêtre ; elle se redessine.
    pub fn changer(&self, f: impl FnOnce(&mut T)) {
        f(&mut verrou(&self.partage).donnees);
        self.autres.redessiner();
    }

    /// Lit ce que montre la fenêtre.
    pub fn lire<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(&verrou(&self.partage).donnees)
    }

    /// Ferme la fenêtre (à sa prochaine image).
    pub fn fermer(&self) {
        verrou(&self.partage).fermer = true;
        self.autres.redessiner();
    }

    /// La fenêtre est-elle encore ouverte ?
    pub fn ouverte(&self) -> bool {
        !verrou(&self.partage).fermer && self.autres.ouverte(&self.cle)
    }

    pub fn cle(&self) -> &str {
        &self.cle
    }
}

fn nouvelle<T>(autres: &AutresFenetres, cle: &str, donnees: T) -> Poignee<T> {
    Poignee {
        partage: Arc::new(Mutex::new(Partage {
            donnees,
            fermer: false,
        })),
        autres: autres.clone(),
        cle: cle.into(),
    }
}

// ======================================================================================
// La fenêtre simple
// ======================================================================================

/// Ce que l'utilisateur fait d'une fenêtre simple.
#[derive(Debug, Clone, PartialEq)]
pub enum Reponse {
    Clic(Clic),
    /// La fenêtre s'est fermée (croix, Échap, ou la poignée).
    Fermee,
}

pub type FenetreSimple = Poignee<Simple>;

struct ContenuSimple {
    partage: Partagee<Simple>,
    textes: Textes,
    etat: EtatSimple,
    rappel: Box<dyn FnMut(Reponse) + Send>,
    /// La hauteur demandée à Windows (points).
    hauteur_posee: f32,
}

impl Contenu for ContenuSimple {
    fn dessiner(&mut self, ui: &mut Ui, cadre: &mut Cadre<'_>) {
        let p = cadre.palette;
        let echap = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let (clic, largeur) = {
            let g = verrou(&self.partage);
            if g.fermer {
                cadre.fermer();
                return;
            }
            let largeur = g.donnees.largeur_fenetre();
            let mut clic = None;
            egui::Frame::new().fill(couleur(p.fond)).show(ui, |ui| {
                clic = dessiner_simple(ui, p, &self.textes, &g.donnees, &mut self.etat);
            });
            (clic, largeur)
        };
        if echap && !self.etat.abandonner() {
            cadre.fermer();
        }
        let h = self.etat.hauteur;
        if h > 0.0 && (h - self.hauteur_posee).abs() > 1.0 {
            // La fenêtre prend la hauteur de son contenu.
            self.hauteur_posee = h;
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::InnerSize(vec2(largeur, h)));
        }
        if let Some(c) = clic {
            (self.rappel)(Reponse::Clic(c));
        }
    }

    fn fermee(&mut self, _place: Option<Place>) {
        verrou(&self.partage).fermer = true;
        (self.rappel)(Reponse::Fermee);
    }
}

/// Ouvre une fenêtre simple parmi les fenêtres de la boucle. `ouverture` donne son titre (la
/// barre de titre) et son icône ; sa taille suit le contenu. `None` si une fenêtre de cette
/// clé est déjà ouverte (elle passe devant, inchangée).
pub fn ouvrir_simple(
    autres: &AutresFenetres,
    cle: &str,
    ouverture: Ouverture,
    textes: &Textes,
    simple: Simple,
    rappel: impl FnMut(Reponse) + Send + 'static,
) -> Option<FenetreSimple> {
    let mut o = ouverture.fixe();
    o.taille = [simple.largeur_fenetre(), 160.0];
    let poignee = nouvelle(autres, cle, simple);
    let contenu = ContenuSimple {
        partage: poignee.partage.clone(),
        textes: textes.clone(),
        etat: EtatSimple::default(),
        rappel: Box::new(rappel),
        hauteur_posee: 0.0,
    };
    autres.ouvrir(cle, o, contenu).then_some(poignee)
}

// ======================================================================================
// La fenêtre de lecture
// ======================================================================================

/// Ce que l'utilisateur fait de la fenêtre de lecture.
#[derive(Debug, Clone, PartialEq)]
pub enum ReponseLecture {
    /// Un clic sur ce mot (son rang) : reprendre la lecture là.
    Mot(usize),
    /// La fenêtre s'est fermée (croix, Échap, ou la poignée).
    Fermee,
}

pub type FenetreLecture = Poignee<Lecture>;

/// Une barre au-dessus du texte, dessinée par le logiciel (pause, vitesse, voix…).
pub type BarreLecture = Box<dyn FnMut(&mut Ui, &Palette) + Send>;

struct ContenuLecture {
    partage: Partagee<Lecture>,
    etat: EtatLecture,
    barre: Option<BarreLecture>,
    rappel: Box<dyn FnMut(ReponseLecture) + Send>,
}

impl Contenu for ContenuLecture {
    fn dessiner(&mut self, ui: &mut Ui, cadre: &mut Cadre<'_>) {
        let p = cadre.palette;
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            cadre.fermer();
        }
        let mut clic = None;
        egui::Frame::new().fill(couleur(p.fond)).show(ui, |ui| {
            ui.set_min_size(ui.available_size());
            if let Some(b) = self.barre.as_mut() {
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_centered(|ui| b(ui, p));
                    });
                ui.separator();
            }
            let g = verrou(&self.partage);
            if g.fermer {
                cadre.fermer();
                return;
            }
            clic = dessiner_lecture(ui, p, &g.donnees, &mut self.etat);
        });
        if let Some(m) = clic {
            (self.rappel)(ReponseLecture::Mot(m));
        }
    }

    fn fermee(&mut self, _place: Option<Place>) {
        verrou(&self.partage).fermer = true;
        (self.rappel)(ReponseLecture::Fermee);
    }
}

/// Ouvre la fenêtre de lecture parmi les fenêtres de la boucle. `None` si une fenêtre de
/// cette clé est déjà ouverte (elle passe devant, inchangée).
pub fn ouvrir_lecture(
    autres: &AutresFenetres,
    cle: &str,
    ouverture: Ouverture,
    lecture: Lecture,
    barre: Option<BarreLecture>,
    rappel: impl FnMut(ReponseLecture) + Send + 'static,
) -> Option<FenetreLecture> {
    let poignee = nouvelle(autres, cle, lecture);
    let contenu = ContenuLecture {
        partage: poignee.partage.clone(),
        etat: EtatLecture::default(),
        barre,
        rappel: Box::new(rappel),
    };
    autres.ouvrir(cle, ouverture, contenu).then_some(poignee)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::autres_fenetres::essais::{CROIX, ORDRES, contexte, image};
    use crate::autres_fenetres::id_de;
    use crate::dessin::toutes_faites::essais::simple;
    use crate::textes::FRANCAIS;

    /// Une fenêtre simple et une fenêtre de lecture s'ouvrent dans la boucle, se changent et
    /// se ferment par leur poignée ou par la croix ; le rappel l'apprend. La fenêtre simple
    /// demande la hauteur de son contenu.
    #[test]
    fn fenetres_toutes_faites_dans_la_boucle() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        let recu = Arc::new(Mutex::new(Vec::<String>::new()));
        let r1 = recu.clone();
        let simple = ouvrir_simple(
            &f,
            "simple",
            Ouverture::new("Démo", [0.0, 0.0]),
            &FRANCAIS,
            simple(),
            move |r| r1.lock().unwrap().push(format!("{r:?}")),
        )
        .expect("ouverte");
        let r2 = recu.clone();
        let lecture = ouvrir_lecture(
            &f,
            "lecture",
            Ouverture::new("Lecture", [500.0, 400.0]),
            Lecture::de("Un texte inventé."),
            Some(Box::new(|ui: &mut Ui, _: &Palette| {
                ui.label("Pause");
            })),
            move |r| r2.lock().unwrap().push(format!("{r:?}")),
        )
        .expect("ouverte");
        assert!(
            ouvrir_lecture(
                &f,
                "lecture",
                Ouverture::new("Lecture", [500.0, 400.0]),
                Lecture::default(),
                None,
                |_| {}
            )
            .is_none()
        );
        let (d, _) = image(&ctx, &f);
        assert_eq!(d.len(), 2);
        let _ = image(&ctx, &f);
        let largeur = simple.lire(|s| s.largeur_fenetre());
        assert_eq!(largeur, LARGEUR_SIMPLE + 48.0);
        let tailles: Vec<egui::Vec2> = ORDRES.with(|o| {
            o.borrow()
                .iter()
                .filter_map(|(id, c)| match c {
                    ViewportCommand::InnerSize(t) if *id == id_de("simple") => Some(*t),
                    _ => None,
                })
                .collect()
        });
        let t = tailles
            .last()
            .expect("la fenêtre prend la hauteur de son contenu");
        assert_eq!(t.x, largeur);
        assert!(t.y > 100.0 && t.y < HAUTEUR_MAX, "{t:?}");
        lecture.changer(|l| l.mot = Some(1));
        simple.changer(|s| s.barre = Some(1.0));
        simple.fermer();
        assert!(!simple.ouverte());
        let _ = image(&ctx, &f);
        assert_eq!(f.cles(), vec!["lecture".to_string()]);
        CROIX.with(|c| c.borrow_mut().push(id_de("lecture")));
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().clear());
        assert!(f.cles().is_empty());
        assert!(!lecture.ouverte());
        assert_eq!(*recu.lock().unwrap(), vec!["Fermee", "Fermee"]);
    }
}
