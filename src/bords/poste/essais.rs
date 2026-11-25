//! Les essais du guichet, sans fenêtre ni matériel : un faux réveil prend la place du message
//! Windows à la fenêtre cachée.

#![allow(clippy::unwrap_used)]

use std::sync::atomic::{AtomicIsize, AtomicUsize};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Barrier, Mutex};

use super::*;
use crate::fenetres::{Demande, Reception, canal};
use crate::horloge::Horloge;

/// Le faux réveil : il compte les réveils et, si on le lui demande, prévient un faux fil de
/// l'icône.
#[derive(Default)]
struct FauxReveil {
    compte: AtomicUsize,
    signal: Mutex<Option<Sender<()>>>,
}

impl Reveiller for FauxReveil {
    fn reveiller(&self) {
        self.compte.fetch_add(1, Ordering::SeqCst);
        if let Some(s) = self.signal.lock().unwrap().as_ref() {
            let _ = s.send(());
        }
    }
}

impl FauxReveil {
    fn compte(&self) -> usize {
        self.compte.load(Ordering::SeqCst)
    }
}

/// Un guichet branché sur le faux réveil, et l'autre bout du canal de la boucle des fenêtres.
fn guichet() -> (Poste, Arc<FauxReveil>, Reception) {
    let (demandes, reception) = canal();
    let reveil = Arc::new(FauxReveil::default());
    let poste = Poste::nouveau(Arc::new(Liens::default()), demandes, reveil.clone());
    (poste, reveil, reception)
}

/// Le texte de chaque notification en attente (les autres messages sont ignorés).
fn textes(commandes: Vec<Commande>) -> Vec<String> {
    commandes
        .into_iter()
        .filter_map(|c| match c {
            Commande::Notifier { texte, .. } => Some(texte),
            _ => None,
        })
        .collect()
}

/// Les numéros reçus de `fil`, dans l'ordre (textes « fil-numéro »).
fn de_ce_fil(textes: &[String], fil: usize) -> Vec<usize> {
    textes
        .iter()
        .filter_map(|t| t.split_once('-'))
        .filter(|(f, _)| f.parse::<usize>().ok() == Some(fil))
        .map(|(_, n)| n.parse().unwrap())
        .collect()
}

/// Le guichet se garde et se clone dans n'importe quel fil.
#[test]
fn le_guichet_passe_d_un_fil_a_l_autre() {
    fn assure<T: Send + Sync + Clone + 'static>() {}
    assure::<Poste>();
}

/// Une rafale de demandes depuis huit fils à la fois, autant que la boîte en prend : aucune ne
/// se perd, chacune réveille le fil de l'icône, et l'ordre de chaque fil est gardé.
#[test]
fn rafales_de_plusieurs_fils() {
    const FILS: usize = 8;
    const PAR_FIL: usize = CAPACITE / FILS;
    let (poste, reveil, reception) = guichet();
    let depart = Barrier::new(FILS);
    std::thread::scope(|s| {
        for f in 0..FILS {
            let (poste, depart) = (poste.clone(), &depart);
            s.spawn(move || {
                depart.wait();
                for n in 0..PAR_FIL {
                    poste.notifier(&format!("{f}-{n}"), false, None);
                }
            });
        }
    });
    assert_eq!(reveil.compte(), FILS * PAR_FIL);
    let recus = textes(poste.commandes());
    assert_eq!(recus.len(), FILS * PAR_FIL);
    for f in 0..FILS {
        assert_eq!(de_ce_fil(&recus, f), (0..PAR_FIL).collect::<Vec<_>>(), "fil {f}");
    }
    // Rien n'est allé à la boucle des fenêtres, et la boîte est vide une fois lue.
    assert_eq!(reception.prendre(), Ok(None));
    assert!(poste.commandes().is_empty());
}

/// Une rafale plus grande que la boîte : les messages en trop sont refusés (la mémoire ne
/// grandit pas), mais chaque demande réveille quand même le fil, et ce qui est gardé est le
/// début de chaque fil, dans l'ordre. Une fois la boîte lue, elle reprend des messages.
#[test]
fn boite_pleine_en_rafale() {
    const FILS: usize = 4;
    const PAR_FIL: usize = 40;
    let (poste, reveil, _reception) = guichet();
    std::thread::scope(|s| {
        for f in 0..FILS {
            let poste = poste.clone();
            s.spawn(move || {
                for n in 0..PAR_FIL {
                    poste.notifier(&format!("{f}-{n}"), true, None);
                }
            });
        }
    });
    assert_eq!(reveil.compte(), FILS * PAR_FIL);
    let recus = textes(poste.commandes());
    assert_eq!(recus.len(), CAPACITE);
    for f in 0..FILS {
        let gardes = de_ce_fil(&recus, f);
        assert_eq!(gardes, (0..gardes.len()).collect::<Vec<_>>(), "fil {f}");
    }
    poste.infobulle("De nouveau.");
    assert!(matches!(
        poste.commandes().as_slice(),
        [Commande::Infobulle(t)] if t == "De nouveau."
    ));
}

/// Un faux fil de l'icône vide la boîte à chaque réveil pendant que quatre fils y déposent :
/// il reçoit exactement ce qui a été accepté, une seule fois, dans l'ordre de chaque fil, et
/// rien ne reste en attente après le dernier réveil.
#[test]
fn le_fil_reveille_trouve_tout() {
    const FILS: usize = 4;
    const PAR_FIL: usize = 500;
    let boite = Boite::<(usize, usize)>::default();
    let reveil = FauxReveil::default();
    let (signal, sonne) = channel();
    *reveil.signal.lock().unwrap() = Some(signal);
    let acceptes = AtomicUsize::new(0);
    let recus = std::thread::scope(|s| {
        let boite = &boite;
        let consommateur = s.spawn(move || {
            let mut recus = Vec::new();
            while sonne.recv().is_ok() {
                recus.extend(boite.vider());
            }
            recus
        });
        let producteurs: Vec<_> = (0..FILS)
            .map(|f| {
                let (reveil, acceptes) = (&reveil, &acceptes);
                s.spawn(move || {
                    for n in 0..PAR_FIL {
                        if boite.deposer((f, n)) {
                            acceptes.fetch_add(1, Ordering::SeqCst);
                        }
                        reveil.reveiller();
                    }
                })
            })
            .collect();
        for p in producteurs {
            p.join().unwrap();
        }
        // Plus aucun dépôt : le faux fil de l'icône finit les réveils en attente, puis s'arrête.
        *reveil.signal.lock().unwrap() = None;
        consommateur.join().unwrap()
    });
    assert_eq!(reveil.compte(), FILS * PAR_FIL);
    assert_eq!(recus.len(), acceptes.load(Ordering::SeqCst));
    assert!(!recus.is_empty());
    for f in 0..FILS {
        let de_f: Vec<usize> = recus.iter().filter(|m| m.0 == f).map(|m| m.1).collect();
        assert!(de_f.windows(2).all(|w| w[0] < w[1]), "fil {f} : {de_f:?}");
    }
    assert!(boite.vider().is_empty());
}

/// Chaque demande va au bon fil : notifier, l'infobulle, l'icône et le code du logiciel au fil
/// de l'icône (réveillé) ; redessiner et ouvrir les Paramètres à la boucle des fenêtres (le fil
/// de l'icône n'est pas dérangé) ; quitter ferme les fenêtres puis arrête le fil de l'icône.
#[test]
fn chaque_demande_va_au_bon_fil() {
    let (poste, reveil, reception) = guichet();
    let adresse = Arc::new(AtomicIsize::new(0));
    poste.notifier("Prêt.", false, None);
    poste.infobulle("Fictif");
    poste.redessiner_icone();
    let a = adresse.clone();
    poste.sur_le_fil_principal(move |h| a.store(h, Ordering::SeqCst));
    assert_eq!(reveil.compte(), 4);
    assert_eq!(reception.prendre(), Ok(None));
    let commandes = poste.commandes();
    assert!(matches!(
        commandes.as_slice(),
        [
            Commande::Notifier { .. },
            Commande::Infobulle(_),
            Commande::Icone,
            Commande::Executer(_)
        ]
    ));
    // Le code du logiciel reçoit l'adresse de la fenêtre cachée, sur le fil qui le fait.
    if let Some(Commande::Executer(f)) = commandes.into_iter().last() {
        f(0x1234);
    }
    assert_eq!(adresse.load(Ordering::SeqCst), 0x1234);

    poste.redessiner();
    poste.ouvrir_parametres(Some("general"));
    assert_eq!(
        reception.prendre(),
        Ok(Some(Demande::Parametres(Some("general".into()))))
    );
    assert_eq!(reveil.compte(), 4);
    assert!(poste.commandes().is_empty());

    poste.quitter();
    assert!(poste.liens().quitter.load(Ordering::Acquire));
    assert!(poste.liens().fermer.load(Ordering::Acquire));
    assert_eq!(reveil.compte(), 5);
    assert!(matches!(poste.commandes().as_slice(), [Commande::Quitter]));
}

/// Sans fenêtre cachée (avant `Horloge::lancer`), le vrai réveil ne poste rien et ne bloque
/// pas : les demandes attendent dans la boîte.
#[test]
fn sans_fenetre_les_demandes_attendent() {
    let p = Horloge::new("essai-guichet").poste();
    p.infobulle("Fictif");
    p.redessiner_icone();
    assert_eq!(p.commandes().len(), 2);
}

// Venus de `horloge.rs` (bloc K), vérifications gardées.

/// Trois niveaux de notification.
#[test]
fn notification_a_trois_niveaux() {
    let p = Horloge::new("essai-niveaux").poste();
    p.notifier("Info.", false, None);
    p.notifier_avec("Attention.", Niveau::Avertissement, AuClic::Action("voir".into()));
    p.notifier("Erreur.", true, None);
    let niveaux: Vec<Niveau> = p
        .commandes()
        .iter()
        .filter_map(|c| match c {
            Commande::Notifier { niveau, .. } => Some(*niveau),
            _ => None,
        })
        .collect();
    assert_eq!(
        niveaux,
        [Niveau::Information, Niveau::Avertissement, Niveau::Erreur]
    );
}

/// Du code du logiciel attend son tour sur le fil de la fenêtre cachée.
#[test]
fn code_pour_le_fil_principal() {
    let p = Horloge::new("essai-fil").poste();
    p.sur_le_fil_principal(|_| {});
    assert!(matches!(p.commandes().first(), Some(Commande::Executer(_))));
}

/// Une notification ouvre les Paramètres à sa page, ou rend une action au logiciel.
#[test]
fn clic_sur_une_notification() {
    let h = Horloge::new("essai-notification");
    let p = h.poste();
    p.notifier("Mise à jour prête.", false, Some("mise_a_jour"));
    p.notifier_action("Lecture finie.", false, "montrer");
    let clics: Vec<AuClic> = p
        .commandes()
        .iter()
        .filter_map(|c| match c {
            Commande::Notifier { au_clic, .. } => Some(au_clic.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        clics,
        vec![
            AuClic::Parametres(Some("mise_a_jour".into())),
            AuClic::Action("montrer".into())
        ]
    );
}
