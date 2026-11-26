//! Le fil des fenêtres de la bibliothèque (bloc M) : le volet préparé au repos, les
//! Paramètres et les autres fenêtres, lancé par `Horloge` comme par `zone::FenetresCommunes`
//! (un seul code pour les deux) ; et le menu de secours demandé au fil de l'icône seule.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

use super::decodage::{WM_MENU_DE_SECOURS, jeton};
use crate::Logiciel;
use crate::fenetres::{self, Demande, Liens, Ouvreur, Reception};
use crate::systeme::hwnd;

/// Le temps qu'une fenêtre qui ne répond plus peut retenir la fermeture du logiciel.
pub const ATTENTE_FERMETURE: Duration = Duration::from_secs(5);

/// Attend la fin d'un fil au plus `delai` ; rend faux (et le laisse partir seul) s'il n'a pas
/// fini : le programme se ferme quand même, rien ne le retient.
pub(crate) fn attendre_la_fin(fil: std::thread::JoinHandle<()>, delai: Duration) -> bool {
    let fin = Instant::now() + delai;
    while !fil.is_finished() {
        if Instant::now() >= fin {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let _ = fil.join();
    true
}

// ---------------------------------------------------------------------------------------

/// Le menu de secours de Windows, demandé par le fil des fenêtres (voir
/// [`lancer_le_fil_des_fenetres`]).
pub type Secours = Box<dyn FnMut(&Demande) + Send>;

/// Les vraies fenêtres (egui).
struct Fenetres {
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    polices: egui::FontDefinitions,
    secours: Secours,
}

impl Ouvreur for Fenetres {
    fn surface(
        &mut self,
        demandes: &Reception,
        premiere: Option<Demande>,
    ) -> Result<Option<Demande>, (String, Option<Demande>)> {
        crate::volet::veiller(
            self.logiciel.clone(),
            self.liens.clone(),
            self.polices.clone(),
            demandes,
            premiere,
        )
    }

    fn parametres(&mut self, page: Option<String>) -> Result<(), String> {
        crate::parametres::fenetre::ouvrir_seuls(
            self.logiciel.clone(),
            Some(self.liens.clone()),
            page,
            self.polices.clone(),
        )
    }

    fn secours(&mut self, d: &Demande) {
        (self.secours)(d);
    }

    fn noter(&mut self, ligne: String) {
        self.logiciel.noter(ligne);
    }
}

/// Le menu de secours de l'icône unique : demandé à la fenêtre cachée du fil principal, qui
/// l'ouvre au point du clic.
pub(crate) fn secours_de_l_horloge(liens: Arc<Liens>) -> Secours {
    Box::new(move |d: &Demande| {
        let point = match d {
            Demande::Volet { icone, .. } => [icone[0], icone[1]],
            Demande::Menu { ancre, .. } => *ancre,
            _ => return,
        };
        let lp = (i64::from(point[0]) & 0xFFFF) | ((i64::from(point[1]) & 0xFFFF) << 16);
        let h = liens.fenetre.load(Ordering::Acquire);
        if h != 0 {
            // SAFETY: comme `ReveilParMessage::reveiller` (le guichet).
            let _ = unsafe {
                PostMessageW(
                    Some(hwnd(h)),
                    WM_MENU_DE_SECOURS,
                    WPARAM(jeton()),
                    LPARAM(lp as isize),
                )
            };
        }
    })
}

/// Lance **le fil des fenêtres de la bibliothèque** : la fenêtre du volet et du petit menu
/// préparée au repos (`volet::veiller`), les Paramètres et les autres fenêtres dans sa
/// boucle, une fenêtre perdue recréée, la garde du fil ; jusqu'à `Demande::Quitter` (ou
/// [`Liens::tout_fermer`]). Les demandes arrivent par `reception` (l'autre bout de
/// [`fenetres::canal`]). C'est le fil que [`crate::horloge::Horloge::lancer`] lance ; un logiciel qui garde
/// sa propre boucle de messages (plusieurs icônes, `zone`) le lance aussi, par
/// `zone::FenetresCommunes`.
///
/// `secours` : le menu gris de Windows à la place du volet ou du petit menu de cette demande
/// (les fenêtres modernes sont impossibles, ou perdues en rafale). Il est appelé **sur le fil
/// des fenêtres** : il doit seulement transmettre la demande au fil de l'icône (un message
/// posté à sa fenêtre), jamais ouvrir le menu lui-même.
pub fn lancer_le_fil_des_fenetres(
    logiciel: Arc<dyn Logiciel>,
    liens: Arc<Liens>,
    reception: Reception,
    secours: Secours,
) -> Result<std::thread::JoinHandle<()>, String> {
    std::thread::Builder::new()
        .name("fenêtres".into())
        .spawn(move || fil_des_fenetres(logiciel, liens, reception, secours))
        .map_err(|e| e.to_string())
}

fn fil_des_fenetres(logiciel: Arc<dyn Logiciel>, liens: Arc<Liens>, reception: Reception, secours: Secours) {
    // Le lieu d'une panique est gardé pour le journal (la boucle des fenêtres l'attrape et
    // recrée la fenêtre) ; si le fil s'arrête quand même, l'icône passe au menu de secours.
    crate::sonde::retenir_les_paniques();
    let pour_la_garde = logiciel.clone();
    let _garde = fenetres::GardeDuFil::new(liens.clone(), move |t| pour_la_garde.noter(t));
    // Surveillé : une image demandée le réveille s'il dort (le volet caché ne tourne qu'à la demande).
    let pour_reveiller = liens.clone();
    let _surveille = crate::sonde::inscrire(
        "fenêtres",
        Some(Box::new(move || {
            if let Ok(c) = pour_reveiller.contexte.try_lock()
                && let Some(c) = c.as_ref()
            {
                c.request_repaint();
            }
        })),
    );
    match crate::polices::polices() {
        Ok(polices) => {
            let mut f = Fenetres {
                logiciel,
                liens: liens.clone(),
                polices,
                secours,
            };
            fenetres::boucle_des_fenetres(&liens, &mut f, &reception);
        }
        Err(e) => {
            logiciel.noter(format!("Interface : {e} ; menu de secours de Windows."));
            liens.egui_impossible.store(true, Ordering::Release);
            let mut f = Fenetres {
                logiciel,
                liens: liens.clone(),
                polices: egui::FontDefinitions::empty(),
                secours,
            };
            // Sans polices : seul le menu de secours (fil principal).
            while let Some(d) = reception.recevoir() {
                if d == Demande::Quitter {
                    break;
                }
                f.secours(&d);
            }
        }
    }
}
