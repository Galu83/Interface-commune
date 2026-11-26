//! Les essais de la zone (bloc M) : ceux de l'ancien `zone.rs`, plus la remise des icônes
//! après le redémarrage de l'Explorateur, le rappel de l'icône seule et la procédure de plus.
//! Les fenêtres sont cachées ; aucune icône n'est donnée à Windows (les appels à
//! `Shell_NotifyIconW` sont notés dans [`APPELS`]).

#![allow(clippy::unwrap_used)]

use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_ENDSESSION};

use super::super::communes::{Aiguilleur, SuiteDuGeste};
use super::super::decodage::Bouton;
use super::*;
use crate::Logiciel;
use crate::clics;
use crate::fenetres::{self, Demande, Liens};
use crate::theme::Surface;

/// Chaque message de Windows qui compte, et lui seul.
#[test]
fn messages_de_l_icone() {
    assert_eq!(Message::de(0x0201), Some(Message::Appui(Bouton::Gauche)));
    assert_eq!(Message::de(0x0205), Some(Message::Relache(Bouton::Droit)));
    assert_eq!(Message::de(0x0203), Some(Message::DoubleClic(Bouton::Gauche)));
    assert_eq!(Message::de(0x0400), Some(Message::Selection));
    assert_eq!(Message::de(0x0401), Some(Message::Clavier));
    assert_eq!(Message::de(0x007B), Some(Message::MenuContextuel));
    assert_eq!(Message::de(0x0405), Some(Message::Notification));
    // Survol, ouverture d'info-bulle : rien.
    assert_eq!(Message::de(0x0200), None);
    assert_eq!(Message::de(0x0406), None);
}

/// Les gestes du volet et du petit menu (`clics`) se tirent des messages.
#[test]
fn gestes_pour_les_clics_communs() {
    assert_eq!(Message::Selection.geste(), Some(clics::Evenement::Selection));
    assert_eq!(
        Message::Relache(Bouton::Gauche).geste(),
        Some(clics::Evenement::Relache)
    );
    assert_eq!(Message::Relache(Bouton::Droit).geste(), None);
    assert_eq!(Message::Notification.geste(), None);
}

/// Le GUID ne change ni d'un lancement à l'autre ni avec la casse du chemin ; il change
/// avec l'icône, le chemin et la graine ; c'est un GUID de version 4.
#[test]
fn guid_stable_et_unique() {
    let a = guid_stable("Essai", r"C:\Programmes\Essai\essai.exe", "souris");
    assert_eq!(
        a,
        guid_stable("Essai", r"c:\programmes\essai\ESSAI.exe", "souris")
    );
    assert_ne!(
        a,
        guid_stable("Essai", r"C:\Programmes\Essai\essai.exe", "clavier")
    );
    assert_ne!(a, guid_stable("Essai", r"D:\Essai\essai.exe", "souris"));
    assert_ne!(
        a,
        guid_stable("Autre", r"C:\Programmes\Essai\essai.exe", "souris")
    );
    assert_eq!((a >> 76) & 0xF, 4, "version 4");
    assert_eq!((a >> 62) & 0x3, 2, "variante");
    // Figé : un changement ferait oublier à Windows les icônes montrées.
    assert_eq!(a, 0xc70b_4318_db8d_4747_8c46_ffab_623b_aed8);
}

/// Les numéros donnés à Windows suivent l'ordre des lignes, sous-menus compris ; les
/// « & » sont doublés.
#[test]
fn menu_windows_numerote_les_choix() {
    let lignes = vec![
        LigneMenu::etat("Souris — 80 %"),
        LigneMenu::Trait,
        LigneMenu::Sous {
            texte: "Styles".into(),
            actif: true,
            lignes: vec![
                LigneMenu::Choix {
                    cle: "style:pixel".into(),
                    texte: "Pixel".into(),
                    coche: true,
                    actif: true,
                },
                LigneMenu::choix("style:lisse", "Lisse"),
            ],
        },
        LigneMenu::choix("quitter", "Quitter"),
    ];
    let mut c = Vec::new();
    cles(&lignes, &mut c);
    assert_eq!(c, ["", "style:pixel", "style:lisse", "quitter"]);
    assert_eq!(texte_menu("Ventes & achats"), "Ventes && achats");
}

#[test]
fn image_depuis_rvba() {
    let i = Image::depuis_rvba(1, vec![1, 2, 3, 4]);
    assert_eq!(i.bgra, [3, 2, 1, 4]);
}

/// L'ajout d'une icône : `NIM_ADD` seul s'il réussit ; sinon `NIM_MODIFY` (icône déjà là) ;
/// la version 4 est redemandée après l'un ou l'autre, jamais si les deux échouent.
#[test]
fn ajout_avec_repli_sur_la_modification() {
    use std::cell::Cell;
    let essai = |ajout: bool, modif: bool| {
        let (a, m, v) = (Cell::new(0), Cell::new(0), Cell::new(0));
        let ok = ajouter_avec_repli(
            || {
                a.set(a.get() + 1);
                ajout
            },
            || {
                m.set(m.get() + 1);
                modif
            },
            || v.set(v.get() + 1),
        );
        (ok, a.get(), m.get(), v.get())
    };
    // Ajout réussi : pas de modification.
    assert_eq!(essai(true, false), (true, 1, 0, 1));
    // L'icône existe déjà : la modification la remet, puis la version 4.
    assert_eq!(essai(false, true), (true, 1, 1, 1));
    // Windows refuse tout : faux, et pas de version à redemander.
    assert_eq!(essai(false, false), (false, 1, 1, 0));
}

/// `WM_ENDSESSION` n'est une fin de session que si `wparam` est vrai ; les options de
/// `lparam` sont décodées, tout autre message est ignoré.
#[test]
fn fin_de_session_decodee() {
    const WM_QUERYENDSESSION: u32 = 0x0011;
    assert_eq!(
        FinDeSession::de(WM_ENDSESSION, 1, 0),
        Some(FinDeSession::default())
    );
    // Fermeture annulée par un autre logiciel : rien à enregistrer.
    assert_eq!(FinDeSession::de(WM_ENDSESSION, 0, 0), None);
    assert_eq!(FinDeSession::de(WM_QUERYENDSESSION, 1, 0), None);
    assert_eq!(FinDeSession::de(WM_DISPLAYCHANGE, 1, 0), None);
    let deconnexion = FinDeSession::de(WM_ENDSESSION, 1, 0x8000_0000_u32 as isize).unwrap();
    assert_eq!(
        deconnexion,
        FinDeSession {
            deconnexion: true,
            ..Default::default()
        }
    );
    let tout = FinDeSession::de(WM_ENDSESSION, 1, 0xC000_0001_u32 as isize).unwrap();
    assert!(tout.deconnexion && tout.critique && tout.application);
    // Les bits hauts d'un `lparam` de 64 bits ne comptent pas.
    let large = (1isize << 40) | 0x4000_0000;
    let c = FinDeSession::de(WM_ENDSESSION, 1, large).unwrap();
    assert!(c.critique && !c.deconnexion && !c.application);
}

/// La fenêtre cachée, pour de vrai : `WM_ENDSESSION` (envoyé, comme Windows le fait) appelle
/// la fonction du logiciel **avant le retour** du message, avec les options ; il reste lisible
/// ensuite par `fin_de_session` ; une fermeture annulée (`wparam` faux) n'appelle rien ; la
/// fonction est gardée d'un message à l'autre ; une fonction qui appelle la zone ne bloque rien.
#[test]
fn fin_de_session_recue_par_la_fenetre_cachee() {
    use std::rc::Rc;
    use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
    let mut z = Zone::creer("EssaiZoneFin", "Essai").unwrap();
    assert_eq!(z.fin_de_session(), None);
    let vus: Rc<RefCell<Vec<FinDeSession>>> = Rc::default();
    let notes = vus.clone();
    z.sur_fin_de_session(move |f| notes.borrow_mut().push(*f));
    let fenetre = z.fenetre;
    let envoyer = |wp: usize, lp: isize| {
        // SAFETY: fenêtre de ce fil, message sans pointeur ; la procédure rend la main avant
        // le retour de `SendMessageW`.
        unsafe { SendMessageW(fenetre, WM_ENDSESSION, Some(WPARAM(wp)), Some(LPARAM(lp))) }
    };
    // Annulée : rien.
    assert_eq!(envoyer(0, 0).0, 0);
    assert!(vus.borrow().is_empty());
    assert_eq!(z.fin_de_session(), None);
    // Déconnexion : la fonction a déjà tout vu au retour du message.
    assert_eq!(envoyer(1, 0x8000_0000_u32 as isize).0, 0, "message traité");
    assert_eq!(
        *vus.borrow(),
        [FinDeSession {
            deconnexion: true,
            ..Default::default()
        }]
    );
    assert_eq!(z.fin_de_session().map(|f| f.deconnexion), Some(true));
    assert_eq!(z.fin_de_session(), None, "lue une seule fois");
    // La fonction est gardée : un arrêt, ensuite.
    envoyer(1, 0);
    assert_eq!(vus.borrow().len(), 2);
    // Sans fonction du logiciel : le signal reste lisible, sans erreur.
    let mut z2 = Zone::creer("EssaiZoneFin2", "Essai").unwrap();
    // SAFETY: comme ci-dessus, pour la seconde fenêtre.
    unsafe { SendMessageW(z2.fenetre, WM_ENDSESSION, Some(WPARAM(1)), Some(LPARAM(0))) };
    assert!(z2.fin_de_session().is_some());
}

/// La fenêtre cachée, pour de vrai, sans rien montrer : aucune icône n'est donnée à
/// Windows (les rappels sont inscrits à la main), des gestes lui sont postés comme
/// Windows le ferait. Chacun revient avec son icône et son point, dans l'ordre ; un
/// message d'une icône inconnue et le survol sont ignorés ; « TaskbarCreated » demande
/// de remettre les icônes.
#[test]
fn gestes_recus_par_la_fenetre_cachee() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };
    let mut z = Zone::creer("EssaiZoneGestes", "Essai").unwrap();
    {
        let mut p = z.partage.borrow_mut();
        p.rappels
            .insert(PREMIER_MESSAGE, ("souris".into(), Ident::Guid(1)));
        p.rappels
            .insert(PREMIER_MESSAGE + 1, ("clavier".into(), Ident::Guid(2)));
    }
    let point = |x: u16, y: u16| WPARAM(usize::from(x) | (usize::from(y) << 16));
    let poster = |msg: u32, wp: WPARAM, lp: isize| {
        // SAFETY: fenêtre de ce fil, message sans pointeur.
        unsafe { PostMessageW(Some(z.fenetre), msg, wp, LPARAM(lp)) }.unwrap();
    };
    poster(PREMIER_MESSAGE, point(100, 200), 0x0400); // NIN_SELECT
    poster(PREMIER_MESSAGE + 1, point(300, 400), 0x007B); // WM_CONTEXTMENU
    poster(PREMIER_MESSAGE, point(1, 1), 0x0200); // survol : ignoré
    poster(PREMIER_MESSAGE + 7, point(1, 1), 0x0400); // icône inconnue : ignorée
    poster(PREMIER_MESSAGE + 1, point(5, 6), 0x0405); // clic sur la notification
    let barre = z.partage.borrow().taskbar_created;
    poster(barre, WPARAM(0), 0);
    let mut m = MSG::default();
    // SAFETY: boucle des messages de ce fil, sur un MSG local.
    unsafe {
        while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
    assert!(z.partage.borrow().a_remettre, "Explorateur redémarré");
    let e = z.evenements();
    assert!(!z.partage.borrow().a_remettre, "icônes remises");
    let vu: Vec<(&str, Message, [i32; 2])> =
        e.iter().map(|e| (e.icone.as_str(), e.message, e.ancre)).collect();
    assert_eq!(
        vu,
        [
            ("souris", Message::Selection, [100, 200]),
            ("clavier", Message::MenuContextuel, [300, 400]),
            ("clavier", Message::Notification, [5, 6]),
        ]
    );
    // Sans icône dans la barre, la place est un petit carré autour de la souris.
    assert!(e.iter().all(|e| e.rect[2] - e.rect[0] == 16));
    assert!(z.evenements().is_empty());
}

/// Les gestes des icônes d'une zone passent par les mécanismes communs, comme l'icône
/// unique : clic gauche, le volet contre l'icône ; un second clic le referme ; clic droit,
/// le petit menu au point du clic ; double-clic, les Paramètres (et le clic qui le suit ne
/// rouvre rien) ; la bulle revient au logiciel ; sans fenêtres modernes, le menu de
/// secours ; le fil des fenêtres parti, le secours pour de bon. Sans aucune fenêtre.
#[test]
fn gestes_par_les_mecanismes_communs() {
    use crate::fenetres::Ouvert;
    let (demandes, reception) = fenetres::canal();
    let liens = Arc::new(Liens::default());
    let mut a = Aiguilleur::new(liens.clone(), demandes);
    let t0 = Instant::now();
    let t = |ms: u64| t0 + Duration::from_millis(ms);
    let double = Duration::from_millis(500);
    assert_eq!(
        a.recevoir(&geste("souris", Message::Selection), t(0), double),
        SuiteDuGeste::Rien
    );
    assert!(matches!(
        reception.prendre(),
        Ok(Some(Demande::Volet {
            icone: [1490, 1040, 1514, 1072],
            ..
        }))
    ));
    // Le volet est montré : un second clic le referme, sans nouvelle demande.
    liens.mettre(Ouvert::Surface(Surface::Volet));
    assert_eq!(
        a.recevoir(&geste("souris", Message::Selection), t(1_000), double),
        SuiteDuGeste::Rien
    );
    assert!(liens.fermer.load(Ordering::Acquire));
    assert_eq!(reception.prendre(), Ok(None));
    liens.mettre(Ouvert::Rien);
    // Clic droit sur une autre icône : le petit menu au point du clic.
    a.recevoir(&geste("clavier", Message::MenuContextuel), t(2_000), double);
    assert!(matches!(
        reception.prendre(),
        Ok(Some(Demande::Menu {
            ancre: [1500, 1050],
            ..
        }))
    ));
    // Le survol et le clic du milieu ne comptent pas.
    assert_eq!(
        a.recevoir(&geste("souris", Message::Appui(Bouton::Milieu)), t(2_100), double),
        SuiteDuGeste::Rien
    );
    assert_eq!(reception.prendre(), Ok(None));
    // La bulle.
    assert_eq!(
        a.recevoir(&geste("souris", Message::Notification), t(2_200), double),
        SuiteDuGeste::Notification("souris".into())
    );
    // Double-clic : les Paramètres ; le NIN_SELECT qui suit ne rouvre pas le volet.
    a.recevoir(
        &geste("souris", Message::DoubleClic(Bouton::Gauche)),
        t(3_000),
        double,
    );
    a.recevoir(
        &geste("souris", Message::Relache(Bouton::Gauche)),
        t(3_050),
        double,
    );
    assert_eq!(reception.prendre(), Ok(Some(Demande::Parametres(None))));
    a.recevoir(&geste("souris", Message::Selection), t(3_060), double);
    assert_eq!(reception.prendre(), Ok(None));
}

/// Un geste d'essai sur une icône, toujours à la même place.
fn geste(icone: &str, message: Message) -> Evenement {
    Evenement {
        icone: icone.into(),
        message,
        ancre: [1500, 1050],
        rect: [1490, 1040, 1514, 1072],
    }
}

/// Sans fenêtres modernes, ou le fil des fenêtres parti : le menu de secours.
#[test]
fn gestes_vers_le_secours() {
    let (demandes, reception) = fenetres::canal();
    let liens = Arc::new(Liens::default());
    let mut a = Aiguilleur::new(liens.clone(), demandes);
    let t0 = Instant::now();
    let t = |ms: u64| t0 + Duration::from_millis(ms);
    let double = Duration::from_millis(500);
    // Sans fenêtres modernes : le menu de secours, au point du clic droit ou à l'icône.
    liens.egui_impossible.store(true, Ordering::Release);
    assert_eq!(
        a.recevoir(&geste("clavier", Message::MenuContextuel), t(5_000), double),
        SuiteDuGeste::Secours {
            icone: "clavier".into(),
            point: [1500, 1050]
        }
    );
    assert_eq!(
        a.recevoir(&geste("souris", Message::Selection), t(6_000), double),
        SuiteDuGeste::Secours {
            icone: "souris".into(),
            point: [1490, 1040]
        }
    );
    // Le fil des fenêtres parti : le secours, pour de bon.
    liens.egui_impossible.store(false, Ordering::Release);
    drop(reception);
    assert!(matches!(
        a.recevoir(&geste("souris", Message::Selection), t(8_000), double),
        SuiteDuGeste::Secours { .. }
    ));
    assert!(liens.egui_impossible.load(Ordering::Acquire));
}

/// Un logiciel d'essai, avec ou sans fenêtre principale ; il compte ses ouvertures.
struct AvecFenetre {
    identite: crate::Identite,
    principale: Option<crate::FenetrePrincipale>,
    ouvertures: std::sync::atomic::AtomicUsize,
}

impl crate::Reglages for AvecFenetre {
    fn identite(&self) -> &crate::Identite {
        &self.identite
    }
    fn action(&self, _: &str) {}
    fn rechercher_mise_a_jour(&self) {}
    fn communs(&self) -> crate::Communs {
        crate::Communs::default()
    }
    fn changer(&self, _: &str, _: crate::Valeur) {}
}

impl Logiciel for AvecFenetre {
    fn etat(&self) -> crate::Etat {
        crate::Etat {
            genre: crate::GenreEtat::Pret,
            titre: "Exemple".into(),
            ligne: "Prêt".into(),
            detail: None,
        }
    }
    fn quitter(&self) {}
    fn noter(&self, _: String) {}
    fn fenetre_principale(&self) -> Option<crate::FenetrePrincipale> {
        self.principale
    }
    fn ouvrir_fenetre_principale(&self) {
        self.ouvertures.fetch_add(1, Ordering::AcqRel);
    }
}

fn avec_fenetre(principale: Option<crate::FenetrePrincipale>) -> Arc<AvecFenetre> {
    Arc::new(AvecFenetre {
        identite: crate::Identite {
            nom: "Exemple".into(),
            version: "0.0.0".into(),
            licence: "MIT".into(),
            description: "Essai.".into(),
            icone: |t, _| vec![255; (t * t * 4) as usize],
            logo: None,
        },
        principale,
        ouvertures: Default::default(),
    })
}

/// Avec une fenêtre principale : le double-clic l'ouvre (le volet du premier clic se
/// ferme), pas les Paramètres ; le clic gauche ouvre le volet. Avec `clic_gauche` : le
/// clic gauche l'ouvre aussi. Sans : rien ne change (les Paramètres au double-clic).
#[test]
fn double_clic_la_fenetre_principale() {
    use crate::fenetres::Ouvert;
    let t0 = Instant::now();
    let t = |ms: u64| t0 + Duration::from_millis(ms);
    let double = Duration::from_millis(500);
    let geste = |message: Message| Evenement {
        icone: "principale".into(),
        message,
        ancre: [1500, 1050],
        rect: [1490, 1040, 1514, 1072],
    };
    let double_clic = |a: &mut Aiguilleur, debut: u64| {
        a.recevoir(&geste(Message::DoubleClic(Bouton::Gauche)), t(debut), double);
        a.recevoir(&geste(Message::Relache(Bouton::Gauche)), t(debut + 50), double);
        a.recevoir(&geste(Message::Selection), t(debut + 60), double);
    };

    // Fenêtre principale, clic gauche : le volet ; double-clic : la fenêtre principale.
    let l = avec_fenetre(Some(crate::FenetrePrincipale::default()));
    let (demandes, reception) = fenetres::canal();
    let liens = Arc::new(Liens::default());
    let mut a = Aiguilleur::new(liens.clone(), demandes).avec_logiciel(l.clone());
    a.recevoir(&geste(Message::Selection), t(0), double);
    assert!(matches!(reception.prendre(), Ok(Some(Demande::Volet { .. }))));
    liens.mettre(Ouvert::Surface(Surface::Volet));
    double_clic(&mut a, 150);
    assert_eq!(l.ouvertures.load(Ordering::Acquire), 1);
    assert!(liens.fermer.load(Ordering::Acquire));
    assert_eq!(reception.prendre(), Ok(None));

    // `clic_gauche` : le clic gauche ouvre la fenêtre principale, sans volet.
    let l = avec_fenetre(Some(crate::FenetrePrincipale { clic_gauche: true }));
    let (demandes, reception) = fenetres::canal();
    let mut a = Aiguilleur::new(Arc::new(Liens::default()), demandes).avec_logiciel(l.clone());
    a.recevoir(&geste(Message::Selection), t(0), double);
    assert_eq!(l.ouvertures.load(Ordering::Acquire), 1);
    assert_eq!(reception.prendre(), Ok(None));
    // Le clic droit garde le petit menu.
    a.recevoir(&geste(Message::MenuContextuel), t(2_000), double);
    assert!(matches!(reception.prendre(), Ok(Some(Demande::Menu { .. }))));

    // Sans fenêtre principale : le double-clic ouvre les Paramètres, comme avant.
    let l = avec_fenetre(None);
    let (demandes, reception) = fenetres::canal();
    let mut a = Aiguilleur::new(Arc::new(Liens::default()), demandes).avec_logiciel(l.clone());
    double_clic(&mut a, 0);
    assert_eq!(l.ouvertures.load(Ordering::Acquire), 0);
    assert_eq!(reception.prendre(), Ok(Some(Demande::Parametres(None))));
}

/// Les appels notés depuis le dernier relevé : (opération, numéro, GUID, message de rappel).
fn releve() -> Vec<(u32, u32, u128, u32)> {
    APPELS.with_borrow_mut(std::mem::take)
}

/// Vide la file des messages de ce fil, en les donnant aux fenêtres.
fn distribuer() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };
    let mut m = MSG::default();
    // SAFETY: boucle des messages de ce fil, sur un MSG local.
    unsafe {
        while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
}

/// L'Explorateur redémarre (« TaskbarCreated ») : rien n'est refait pendant le message ;
/// au passage suivant de la boucle (`evenements`), chaque icône est remise, l'icône seule
/// par son numéro (uID 1, sans GUID), celle de la zone par son GUID, chacune avec son
/// message de rappel et la version 4 ; une seule fois. À la destruction, chacune est
/// retirée. Sans rien montrer.
#[test]
fn remise_apres_le_redemarrage_de_l_explorateur() {
    let classe = HSTRING::from("EssaiZoneRemise");
    let mut z = Zone::avec_classe(&classe, &classe, ClasseDejaLa::Reprendre, "Essai").unwrap();
    releve();
    assert!(z.poser_hicon("seule", Ident::Numero(1), WM_APP + 1, HICON::default(), "Essai"));
    assert_eq!(
        releve(),
        [
            (NIM_ADD.0, 1, 0, WM_APP + 1),
            (NIM_SETVERSION.0, 1, 0, WM_APP + 1)
        ]
    );
    let image = Image {
        taille: 16,
        bgra: vec![255; 16 * 16 * 4],
    };
    assert!(z.poser("souris", &image, "Souris"));
    let g = z.guid("souris");
    assert_eq!(
        releve(),
        [
            (NIM_DELETE.0, 0, g, 0),
            (NIM_ADD.0, 0, g, PREMIER_MESSAGE),
            (NIM_SETVERSION.0, 0, g, PREMIER_MESSAGE)
        ]
    );
    // SAFETY: fenêtre de ce fil, message sans pointeur.
    unsafe { PostMessageW(Some(z.fenetre), z.taskbar_created, WPARAM(0), LPARAM(0)) }.unwrap();
    distribuer();
    assert!(z.partage.borrow().a_remettre);
    assert!(releve().is_empty(), "rien pendant le message");
    assert!(z.evenements().is_empty());
    assert_eq!(
        releve(),
        [
            (NIM_ADD.0, 1, 0, WM_APP + 1),
            (NIM_SETVERSION.0, 1, 0, WM_APP + 1),
            (NIM_ADD.0, 0, g, PREMIER_MESSAGE),
            (NIM_SETVERSION.0, 0, g, PREMIER_MESSAGE)
        ]
    );
    z.remettre_si_besoin();
    assert!(releve().is_empty(), "une seule fois");
    // Le changement d'info-bulle de l'icône seule : une modification, par son numéro.
    assert!(z.modifier("seule", None, Some("Autre")));
    assert_eq!(releve(), [(NIM_MODIFY.0, 1, 0, 0)]);
    z.detruire();
    let mut fin = releve();
    fin.sort();
    assert_eq!(fin, [(NIM_DELETE.0, 0, g, 0), (NIM_DELETE.0, 1, 0, 0)]);
}

/// L'icône seule : ses gestes, lus par `lire_rappel`, ne passent pas par `evenements` ; une
/// icône inconnue ne donne rien.
#[test]
fn rappel_de_l_icone_seule() {
    let classe = HSTRING::from("EssaiZoneRappel");
    let mut z = Zone::avec_classe(&classe, &classe, ClasseDejaLa::Reprendre, "Essai").unwrap();
    z.poser_hicon("seule", Ident::Numero(1), WM_APP + 1, HICON::default(), "Essai");
    let point = WPARAM(100 | (200 << 16));
    let e = z.lire_rappel("seule", point, LPARAM(0x0400)).unwrap();
    assert_eq!(
        (e.icone.as_str(), e.message, e.ancre),
        ("seule", Message::Selection, [100, 200])
    );
    assert!(z.lire_rappel("autre", point, LPARAM(0x0400)).is_none());
    assert!(z.lire_rappel("seule", point, LPARAM(0x0200)).is_none(), "survol");
    // SAFETY: fenêtre de ce fil, message sans pointeur.
    unsafe { PostMessageW(Some(z.fenetre), WM_APP + 1, point, LPARAM(0x0400)) }.unwrap();
    distribuer();
    assert!(z.evenements().is_empty());
    releve();
}

/// La procédure de plus : appelée avant la zone, son résultat est rendu à Windows ; ce
/// qu'elle laisse (`None`) va à la zone (ici, la fin de session) ; un message envoyé pendant
/// son propre appel va à la zone seule ; elle est gardée d'un message à l'autre.
#[test]
fn procedure_de_plus_avant_la_zone() {
    let mut z = Zone::creer("EssaiZoneComplement", "Essai").unwrap();
    let vus: Rc<RefCell<Vec<u32>>> = Rc::default();
    let notes = vus.clone();
    z.completer(Box::new(move |h, msg, _, _| {
        notes.borrow_mut().push(msg);
        if msg == WM_APP + 51 {
            // SAFETY: fenêtre de ce fil, message sans pointeur ; envoyé pendant l'appel.
            let r = unsafe { SendMessageW(h, WM_APP + 50, None, None) };
            return Some(LRESULT(r.0 + 100));
        }
        (msg == WM_APP + 50).then_some(LRESULT(7))
    }));
    let fenetre = z.fenetre;
    let envoyer = |msg: u32, wp: usize| {
        // SAFETY: fenêtre de ce fil, message sans pointeur.
        unsafe { SendMessageW(fenetre, msg, Some(WPARAM(wp)), Some(LPARAM(0))) }.0
    };
    assert_eq!(envoyer(WM_APP + 50, 0), 7);
    assert_eq!(envoyer(WM_ENDSESSION, 1), 0);
    assert!(z.fin_de_session().is_some(), "laissé à la zone");
    // Pendant son appel, le message envoyé va à la zone seule (traitement par défaut : 0).
    assert_eq!(envoyer(WM_APP + 51, 0), 100);
    assert_eq!(envoyer(WM_APP + 50, 0), 7, "gardée");
    assert_eq!(
        *vus.borrow(),
        [WM_APP + 50, WM_ENDSESSION, WM_APP + 51, WM_APP + 50]
    );
}
