//! Les essais de l'instance unique, sans fenêtre : le codage, la lecture et les décisions
//! (purs) ; la lecture prudente d'un `WM_COPYDATA` fabriqué à la main ; le vrai verrou dans
//! `Local\`, avec des noms inventés propres à ce processus. Aucun essai n'ouvre de fenêtre ni
//! n'en cherche une qui existerait vraiment.

// Dans un essai, un échec doit arrêter l'essai : `unwrap` y sont voulus.
#![allow(clippy::unwrap_used)]

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};

use super::codage::{MARQUE_ARGUMENTS, coder_arguments, nom_de_la_fenetre, nom_du_verrou};
use super::*;

/// Un nom de logiciel inventé, propre à cet essai et à ce processus.
fn nom_d_essai(quoi: &str) -> String {
    format!("Essai-bloc-n-{quoi}-{}", std::process::id())
}

/// Le verrou `<espace>\<nom>-une-seule` existe-t-il ? (La poignée ouverte est refermée.)
fn verrou_present(espace: &str, nom: &str) -> bool {
    let n = HSTRING::from(format!("{espace}\\{nom}-une-seule"));
    // SAFETY: nom vivant pendant l'appel ; la poignée rendue est fermée aussitôt.
    unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, &n) }
        .map(|h| {
            // SAFETY: poignée ouverte juste au-dessus, fermée une fois.
            let _ = unsafe { CloseHandle(h) };
        })
        .is_ok()
}

/// Les arguments d'un second lancement font l'aller-retour ; tout ce qui ne vient pas de
/// `coder_arguments` est refusé.
#[test]
fn arguments_du_second_lancement() {
    let args: Vec<String> = ["--choisir=2", "un deux", "é"].map(String::from).into();
    let octets = coder_arguments(&args).unwrap();
    assert_eq!(decoder_arguments(&octets), Some(args));
    assert_eq!(coder_arguments(&[]).as_deref(), Some(&[][..]));
    assert_eq!(decoder_arguments(&[]), Some(vec![]));
    // Trop, trop gros, un zéro dedans.
    let trop: Vec<String> = (0..=ARGUMENTS_MAX).map(|i| i.to_string()).collect();
    assert_eq!(coder_arguments(&trop), None);
    assert_eq!(coder_arguments(&["x".repeat(ARGUMENTS_OCTETS_MAX)]), None);
    assert_eq!(coder_arguments(&["a\0b".into()]), None);
    // Reçu : pas de zéro final, UTF-8 invalide, trop long, trop d'arguments.
    assert_eq!(decoder_arguments(b"abc"), None);
    assert_eq!(decoder_arguments(&[0xFF, 0]), None);
    assert_eq!(decoder_arguments(&vec![b'a'; ARGUMENTS_OCTETS_MAX + 1]), None);
    assert_eq!(decoder_arguments(&[0u8; ARGUMENTS_MAX + 1]), None);
}

/// Les bornes exactes passent : 16 arguments, 4 096 octets en tout (zéros compris) ; un
/// argument vide reste un argument.
#[test]
fn bornes_exactes() {
    assert_eq!((ARGUMENTS_MAX, ARGUMENTS_OCTETS_MAX), (16, 4096));
    let seize: Vec<String> = (0..ARGUMENTS_MAX).map(|i| format!("--a{i}")).collect();
    let octets = coder_arguments(&seize).unwrap();
    assert_eq!(decoder_arguments(&octets), Some(seize));
    // Un seul argument de 4 095 octets + son zéro : 4 096, la borne.
    let plein = vec!["y".repeat(ARGUMENTS_OCTETS_MAX - 1)];
    let octets = coder_arguments(&plein).unwrap();
    assert_eq!(octets.len(), ARGUMENTS_OCTETS_MAX);
    assert_eq!(decoder_arguments(&octets), Some(plein));
    let vides = vec![String::new(), "x".into(), String::new()];
    assert_eq!(coder_arguments(&vides).as_deref(), Some(&b"\0x\0\0"[..]));
    assert_eq!(decoder_arguments(b"\0x\0\0"), Some(vides));
}

/// Les noms du rendez-vous entre deux copies sont des promesses (C-18) : le verrou dans
/// `Local\`, la fenêtre cachée `<nom>-principal`.
#[test]
fn noms_du_rendez_vous() {
    assert_eq!(nom_du_verrou("Fictif"), "Local\\Fictif-une-seule");
    assert_eq!(nom_de_la_fenetre("Fictif"), "Fictif-principal");
    assert_eq!(classe("Fictif"), HSTRING::from("Fictif-principal"));
    assert_eq!(MARQUE_ARGUMENTS.to_be_bytes()[4..], *b"ICAR");
}

/// Ce que dit un second lancement, selon son réveil.
#[test]
fn decisions_du_second_lancement() {
    assert_eq!(que_dire(&Reveil::Rien), Decision::SArreter);
    assert_eq!(que_dire(&Reveil::Parametres), Decision::Ouvrir);
    assert_eq!(que_dire(&Reveil::default()), Decision::Ouvrir);
    let args = vec!["--choisir=2".to_string()];
    assert_eq!(
        que_dire(&Reveil::Arguments(args.clone())),
        Decision::Transmettre(coder_arguments(&args).unwrap())
    );
    // Hors des bornes : rien ne part.
    let trop: Vec<String> = (0..=ARGUMENTS_MAX).map(|i| i.to_string()).collect();
    assert_eq!(
        que_dire(&Reveil::Arguments(trop)),
        Decision::Refuser(ErreurUnique::ArgumentsRefuses)
    );
}

/// La traduction pour l'API d'aujourd'hui, et la ligne du journal d'une demande perdue.
#[test]
fn traduction_pour_l_api() {
    assert_eq!(traduire(Issue::Seul(())), Ok(()));
    assert_eq!(traduire(Issue::<()>::Prevenue), Err(true));
    assert_eq!(traduire(Issue::<()>::RienADemander), Err(true));
    for e in [
        ErreurUnique::FenetreIntrouvable,
        ErreurUnique::MessageNonParti,
        ErreurUnique::PasDeReponse,
        ErreurUnique::ArgumentsNonPris,
        ErreurUnique::ArgumentsRefuses,
    ] {
        assert_eq!(traduire(Issue::<()>::Perdue(e)), Err(false));
        let ligne = ligne_du_journal(&Issue::<()>::Perdue(e)).unwrap();
        assert!(
            ligne.starts_with("Second lancement : demande perdue ("),
            "{ligne}"
        );
        assert!(ligne.contains(&e.to_string()), "{ligne}");
    }
    assert_eq!(ligne_du_journal(&Issue::Seul(())), None);
    assert_eq!(ligne_du_journal(&Issue::<()>::Prevenue), None);
    assert_eq!(ligne_du_journal(&Issue::<()>::RienADemander), None);
}

/// Un `WM_COPYDATA` sans la marque, vide ou mal fait n'est pas lu ; avec elle, il l'est.
#[test]
fn copie_de_donnees_lue_avec_prudence() {
    let octets = coder_arguments(&["--choisir=3".to_string()]).unwrap();
    let mut copie = COPYDATASTRUCT {
        dwData: MARQUE_ARGUMENTS,
        cbData: octets.len() as u32,
        lpData: octets.as_ptr().cast_mut().cast(),
    };
    assert_eq!(
        lire_arguments(&raw const copie as isize),
        Some(vec!["--choisir=3".to_string()])
    );
    copie.dwData = 7;
    assert_eq!(lire_arguments(&raw const copie as isize), None);
    copie.dwData = MARQUE_ARGUMENTS;
    copie.cbData = (ARGUMENTS_OCTETS_MAX + 1) as u32;
    assert_eq!(lire_arguments(&raw const copie as isize), None);
    copie.cbData = 1;
    copie.lpData = std::ptr::null_mut();
    assert_eq!(lire_arguments(&raw const copie as isize), None);
    assert_eq!(lire_arguments(0), None);
    // La vérification avant lecture, seule.
    assert!(copie_a_lire(MARQUE_ARGUMENTS, ARGUMENTS_OCTETS_MAX, false));
    assert!(!copie_a_lire(MARQUE_ARGUMENTS, ARGUMENTS_OCTETS_MAX + 1, false));
    assert!(!copie_a_lire(MARQUE_ARGUMENTS, 0, true));
    assert!(!copie_a_lire(0, 0, false));
}

/// Un second lancement ne fait rien quand c'est ce qu'on lui a dit, et un seul logiciel à la
/// fois tient le verrou.
#[test]
fn second_lancement_sans_rien_dire() {
    let nom = nom_d_essai("rien");
    let premier = instance_unique_avec(&nom, &Reveil::Rien).unwrap();
    // Le verrou est pris : le second est prévenu qu'il n'a rien à faire (sans fenêtre à
    // chercher, donc sans rien dire à personne).
    assert!(matches!(instance_unique_avec(&nom, &Reveil::Rien), Err(true)));
    // Sans fenêtre cachée à l'autre bout, les arguments ne partent pas, ni la demande
    // d'ouverture.
    assert!(matches!(
        instance_unique_avec(&nom, &Reveil::Arguments(vec!["--x".into()])),
        Err(false)
    ));
    assert!(matches!(instance_unique(&nom), Err(false)));
    // Des arguments hors des bornes : perdus aussi, sans rien chercher.
    let trop = vec!["z".repeat(ARGUMENTS_OCTETS_MAX)];
    assert!(matches!(
        instance_unique_avec(&nom, &Reveil::Arguments(trop)),
        Err(false)
    ));
    drop(premier);
    assert!(instance_unique_avec(&nom, &Reveil::Rien).is_ok());
}

/// Le vrai verrou vit dans `Local\` (la session), jamais dans `Global\` ; un second lancement
/// ne le garde pas vivant après lui ; la fin de la première copie le libère (C-18).
#[test]
fn verrou_dans_la_session() {
    let nom = nom_d_essai("local");
    assert!(!verrou_present("Local", &nom), "verrou déjà là avant l'essai");
    let premiere = prendre_le_verrou(&nom).unwrap();
    assert!(verrou_present("Local", &nom));
    assert!(!verrou_present("Global", &nom));
    // Le second essai ne l'obtient pas, et referme aussitôt sa poignée.
    assert!(prendre_le_verrou(&nom).is_none());
    drop(premiere);
    // Plus aucune poignée : le verrou a disparu de la session.
    assert!(!verrou_present("Local", &nom));
    let encore = prendre_le_verrou(&nom).unwrap();
    assert!(verrou_present("Local", &nom));
    drop(encore);
}
