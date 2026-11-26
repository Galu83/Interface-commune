//! Les essais de la décision « bloqué » : une fausse horloge (une `Duration` depuis une
//! origine inventée) et des fils simulés (des relevés écrits à la main). Aucune attente réelle.

use std::time::Duration;

use super::*;

fn s(n: u64) -> Duration {
    Duration::from_secs(n)
}

/// Un fil simulé : sa clé, son nom, son compteur, sa dernière étape, sa boucle imbriquée.
fn fil(cle: u8, nom: &'static str, battements: u64) -> Releve<u8, Duration> {
    Releve {
        cle,
        nom,
        battements,
        etape: "",
        imbriquee: None,
    }
}

fn lignes(c: &[Constat]) -> Vec<String> {
    c.iter().map(Constat::ligne).collect()
}

/// Les délais par défaut : un tour toutes les 5 s, bloqué après 15 s.
#[test]
fn delais_par_defaut() {
    let d = Delais::default();
    assert_eq!(d.pas, s(5));
    assert_eq!(d.blocage, s(15));
}

/// C-107 : une ligne quand le fil est bloqué (une seule, même longtemps après), puis une
/// ligne de reprise avec la durée totale ; rien tant qu'il bat.
#[test]
fn un_fil_bloque_donne_une_ligne_puis_une_de_reprise() {
    let mut f = fil(1, "icône", 1);
    f.etape = "Shell_NotifyIcon";
    let mut sv = Surveillance::new(s(15));
    assert!(sv.tour(s(0), &[f]).is_empty());
    assert!(sv.tour(s(5), &[f]).is_empty());
    assert!(sv.tour(s(10), &[f]).is_empty());
    let l = lignes(&sv.tour(s(15), &[f]));
    assert_eq!(
        l,
        ["Le fil « icône » ne répond plus depuis 15 s : dernière étape : Shell_NotifyIcon."]
    );
    assert!(sv.tour(s(20), &[f]).is_empty());
    assert!(sv.tour(s(7800), &[f]).is_empty());
    f.battements = 2;
    assert_eq!(
        sv.tour(s(7805), &[f]),
        [Constat::Repart {
            fil: "icône",
            apres: s(7805)
        }]
    );
    assert_eq!(
        lignes(&sv.tour(s(7805), &[f])),
        Vec::<String>::new(),
        "reparti : rien de plus"
    );
    f.battements = 3;
    assert!(sv.tour(s(7810), &[f]).is_empty());
}

/// Le relevé dit la boucle imbriquée où le fil est pris, et depuis quand.
#[test]
fn un_fil_dans_une_boucle_imbriquee_est_dit_comme_tel() {
    let mut f = fil(1, "icône", 1);
    let mut sv = Surveillance::new(s(15));
    assert!(sv.tour(s(0), &[f]).is_empty());
    f.imbriquee = Some(("TrackPopupMenu", s(2)));
    let l = lignes(&sv.tour(s(20), &[f]));
    assert_eq!(l.len(), 1, "{l:?}");
    assert!(
        l[0].contains("dans « TrackPopupMenu » depuis 18 s, dernière étape : aucune."),
        "{l:?}"
    );
}

/// Un fil qui avance n'écrit rien, si longtemps qu'il tourne.
#[test]
fn un_fil_qui_avance_n_ecrit_rien() {
    let mut sv = Surveillance::new(s(15));
    for i in 0..40u64 {
        let fils = [fil(1, "fenêtres", i), fil(2, "icône", i)];
        assert!(sv.tour(s(5 * i), &fils).is_empty());
    }
}

/// Deux fils, un seul bloqué : seul lui est dit, sans étape (« aucune »).
#[test]
fn deux_fils_un_seul_bloque() {
    let mut sv = Surveillance::new(s(15));
    let mut v = Vec::new();
    for i in 0..5u64 {
        v.extend(lignes(
            &sv.tour(s(5 * i), &[fil(1, "icône", i), fil(2, "fenêtres", 0)]),
        ));
    }
    assert_eq!(
        v,
        ["Le fil « fenêtres » ne répond plus depuis 15 s : dernière étape : aucune."]
    );
}

/// Un fil qui sort de la surveillance est oublié : réinscrit sous la même clé, il repart de
/// zéro.
#[test]
fn un_fil_sorti_est_oublie() {
    let f = fil(1, "essai", 0);
    let mut sv = Surveillance::new(s(15));
    assert!(sv.tour(s(0), &[f]).is_empty());
    assert!(sv.tour(s(10), &[]).is_empty());
    assert!(sv.tour(s(20), &[f]).is_empty(), "revu comme la première fois");
    assert_eq!(sv.tour(s(35), &[f]).len(), 1);
}

/// Les délais réglés (`sonde::regler`) font foi : bloqué après 200 ms, pas plus tôt.
#[test]
fn les_delais_regles_font_foi() {
    let ms = Duration::from_millis;
    let mut sv = Surveillance::des_delais(Delais {
        pas: ms(50),
        blocage: ms(200),
    });
    let f = fil(1, "essai", 0);
    for t in (0..200).step_by(50) {
        assert!(sv.tour(ms(t), &[f]).is_empty(), "à {t} ms");
    }
    assert_eq!(sv.tour(ms(200), &[f]).len(), 1);
}

/// C-107 : le PC dort ; au réveil, aucun fil n'est dit bloqué, une seule ligne dit la reprise ;
/// un vrai blocage ensuite est toujours signalé.
#[test]
fn la_veille_du_pc_n_est_pas_un_blocage() {
    let mut sv = Surveillance::avec_pas(s(15), s(5));
    let deux = |n| [fil(1, "écrans", n), fil(2, "placement", n)];
    assert!(sv.tour(s(0), &deux(1)).is_empty());
    assert!(sv.tour(s(5), &deux(1)).is_empty());
    // Quatre heures de sommeil : les fils n'ont pas battu (le PC dormait).
    let reveil = s(5 + 15_900);
    assert_eq!(
        lignes(&sv.tour(reveil, &deux(1))),
        ["Reprise après une veille d'environ 15900 s : les mesures de blocage repartent de zéro."]
    );
    assert!(sv.tour(reveil + s(5), &deux(2)).is_empty());
    assert!(sv.tour(reveil + s(10), &deux(3)).is_empty());
    // Un vrai blocage après la veille : « placement » ne bat plus depuis reveil + 10 s.
    let t = reveil + s(10);
    let mut v = Vec::new();
    for (i, d) in [5, 10, 15].into_iter().enumerate() {
        let n = 4 + i as u64;
        v.extend(lignes(
            &sv.tour(t + s(d), &[fil(1, "écrans", n), fil(2, "placement", 3)]),
        ));
    }
    assert_eq!(
        v,
        ["Le fil « placement » ne répond plus depuis 15 s : dernière étape : aucune."]
    );
}

/// Sans pas connu, un grand écart reste un blocage : l'horloge est celle de l'appelant.
#[test]
fn sans_pas_un_grand_ecart_reste_un_blocage() {
    let f = fil(1, "icône", 1);
    let mut sv = Surveillance::new(s(15));
    assert!(sv.tour(s(0), &[f]).is_empty());
    let l = lignes(&sv.tour(s(15_900), &[f]));
    assert_eq!(l.len(), 1, "{l:?}");
    assert!(l[0].contains("ne répond plus"), "{l:?}");
}

/// Une veille se reconnaît à un écart de plus de trois pas, et de plus de 3 s.
#[test]
fn reconnaitre_une_veille() {
    let ms = Duration::from_millis;
    assert!(!est_une_veille(s(5), s(5)));
    assert!(!est_une_veille(s(15), s(5)));
    assert!(est_une_veille(s(16), s(5)));
    // Un pas court (le chien des créations) : jamais moins de 3 s.
    assert!(!est_une_veille(ms(2_500), ms(100)));
    assert!(est_une_veille(ms(3_100), ms(100)));
}

fn en_creation(id: u64, nom: &str, debut: Duration) -> EnCreation<Duration> {
    EnCreation {
        id,
        nom: nom.into(),
        debut,
        debut_utc: 14 * 3600 + 3 * 60 + 22,
        signalee: false,
    }
}

/// C-108 : une création de fenêtre qui dure donne une seule ligne, avec le nom et l'heure du
/// début ; rien pour une création courte ; la fin est dite seulement après l'alerte.
#[test]
fn creation_longue_donne_une_ligne() {
    let mut l = vec![en_creation(1, "Paramètres", s(0)), en_creation(2, "Courte", s(5))];
    assert!(lignes_des_creations(&mut l, s(2), SEUIL_DE_CREATION).is_empty());
    assert_eq!(fin_de_creation(&l[0], s(2)), None);
    let v = lignes_des_creations(&mut l, s(6), SEUIL_DE_CREATION);
    assert_eq!(
        v,
        [
            "Fenêtre « Paramètres » : sa création dure depuis 6 s (commencée à 14:03:22 UTC) ; le fil des fenêtres ne rend plus la main."
        ]
    );
    assert!(
        lignes_des_creations(&mut l, s(60), SEUIL_DE_CREATION)
            .iter()
            .all(|x| !x.contains("Paramètres"))
    );
    assert_eq!(
        fin_de_creation(&l[0], s(61)).as_deref(),
        Some(
            "Fenêtre « Paramètres » : la création, commencée à 14:03:22 UTC, s'est enfin terminée après 61 s."
        )
    );
}

/// Le chien des créations : après une veille du PC, une création repart de zéro, sans
/// alerte ; une vraie longueur ensuite est signalée, une fois.
#[test]
fn la_veille_n_allonge_pas_une_creation() {
    let pas = s(1);
    let seuil = s(3);
    let mut l = vec![en_creation(1, "Paramètres", s(0))];
    let mut dernier = s(0);
    assert!(tour_des_creations(&mut l, s(1), &mut dernier, pas, seuil).is_empty());
    let reveil = s(3_600);
    assert!(tour_des_creations(&mut l, reveil, &mut dernier, pas, seuil).is_empty());
    assert!(tour_des_creations(&mut l, reveil + s(1), &mut dernier, pas, seuil).is_empty());
    assert!(tour_des_creations(&mut l, reveil + s(2), &mut dernier, pas, seuil).is_empty());
    let v = tour_des_creations(&mut l, reveil + s(3), &mut dernier, pas, seuil);
    assert_eq!(v.len(), 1, "{v:?}");
    assert!(v[0].contains("« Paramètres »"), "{v:?}");
    assert!(tour_des_creations(&mut l, reveil + s(4), &mut dernier, pas, seuil).is_empty());
}

/// Le pas du chien des créations : un tiers du seuil, entre 20 ms et 1 s.
#[test]
fn pas_du_chien_des_creations() {
    let ms = Duration::from_millis;
    assert_eq!(pas_des_creations(SEUIL_DE_CREATION), s(1));
    assert_eq!(pas_des_creations(ms(300)), ms(100));
    assert_eq!(pas_des_creations(ms(30)), ms(20));
    assert_eq!(pas_des_creations(s(60)), s(1));
}

/// Les autres lignes, mot pour mot.
#[test]
fn les_autres_lignes() {
    assert_eq!(heure_du_jour(14 * 3600 + 3 * 60 + 22), "14:03:22");
    assert_eq!(heure_du_jour(86_400 + 59), "00:00:59");
    assert_eq!(
        ligne_du_clic("droit", 9 * 3600),
        "Icône : clic droit reçu à 09:00:00 UTC."
    );
    assert_eq!(
        debut_d_imbriquee("menu d'essai"),
        "Boucle imbriquée « menu d'essai » : début."
    );
    assert_eq!(
        fin_d_imbriquee("menu d'essai", Duration::from_millis(1234)),
        "Boucle imbriquée « menu d'essai » : fin après 1234 ms."
    );
}

/// C-109 : le lieu d'une panique, sans le chemin du PC, quel que soit le séparateur.
#[test]
fn lieu_de_panique_sans_le_chemin_du_pc() {
    assert_eq!(
        lieu_court(
            r"C:\Users\exemple\.cargo\registry\une-crate-1.0.0\src\native\glow_integration.rs",
            1162
        ),
        "native/glow_integration.rs:1162"
    );
    assert_eq!(lieu_court("src/volet.rs", 7), "src/volet.rs:7");
    assert_eq!(lieu_court("lib.rs", 1), "lib.rs:1");
    assert_eq!(lieu_court("", 3), ":3");
}
