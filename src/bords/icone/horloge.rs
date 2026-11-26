//! L'icône seule près de l'horloge et le fil principal (bloc M) : une façade sur une
//! [`Zone`] à une seule icône. La zone tient la fenêtre cachée, l'icône, sa remise quand
//! l'Explorateur redémarre, les notifications et le menu gris ; ce fichier y ajoute ce qui est
//! propre à l'icône seule : la boucle des messages de la bibliothèque, le guichet
//! ([`Poste`]), le fil des fenêtres, l'instance unique et le menu de secours.
//!
//! Chaque `unsafe` est commenté.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use windows::Win32::Foundation::{HANDLE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DEVICE_NOTIFY_WINDOW_HANDLE, DispatchMessageW, GetMessageW, KillTimer, MSG, PBT_POWERSETTINGCHANGE,
    PostMessageW, PostQuitMessage, SetTimer, TranslateMessage, WM_CLOSE, WM_COPYDATA, WM_DESTROY,
    WM_DISPLAYCHANGE, WM_ENDSESSION, WM_NULL, WM_POWERBROADCAST, WM_SETTINGCHANGE, WM_TIMER,
};
use windows::core::HSTRING;

use super::communes::{Aiguilleur, SuiteDuGeste};
use super::decodage::{
    Alimentation, Destinataire, ECRAN_DE_LA_CONSOLE, ID_ICONE, LigneMenu, MINUTERIE_ICONE, WM_COMMANDE,
    WM_ICONE, WM_MENU_DE_SECOURS, alimentation, attente_icone, destinataire, jeton, message_permis,
};
use super::fil::{ATTENTE_FERMETURE, attendre_la_fin, lancer_le_fil_des_fenetres, secours_de_l_horloge};
use super::image::fabriquer_icone;
use super::zone::{Ident, Zone};
use crate::Logiciel;
use crate::bords::poste::{AuClic, Commande, Poste, ReveilParMessage};
use crate::bords::systeme::cachee::ClasseDejaLa;
use crate::bords::systeme::echelle::reglage_dpi;
use crate::bords::unique::{WM_OUVRIR_PARAMETRES, classe, lire_arguments};
use crate::fenetres::{self, Demande, Liens, Reception};
use crate::menu::{self, Choix};
use crate::systeme::{choix_windows, hwnd, temps_double_clic};
use ic_coeur::gestes::aiguilleur::fermer_avant_la_fenetre_principale;

/// L'identifiant de l'icône seule dans sa zone.
const ICONE: &str = "horloge";

/// Un message de Windows reçu par la fenêtre cachée du logiciel (celle de l'icône).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageWindows {
    /// La fenêtre cachée (son adresse).
    pub fenetre: isize,
    pub message: u32,
    pub wparam: usize,
    pub lparam: isize,
}

/// Ce que le logiciel fait des messages de Windows de la fenêtre cachée (`WM_HOTKEY`, ses
/// messages à lui, `WM_POWERBROADCAST`…), sur le fil de l'icône. `Some(résultat)` : traité ;
/// `None` : Windows fait le traitement par défaut. Il voit aussi, sans pouvoir les arrêter,
/// `WM_QUERYENDSESSION`, `WM_ENDSESSION`, `WM_SETTINGCHANGE`, `WM_DISPLAYCHANGE` et
/// `WM_POWERBROADCAST` (veille, reprise, écran allumé ou éteint). Il ne voit jamais les
/// messages propres à la bibliothèque.
///
/// **Fin de session** : le récepteur est appelé de façon synchrone, avant que la bibliothèque
/// ferme ses fenêtres. C'est là qu'enregistrer, sans attendre : Windows ne laisse que quelques
/// secondes. `crate::zone::FinDeSession::de(message, wparam, lparam)` décode `WM_ENDSESSION`
/// (`Some` seulement si la session se ferme vraiment).
pub type Recepteur = Box<dyn FnMut(&MessageWindows) -> Option<isize>>;

/// Ce que le logiciel fait des arguments d'un second lancement
/// ([`crate::horloge::Reveil::Arguments`]), sur le fil de l'icône : à rendre vite (poster un
/// message, noter un ordre), le second lancement attend jusqu'à 3 s.
pub type RecepteurArguments = Box<dyn FnMut(Vec<String>)>;

/// Transmet un `WM_POWERBROADCAST` à [`crate::reprise`].
///
/// # Safety
/// `lp` est le `lParam` du message reçu à l'instant (pour PBT_POWERSETTINGCHANGE, un
/// POWERBROADCAST_SETTING que Windows garde valable pendant le message).
unsafe fn signaler_l_alimentation(wp: usize, lp: isize) {
    let pbt = wp as u32;
    let ecran = if pbt == PBT_POWERSETTINGCHANGE && lp != 0 {
        // SAFETY: voir la fonction ; on ne lit le DWORD de `Data` qu'après avoir vérifié le
        // GUID et la longueur annoncée.
        unsafe {
            let r = lp as *const windows::Win32::System::Power::POWERBROADCAST_SETTING;
            let ps = std::ptr::addr_of!((*r).PowerSetting).read_unaligned();
            let long = std::ptr::addr_of!((*r).DataLength).read_unaligned();
            (ps == ECRAN_DE_LA_CONSOLE && long >= 4)
                .then(|| std::ptr::addr_of!((*r).Data).cast::<u32>().read_unaligned())
        }
    } else {
        None
    };
    match alimentation(pbt, ecran) {
        Some(Alimentation::Veille) => crate::bords::reprise::veille(),
        Some(Alimentation::Reprise) => crate::bords::reprise::reveil(),
        Some(Alimentation::Ecran { allume }) => crate::bords::reprise::ecran(allume),
        None => {}
    }
}

/// L'icône près de l'horloge, avant son lancement.
pub struct Horloge {
    nom: String,
    poste: Poste,
    reception: Reception,
    messages: Option<Recepteur>,
    arguments: Option<RecepteurArguments>,
    sans_fantome: bool,
}

impl Horloge {
    /// `nom` : le nom court du logiciel, le même que pour [`crate::horloge::instance_unique`].
    pub fn new(nom: &str) -> Horloge {
        let (demandes, reception) = fenetres::canal();
        let liens = Arc::new(Liens::default());
        // Le guichet réveille la fenêtre cachée par son message des commandes, avec son jeton.
        let reveil = Arc::new(ReveilParMessage {
            liens: liens.clone(),
            message: WM_COMMANDE,
            jeton: jeton(),
        });
        Horloge {
            nom: nom.into(),
            poste: Poste::nouveau(liens, demandes, reveil),
            reception,
            messages: None,
            arguments: None,
            sans_fantome: false,
        }
    }

    /// Windows ne remplace jamais une fenêtre du logiciel par son « fantôme » (cadre gris,
    /// « Ne répond pas ») quand elle est occupée un moment : voir
    /// [`crate::systeme::sans_fantome`]. À choisir pour un logiciel dont les fenêtres peuvent
    /// être retenues par un travail long, qu'on ne veut pas voir arrêter de force.
    pub fn sans_fantome(mut self) -> Horloge {
        self.sans_fantome = true;
        self
    }

    /// Reçoit les arguments qu'un second lancement transmet
    /// ([`crate::horloge::Reveil::Arguments`] de [`crate::horloge::instance_unique_avec`]), par
    /// exemple `--choisir=2`. Sans récepteur, ils sont ignorés.
    pub fn recevoir_arguments(mut self, f: impl FnMut(Vec<String>) + 'static) -> Horloge {
        self.arguments = Some(Box::new(f));
        self
    }

    /// Les messages de Windows du logiciel passent par la boucle de l'icône (une seule boucle
    /// pour tout le programme) : voir [`Recepteur`].
    pub fn recevoir_messages(mut self, f: impl FnMut(&MessageWindows) -> Option<isize> + 'static) -> Horloge {
        self.messages = Some(Box::new(f));
        self
    }

    pub fn poste(&self) -> Poste {
        self.poste.clone()
    }

    /// Lance le fil des fenêtres (le volet préparé au repos), pose l'icône et traite les
    /// messages jusqu'à « Quitter ». `parametres` : les Paramètres à ouvrir tout de suite
    /// (`--parametres <page>` : `Some(page)`).
    pub fn lancer(
        self,
        logiciel: Arc<dyn Logiciel>,
        parametres: Option<Option<String>>,
    ) -> Result<(), String> {
        reglage_dpi();
        let Horloge {
            nom,
            poste,
            reception,
            messages,
            arguments,
            sans_fantome,
        } = self;
        if sans_fantome {
            crate::systeme::sans_fantome();
        }
        if let Some(page) = parametres {
            let _ = poste.demandes().send(Demande::Parametres(page));
        }
        // La sonde des fils écrit dans le journal du logiciel (clics, boucles imbriquées, blocages).
        let pour_la_sonde = logiciel.clone();
        crate::sonde::brancher_le_journal(move |ligne| pour_la_sonde.noter(ligne));
        let fil = lancer_le_fil_des_fenetres(
            logiciel.clone(),
            poste.liens().clone(),
            reception,
            secours_de_l_horloge(poste.liens().clone()),
        )?;
        let pour_noter = logiciel.clone();
        let r = boucle(&nom, logiciel, poste.clone(), messages, arguments);
        poste.liens().tout_fermer();
        let _ = poste.demandes().send(Demande::Quitter);
        if !attendre_la_fin(fil, ATTENTE_FERMETURE) {
            pour_noter.noter("Une fenêtre ne répond plus : le logiciel se ferme sans l'attendre.".into());
        }
        r
    }
}

// ---------------------------------------------------------------------------------------
// Le fil principal

/// L'état du fil principal : la zone à une icône, et ce que la bibliothèque en fait. Rangé
/// dans un `Rc` tenu par [`boucle`] ; la procédure de la zone le joint par son complément
/// (un lien faible) : aucune variable globale.
struct Principal {
    logiciel: Arc<dyn Logiciel>,
    poste: Poste,
    zone: Zone,
    taskbar_created: u32,
    /// L'icône du logiciel a pu être fabriquée.
    icone_faite: bool,
    barre_sombre: bool,
    /// Le suivi des clics et l'ordre qui en découle, les mêmes que pour les icônes d'une zone.
    aiguilleur: Aiguilleur,
    /// Ce que fait un clic sur la dernière notification.
    au_clic: AuClic,
    /// Les messages de Windows du logiciel.
    messages: Option<Recepteur>,
    /// Les arguments d'un second lancement.
    arguments: Option<RecepteurArguments>,
    /// Le dernier changement d'icône, et un changement qui attend la fin du délai.
    icone_changee: Option<Instant>,
    icone_attendue: bool,
}

impl Principal {
    fn liens(&self) -> &Liens {
        self.poste.liens()
    }

    /// Un message de la fenêtre cachée ; `Some(résultat)` s'il est traité ici, `None` pour la
    /// zone puis le traitement par défaut de Windows.
    fn message(&mut self, msg: u32, wp: usize, lp: isize) -> Option<isize> {
        // Une remise demandée pendant un menu (l'état était occupé) : faite maintenant.
        self.zone.remettre_si_besoin();
        if !message_permis(msg, wp) {
            // Message interne sans le jeton : venu d'un autre programme, ignoré.
            return Some(0);
        }
        match destinataire(msg, wp, self.taskbar_created) {
            Destinataire::Bibliotheque => {}
            Destinataire::LesDeux => {
                let _ = self.transmettre(msg, wp, lp);
            }
            Destinataire::Logiciel => return self.transmettre(msg, wp, lp),
        }
        self.bibliotheque(msg, wp, lp)
    }

    /// Les messages propres à la bibliothèque, et ceux qu'elle voit avec le logiciel.
    fn bibliotheque(&mut self, msg: u32, wp: usize, lp: isize) -> Option<isize> {
        match msg {
            WM_ICONE => self.icone(wp, lp),
            WM_COMMANDE => self.commandes(),
            WM_TIMER => {
                // La minuterie de l'icône (les autres vont au logiciel).
                // SAFETY: minuterie de cette fenêtre.
                let _ = unsafe { KillTimer(Some(self.zone.hwnd()), MINUTERIE_ICONE) };
                self.icone_attendue = false;
                self.changer_icone();
            }
            // Relancé alors qu'il tourne déjà : sa fenêtre principale, sinon ses Paramètres.
            WM_OUVRIR_PARAMETRES => self.ouvrir_le_logiciel(),
            WM_COPYDATA => {
                // Les arguments d'un second lancement (marque, taille et UTF-8 vérifiés).
                let Some(args) = lire_arguments(lp) else {
                    return Some(0);
                };
                if let Some(f) = self.arguments.as_mut() {
                    f(args);
                }
                return Some(1);
            }
            WM_MENU_DE_SECOURS => {
                let x = (lp & 0xFFFF) as i16;
                let y = ((lp >> 16) & 0xFFFF) as i16;
                self.menu_de_secours(i32::from(x), i32::from(y));
            }
            WM_SETTINGCHANGE => {
                self.redessiner_icone(false);
                return None;
            }
            // Windows refait l'affichage : déjà signalé par la zone (`affichage_change`).
            WM_DISPLAYCHANGE => return None,
            WM_POWERBROADCAST => {
                // Veille, reprise, écran allumé ou éteint : le dessin des fenêtres attend la fin
                // du réveil de Windows (`crate::reprise`).
                // SAFETY: pour PBT_POWERSETTINGCHANGE, Windows passe dans `lp` un
                // POWERBROADCAST_SETTING valable pendant ce message.
                unsafe { signaler_l_alimentation(wp, lp) };
                return None;
            }
            WM_ENDSESSION => {
                if wp != 0 {
                    // La session se ferme : le programme peut être arrêté dès le retour.
                    self.logiciel.noter("Windows ferme la session.".into());
                    self.liens().tout_fermer();
                    self.fermer();
                }
            }
            WM_CLOSE => self.fermer(),
            // SAFETY: fin de la boucle des messages de ce fil.
            WM_DESTROY => unsafe { PostQuitMessage(0) },
            // L'Explorateur a redémarré : remettre l'icône.
            m if m == self.taskbar_created && m != 0 => self.zone.remettre_tout(),
            _ => return None,
        }
        Some(0)
    }

    /// Un message de rappel de l'icône : un geste, ou un clic sur la notification.
    fn icone(&mut self, wp: usize, lp: isize) {
        let Some(e) = self.zone.lire_rappel(ICONE, WPARAM(wp), LPARAM(lp)) else {
            return;
        };
        // Lu avant l'envoi du clic : la note ne s'écrit qu'à la première fois.
        let deja_impossible = self.liens().egui_impossible.load(Ordering::Acquire);
        if let Some(g) = e.message.geste() {
            crate::sonde::etape("geste sur l'icône");
            crate::sonde::clic(g.nom());
        }
        match self.aiguilleur.recevoir(&e, Instant::now(), temps_double_clic()) {
            SuiteDuGeste::Rien => {}
            SuiteDuGeste::Notification(_) => self.clic_bulle(),
            SuiteDuGeste::Secours { point, .. } => {
                if !deja_impossible {
                    // Le fil des fenêtres n'existe plus : le menu de secours, pour que le
                    // clic ouvre quand même quelque chose.
                    self.logiciel.noter(
                        "Clic sur l'icône : le fil des fenêtres ne répond plus ; menu de secours de Windows."
                            .into(),
                    );
                }
                self.menu_de_secours(point[0], point[1]);
            }
        }
    }

    fn redessiner_icone(&mut self, toujours: bool) {
        let barre = choix_windows().barre_sombre;
        if !toujours && self.icone_faite && barre == self.barre_sombre {
            return;
        }
        self.barre_sombre = barre;
        let icone = fabriquer_icone(&*self.logiciel, barre);
        self.icone_faite = icone.is_some();
        // L'ancienne image est détruite par la zone, après le changement.
        self.zone.modifier(ICONE, Some(icone.unwrap_or_default()), None);
    }

    fn commandes(&mut self) {
        for c in self.poste.commandes() {
            match c {
                Commande::Notifier {
                    texte,
                    niveau,
                    au_clic,
                } => {
                    self.au_clic = au_clic;
                    crate::sonde::etape("Shell_NotifyIcon (notification)");
                    let titre = self.logiciel.identite().nom.clone();
                    let _ = self.zone.notifier(ICONE, &titre, &texte, niveau);
                }
                Commande::Infobulle(t) => {
                    let _ = self.zone.modifier(ICONE, None, Some(&t));
                }
                Commande::Icone => self.icone_demandee(),
                Commande::Executer(f) => f(self.zone.fenetre()),
                Commande::Quitter => {
                    self.fermer();
                    return;
                }
            }
        }
    }

    /// La fenêtre principale du logiciel : le volet ou le petit menu ouvert par le premier
    /// clic d'un double-clic se ferme, pour lui laisser la place.
    fn ouvrir_fenetre_principale(&self) {
        if fermer_avant_la_fenetre_principale(self.liens().ouvert()) {
            self.liens().fermer_la_fenetre();
        }
        self.logiciel.ouvrir_fenetre_principale();
    }

    /// Ouvre le logiciel : sa fenêtre principale s'il en a une, sinon ses Paramètres.
    fn ouvrir_le_logiciel(&self) {
        if self.logiciel.fenetre_principale().is_some() {
            self.ouvrir_fenetre_principale();
        } else {
            self.ouvrir_parametres(None);
        }
    }

    fn ouvrir_parametres(&self, page: Option<String>) {
        self.liens().montrer_page(self.poste.demandes(), page);
    }

    /// L'icône doit changer : tout de suite, ou à la fin du délai contre le clignotement.
    fn icone_demandee(&mut self) {
        if self.icone_attendue {
            return;
        }
        match attente_icone(self.icone_changee, Instant::now()) {
            None => self.changer_icone(),
            Some(d) => {
                self.icone_attendue = true;
                // SAFETY: minuterie de la fenêtre de ce fil ; WM_TIMER la rappelle.
                unsafe {
                    SetTimer(
                        Some(self.zone.hwnd()),
                        MINUTERIE_ICONE,
                        d.as_millis().max(1) as u32,
                        None,
                    )
                };
            }
        }
    }

    fn changer_icone(&mut self) {
        self.icone_changee = Some(Instant::now());
        self.redessiner_icone(true);
    }

    /// Passe un message au logiciel ; `Some(résultat)` s'il l'a traité.
    fn transmettre(&mut self, msg: u32, wp: usize, lp: isize) -> Option<isize> {
        let m = MessageWindows {
            fenetre: self.zone.fenetre(),
            message: msg,
            wparam: wp,
            lparam: lp,
        };
        self.messages.as_mut().and_then(|f| f(&m))
    }

    /// Un clic sur la dernière notification.
    fn clic_bulle(&self) {
        match self.au_clic.clone() {
            AuClic::Parametres(page) => self.ouvrir_parametres(page),
            AuClic::Action(cle) => self.logiciel.action(&cle),
        }
    }

    /// Le menu gris de Windows : seulement en secours, si le menu moderne ne peut pas
    /// s'ouvrir (carte graphique sans OpenGL, polices absentes). Les mêmes lignes, dans le
    /// même ordre ; « Ouvrir `<logiciel>` » en gras (la ligne par défaut).
    fn menu_de_secours(&mut self, x: i32, y: i32) {
        let l = &self.logiciel;
        let nom = l.fenetre_principale().map(|_| l.identite().nom.clone());
        let lignes = menu::lignes_avec(l.textes(), nom.as_deref(), &l.actions_menu(), &l.lignes_en_plus());
        let mut defaut = None;
        let gris: Vec<LigneMenu> = lignes
            .iter()
            .enumerate()
            .map(|(i, l)| match l {
                None => LigneMenu::Trait,
                Some(l) => {
                    if l.quoi == Choix::Ouvrir {
                        defaut = Some(i.to_string());
                    }
                    LigneMenu::choix(&i.to_string(), &l.texte)
                }
            })
            .collect();
        let choix = self.zone.menu_windows_complet(
            [x, y],
            &gris,
            defaut.as_deref(),
            "TrackPopupMenu (menu de secours)",
        );
        let Some(Some(l)) = choix
            .and_then(|c| c.parse::<usize>().ok())
            .and_then(|i| lignes.get(i))
        else {
            return;
        };
        match &l.quoi {
            Choix::Action(cle) => self.logiciel.action(cle),
            Choix::Ouvrir => self.ouvrir_fenetre_principale(),
            Choix::Parametres => self.ouvrir_parametres(None),
            Choix::RechercherMiseAJour => {
                self.logiciel.rechercher_mise_a_jour();
                self.ouvrir_parametres(Some(crate::parametres::modele::MISE_A_JOUR.into()));
            }
            Choix::Quitter => self.logiciel.quitter(),
        }
    }

    fn fermer(&mut self) {
        // Retire l'icône, puis détruit la fenêtre (WM_DESTROY arrive pendant que l'état est
        // emprunté : finir ici).
        self.zone.detruire();
        // SAFETY: fin de la boucle des messages de ce fil.
        unsafe { PostQuitMessage(0) };
    }
}

/// Crée la zone à une icône (la fenêtre cachée de l'instance unique) et pose l'icône.
fn creer(
    nom: &str,
    logiciel: Arc<dyn Logiciel>,
    poste: Poste,
    messages: Option<Recepteur>,
    arguments: Option<RecepteurArguments>,
) -> Result<Principal, String> {
    // La seule de sa classe : une deuxième instance la retrouve par ce nom.
    let mut zone = Zone::avec_classe(&classe(nom), &HSTRING::from(nom), ClasseDejaLa::Refuser, nom)?;
    poste.liens().fenetre.store(zone.fenetre(), Ordering::Release);
    // L'écran allumé ou éteint, par WM_POWERBROADCAST (Windows envoie l'état tout de suite,
    // puis à chaque changement) : pendant un écran éteint, les fenêtres ne dessinent pas
    // (`crate::reprise`). L'inscription vit autant que la fenêtre. Sans elle (refusée), seules
    // la veille et la reprise comptent.
    // SAFETY: fenêtre de ce fil, qui vient d'être créée ; GUID constant.
    let _ = unsafe {
        windows::Win32::System::Power::RegisterPowerSettingNotification(
            HANDLE(zone.hwnd().0),
            &ECRAN_DE_LA_CONSOLE,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        )
    };
    let barre_sombre = choix_windows().barre_sombre;
    let icone = fabriquer_icone(&*logiciel, barre_sombre);
    // L'icône seule garde son numéro (sans GUID) et son message de rappel.
    zone.poser_hicon(
        ICONE,
        Ident::Numero(ID_ICONE),
        WM_ICONE,
        icone.unwrap_or_default(),
        nom,
    );
    let aiguilleur =
        Aiguilleur::new(poste.liens().clone(), poste.demandes().clone()).avec_logiciel(logiciel.clone());
    Ok(Principal {
        logiciel,
        taskbar_created: zone.taskbar_created(),
        zone,
        poste,
        icone_faite: icone.is_some(),
        barre_sombre,
        aiguilleur,
        au_clic: AuClic::default(),
        messages,
        arguments,
        icone_changee: None,
        icone_attendue: false,
    })
}

/// Crée la fenêtre cachée et l'icône, puis traite les messages jusqu'à « Quitter ».
fn boucle(
    nom: &str,
    logiciel: Arc<dyn Logiciel>,
    poste: Poste,
    messages: Option<Recepteur>,
    arguments: Option<RecepteurArguments>,
) -> Result<(), String> {
    let principal = Rc::new(RefCell::new(creer(nom, logiciel, poste, messages, arguments)?));
    let h_reveil = principal.try_borrow().map_err(|e| e.to_string())?.zone.fenetre();
    // Surveillé : sa boucle de messages bat à chaque message ; un message vide (WM_NULL) posté par
    // la surveillance le tire de son attente.
    let _surveille = crate::sonde::inscrire(
        "icône",
        Some(Box::new(move || {
            // SAFETY: PostMessageW n'attend rien et accepte une fenêtre déjà détruite (il échoue).
            let _ = unsafe { PostMessageW(Some(hwnd(h_reveil)), WM_NULL, WPARAM(0), LPARAM(0)) };
        })),
    );
    // Les messages de la fenêtre passent d'abord par l'état du fil principal ; pendant qu'il
    // est occupé (un menu ouvert), ils vont à la zone seule.
    let faible = Rc::downgrade(&principal);
    principal
        .try_borrow()
        .map_err(|e| e.to_string())?
        .zone
        .completer(Box::new(move |_h, msg, wp, lp| {
            let p = faible.upgrade()?;
            let mut p = p.try_borrow_mut().ok()?;
            let r = p.message(msg, wp.0, lp.0);
            r.map(LRESULT)
        }));
    let mut m = MSG::default();
    // SAFETY: boucle des messages classique de ce fil, sur un MSG local.
    unsafe {
        crate::sonde::etape("attente d'un message");
        while GetMessageW(&mut m, None, 0, 0).as_bool() {
            crate::sonde::battement();
            crate::sonde::etape("traitement d'un message");
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
            crate::sonde::etape("attente d'un message");
        }
    }
    // La zone retire l'icône et détruit la fenêtre, si `fermer` ne l'a pas déjà fait.
    drop(principal);
    Ok(())
}
