//! « Conserver ces modifications ? » dans les Paramètres, mis en page par egui seul, sans
//! aucune fenêtre (rien ne s'ouvre chez le propriétaire pendant les essais), avec un logiciel
//! inventé : la carte, ses deux boutons, Entrée et Échap, le délai, la fermeture, un brouillon
//! entier, deux changements de suite.

// Dans un essai, un verrou empoisonné doit faire échouer l'essai.
#![allow(clippy::expect_used)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

use interface_commune::parametres::modele::DUREE_CONSERVER;
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::{
    Communs, Identite, Instantane, Page, Palette, Reglages, Textes, Valeur, apparence, polices, textes,
};

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

/// Ce que la page de l'essai doit changer à la prochaine image.
#[derive(Debug, Clone, PartialEq)]
enum Demande {
    /// Le mode d'affichage passe de… à… (à confirmer).
    Mode(&'static str, &'static str),
    /// Un chiffre de plus dans le brouillon (à confirmer).
    Brouillon,
}

/// Un logiciel inventé : un mode d'affichage et un brouillon de chiffres, à confirmer.
struct Ecran {
    identite: Identite,
    anglais: bool,
    mode: Mutex<String>,
    brouillon: Mutex<Vec<i32>>,
    /// Tout ce que la bibliothèque a rendu par `changer`.
    changes: Mutex<Vec<(String, Valeur)>>,
    demande: Mutex<Option<Demande>>,
}

impl Ecran {
    fn new(anglais: bool) -> Ecran {
        Ecran {
            identite: Identite {
                nom: "Démo".into(),
                version: "1.2.3".into(),
                licence: "MIT".into(),
                description: String::new(),
                icone,
                logo: None,
            },
            anglais,
            mode: Mutex::new("étendu".into()),
            brouillon: Mutex::new(vec![1]),
            changes: Mutex::default(),
            demande: Mutex::default(),
        }
    }

    fn mode(&self) -> String {
        self.mode.lock().expect("verrou").clone()
    }

    fn brouillon(&self) -> Vec<i32> {
        self.brouillon.lock().expect("verrou").clone()
    }

    fn demander(&self, d: Demande) {
        *self.demande.lock().expect("verrou") = Some(d);
    }
}

impl Reglages for Ecran {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn textes(&self) -> &Textes {
        if self.anglais {
            &textes::ANGLAIS
        } else {
            &textes::FRANCAIS
        }
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("ecran", "Écran", '\u{E7F4}')]
    }

    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        ui.label("Une page de l'essai");
        let demande = self.demande.lock().expect("verrou").take();
        match demande {
            Some(Demande::Mode(avant, apres)) => {
                *self.mode.lock().expect("verrou") = apres.into();
                page.changer_a_confirmer(
                    "mode",
                    "Mode d'affichage",
                    Valeur::Texte(avant.into()),
                    Valeur::Texte(apres.into()),
                );
            }
            Some(Demande::Brouillon) => {
                self.brouillon.lock().expect("verrou").push(2);
                page.brouillon_a_confirmer("Réglages");
            }
            None => {}
        }
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, cle: &str, valeur: Valeur) {
        if let ("mode", Valeur::Texte(t)) = (cle, &valeur) {
            *self.mode.lock().expect("verrou") = t.clone();
        }
        self.changes.lock().expect("verrou").push((cle.into(), valeur));
    }

    fn instantane(&self) -> Option<Instantane> {
        Some(Instantane::new(self.brouillon()))
    }

    fn remettre(&self, instantane: &Instantane) {
        if let Some(v) = instantane.get::<Vec<i32>>() {
            *self.brouillon.lock().expect("verrou") = v.clone();
        }
    }
}

/// Un banc d'essai : un contexte egui gardé d'une image à l'autre (le focus, les clics),
/// des Paramètres dessinés sans fenêtre.
struct Banc {
    ctx: egui::Context,
    palette: Palette,
}

impl Banc {
    fn new() -> Banc {
        let ctx = egui::Context::default();
        ctx.set_fonts(polices::polices_ou_vides());
        let a = apparence::Apparence::de(Default::default(), Some(false));
        a.appliquer(&ctx);
        Banc {
            ctx,
            palette: a.palette.clone(),
        }
    }

    /// Une image avec ces événements ; rend les textes dessinés (avec leur place).
    fn image(
        &self,
        s: &mut Parametres,
        l: &dyn Reglages,
        evenements: Vec<egui::Event>,
    ) -> Vec<(String, egui::Rect)> {
        let entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(920.0, 680.0),
            )),
            events: evenements,
            ..Default::default()
        };
        let p = self.palette.clone();
        let mut sortie = self.ctx.run_ui(entree, |ui| s.dessiner(ui, &p, l));
        sortie.textures_delta.clear();
        let mut v = Vec::new();
        for c in sortie.shapes {
            textes_de(&c.shape, &mut v);
        }
        v
    }

    /// Plusieurs images sans rien faire (le focus se pose, la mise en page se fixe).
    fn calme(&self, s: &mut Parametres, l: &dyn Reglages) -> Vec<(String, egui::Rect)> {
        let mut v = Vec::new();
        for _ in 0..4 {
            v = self.image(s, l, Vec::new());
        }
        v
    }

    fn touche(&self, s: &mut Parametres, l: &dyn Reglages, key: egui::Key) {
        let evenement = |pressed| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        self.image(s, l, vec![evenement(true), evenement(false)]);
        self.calme(s, l);
    }

    fn clic(&self, s: &mut Parametres, l: &dyn Reglages, au: egui::Pos2) {
        let bouton = |pressed| egui::Event::PointerButton {
            pos: au,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        self.image(s, l, vec![egui::Event::PointerMoved(au)]);
        self.image(s, l, vec![bouton(true)]);
        self.image(s, l, vec![bouton(false)]);
        self.calme(s, l);
    }
}

fn textes_de(s: &egui::Shape, v: &mut Vec<(String, egui::Rect)>) {
    match s {
        egui::Shape::Vec(l) => l.iter().for_each(|s| textes_de(s, v)),
        egui::Shape::Text(t) => v.push((
            t.galley.text().to_string(),
            egui::Rect::from_min_size(t.pos, t.galley.size()),
        )),
        _ => {}
    }
}

fn a_le_texte(v: &[(String, egui::Rect)], cherche: &str) -> bool {
    v.iter().any(|(t, _)| t == cherche)
}

fn place_du_texte(v: &[(String, egui::Rect)], cherche: &str) -> Option<egui::Rect> {
    v.iter().find(|(t, _)| t == cherche).map(|(_, r)| *r)
}

/// Des Paramètres sur la page de l'essai, avec un changement de mode à confirmer déjà fait.
fn ouvert(l: &Ecran, b: &Banc) -> Parametres {
    let mut s = Parametres::new(l, Some("ecran"));
    b.calme(&mut s, l);
    l.demander(Demande::Mode("étendu", "cloné"));
    b.image(&mut s, l, Vec::new());
    s
}

fn mode_a_rendre(l: &Ecran) -> Vec<(String, Valeur)> {
    l.changes.lock().expect("verrou").clone()
}

/// Le changement est appliqué tout de suite ; la carte pose la question, le compte à rebours
/// (15 s) et les deux boutons, en français comme en anglais ; rien de tout cela avant.
#[test]
fn la_carte_pose_la_question() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = Parametres::new(&l, Some("ecran"));
    let avant = b.calme(&mut s, &l);
    assert!(!a_le_texte(&avant, "Conserver ces modifications ?"));
    l.demander(Demande::Mode("étendu", "cloné"));
    let v = b.calme(&mut s, &l);
    // Effet immédiat : le logiciel a le nouveau mode, la carte attend la réponse.
    assert_eq!(l.mode(), "cloné");
    assert!(s.confirmation.en_attente(Instant::now()).is_some());
    // Pas de bulle « Annuler » pour ce changement : la carte la remplace.
    assert!(s.annulation.a_annuler(Instant::now()).is_none());
    if polices::polices().is_err() {
        return;
    }
    for t in [
        "Conserver ces modifications ?",
        "« Mode d'affichage » modifié.",
        "Retour au réglage d'avant dans 15 s",
        "Conserver",
        "Revenir",
    ] {
        assert!(a_le_texte(&v, t), "{t} : {v:?}");
    }
    // En anglais, tous les mots sont anglais.
    let en = Ecran::new(true);
    let mut s = Parametres::new(&en, Some("ecran"));
    b.calme(&mut s, &en);
    en.demander(Demande::Mode("étendu", "cloné"));
    let v = b.calme(&mut s, &en);
    for t in [
        "Keep these changes?",
        "Going back to the previous setting in 15 s",
        "Keep",
        "Revert",
    ] {
        assert!(a_le_texte(&v, t), "{t} : {v:?}");
    }
    assert!(!a_le_texte(&v, "Revenir"));
}

/// Entrée = « Revenir » (le bouton par défaut, focus posé dessus).
#[test]
fn entree_fait_revenir() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = ouvert(&l, &b);
    b.calme(&mut s, &l);
    assert_eq!(l.mode(), "cloné");
    b.touche(&mut s, &l, egui::Key::Enter);
    assert_eq!(l.mode(), "étendu", "Entrée = Revenir");
    assert!(s.confirmation.en_attente(Instant::now()).is_none());
    let v = b.calme(&mut s, &l);
    assert!(
        !a_le_texte(&v, "Conserver ces modifications ?"),
        "la carte a disparu"
    );
    // Revenu par le même chemin qu'« Annuler » : `Reglages::changer`, avec la valeur d'avant.
    assert_eq!(
        mode_a_rendre(&l).last(),
        Some(&("mode".to_string(), Valeur::Texte("étendu".into())))
    );
}

/// Échap = « Revenir ».
#[test]
fn echap_fait_revenir() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = ouvert(&l, &b);
    b.calme(&mut s, &l);
    b.touche(&mut s, &l, egui::Key::Escape);
    assert_eq!(l.mode(), "étendu", "Échap = Revenir");
    assert!(s.confirmation.en_attente(Instant::now()).is_none());
}

/// Un clic sur « Conserver » garde le changement : plus de carte, rien n'est rendu ; un clic
/// sur « Revenir » le défait.
#[test]
fn clic_conserver_et_clic_revenir() {
    if polices::polices().is_err() {
        return;
    }
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = ouvert(&l, &b);
    let v = b.calme(&mut s, &l);
    let conserver = place_du_texte(&v, "Conserver").expect("bouton Conserver");
    let revenir = place_du_texte(&v, "Revenir").expect("bouton Revenir");
    assert!(conserver.center().x < revenir.center().x, "Revenir est à droite");
    let rendus = mode_a_rendre(&l).len();
    b.clic(&mut s, &l, conserver.center());
    assert_eq!(l.mode(), "cloné", "Conserver garde le changement");
    assert!(s.confirmation.en_attente(Instant::now()).is_none());
    assert_eq!(mode_a_rendre(&l).len(), rendus, "rien n'est rendu au logiciel");
    let v = b.calme(&mut s, &l);
    assert!(!a_le_texte(&v, "Conserver ces modifications ?"));
    // Un autre changement, puis « Revenir ».
    l.demander(Demande::Mode("cloné", "second écran seul"));
    let v = b.calme(&mut s, &l);
    assert_eq!(l.mode(), "second écran seul");
    let revenir = place_du_texte(&v, "Revenir").expect("bouton Revenir");
    b.clic(&mut s, &l, revenir.center());
    assert_eq!(l.mode(), "cloné", "Revenir rend la valeur d'avant");
}

/// Sans réponse, au bout de 15 s : retour automatique, une seule fois.
#[test]
fn delai_ecoule_retour_automatique() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = ouvert(&l, &b);
    assert_eq!(DUREE_CONSERVER, Duration::from_secs(15));
    let debut = Instant::now();
    assert!(!s.tic_a(&l, debut + Duration::from_secs(14)));
    assert_eq!(l.mode(), "cloné");
    assert!(s.tic_a(&l, debut + DUREE_CONSERVER + Duration::from_secs(1)));
    assert_eq!(l.mode(), "étendu", "retour automatique");
    assert!(!s.tic_a(&l, debut + Duration::from_secs(60)));
    let rendus = mode_a_rendre(&l).len();
    s.tic_a(&l, debut + Duration::from_secs(90));
    assert_eq!(mode_a_rendre(&l).len(), rendus, "rendu une seule fois");
}

/// Fermer les Paramètres pendant le compte à rebours = « Revenir », avant l'enregistrement du
/// logiciel ; la fenêtre des Paramètres le fait elle-même.
#[test]
fn fermer_les_parametres_fait_revenir() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = ouvert(&l, &b);
    assert_eq!(l.mode(), "cloné");
    s.fermer(&l);
    assert_eq!(l.mode(), "étendu");
    assert!(s.confirmation.en_attente(Instant::now()).is_none());
    // La fenêtre des Paramètres appelle `fermer` à sa fermeture ; rien n'attend : elle passe.
    use interface_commune::autres_fenetres::Contenu;
    let l = std::sync::Arc::new(Ecran::new(false));
    let mut contenu =
        interface_commune::parametres::fenetre::ContenuParametres::new(l, Default::default(), Some("ecran"));
    assert!(contenu.peut_fermer(false));
}

/// Deux changements de suite : une seule carte ; la valeur d'avant le PREMIER revient, avec
/// un autre réglage changé entre-temps.
#[test]
fn deux_changements_de_suite() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = ouvert(&l, &b);
    l.demander(Demande::Mode("cloné", "second écran seul"));
    let v = b.calme(&mut s, &l);
    assert_eq!(l.mode(), "second écran seul");
    if polices::polices().is_ok() {
        assert_eq!(
            v.iter()
                .filter(|(t, _)| t == "Conserver ces modifications ?")
                .count(),
            1,
            "une seule carte"
        );
    }
    b.touche(&mut s, &l, egui::Key::Enter);
    assert_eq!(l.mode(), "étendu", "l'état d'avant le premier changement");
}

/// Un brouillon entier : « Revenir » le remet d'un coup à l'état d'avant, par `remettre`.
#[test]
fn brouillon_a_confirmer() {
    let b = Banc::new();
    let l = Ecran::new(false);
    let mut s = Parametres::new(&l, Some("ecran"));
    b.calme(&mut s, &l);
    l.demander(Demande::Brouillon);
    b.image(&mut s, &l, Vec::new());
    assert_eq!(l.brouillon(), vec![1, 2]);
    assert!(s.confirmation.en_attente(Instant::now()).is_some());
    s.revenir(&l);
    assert_eq!(l.brouillon(), vec![1], "le brouillon d'avant est remis");
    assert!(s.confirmation.en_attente(Instant::now()).is_none());
}
