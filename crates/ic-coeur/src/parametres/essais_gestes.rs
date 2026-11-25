//! Essais des gestes des Paramètres sur une fausse horloge (`Duration` depuis le départ) : la
//! bulle « Annuler » (C-70), « Conserver ces modifications ? », ce qui est rendu au logiciel
//! ([`Envoi`]), et le bandeau « Redémarrer maintenant » (C-78).

use std::time::Duration;

use super::envois;
use super::*;
use crate::contrat::{Instantane, Valeur};

/// L'instant `ms` millisecondes après le départ de la fausse horloge.
fn a(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

fn b(v: bool) -> Valeur {
    Valeur::Booleen(v)
}

fn n(v: f64) -> Valeur {
    Valeur::Nombre(v)
}

fn changer_vers(cle: &str, v: Valeur) -> Envoi {
    Envoi::Changer(cle.to_string(), v)
}

#[test]
fn annuler_un_brouillon_entier() {
    let t = a(0);
    let mut an = Annulation::default();
    an.changer_brouillon("Réglages", Instantane::new(vec![1, 2]), t);
    // Une frappe de plus : « Annuler » garde l'état d'avant la première.
    an.changer_brouillon("Réglages", Instantane::new(vec![9, 9]), a(300));
    assert!(an.a_annuler(a(300)).is_some());
    // Un brouillon ne se défait pas réglage par réglage.
    assert!(an.annuler(a(400)).is_empty());
    let avant = an.annuler_brouillon(a(400)).unwrap();
    assert_eq!(avant.get::<Vec<i32>>(), Some(&vec![1, 2]));
    assert_eq!(avant.get::<String>(), None);
    assert!(an.annuler_brouillon(a(400)).is_none());
    // Trop tard : la bulle a disparu.
    an.changer_brouillon("Réglages", Instantane::new(1), t);
    assert!(an.annuler_brouillon(t + DUREE_ANNULER).is_none());
    // Un réglage seul n'est pas un brouillon.
    an.changer("x", "X", b(false), &b(true), t);
    assert!(an.annuler_brouillon(t).is_none());
    assert_eq!(an.annuler(t).len(), 1);
}

/// Effet immédiat et « Annuler » : la bulle garde la valeur d'avant, disparaît après
/// quelques secondes ; des changements de suite du même réglage ne font qu'un.
#[test]
fn effet_immediat_et_annuler() {
    let t = a(0);
    let mut an = Annulation::default();
    an.changer("theme", "Thème", b(false), &b(true), t);
    assert_eq!(an.a_annuler(t).map(|x| x.titre.as_str()), Some("Thème"));
    assert!(an.reste(a(1_000)).is_some());
    assert_eq!(an.annuler(a(1_000)), vec![("theme".into(), b(false))]);
    assert!(an.annuler(a(1_000)).is_empty());
    // Un curseur : 1 → 2 → 3, « Annuler » revient à 1.
    an.changer("vitesse", "Vitesse", n(1.0), &n(2.0), t);
    an.changer("vitesse", "Vitesse", n(2.0), &n(3.0), a(300));
    assert_eq!(an.a_annuler(t).map(|x| x.avant.clone()), Some(n(1.0)));
    // Revenu à la valeur de départ : plus rien à annuler.
    an.changer("vitesse", "Vitesse", n(3.0), &n(1.0), a(600));
    assert!(an.a_annuler(a(600)).is_none());
    // La bulle disparaît après DUREE_ANNULER ; trop tard pour annuler.
    an.changer(
        "nom",
        "Nom",
        Valeur::Texte("a".into()),
        &Valeur::Texte("b".into()),
        t,
    );
    assert!(an.a_annuler(t + DUREE_ANNULER).is_none());
    assert!(an.annuler(t + DUREE_ANNULER).is_empty());
    // Un changement qui ne change rien n'ouvre pas de bulle.
    an.changer("x", "X", b(true), &b(true), t);
    assert!(an.a_annuler(t).is_none());
    // Un groupe : une seule bulle, « Annuler » remet tout ; ce qui ne change pas est oublié.
    an.changer_groupe(
        "Disposition",
        vec![
            ("gauche".into(), n(1.0), n(2.0)),
            ("pareil".into(), b(true), b(true)),
            ("droite".into(), n(3.0), n(4.0)),
        ],
        t,
    );
    assert_eq!(an.a_annuler(t).map(|x| x.titre.as_str()), Some("Disposition"));
    assert_eq!(
        an.annuler(t),
        vec![("gauche".into(), n(1.0)), ("droite".into(), n(3.0))]
    );
    an.changer_groupe("Rien", vec![("x".into(), b(true), b(true))], t);
    assert!(an.a_annuler(t).is_none());
}

/// La bulle reste six secondes, pas une milliseconde de plus ; une horloge qui recule ne la
/// fait pas durer plus.
#[test]
fn la_bulle_disparait_apres_six_secondes() {
    assert_eq!(DUREE_ANNULER, Duration::from_secs(6));
    let mut an = Annulation::default();
    an.changer("theme", "Thème", b(false), &b(true), a(10_000));
    assert_eq!(an.reste(a(10_000)), Some(DUREE_ANNULER));
    assert_eq!(an.reste(a(15_999)), Some(a(1)));
    assert!(an.a_annuler(a(16_000)).is_none());
    // Un instant d'avant le changement compte comme zéro.
    assert_eq!(an.reste(a(9_000)), Some(DUREE_ANNULER));
}

/// Le compte à rebours : 15 s, puis 14… jusqu'à 1 ; il disparaît à 0 ; le prochain
/// redessin tombe juste après le changement de seconde.
#[test]
fn conserver_compte_a_rebours() {
    let t = a(0);
    let mut c = Confirmation::default();
    assert_eq!(DUREE_CONSERVER, Duration::from_secs(15));
    assert!(c.en_attente(t).is_none());
    assert_eq!(c.secondes(t), None);
    c.changer(
        "mode",
        "Mode",
        Valeur::Texte("a".into()),
        &Valeur::Texte("b".into()),
        t,
    );
    assert_eq!(c.en_attente(t).map(|x| x.titre.as_str()), Some("Mode"));
    assert_eq!(c.secondes(t), Some(15));
    assert_eq!(c.secondes(a(1)), Some(15));
    assert_eq!(c.secondes(a(1_000)), Some(14));
    assert_eq!(c.secondes(a(14_000)), Some(1));
    assert_eq!(c.secondes(a(14_999)), Some(1));
    assert_eq!(c.secondes(a(15_000)), None);
    assert_eq!(c.reste(a(5_000)), Some(a(10_000)));
    // Au milieu d'une seconde : redessiner à la fin de celle-ci ; pile sur une seconde :
    // dans une seconde. Toujours un peu après le changement.
    assert_eq!(c.prochain_changement(a(500)), Some(a(505)));
    assert_eq!(c.prochain_changement(a(1_000)), Some(a(1_005)));
    assert_eq!(c.prochain_changement(a(15_000)), None);
}

/// « Conserver » : les changements restent, plus rien n'attend, rien n'est rendu.
#[test]
fn conserver_garde_les_changements() {
    let t = a(0);
    let mut c = Confirmation::default();
    c.changer("x", "X", b(false), &b(true), t);
    assert!(c.conserver(a(3_000)));
    assert!(c.en_attente(a(3_000)).is_none());
    assert!(c.revenir().est_vide());
    assert!(c.echu(a(60_000)).is_none());
    // Rien n'attend : rien à conserver.
    assert!(!c.conserver(t));
    // Un changement qui ne change rien n'ouvre rien.
    c.changer("x", "X", b(true), &b(true), t);
    assert!(c.en_attente(t).is_none());
}

/// « Revenir » rend la valeur d'avant ; la fermeture des Paramètres fait de même (même
/// appel) ; une seconde fois : rien.
#[test]
fn revenir_rend_la_valeur_d_avant() {
    let t = a(0);
    let mut c = Confirmation::default();
    c.changer("mode", "Mode", n(1.0), &n(2.0), t);
    let r = c.revenir();
    assert_eq!(r.valeurs, vec![("mode".to_string(), n(1.0))]);
    assert!(r.brouillon.is_none());
    assert!(c.en_attente(t).is_none());
    assert!(c.revenir().est_vide());
}

/// Délai écoulé : retour automatique, une seule fois ; trop tard pour « Conserver ».
#[test]
fn delai_ecoule_retour_automatique() {
    let t = a(0);
    let mut c = Confirmation::default();
    c.changer("mode", "Mode", n(1.0), &n(2.0), t);
    assert!(c.echu(a(14_999)).is_none());
    assert!(c.en_attente(a(14_999)).is_some());
    // Un clic arrivé à l'instant où le délai finit ne garde rien.
    assert!(!c.conserver(t + DUREE_CONSERVER));
    let r = c.echu(t + DUREE_CONSERVER).expect("retour automatique");
    assert_eq!(r.valeurs, vec![("mode".to_string(), n(1.0))]);
    assert!(c.echu(t + DUREE_CONSERVER).is_none());
    assert!(c.revenir().est_vide());
}

/// Deux changements de suite : une seule confirmation, l'état d'avant le PREMIER est
/// rendu, et le compte à rebours repart au second. Revenir à la valeur de départ : plus
/// rien à confirmer.
#[test]
fn deux_changements_gardent_l_avant_du_premier() {
    let t = a(0);
    let mut c = Confirmation::default();
    c.changer("mode", "Mode", n(1.0), &n(2.0), t);
    c.changer("mode", "Mode", n(2.0), &n(3.0), a(10_000));
    // Le compte à rebours est reparti : 15 s après le second.
    assert_eq!(c.secondes(a(10_000)), Some(15));
    assert!(c.echu(a(20_000)).is_none());
    // Un autre réglage s'ajoute ; le titre est celui du dernier.
    c.changer("souris", "Souris", n(5.0), &n(6.0), a(12_000));
    assert_eq!(c.en_attente(a(12_000)).map(|x| x.titre.as_str()), Some("Souris"));
    let r = c.revenir();
    assert_eq!(
        r.valeurs,
        vec![("mode".to_string(), n(1.0)), ("souris".to_string(), n(5.0))]
    );
    // Revenu à la valeur de départ à la main : plus rien à confirmer.
    c.changer("mode", "Mode", n(1.0), &n(2.0), t);
    c.changer("mode", "Mode", n(2.0), &n(1.0), a(500));
    assert!(c.en_attente(a(500)).is_none());
    assert!(c.revenir().est_vide());
    // Un groupe, puis un autre changement : tout revient, une seule fois chacun.
    c.changer_groupe(
        "Écran",
        vec![
            ("gauche".into(), n(1.0), n(2.0)),
            ("pareil".into(), n(7.0), n(7.0)),
            ("droite".into(), n(3.0), n(4.0)),
        ],
        t,
    );
    c.changer("gauche", "Gauche", n(2.0), &n(9.0), a(100));
    let r = c.revenir();
    assert_eq!(
        r.valeurs,
        vec![("gauche".to_string(), n(1.0)), ("droite".to_string(), n(3.0))]
    );
    c.changer_groupe("Rien", vec![("x".into(), n(1.0), n(1.0))], t);
    assert!(c.en_attente(t).is_none());
}

/// Un brouillon entier : l'état d'avant le premier changement est rendu, avec les
/// valeurs seules s'il y en a ; un second brouillon n'écrase pas le premier.
#[test]
fn brouillon_a_confirmer() {
    let t = a(0);
    let mut c = Confirmation::default();
    c.changer_brouillon("Réglages", Instantane::new(vec![1, 2]), t);
    c.changer_brouillon("Réglages", Instantane::new(vec![9]), a(2_000));
    c.changer("mode", "Mode", b(false), &b(true), a(3_000));
    assert_eq!(c.secondes(a(3_000)), Some(15));
    let r = c.revenir();
    assert!(!r.est_vide());
    assert_eq!(
        r.brouillon.as_ref().and_then(|x| x.get::<Vec<i32>>()),
        Some(&vec![1, 2])
    );
    assert_eq!(r.valeurs, vec![("mode".to_string(), b(false))]);
    assert!(c.revenir().est_vide());
}

/// Un changement est rendu au logiciel tout de suite, même s'il ne change rien (comme
/// avant) ; un groupe ne rend que ce qui change ; « Annuler » rend les valeurs d'avant.
#[test]
fn envois_d_un_changement_et_d_annuler() {
    let mut an = Annulation::default();
    assert_eq!(
        envois::changer(&mut an, "theme", "Thème", b(false), b(true), a(0)),
        [changer_vers("theme", b(true))]
    );
    assert_eq!(
        envois::changer(&mut an, "x", "X", b(true), b(true), a(0)),
        [changer_vers("x", b(true))]
    );
    let groupe = vec![
        ("gauche".into(), n(1.0), n(2.0)),
        ("pareil".into(), b(true), b(true)),
    ];
    assert_eq!(
        envois::changer_groupe(&mut an, "Disposition", groupe, a(100)),
        [changer_vers("gauche", n(2.0))]
    );
    assert_eq!(envois::annuler(&mut an, a(200)), [changer_vers("gauche", n(1.0))]);
    assert!(envois::annuler(&mut an, a(200)).is_empty());
    // Un brouillon : « Annuler » le remet entier.
    let avant = Instantane::new(3_u8);
    an.changer_brouillon("Réglages", avant.clone(), a(300));
    assert_eq!(envois::annuler(&mut an, a(400)), [Envoi::Remettre(avant)]);
}

/// Le délai de « Conserver » écoulé : le brouillon d'avant d'abord, puis les valeurs, et la
/// bulle se ferme ; une seule fois.
#[test]
fn echu_rend_le_brouillon_d_abord() {
    let mut an = Annulation::default();
    let mut co = Confirmation::default();
    let avant = Instantane::new("brouillon d'avant".to_string());
    assert!(envois::brouillon_a_confirmer(&mut an, &mut co, "Réglages", avant.clone(), a(0)).is_empty());
    let v = envois::changer_a_confirmer(&mut an, &mut co, "mode", "Mode", n(1.0), n(2.0), a(1_000));
    assert_eq!(v, [changer_vers("mode", n(2.0))]);
    an.changer("autre", "Autre", b(false), &b(true), a(2_000));
    assert!(envois::echu(&mut an, &mut co, a(15_999)).is_none());
    assert_eq!(
        envois::echu(&mut an, &mut co, a(16_000)),
        Some(vec![Envoi::Remettre(avant), changer_vers("mode", n(1.0))])
    );
    assert!(an.a_annuler(a(16_000)).is_none());
    assert!(envois::echu(&mut an, &mut co, a(16_000)).is_none());
}

/// Un changement à confirmer pendant le compte à rebours : l'« avant » du premier reste, et
/// le compte repart à 15 s ; il ferme la bulle « Annuler ». Après le délai, le retour passe
/// d'abord, puis le nouveau changement.
#[test]
fn changer_a_confirmer_pendant_et_apres_le_delai() {
    let mut an = Annulation::default();
    let mut co = Confirmation::default();
    an.changer("theme", "Thème", b(false), &b(true), a(0));
    envois::changer_a_confirmer(&mut an, &mut co, "mode", "Mode", n(1.0), n(2.0), a(0));
    assert!(an.a_annuler(a(0)).is_none());
    let v = envois::changer_a_confirmer(&mut an, &mut co, "mode", "Mode", n(2.0), n(3.0), a(10_000));
    assert_eq!(v, [changer_vers("mode", n(3.0))]);
    assert_eq!(co.secondes(a(10_000)), Some(15));
    assert_eq!(
        co.en_attente(a(10_000)).map(|x| x.valeurs.clone()),
        Some(vec![("mode".into(), n(1.0))])
    );
    // Le délai est passé sans réponse : le retour d'abord, puis le nouveau changement.
    let v = envois::changer_a_confirmer(&mut an, &mut co, "souris", "Souris", n(5.0), n(6.0), a(30_000));
    assert_eq!(v, [changer_vers("mode", n(1.0)), changer_vers("souris", n(6.0))]);
    // Rien ne change : rien n'est rendu, et rien n'attend de plus.
    let v = envois::changer_a_confirmer(&mut an, &mut co, "souris", "Souris", n(6.0), n(6.0), a(31_000));
    assert!(v.is_empty());
    assert_eq!(
        envois::revenir(&mut an, &mut co),
        [changer_vers("souris", n(5.0))]
    );
    assert!(envois::revenir(&mut an, &mut co).is_empty());
}

/// Un groupe à confirmer : ce qui change est rendu ; un groupe qui ne change rien n'ouvre
/// rien et ne ferme pas la bulle.
#[test]
fn groupe_a_confirmer() {
    let mut an = Annulation::default();
    let mut co = Confirmation::default();
    an.changer("theme", "Thème", b(false), &b(true), a(0));
    let rien = vec![("x".into(), n(1.0), n(1.0))];
    assert!(envois::changer_groupe_a_confirmer(&mut an, &mut co, "Rien", rien, a(0)).is_empty());
    assert!(an.a_annuler(a(0)).is_some());
    assert!(co.en_attente(a(0)).is_none());
    let groupe = vec![
        ("gauche".into(), n(1.0), n(2.0)),
        ("pareil".into(), n(7.0), n(7.0)),
        ("droite".into(), n(3.0), n(4.0)),
    ];
    let v = envois::changer_groupe_a_confirmer(&mut an, &mut co, "Écran", groupe, a(100));
    assert_eq!(
        v,
        [changer_vers("gauche", n(2.0)), changer_vers("droite", n(4.0))]
    );
    assert!(an.a_annuler(a(100)).is_none());
    assert_eq!(
        envois::revenir(&mut an, &mut co),
        [changer_vers("gauche", n(1.0)), changer_vers("droite", n(3.0))]
    );
}

/// C-78 : la dernière raison demandée l'emporte ; une image sans demande ne l'efface pas.
#[test]
fn bandeau_redemarrer() {
    let mut bandeau = None;
    noter_redemarrage(&mut bandeau, None);
    assert_eq!(bandeau, None);
    noter_redemarrage(&mut bandeau, Some("Nouvelle langue".into()));
    noter_redemarrage(&mut bandeau, None);
    assert_eq!(bandeau.as_deref(), Some("Nouvelle langue"));
    noter_redemarrage(&mut bandeau, Some("Nouveau pilote".into()));
    assert_eq!(bandeau.as_deref(), Some("Nouveau pilote"));
}

/// Un retour rend le brouillon d'abord, puis les valeurs dans l'ordre.
#[test]
fn retour_en_envois() {
    let avant = Instantane::new(1_u8);
    let r = Retour {
        valeurs: vec![("a".into(), n(1.0)), ("b".into(), n(2.0))],
        brouillon: Some(avant.clone()),
    };
    assert_eq!(
        r.envois(),
        [
            Envoi::Remettre(avant),
            changer_vers("a", n(1.0)),
            changer_vers("b", n(2.0))
        ]
    );
    assert!(Retour::default().envois().is_empty());
}
