//! La fenêtre du volet et du petit menu, préparée au repos, gardée cachée et montrée au clic ;
//! le détail : [`crate::volet`].

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use eframe::egui::{self, FontDefinitions};

use crate::Logiciel;
use crate::apparence::Apparence;
use crate::bords::verrou::verrou;
use crate::fenetres::{Demande, Garde, Liens, Reception};
use crate::menu::Choix;
use crate::place::Rect;
use crate::surface::{Choisi, LARGEUR, LARGEUR_MENU};
use crate::theme::Surface;

fn rang(s: Surface) -> usize {
    match s {
        Surface::Volet => 0,
        Surface::Menu => 1,
    }
}

/// Écart entre deux images d'un fondu (ouverture ou fermeture) : environ 60 par seconde.
/// Sans synchronisation verticale, rien d'autre ne cadence le dessin.
pub const IMAGE_DE_FONDU: Duration = Duration::from_millis(16);

/// Options communes des fenêtres : n'importe quel fil, fenêtre cachée jusqu'à sa mise en
/// place.
///
/// Sans synchronisation verticale (`glow_options.vsync: false`) : avec elle, SwapBuffers attend le
/// prochain rafraîchissement de l'écran. Quand un écran est retiré ou rendu (il passe à un
/// autre PC, il se met en veille), Windows refait l'affichage et cette attente a duré des
/// dizaines de secondes, voire des minutes : le fil des fenêtres restait bloqué dans
/// SwapBuffers, et le volet comme le petit menu ne répondaient plus. Ces fenêtres ne
/// dessinent qu'à la demande : elles n'ont pas besoin d'être cadencées par l'écran, et les
/// fondus se cadencent eux-mêmes ([`IMAGE_DE_FONDU`]).
pub fn options(viewport: egui::ViewportBuilder) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport,
        glow_options: eframe::egui_glow::GlowConfiguration {
            vsync: false,
            ..Default::default()
        },
        event_loop_builder: Some(Box::new(|b| {
            use winit::platform::windows::EventLoopBuilderExtWindows;
            b.with_any_thread(true);
        })),
        centered: false,
        persist_window: false,
        ..Default::default()
    }
}

/// Fait ce qui a été choisi dans le volet ou le petit menu, une fois la fenêtre cachée ; rend
/// la fenêtre à ouvrir ensuite, s'il y en a une. N'attend jamais : le logiciel reçoit l'ordre
/// et rend la main.
pub fn faire(choisi: Option<Choisi>, logiciel: &Arc<dyn Logiciel>) -> Option<Demande> {
    match choisi? {
        Choisi::Rapide(cle) | Choisi::Menu(Choix::Action(cle)) => {
            logiciel.action(&cle);
            None
        }
        Choisi::Parametres | Choisi::Menu(Choix::Parametres) => Some(Demande::Parametres(None)),
        Choisi::Menu(Choix::Ouvrir) => {
            // La fenêtre principale est au logiciel : il l'ouvre sans attendre.
            logiciel.ouvrir_fenetre_principale();
            None
        }
        Choisi::Menu(Choix::RechercherMiseAJour) => {
            // La recherche part sur son propre fil ; sa réponse s'affiche dans les
            // Paramètres, page « Mise à jour ».
            logiciel.rechercher_mise_a_jour();
            Some(Demande::Parametres(Some(
                crate::parametres::modele::MISE_A_JOUR.into(),
            )))
        }
        Choisi::Menu(Choix::Quitter) => {
            logiciel.quitter();
            None
        }
    }
}

/// Écrit au journal sur un fil à part : le fil des fenêtres ne touche jamais au disque
/// pendant une ouverture.
fn noter(logiciel: &Arc<dyn Logiciel>, texte: String) {
    let l = logiciel.clone();
    let _ = std::thread::Builder::new()
        .name("journal du volet".into())
        .spawn(move || l.noter(texte));
}

/// Une image du volet ou du petit menu pendant son ouverture, pour les mesures
/// ([`observer`]).
#[derive(Debug, Clone, Copy)]
pub struct ImageVue {
    pub surface: Surface,
    /// Du clic sur l'icône à la fin du dessin de cette image (avant l'échange des tampons).
    pub depuis_clic: Duration,
    /// L'image est encore une image de mise en place (fenêtre voilée, rien à l'écran).
    pub mise_en_place: bool,
    /// L'opacité posée pour cette image (0 à 255).
    pub opacite: u8,
    /// L'animation d'ouverture est finie : la fenêtre est à sa place, opaque.
    pub finie: bool,
}

type Observateur = Box<dyn FnMut(ImageVue) + Send>;

/// Le mode d'essai des mesures : chaque image d'une ouverture est rendue à l'observateur ;
/// la fenêtre reste voilée et ne prend jamais le premier plan (rien ne s'ouvre à l'écran,
/// le clavier reste à l'application en cours).
// ÉTAT GLOBAL : posé par `observer()`, fonction publique sans poignée (la boucle et le volet
// sont créés par la bibliothèque, l'essai de mesure n'a rien à leur passer).
static OBSERVATEUR: std::sync::Mutex<Option<Observateur>> = std::sync::Mutex::new(None);
// ÉTAT GLOBAL : même raison que `OBSERVATEUR`, lu à chaque image sans prendre de verrou.
static DISCRET: AtomicBool = AtomicBool::new(false);

/// Pour les mesures (`examples/mesure_ouverture.rs`) seulement : `Some`, chaque image d'une
/// ouverture est rendue à `f`, la fenêtre reste voilée (invisible) et ne prend jamais le
/// focus ; `None` revient au fonctionnement normal.
pub fn observer(f: Option<Observateur>) {
    DISCRET.store(f.is_some(), Ordering::Release);
    *verrou(&OBSERVATEUR) = f;
}

fn discret() -> bool {
    DISCRET.load(Ordering::Acquire)
}

/// La première ouverture de chaque surface, et la première préparation, depuis le lancement.
// ÉTAT GLOBAL : « depuis le lancement » ; doit survivre à la recréation de la boucle après une
// panique (un objet de la boucle serait remis à zéro avec elle).
static DEJA_OUVERTE: [AtomicBool; 2] = [AtomicBool::new(false), AtomicBool::new(false)];
// ÉTAT GLOBAL : même raison que `DEJA_OUVERTE`.
static DEJA_PREPAREE: AtomicBool = AtomicBool::new(false);

/// Une demande d'ouverture, reçue par la fenêtre.
#[derive(Debug, Clone, Copy)]
struct Montrer {
    surface: Surface,
    icone: Rect,
    /// Le point du clic droit (le petit menu) ; le centre de l'icône pour le volet.
    ancre: [i32; 2],
    clic: Instant,
    recue: Instant,
}

impl Montrer {
    fn de(d: &Demande) -> Option<Montrer> {
        let recue = Instant::now();
        match *d {
            Demande::Volet { icone, clic } => Some(Montrer {
                surface: Surface::Volet,
                icone,
                ancre: centre(icone),
                clic,
                recue,
            }),
            Demande::Menu { icone, ancre, clic } => Some(Montrer {
                surface: Surface::Menu,
                icone,
                ancre,
                clic,
                recue,
            }),
            _ => None,
        }
    }
}

fn centre(r: Rect) -> [i32; 2] {
    [r[0] / 2 + r[2] / 2, r[1] / 2 + r[3] / 2]
}

/// Où en est la fenêtre.
#[derive(Clone)]
enum Etape {
    /// Préparation au repos, fenêtre cachée : le volet, puis le petit menu, sont dessinés une
    /// fois (polices et images prêtes).
    Preparation(u8),
    /// Prête et cachée.
    Cachee,
    /// Montrée encore invisible (opacité 0) : écran, taille, place.
    MiseEnPlace { m: Montrer, images: u8 },
    /// Montrée : l'animation d'ouverture, puis l'attente d'un choix.
    Montree {
        m: Montrer,
        debut: Instant,
        /// L'animation déjà jouée, avancée image par image ([`avancer`](ic_coeur::theme::avancer)) : un retard de
        /// l'écran ne la fait jamais sauter à la fin.
        anime: Duration,
        /// La fin de l'image précédente.
        derniere: Instant,
        /// La première image montrée (celle qui lève le voile) a été présentée.
        presentee: bool,
        finie: bool,
        /// La taille a changé (contenu nouveau) : la replacer contre la barre.
        replacer: bool,
        /// Clics et touches ignorés pendant les premiers instants.
        garde: Garde,
    },
    /// Le fondu de fermeture, puis ce qui a été choisi.
    Fermeture {
        surface: Surface,
        debut: Instant,
        puis: Option<Choisi>,
    },
}

/// Ce que la fenêtre rend au fil des fenêtres en se fermant.
#[derive(Default)]
struct Sortie {
    /// La demande pour une autre fenêtre (les Paramètres…).
    suite: Option<Demande>,
    /// La demande de volet ou de menu en cours (pour le menu de secours, en cas d'erreur).
    en_cours: Option<Demande>,
}

/// Ouvre les Paramètres à cette page, dans la boucle, ou les amène à cette page et devant
/// s'ils sont déjà ouverts.
fn ouvrir_parametres(liens: &Arc<Liens>, logiciel: &Arc<dyn Logiciel>, page: Option<String>) {
    crate::parametres::fenetre::ouvrir_dans(liens, logiciel, page);
}

mod app;
mod fenetre;
mod veille;

pub use veille::{PAS_DE_VEILLE, Veilleur};

use fenetre::Volet;

/// Les flèches haut et bas passent d'une ligne à l'autre, comme Tab et Maj+Tab (C-33) : écrit
/// dans le dessin (`src/dessin/volet/clavier.rs`, bloc G), repris à son chemin d'aujourd'hui.
pub use crate::dessin::volet::clavier::fleches_en_tab;

/// La fenêtre du volet est-elle voilée à cette étape ? Oui, sauf montrée pour de vrai
/// (animation d'ouverture, attente d'un choix, fermeture).
fn voilee(e: &Etape) -> bool {
    !matches!(e, Etape::Montree { .. } | Etape::Fermeture { .. })
}

/// Prépare au repos la fenêtre du volet et du petit menu (cachée, invisible), puis la garde :
/// chaque demande de volet ou de petit menu la montre sans la reconstruire ; les Paramètres
/// et les autres fenêtres s'ouvrent dans sa boucle. Rend la main quand le logiciel se ferme
/// (`Demande::Quitter` ou `None`).
pub fn veiller(
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    polices: FontDefinitions,
    demandes: &Reception,
    premiere: Option<Demande>,
) -> Result<Option<Demande>, (String, Option<Demande>)> {
    let sortie = Rc::new(RefCell::new(Sortie {
        suite: None,
        en_cours: premiere.clone(),
    }));
    let s2 = sortie.clone();
    let creee_a = Instant::now();
    let viewport = egui::ViewportBuilder::default()
        .with_title(format!("Volet de {}", logiciel.identite().nom))
        .with_inner_size([LARGEUR, 300.0])
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top()
        .with_taskbar(false)
        .with_visible(false);
    let (l2, liens2) = (logiciel.clone(), liens.clone());
    let nom = logiciel.identite().nom.clone();
    crate::sonde::etape("volet : création de la fenêtre et du contexte OpenGL");
    let r = eframe::run_native(
        &nom,
        options(viewport),
        Box::new(move |cc| {
            cc.egui_ctx.set_fonts(polices);
            let apparence = Apparence::new(None);
            apparence.appliquer(&cc.egui_ctx);
            *verrou(&liens2.contexte) = Some(cc.egui_ctx.clone());
            liens2.autres.lier(Some(cc.egui_ctx.clone()));
            // Une demande envoyée réveille cette fenêtre, même cachée.
            demandes.lecteur(Some(cc.egui_ctx.clone()));
            let pour_fermer = liens2.clone();
            let veilleur = Veilleur::new(move || pour_fermer.fermer_la_fenetre());
            Ok(Box::new(Volet {
                logiciel: l2,
                liens: liens2,
                demandes,
                apparence,
                hwnd: None,
                transparente: false,
                voilee: None,
                etape: Etape::Preparation(0),
                tailles: [egui::vec2(LARGEUR, 300.0), egui::vec2(LARGEUR_MENU, 200.0)],
                borne: None,
                effets: true,
                a_eu_le_focus: false,
                focus_redemande: false,
                sortie_voulue: None,
                fin_voulue: false,
                sortie: s2,
                creee_a,
                premiere,
                vue: 0,
                veilleur,
            }))
        }),
    );
    demandes.lecteur(None);
    liens.autres.lier(None);
    liens.surface_prete.store(0, Ordering::Release);
    liens.autres.racine(0);
    demandes.racine(0);
    let mut s = sortie.borrow_mut();
    match r {
        Ok(()) => Ok(s.suite.take()),
        Err(e) => Err((e.to_string(), s.en_cours.take())),
    }
}

#[cfg(test)]
mod essais {
    use super::*;
    use crate::theme;

    /// Le veilleur public, sans fenêtre : sans appui de souris dehors, `fermer` n'est jamais
    /// appelée ; une fenêtre qui n'existe pas n'est pas surveillée ; il se retire à sa
    /// destruction (le fil se termine, sans attente).
    #[test]
    fn veilleur_public_sans_fenetre() {
        let appels = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let a2 = appels.clone();
        let v = Veilleur::new(move || {
            a2.fetch_add(1, Ordering::AcqRel);
        });
        // Une poignée qui n'est pas une fenêtre : rien n'est lu, rien n'est fermé.
        v.activer(0x7fff_fff0);
        std::thread::sleep(PAS_DE_VEILLE * 4);
        v.desactiver();
        std::thread::sleep(PAS_DE_VEILLE * 2);
        drop(v);
        assert_eq!(appels.load(Ordering::Acquire), 0);
    }

    /// Volet, petit menu et Paramètres sans synchronisation verticale : avec elle, le fil des
    /// fenêtres restait bloqué dans SwapBuffers pendant des minutes après le retrait d'un
    /// écran (il passait à un autre PC), et rien ne s'ouvrait plus.
    #[test]
    fn fenetres_sans_synchronisation_verticale() {
        let o = options(egui::ViewportBuilder::default());
        assert!(!o.glow_options.vsync, "SwapBuffers ne doit pas attendre l'écran");
        // Les fondus se cadencent eux-mêmes, sans tourner à vide.
        assert!(IMAGE_DE_FONDU >= Duration::from_millis(10));
        assert!(IMAGE_DE_FONDU <= Duration::from_millis(20));
    }

    /// Préparée, cachée ou en train de se mettre en place : voilée (eframe la montre d'office
    /// à sa première image, et winit lui retire son opacité 0). Le voile ne se lève qu'à
    /// l'ouverture pour de vrai, et revient à la fermeture.
    #[test]
    fn voilee_sauf_montree_pour_de_vrai() {
        let maintenant = Instant::now();
        let m = Montrer {
            surface: Surface::Volet,
            icone: [0, 0, 16, 16],
            ancre: [8, 8],
            clic: maintenant,
            recue: maintenant,
        };
        assert!(voilee(&Etape::Preparation(0)));
        assert!(voilee(&Etape::Preparation(2)));
        assert!(voilee(&Etape::Cachee));
        assert!(voilee(&Etape::MiseEnPlace { m, images: 1 }));
        assert!(!voilee(&Etape::Montree {
            m,
            debut: maintenant,
            anime: Duration::ZERO,
            derniere: maintenant,
            presentee: false,
            finie: false,
            replacer: false,
            garde: Garde::default(),
        }));
        assert!(!voilee(&Etape::Fermeture {
            surface: Surface::Menu,
            debut: maintenant,
            puis: None,
        }));
    }

    /// Prête avant d'être montrée : le voile se lève à la fin de la dernière image de mise en
    /// place, quand la fenêtre est déjà à sa taille, à sa place et à l'opacité de la première
    /// image de l'ouverture. Cette image-là est déjà bien visible (l'ancienne ouverture
    /// montrait d'abord une fenêtre transparente, puis la faisait monter et apparaître en
    /// 250 ms : une attente ressentie à chaque clic).
    #[test]
    fn premiere_image_montree_deja_visible() {
        let premiere = theme::ouverture(0, true);
        assert!(premiere.opacite >= 128, "{}", premiere.opacite);
        const { assert!(theme::OUVERTURE_FONDU_MS <= 100) };
        // « Effets d'animation » coupés : opaque tout de suite.
        assert_eq!(theme::ouverture(0, false).opacite, 255);
        assert!(theme::ouverture(0, false).finie);
    }
}
