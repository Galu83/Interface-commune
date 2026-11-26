//! Les Paramètres mis en page par egui seul, sans aucune fenêtre (rien ne s'ouvre chez le
//! propriétaire pendant les essais), avec un logiciel inventé. Le volet et le petit menu sont
//! dans `tests/sans_fenetre.rs` ; ce qu'ils partagent, dans `tests/sans_fenetre/commun.rs`.

// Dans un essai, un verrou empoisonné doit faire échouer l'essai.
#![allow(clippy::expect_used)]

// Le logiciel inventé et les images sans fenêtre, partagés avec `sans_fenetre.rs` : chaque
// fichier d'essais n'en prend qu'une partie, le reste n'y sert pas.
#[allow(dead_code)]
#[path = "sans_fenetre/commun.rs"]
mod commun;

use commun::*;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::{Communs, EtatMiseAJour, Identite, Page, Reglages, Textes, Valeur, dessin, polices};

/// Chaque page des Paramètres se dessine sans fenêtre ; l'en-tête montre le nom et
/// « Version x.y.z » ; une page qui demande un redémarrage montre le bandeau.
#[test]
fn parametres_sans_fenetre() {
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("mise_a_jour"));
    assert_eq!(s.cle(), "mise_a_jour");
    let cles: Vec<String> = s.pages.iter().map(|p| p.cle.clone()).collect();
    assert_eq!(
        cles,
        [
            "accueil",
            "ecrans",
            "dispositions",
            "general",
            "mise_a_jour",
            "a_propos"
        ]
    );
    for cle in &cles {
        s.aller_a(cle);
        let textes = image(egui::vec2(920.0, 680.0), 1.0, |ui, p| s.dessiner(ui, p, &demo));
        if polices::polices().is_ok() {
            let tous: Vec<&str> = textes.iter().map(|(t, _)| t.as_str()).collect();
            assert!(tous.contains(&"Démo"), "{cle} : {tous:?}");
            assert!(tous.contains(&"Version 1.2.3"), "{cle} : {tous:?}");
            assert!(tous.contains(&"Mise à jour"), "{cle} : {tous:?}");
        }
    }
    assert!(s.redemarrage.is_some(), "le bandeau de redémarrage est demandé");
}

/// Effet immédiat : un changement part tout de suite au logiciel ; « Annuler » lui rend la
/// valeur d'avant.
#[test]
fn effet_immediat_et_annuler_vers_le_logiciel() {
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, None);
    s.changer(
        &demo,
        "general.demarrage",
        "Démarrer avec Windows",
        Valeur::Booleen(true),
        Valeur::Booleen(false),
    );
    assert_eq!(
        demo.changes.lock().expect("verrou").last(),
        Some(&("general.demarrage".to_string(), Valeur::Booleen(false)))
    );
    for (cle, avant) in s.annulation.annuler(std::time::Instant::now()) {
        demo.changer(&cle, avant);
    }
    assert_eq!(
        demo.changes.lock().expect("verrou").last(),
        Some(&("general.demarrage".to_string(), Valeur::Booleen(true)))
    );
    assert!(demo.actions.lock().expect("verrou").is_empty());
}

/// Un logiciel qui ne prend que les Paramètres (trait `Reglages` seul), en anglais : tous
/// les mots communs sont en anglais, aucun texte français imposé.
#[test]
fn parametres_seuls_en_anglais() {
    let seul = Seul { identite: identite() };
    assert_eq!(Parametres::titre_fenetre(&seul), "Démo Settings");
    let mut s = Parametres::new(&seul, None);
    let titres: Vec<&str> = s.pages.iter().map(|p| p.titre.as_str()).collect();
    assert_eq!(titres, ["General", "Update", "About"]);
    for cle in ["general", "mise_a_jour", "a_propos"] {
        s.aller_a(cle);
        let v = image_parametres(&mut s, &seul);
        if polices::polices().is_err() {
            return;
        }
        let tous: Vec<&str> = v
            .iter()
            .filter_map(|d| match d {
                Dessine::Texte { texte, .. } => Some(texte.as_str()),
                _ => None,
            })
            .collect();
        for francais in [
            "Général",
            "Mise à jour",
            "À propos",
            "Ouvrir le journal",
            "Rechercher une mise à jour",
            "Dossier des installateurs",
        ] {
            assert!(!tous.contains(&francais), "{cle} : « {francais} » dans {tous:?}");
        }
        assert!(tous.contains(&"Version 1.2.3"), "{cle} : {tous:?}");
    }
}

/// L'en-tête rend celui de PowerPeek : le vrai logo (une image de 32 points), le nom en 16
/// points, la version en 12 ; des lignes de navigation de 38 points ; Général, Mise à jour et
/// À propos juste après les pages du logiciel, 6 points plus bas, jamais au bas de la
/// fenêtre ; une colonne de 250 points au moins, plus large pour un nom de page long.
#[test]
fn entete_et_navigation_comme_powerpeek() {
    if polices::polices().is_err() {
        return;
    }
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("ecrans"));
    let v = image_parametres(&mut s, &demo);
    let logo = v
        .iter()
        .find_map(|d| match d {
            Dessine::Image(r) => Some(*r),
            _ => None,
        })
        .expect("le logo est une image");
    assert!(
        (logo.width() - 32.0).abs() < 0.5 && (logo.height() - 32.0).abs() < 0.5,
        "{logo:?}"
    );
    let (nom, taille_nom) = texte(&v, "Démo").expect("le nom");
    assert_eq!(taille_nom, 16.0);
    assert!(nom.left() > logo.right(), "le nom est à droite du logo");
    let (version, taille_version) = texte(&v, "Version 1.2.3").expect("la version");
    assert_eq!(taille_version, 12.0);
    assert!(version.top() >= nom.bottom() - 1.0, "la version est sous le nom");
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    let pas = y("Dispositions") - y("Écrans");
    let espacement = 2.0 + 8.0;
    assert!(
        (pas - 38.0 - espacement).abs() < 0.6,
        "pas entre deux lignes : {pas}"
    );
    let saut = y("Général") - y("Dispositions");
    assert!(
        (saut - pas - 6.0).abs() < 0.6,
        "6 points avant Général : {saut} / {pas}"
    );
    assert!((y("Mise à jour") - y("Général") - pas).abs() < 0.6);
    assert!(
        (y("À propos") - y("Mise à jour") - pas).abs() < 0.6 && y("À propos") < 680.0 - 150.0,
        "les pages communes ne sont pas au bas"
    );
    let largeur = |s: &Parametres, l: &dyn Reglages| {
        let ctx = egui::Context::default();
        ctx.set_fonts(polices::polices_ou_vides());
        let mut w = 0.0;
        for _ in 0..2 {
            let mut o = ctx.run_ui(egui::RawInput::default(), |ui| w = s.largeur_navigation(ui, l));
            o.textures_delta.clear();
        }
        w
    };
    assert_eq!(
        largeur(&s, &demo),
        interface_commune::parametres::LARGEUR_NAVIGATION_MIN
    );
    let longue = Longue(Demo::new());
    let s2 = Parametres::new(&longue, None);
    let w = largeur(&s2, &longue);
    assert!(
        w > interface_commune::parametres::LARGEUR_NAVIGATION_MIN
            && w <= interface_commune::parametres::LARGEUR_NAVIGATION_MAX,
        "{w}"
    );
}

/// Un logiciel aux noms de pages longs.
struct Longue(Demo);

impl Reglages for Longue {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new(
            "longue",
            "Une page au nom vraiment très, très long",
            '\u{E7F4}',
        )]
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// La langue change pendant que les Paramètres sont ouverts : la navigation suit, sur la
/// même page.
#[test]
fn la_langue_change_en_direct() {
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("a_propos"));
    assert_eq!(s.pages[s.page].titre, "À propos");
    demo.anglais.store(true, Ordering::Relaxed);
    let _ = image_parametres(&mut s, &demo);
    assert_eq!(s.cle(), "a_propos");
    assert_eq!(s.pages[s.page].titre, "About");
    assert_eq!(Parametres::titre_fenetre(&demo), "Démo Settings");
}

/// La page Général montre les cartes propres du logiciel, après les réglages communs et
/// avant « Journal » ; un dossier fixe n'a pas de bouton « Choisir… ».
#[test]
fn carte_propre_dans_general_et_dossier_fixe() {
    if polices::polices().is_err() {
        return;
    }
    let demo = Demo::new();
    let mut s = Parametres::new(&demo, Some("general"));
    let v = image_parametres(&mut s, &demo);
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    assert!(y("Notifications") < y("Carte propre"));
    assert!(y("Carte propre") < y("Journal"));
    s.aller_a("mise_a_jour");
    let v = image_parametres(&mut s, &demo);
    assert!(texte(&v, "Choisir…").is_some());
    let fixe = DossierFixe(Demo::new());
    let mut s = Parametres::new(&fixe, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &fixe);
    assert!(texte(&v, "Dossier des installateurs").is_some());
    assert!(texte(&v, "Choisir…").is_none());
}

/// Un logiciel dont le dossier des installateurs ne se choisit pas.
struct DossierFixe(Demo);

impl Reglages for DossierFixe {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        Communs {
            dossier_fixe: true,
            ..self.0.communs()
        }
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Un point d'alerte sur la ligne « Mise à jour » quand une version est prête (par défaut),
/// sur une page propre quand le logiciel le demande, nulle part ailleurs.
#[test]
fn point_d_alerte_dans_la_navigation() {
    if polices::polices().is_err() {
        return;
    }
    let ronds = |l: &dyn Reglages| {
        let mut s = Parametres::new(l, None);
        let v = image_parametres(&mut s, l);
        let lignes: Vec<(String, f32)> = ["Écrans", "Dispositions", "Général", "Mise à jour", "À propos"]
            .iter()
            .map(|t| (t.to_string(), texte(&v, t).expect(t).0.center().y))
            .collect();
        v.iter()
            .filter_map(|d| match d {
                Dessine::Rond(c, r) if (*r - interface_commune::parametres::RAYON_ALERTE).abs() < 0.01 => {
                    lignes
                        .iter()
                        .find(|(_, y)| (c.y - y).abs() < 4.0)
                        .map(|(t, _)| t.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    let demo = Demo::new();
    assert_eq!(ronds(&demo), ["Mise à jour"]);
    let alerte = Alerte(Demo::new());
    assert_eq!(ronds(&alerte), ["Dispositions"]);
}

/// Un logiciel qui signale une de ses pages (et pas la mise à jour).
struct Alerte(Demo);

impl Reglages for Alerte {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        self.0.pages()
    }

    fn communs(&self) -> Communs {
        self.0.communs()
    }

    fn alerte(&self, cle: &str) -> bool {
        cle == "dispositions"
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Un logiciel qui change les pages communes : ses propres `Communs`, sa phrase pour
/// « Rétablir », et une carte propre dans Mise à jour et dans À propos.
struct Variante(Demo, Communs);

impl Reglages for Variante {
    fn identite(&self) -> &Identite {
        &self.0.identite
    }

    fn action(&self, cle: &str) {
        self.0.action(cle);
    }

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        self.1.clone()
    }

    fn changer(&self, cle: &str, valeur: Valeur) {
        self.0.changer(cle, valeur);
    }

    fn dessiner_mise_a_jour(&self, ui: &mut egui::Ui, page: &mut Contexte) {
        dessin::carte_info(ui, page.palette, '\u{E7F4}', "Carte de mise à jour", "Inventée.");
    }

    fn dessiner_a_propos(&self, ui: &mut egui::Ui, page: &mut Contexte) {
        dessin::carte_info(ui, page.palette, '\u{E7F4}', "Carte à propos", "Inventée.");
        // Un groupe de changements d'un seul geste : une seule bulle « Annuler ».
        if page.sous_page.is_none() {
            page.changer_groupe(
                "Disposition",
                vec![
                    ("gauche".into(), Valeur::Nombre(1.0), Valeur::Nombre(2.0)),
                    ("droite".into(), Valeur::Nombre(3.0), Valeur::Nombre(4.0)),
                ],
            );
            page.ouvrir_sous_page(Some("Fait"));
        }
    }

    fn retablir_explication(&self) -> Option<String> {
        Some("Les dispositions inventées sont effacées.".into())
    }
}

/// Démarrage grisé avec sa raison, et la phrase de « Rétablir » fournie par le logiciel.
#[test]
fn demarrage_grise_et_retablir_du_logiciel() {
    if polices::polices().is_err() {
        return;
    }
    let base = Demo::new().communs();
    let l = Variante(
        Demo::new(),
        Communs {
            demarrage_bloque: Some("Refusé par une règle inventée.".into()),
            ..base
        },
    );
    let mut s = Parametres::new(&l, Some("general"));
    let v = image_parametres(&mut s, &l);
    assert!(texte(&v, "Refusé par une règle inventée.").is_some());
    assert!(texte(&v, Textes::default().demarrer_explication).is_none());
    s.confirmer_retablir = true;
    let v = image_parametres(&mut s, &l);
    assert!(texte(&v, "Les dispositions inventées sont effacées.").is_some());
}

/// Journal : l'interrupteur « Journal détaillé » et « Effacer le journal » ne se montrent que
/// si le logiciel les donne ; l'effacement passe par une question de confirmation.
#[test]
fn journal_detaille_et_effacer_avec_confirmation() {
    if polices::polices().is_err() {
        return;
    }
    let t = Textes::default();
    let base = Demo::new().communs();
    let sans = Variante(
        Demo::new(),
        Communs {
            journal_detaille: None,
            journal_effacable: false,
            ..base.clone()
        },
    );
    let mut s = Parametres::new(&sans, Some("general"));
    let v = image_parametres(&mut s, &sans);
    assert!(texte(&v, t.journal_detaille).is_none());
    assert!(texte(&v, t.effacer_le_journal).is_none());
    let avec = Variante(
        Demo::new(),
        Communs {
            journal_detaille: Some(false),
            journal_effacable: true,
            ..base
        },
    );
    let mut s = Parametres::new(&avec, Some("general"));
    let v = image_parametres(&mut s, &avec);
    assert!(texte(&v, t.journal_detaille).is_some());
    assert!(texte(&v, t.effacer_le_journal).is_some());
    assert!(texte(&v, t.oui_effacer).is_none());
    s.confirmer_effacer = true;
    let v = image_parametres(&mut s, &avec);
    assert!(texte(&v, t.effacer_journal_question).is_some());
    assert!(texte(&v, t.oui_effacer).is_some());
}

/// Carte « Journal », journal effaçable, en français : les deux boutons (« Ouvrir le journal »,
/// « Effacer le journal ») ne recouvrent jamais l'explication. La place réservée à droite se
/// mesure sur les boutons réels, pas sur une constante.
#[test]
fn boutons_du_journal_ne_recouvrent_pas_l_explication() {
    if polices::polices().is_err() {
        return;
    }
    let t = Textes::default();
    let l = Variante(
        Demo::new(),
        Communs {
            journal_effacable: true,
            ..Demo::new().communs()
        },
    );
    let mut s = Parametres::new(&l, Some("general"));
    let v = image_parametres(&mut s, &l);
    let explication = texte_souple(&v, t.journal_explication).expect("l'explication du journal");
    for nom in [t.ouvrir_le_journal, t.effacer_le_journal] {
        let (texte_bouton, _) = texte(&v, nom).expect("le bouton");
        // Le texte est centré dans le bouton, 14 points de marge de chaque côté (80 au moins).
        let bord = texte_bouton.center().x - (texte_bouton.width() + 28.0).max(80.0) / 2.0;
        assert!(
            explication.max.x <= bord + 0.5,
            "« {nom} » recouvre l'explication ({explication:?}, bord du bouton à {bord})"
        );
        assert!(
            (texte_bouton.center().y - explication.center().y).abs() < 48.0,
            "« {nom} » sur la même carte"
        );
    }
}

/// Une carte sans explication (« Chercher les mises à jour ») : le titre seul est centré sur la
/// hauteur de la carte, à la hauteur de l'interrupteur, et non collé en haut.
#[test]
fn titre_seul_centre_dans_sa_carte() {
    if polices::polices().is_err() {
        return;
    }
    let t = Textes::default();
    // Recherches actives : la carte n'a pas d'explication.
    let l = Variante(
        Demo::new(),
        Communs {
            recherches: Some(true),
            ..Demo::new().communs()
        },
    );
    let mut s = Parametres::new(&l, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &l);
    let (titre, _) = texte(&v, t.chercher_mises_a_jour).expect("le titre seul");
    let etat = v
        .iter()
        .filter_map(|d| match d {
            Dessine::Texte { texte, rect, .. } if texte == t.active || texte == t.desactive => Some(*rect),
            _ => None,
        })
        .min_by(|a, b| {
            let da = (a.center().y - titre.center().y).abs();
            let db = (b.center().y - titre.center().y).abs();
            da.total_cmp(&db)
        })
        .expect("l'état de l'interrupteur");
    assert!(
        (titre.center().y - etat.center().y).abs() <= 1.5,
        "titre seul pas centré : {titre:?} contre l'interrupteur {etat:?}"
    );
}

/// Notifications : le bouton « Essayer » ne se montre que si le logiciel le demande, et
/// seulement avec la carte « Notifications » ; sans lui, la page ne change pas.
#[test]
fn notifications_essayer_seulement_si_demande() {
    if polices::polices().is_err() {
        return;
    }
    let t = Textes::default();
    let base = Demo::new().communs();
    let essai = |notifications: Option<bool>, notifications_essai: bool| {
        let l = Variante(
            Demo::new(),
            Communs {
                notifications,
                notifications_essai,
                ..base.clone()
            },
        );
        let mut s = Parametres::new(&l, Some("general"));
        let v = image_parametres(&mut s, &l);
        (
            texte(&v, t.notifications).is_some(),
            texte(&v, t.essayer).is_some(),
        )
    };
    assert_eq!(essai(Some(false), false), (true, false));
    assert_eq!(essai(Some(false), true), (true, true));
    assert_eq!(essai(Some(true), true), (true, true));
    assert_eq!(essai(None, true), (false, false));
}

/// Mise à jour : l'interrupteur des recherches, « Automatique », la carte propre après le
/// dossier ; recherches coupées : une information, ni recherche ni dossier.
#[test]
fn mise_a_jour_recherches_et_dossier_automatique() {
    if polices::polices().is_err() {
        return;
    }
    let base = Demo::new().communs();
    let l = Variante(
        Demo::new(),
        Communs {
            recherches: Some(true),
            dossier_automatique: Some(false),
            mise_a_jour: EtatMiseAJour::Information("Dossier introuvable.".into()),
            ..base.clone()
        },
    );
    let mut s = Parametres::new(&l, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &l);
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    assert!(y("Chercher les mises à jour") < y("Dossier des installateurs"));
    assert!(y("Dossier des installateurs") < y("Carte de mise à jour"));
    assert!(texte(&v, "Automatique").is_some());
    assert!(texte(&v, "Dossier introuvable.").is_some());
    assert!(texte(&v, "Rechercher une mise à jour").is_some());
    let coupee = Variante(
        Demo::new(),
        Communs {
            recherches: Some(false),
            mise_a_jour: EtatMiseAJour::Coupee,
            ..base
        },
    );
    let mut s = Parametres::new(&coupee, Some("mise_a_jour"));
    let v = image_parametres(&mut s, &coupee);
    assert!(texte(&v, "Recherche des mises à jour coupée").is_some());
    assert!(texte(&v, "Rechercher une mise à jour").is_none());
    assert!(texte(&v, "Dossier des installateurs").is_none());
    // L'explication passe par les espaces insécables (« Coupé : ») : on cherche sa fin.
    assert!(v.iter().any(|d| matches!(
        d,
        Dessine::Texte { texte, .. } if texte.ends_with("ne cherche pas de nouvelle version.")
    )));
}

/// À propos : « Voir les licences », la carte propre avant les crédits, un long crédit qui
/// passe à la ligne, et un groupe de changements défait d'un seul « Annuler ».
#[test]
fn a_propos_licences_credits_et_groupe() {
    if polices::polices().is_err() {
        return;
    }
    let base = Demo::new().communs();
    let long = "Une bibliothèque inventée au nom vraiment très long, bien plus large que la page".repeat(2);
    let l = Variante(
        Demo::new(),
        Communs {
            fichier_licences: true,
            credits: vec![interface_commune::Credit {
                nom: long.clone(),
                licence: "MIT".into(),
            }],
            ..base
        },
    );
    let mut s = Parametres::new(&l, Some("a_propos"));
    let v = image_parametres(&mut s, &l);
    let y = |t: &str| texte(&v, t).expect(t).0.center().y;
    assert!(y("Licence") < y("Carte à propos"));
    assert!(y("Carte à propos") < y("Crédits"));
    assert!(texte(&v, "Voir les licences").is_some());
    let (credit, _) = texte(&v, &long).expect("crédit");
    assert!(credit.right() <= 920.0, "le crédit déborde : {credit:?}");
    // Les deux changements du groupe sont partis au logiciel ; « Annuler » remet les deux.
    let faits = l.0.changes.lock().expect("verrou").clone();
    assert!(faits.contains(&("gauche".into(), Valeur::Nombre(2.0))));
    assert!(faits.contains(&("droite".into(), Valeur::Nombre(4.0))));
    let defaits = s.annulation.annuler(std::time::Instant::now());
    assert_eq!(
        defaits,
        vec![
            ("gauche".to_string(), Valeur::Nombre(1.0)),
            ("droite".to_string(), Valeur::Nombre(3.0))
        ]
    );
}

/// Un logiciel à brouillon : ses réglages se modifient d'un bloc (un `Vec`), avec un pied de
/// page et une fermeture qui peut être refusée.
struct AvecBrouillon {
    identite: Identite,
    brouillon: Mutex<Vec<i32>>,
    /// La page change le brouillon à la prochaine image (une seule fois).
    modifier: AtomicBool,
    enregistrement_ok: AtomicBool,
    fermetures: Mutex<Vec<bool>>,
}

impl Reglages for AvecBrouillon {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn pages(&self) -> Vec<Page> {
        vec![Page::new("p", "Page du brouillon", '\u{E7F4}')]
    }

    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, page: &mut Contexte) {
        ui.label("Contenu du brouillon");
        if self.modifier.swap(false, Ordering::Relaxed) {
            self.brouillon.lock().expect("verrou").push(2);
            page.brouillon_modifie("Réglages");
        }
    }

    fn dessiner_pied(&self, ui: &mut egui::Ui, _page: &mut Contexte) {
        ui.label("Réglage pas enregistré");
    }

    fn instantane(&self) -> Option<interface_commune::Instantane> {
        Some(interface_commune::Instantane::new(
            self.brouillon.lock().expect("verrou").clone(),
        ))
    }

    fn remettre(&self, instantane: &interface_commune::Instantane) {
        if let Some(v) = instantane.get::<Vec<i32>>() {
            self.brouillon.lock().expect("verrou").clone_from(v);
        }
    }

    fn peut_fermer(&self, imposee: bool) -> bool {
        self.fermetures.lock().expect("verrou").push(imposee);
        self.enregistrement_ok.load(Ordering::Relaxed)
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Un brouillon entier : la page le modifie, la bulle « Annuler » le remet d'un coup à l'état
/// d'avant l'image ; le pied de page est sous toutes les pages, communes comprises ; la
/// fermeture passe par le logiciel, qui peut la refuser.
#[test]
fn brouillon_entier_pied_de_page_et_fermeture() {
    if polices::polices().is_err() {
        return;
    }
    let l = AvecBrouillon {
        identite: identite(),
        brouillon: Mutex::new(vec![1]),
        modifier: AtomicBool::new(true),
        enregistrement_ok: AtomicBool::new(false),
        fermetures: Mutex::default(),
    };
    let mut s = Parametres::new(&l, Some("p"));
    let v = image_parametres(&mut s, &l);
    assert!(texte(&v, "Contenu du brouillon").is_some());
    assert!(
        texte(&v, "Réglage pas enregistré").is_some(),
        "pied sous une page propre"
    );
    assert_eq!(*l.brouillon.lock().expect("verrou"), vec![1, 2]);
    // La bulle est celle du brouillon : seul `annuler_brouillon` la défait, d'un coup.
    assert!(s.annulation.annuler(std::time::Instant::now()).is_empty());
    let avant = s
        .annulation
        .annuler_brouillon(std::time::Instant::now())
        .expect("bulle du brouillon");
    l.remettre(&avant);
    assert_eq!(*l.brouillon.lock().expect("verrou"), vec![1]);
    // Le pied est aussi sous les pages communes.
    let mut g = Parametres::new(&l, Some("general"));
    let v = image_parametres(&mut g, &l);
    assert!(texte(&v, "Réglage pas enregistré").is_some(), "pied sous Général");
    // Fermeture : refusée tant que l'enregistrement échoue, acceptée ensuite.
    let mut contenu = interface_commune::parametres::fenetre::ContenuParametres::new(
        std::sync::Arc::new(AvecBrouillon {
            identite: identite(),
            brouillon: Mutex::default(),
            modifier: AtomicBool::new(false),
            enregistrement_ok: AtomicBool::new(false),
            fermetures: Mutex::default(),
        }),
        Default::default(),
        None,
    );
    use interface_commune::autres_fenetres::Contenu;
    assert!(!contenu.peut_fermer(false), "refusée");
    assert!(
        !contenu.peut_fermer(true),
        "le refus est rendu tel quel, la fenêtre l'ignore"
    );
}

/// Un logiciel dont les pages arrivent après l'ouverture des Paramètres (cas d'une interface
/// qui reçoit sa description d'un autre processus).
struct Tardif {
    identite: Identite,
    accueil: AtomicBool,
    pages: Mutex<Vec<Page>>,
}

impl Reglages for Tardif {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn accueil(&self) -> bool {
        self.accueil.load(Ordering::Relaxed)
    }

    fn pages(&self) -> Vec<Page> {
        self.pages.lock().expect("verrou").clone()
    }

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

/// Des pages qui arrivent après l'ouverture apparaissent dans la navigation à l'image
/// suivante ; la page choisie est gardée ; une page qui disparaît ramène à Général.
#[test]
fn pages_arrivees_apres_l_ouverture() {
    let l = Tardif {
        identite: identite(),
        accueil: AtomicBool::new(false),
        pages: Mutex::new(Vec::new()),
    };
    let cles = |s: &Parametres| s.pages.iter().map(|p| p.cle.clone()).collect::<Vec<_>>();
    let mut s = Parametres::new(&l, None);
    assert_eq!(cles(&s), ["general", "mise_a_jour", "a_propos"]);
    s.aller_a("a_propos");

    // Les pages arrivent : la navigation les montre, « À propos » reste choisie.
    l.accueil.store(true, Ordering::Relaxed);
    *l.pages.lock().expect("verrou") = vec![
        Page::new("voix", "Voix", '\u{E720}'),
        Page::new("lecture", "Lecture", '\u{E7BE}'),
    ];
    image_parametres(&mut s, &l);
    assert_eq!(
        cles(&s),
        ["accueil", "voix", "lecture", "general", "mise_a_jour", "a_propos"]
    );
    assert_eq!(s.cle(), "a_propos");

    // Une page propre choisie reste choisie quand une autre s'ajoute devant elle.
    s.aller_a("lecture");
    l.pages
        .lock()
        .expect("verrou")
        .insert(0, Page::new("dictee", "Dictée", '\u{E720}'));
    s.suivre(&l);
    assert_eq!(s.cle(), "lecture");
    assert_eq!(s.pages.len(), 7);

    // La page montrée disparaît : retour à Général.
    l.pages.lock().expect("verrou").retain(|p| p.cle != "lecture");
    s.suivre(&l);
    assert_eq!(s.cle(), "general");
    assert!(!cles(&s).contains(&"lecture".to_string()));
}
