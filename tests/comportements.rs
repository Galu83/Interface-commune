//! Les comportements de `docs/COMPORTEMENTS.md` qui n'avaient pas d'essai (filets de la
//! refonte, 24 nov. 2026). Chaque essai porte le numéro du comportement qu'il garde.
//!
//! Tout se fait sans fenêtre : les dessins passent par `capture` (rendu logiciel), les appels
//! à Windows ne font que lire (écran, souris, réglages), et le verrou « une seule copie » est
//! pris sous un nom inventé. Ce qui demande une vraie fenêtre (la place retenue 1 s après un
//! déplacement, le message reçu par la copie déjà lancée) reste aux essais de GitHub.

#![cfg(windows)]
// Dans un essai, un échec doit arrêter l'essai ; le verrou et le DPI se lisent par Windows.
#![allow(clippy::expect_used, unsafe_code)]

#[allow(dead_code)]
#[path = "references/logiciel.rs"]
mod logiciel;

use std::sync::{Arc, Mutex};

use interface_commune::autres_fenetres::Contenu;
use interface_commune::capture::{self, Options};
use interface_commune::fenetres::{Demande, Liens, Ouvert, canal};
use interface_commune::horloge::{Reveil, instance_unique_avec, reglage_dpi};
use interface_commune::parametres::fenetre::{ContenuParametres, ouverture};
use interface_commune::parametres::{Contexte, Parametres};
use interface_commune::place::Place;
use interface_commune::theme::ChoixWindows;
use interface_commune::{
    Communs, Etat, GenreEtat, Identite, Logiciel, Page, Reglages, Rgb, Valeur, apparence, dessin, polices,
    seuils, sonde, systeme, textes,
};

fn polices_presentes() -> bool {
    let ok = polices::polices().is_ok();
    if !ok {
        eprintln!("Polices Segoe absentes : essai de dessin sauté.");
    }
    ok
}

/// Un logiciel inventé dont on choisit les pages, l'état et la place retenue.
struct Essai {
    identite: Identite,
    pages: Vec<Page>,
    detail: String,
    place: Option<Place>,
    retenue: Mutex<Option<Place>>,
}

impl Essai {
    fn new() -> Essai {
        Essai {
            identite: logiciel::identite(),
            pages: Vec::new(),
            detail: String::new(),
            place: None,
            retenue: Mutex::new(None),
        }
    }
}

impl Reglages for Essai {
    fn identite(&self) -> &Identite {
        &self.identite
    }
    fn action(&self, _: &str) {}
    fn rechercher_mise_a_jour(&self) {}
    fn pages(&self) -> Vec<Page> {
        self.pages.clone()
    }
    fn dessiner_page(&self, _: &str, _: &mut egui::Ui, page: &mut Contexte) {
        page.redemarrage_necessaire("Un réglage inventé demande un redémarrage.");
    }
    fn communs(&self) -> Communs {
        Communs::default()
    }
    fn changer(&self, _: &str, _: Valeur) {}
    fn place_parametres(&self) -> Option<Place> {
        self.place
    }
    fn retenir_place(&self, place: Place) {
        *self.retenue.lock().expect("verrou") = Some(place);
    }
}

impl Logiciel for Essai {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Essai".into(),
            ligne: "Tout va bien".into(),
            detail: (!self.detail.is_empty()).then(|| self.detail.clone()),
        }
    }
    fn quitter(&self) {}
    fn noter(&self, _: String) {}
}

/// C-17 : la copie déjà lancée montre ses Paramètres (ou la page demandée) quand une
/// deuxième arrive. Le second lancement trouve le verrou pris et prévient la première
/// (`Err`) ; la demande « Paramètres » part vers la boucle des fenêtres, ou, s'ils sont déjà
/// ouverts, change leur page et les met devant.
#[test]
fn c17_la_premiere_copie_montre_ses_parametres() {
    let nom = format!("Essai-filets-c17-{}", std::process::id());
    let premiere = instance_unique_avec(&nom, &Reveil::Parametres).expect("première copie");
    // Aucune fenêtre cachée sous ce nom : la deuxième s'arrête quand même, sans rester accrochée.
    assert_eq!(instance_unique_avec(&nom, &Reveil::Parametres).err(), Some(false));
    drop(premiere);

    let liens = Liens::default();
    let (demandes, reception) = canal();
    liens.montrer_page(&demandes, None);
    assert!(matches!(reception.prendre(), Ok(Some(Demande::Parametres(None)))));
    liens.montrer_page(&demandes, Some("ecrans".into()));
    assert!(matches!(reception.prendre(), Ok(Some(Demande::Parametres(Some(ref p)))) if p == "ecrans"));
    // Déjà ouverts : pas de deuxième fenêtre, la page voulue change.
    liens.mettre(Ouvert::Parametres);
    liens.montrer_page(&demandes, Some("dispositions".into()));
    assert!(matches!(reception.prendre(), Ok(None)));
    assert_eq!(
        liens.page_voulue.lock().expect("verrou").as_deref(),
        Some("dispositions")
    );
}

/// C-18 : une seule copie par session Windows et par logiciel, par le verrou
/// `Local\<nom>-une-seule`. Ce nom exact est une promesse : une nouvelle version doit voir le
/// verrou de l'ancienne pendant une mise à jour.
#[test]
fn c18_une_seule_copie_par_session() {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};
    use windows::core::HSTRING;

    let nom = format!("Essai-filets-c18-{}", std::process::id());
    let ouvrir = |espace: &str| {
        let n = HSTRING::from(format!("{espace}\\{nom}-une-seule"));
        // SAFETY: nom vivant pendant l'appel ; la poignée rendue est fermée aussitôt.
        unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, &n) }
            .map(|h| {
                // SAFETY: poignée ouverte juste au-dessus, fermée une fois.
                let _ = unsafe { CloseHandle(h) };
            })
            .is_ok()
    };
    assert!(!ouvrir("Local"), "verrou déjà là avant l'essai");
    let premiere = instance_unique_avec(&nom, &Reveil::Rien).expect("première copie");
    assert!(ouvrir("Local"), "le verrou doit s'appeler Local\\<nom>-une-seule");
    assert!(!ouvrir("Global"), "le verrou ne vaut que pour la session");
    // La deuxième copie, qui n'a rien à dire, s'arrête.
    assert_eq!(instance_unique_avec(&nom, &Reveil::Rien).err(), Some(true));
    // La première fermée, une nouvelle copie peut démarrer.
    drop(premiere);
    assert!(!ouvrir("Local"));
    assert!(instance_unique_avec(&nom, &Reveil::Rien).is_ok());
}

/// C-25 : rien n'est coupé ; la largeur suit les textes jusqu'à la largeur maximale, puis les
/// phrases passent à la ligne.
#[test]
fn c25_un_long_texte_passe_a_la_ligne() {
    if !polices_presentes() {
        return;
    }
    let mut court = Essai::new();
    court.detail = "Une phrase courte.".into();
    let mut long = Essai::new();
    long.detail = "Une explication inventée, beaucoup trop longue pour une seule ligne du volet : \
                   elle doit passer à la ligne plusieurs fois, sans être coupée au bord droit, \
                   et le volet doit grandir en hauteur plutôt qu'en largeur."
        .into();
    let o = Options::default();
    let a = capture::volet(&court, o).expect("volet court");
    let b = capture::volet(&long, o).expect("volet long");
    let max = (interface_commune::mesures::LARGEUR_MAX + 2.0 * 16.0) as u32;
    assert!(b.largeur <= max, "trop large : {} > {max}", b.largeur);
    assert!(
        b.hauteur >= a.hauteur + 3 * 16,
        "le texte n'est pas passé à la ligne : {} puis {}",
        a.hauteur,
        b.hauteur
    );
}

/// C-78 : un réglage qui demande un redémarrage montre le bandeau « Redémarrer maintenant »,
/// dont le bouton envoie l'action `cles::REDEMARRER` au logiciel.
#[test]
fn c78_bandeau_redemarrer() {
    assert_eq!(interface_commune::cles::REDEMARRER, "redemarrer");
    assert_eq!(textes::FRANCAIS.redemarrer_maintenant, "Redémarrer maintenant");
    if !polices_presentes() {
        return;
    }
    let mut l = Essai::new();
    l.pages = vec![Page::new("ecrans", "Écrans", '\u{E7F4}')];
    let mut s = Parametres::new(&l, Some("ecrans"));
    assert_eq!(s.redemarrage, None);
    let image = capture::parametres_de(&mut s, &l, Options::default()).expect("Paramètres");
    assert_eq!(
        s.redemarrage.as_deref(),
        Some("Un réglage inventé demande un redémarrage.")
    );
    // Le bandeau est orangé, en haut de la page (son dessin exact : images de référence).
    let orange = (0..image.largeur)
        .filter_map(|x| image.pixel(x, 40))
        .any(|[r, g, b, _]| r > g && g > b && r >= 200 && b < 230 && r - b > 15);
    assert!(orange, "pas de bandeau en haut de la page");
}

/// C-79 : les Paramètres s'ouvrent à leur place retenue, toujours en entier sur un écran, et
/// retiennent leur place quand ils se ferment. (Le suivi 1 s après un déplacement et le refus
/// d'une fenêtre réduite ou agrandie demandent une vraie fenêtre : `place::*` et GitHub.)
#[test]
fn c79_place_des_parametres() {
    let (_, travail) = systeme::ecran_autour(0, 0).expect("un écran");
    let mut l = Essai::new();
    l.place = Some([travail[0] + 20, travail[1] + 20, 700, 500]);
    let o = ouverture(&l, false);
    assert_eq!(
        o.place, l.place,
        "la place retenue, qui tient sur l'écran, est gardée"
    );
    // Une place sur un écran débranché : ramenée en entier sur un écran.
    l.place = Some([-15_000, -12_000, 700, 500]);
    let o = ouverture(&l, false);
    let p = o.place.expect("une place");
    let (_, ici) = systeme::ecran_autour(p[0], p[1]).expect("un écran");
    assert!(
        p[0] >= ici[0] && p[1] >= ici[1] && p[0] + p[2] <= ici[2] && p[1] + p[3] <= ici[3],
        "{p:?} hors de {ici:?}"
    );
    // La place d'une fenêtre réduite (-32 000 chez Windows) n'est jamais reprise.
    l.place = Some([-32_000, -32_000, 160, 28]);
    assert_eq!(ouverture(&l, false).place, None);
    // Sans place retenue : centrée sur l'écran de la souris (pas de place imposée).
    l.place = None;
    assert_eq!(ouverture(&l, false).place, None);
    // À la fermeture, la place est retenue par le logiciel.
    let l = Arc::new(Essai::new());
    let mut c = ContenuParametres::new(l.clone(), Arc::new(Mutex::new(None)), None);
    c.fermee(Some([100, 120, 800, 600]));
    assert_eq!(*l.retenue.lock().expect("verrou"), Some([100, 120, 800, 600]));
}

/// C-80 : la colonne de navigation s'élargit pour un long nom de page, sans dépasser sa
/// largeur maximale.
#[test]
fn c80_la_navigation_s_elargit() {
    if !polices_presentes() {
        return;
    }
    let largeur = |titre: &str| {
        let mut l = Essai::new();
        l.pages = vec![Page::new("page", titre, '\u{E7F4}')];
        let s = Parametres::new(&l, None);
        let mut w = 0.0;
        capture::dessin(egui::vec2(400.0, 100.0), Options::default(), |ui, _| {
            w = s.largeur_navigation(ui, &l);
        })
        .expect("dessin");
        w
    };
    let court = largeur("Écrans");
    let long = largeur("Dispositions des écrans de la maison");
    let enorme = largeur(&"Un nom de page vraiment démesuré ".repeat(6));
    assert!(long > court, "{court} puis {long}");
    assert!(enorme >= long && enorme < 600.0, "{long} puis {enorme}");
}

/// C-90 : clair ou sombre et l'accent de Windows, relus toutes les 2 s et appliqués dès
/// qu'ils changent ; un thème imposé (captures, essais) le reste.
#[test]
fn c90_le_theme_de_windows_est_relu() {
    assert_eq!(apparence::RELECTURE, std::time::Duration::from_secs(2));
    let ctx = egui::Context::default();
    let windows = apparence::choix_windows();
    let autre = ChoixWindows {
        accent: Some(Rgb(1, 2, 3)),
        sombre: !windows.sombre,
        ..windows.clone()
    };
    let mut a = apparence::Apparence::de(autre.clone(), None);
    // Avant 2 s, rien n'est relu.
    assert!(!a.suivre(&ctx));
    assert_eq!(a.choix, autre);
    // Relu : le choix de Windows remplace l'ancien, et la palette suit.
    assert!(a.relire(&ctx));
    assert_eq!(a.choix, windows);
    assert_eq!(a.palette.sombre, windows.sombre);
    assert_eq!(ctx.global_style().visuals.dark_mode, windows.sombre);
    assert!(!a.relire(&ctx), "rien n'a changé");
    // Imposé : reste sombre quoi que dise Windows.
    let mut f = apparence::Apparence::de(autre, Some(true));
    f.relire(&ctx);
    assert!(f.choix.sombre && f.palette.sombre);
}

/// C-98 : les nombres ont une espace insécable avant leur unité. Gardé ici pour les seuils et
/// les textes de la bibliothèque ; les écarts connus sont listés (à corriger par la refonte).
#[test]
fn c98_espace_insecable_avant_l_unite() {
    let o = seuils::Options::default();
    assert_eq!(o.unite, "%");
    // Tous les textes, dans les deux langues : un nombre (« {} ») suivi d'une espace normale
    // puis d'une unité. Les écarts connus d'aujourd'hui sont listés : la refonte les corrige
    // (docs/PLAN-REFONTE.md) et les retire d'ici. Un autre écart, hors des textes : le
    // pourcentage des appareils du volet (`briques`, fonction privée), vu sur les images.
    const ECARTS_CONNUS: &[&str] = &[
        "Retour au réglage d'avant dans {} s",
        "Going back to the previous setting in {} s",
    ];
    const UNITES: &[&str] = &["s", "%", "ms", "px", "Mo", "Ko", "Go", "min", "h", "j"];
    let mut trouves = Vec::new();
    for t in [&textes::FRANCAIS, &textes::ANGLAIS] {
        let tout = format!("{t:#?}");
        for ligne in tout.lines() {
            for (i, _) in ligne.match_indices("{} ") {
                let suite = &ligne[i + 3..];
                let mot: String = suite
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '%')
                    .collect();
                if UNITES.contains(&mot.as_str()) {
                    trouves.push(ligne.trim().to_string());
                }
            }
        }
    }
    let nouveaux: Vec<_> = trouves
        .iter()
        .filter(|l| !ECARTS_CONNUS.iter().any(|e| l.contains(e)))
        .collect();
    assert!(
        nouveaux.is_empty(),
        "espace normale avant une unité : {nouveaux:#?}"
    );
    // Un écart connu corrigé doit sortir de la liste.
    for e in ECARTS_CONNUS {
        assert!(
            trouves.iter().any(|l| l.contains(e)),
            "« {e} » est corrigé : retirez-le des écarts connus"
        );
    }
}

/// C-104 : la mise au point (focus du clavier) est visible sur chaque contrôle commun.
#[test]
fn c104_le_focus_se_voit() {
    if !polices_presentes() {
        return;
    }
    let t = &textes::FRANCAIS;
    type Controle =
        fn(&mut egui::Ui, &interface_commune::Palette, &interface_commune::Textes) -> egui::Response;
    let controles: [(&str, Controle); 5] = [
        ("bouton", |ui, p, _| {
            dessin::bouton(ui, p, None, "Valider", dessin::Genre::Normal)
        }),
        ("bouton accent", |ui, p, _| {
            dessin::bouton(ui, p, None, "Valider", dessin::Genre::Accent)
        }),
        ("bouton icône", |ui, p, _| {
            dessin::bouton_icone(ui, p, '\u{E713}', "Réglages")
        }),
        ("interrupteur", |ui, p, t| {
            dessin::interrupteur_ecrit(ui, p, t, &mut true)
        }),
        ("ligne de menu", |ui, p, _| {
            dessin::ligne_menu(ui, p, '\u{E713}', "Paramètres")
        }),
    ];
    for o in [
        Options::default(),
        Options {
            sombre: true,
            ..Options::default()
        },
    ] {
        for (nom, controle) in controles {
            let image = |focus: bool| {
                capture::dessin(egui::vec2(220.0, 60.0), o, |ui, p| {
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        let r = controle(ui, p, t);
                        if focus {
                            r.request_focus();
                        }
                    });
                })
                .expect("dessin")
            };
            assert_ne!(image(false), image(true), "{nom} : le focus ne se voit pas");
        }
    }
}

/// C-110 : le journal du logiciel est branché sur celui de la bibliothèque : chaque ligne de
/// la sonde lui arrive.
#[test]
fn c110_le_journal_est_branche() {
    let lignes = Arc::new(Mutex::new(Vec::<String>::new()));
    let l = lignes.clone();
    sonde::brancher_le_journal(move |ligne| l.lock().expect("verrou").push(ligne));
    sonde::clic("essai-c110");
    let lignes = lignes.lock().expect("verrou");
    assert!(lignes.iter().any(|l| l.contains("clic essai-c110")), "{lignes:?}");
}

/// C-112 : l'écran autour d'un point (et sa zone de travail), la position de la souris, les
/// réglages de Windows lus par la bibliothèque.
#[test]
fn c112_ce_que_la_bibliotheque_lit_dans_windows() {
    let dans = |r: [i32; 4], x: i32, y: i32| x >= r[0] && x < r[2] && y >= r[1] && y < r[3];
    let (ecran, travail) = systeme::ecran_autour(0, 0).expect("un écran");
    assert!(ecran[2] > ecran[0] && ecran[3] > ecran[1]);
    assert!(
        travail[0] >= ecran[0] && travail[1] >= ecran[1] && travail[2] <= ecran[2] && travail[3] <= ecran[3],
        "la zone de travail est dans l'écran : {travail:?} {ecran:?}"
    );
    // Un point loin de tout écran : l'écran le plus proche.
    assert!(systeme::ecran_autour(-90_000, 90_000).is_some());
    if let Some([x, y]) = systeme::position_souris() {
        let (e, _) = systeme::ecran_autour(x, y).expect("l'écran de la souris");
        assert!(dans(e, x, y), "souris ({x}, {y}) hors de {e:?}");
    }
    let _: bool = systeme::effets_d_animation();
    let d = systeme::temps_double_clic();
    assert!(
        d >= std::time::Duration::from_millis(100) && d <= std::time::Duration::from_secs(5),
        "{d:?}"
    );
}

/// C-113 : le réglage d'échelle du processus (chaque écran à sa propre échelle) est posé par
/// `reglage_dpi`, avant toute fenêtre.
#[test]
fn c113_reglage_d_echelle_par_ecran() {
    use windows::Win32::UI::HiDpi::{
        AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        GetThreadDpiAwarenessContext,
    };
    // SAFETY: lectures du réglage du fil, sans pointeur.
    let par_ecran = || unsafe {
        AreDpiAwarenessContextsEqual(
            GetThreadDpiAwarenessContext(),
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        )
        .as_bool()
    };
    reglage_dpi();
    assert!(par_ecran(), "le processus doit être « par écran, v2 »");
    // Une deuxième fois : sans effet, sans erreur.
    reglage_dpi();
    assert!(par_ecran());
}
