//! Les essais du contrat (bloc A), sans fenêtre ni système : un logiciel inventé, « Fictif »
//! (données inventées, règle n° 1), le même nom que celui des images de référence.

use super::*;

/// L'icône inventée : `taille` × `taille` pixels BGRA, bleu pur (B = 255) sur barre claire,
/// rouge pur (R = 255) sur barre sombre ; alpha 200.
fn icone_fictive(taille: u32, barre_sombre: bool) -> Vec<u8> {
    let pixel = if barre_sombre {
        [0, 0, 255, 200]
    } else {
        [255, 0, 0, 200]
    };
    pixel.repeat((taille * taille) as usize)
}

/// Une icône qui n'a jamais la bonne taille.
fn icone_trop_petite(_taille: u32, _barre_sombre: bool) -> Vec<u8> {
    vec![0; 4]
}

/// Un logo inventé, vert (RVBA), seulement à 32 pixels.
fn logo_fictif(taille: u32) -> Option<Vec<u8>> {
    (taille == 32).then(|| [0, 255, 0, 255].repeat(32 * 32))
}

/// Un logo qui rend une image de la mauvaise taille.
fn logo_tronque(_taille: u32) -> Option<Vec<u8>> {
    Some(vec![0; 8])
}

fn fictif() -> Identite {
    Identite {
        nom: "Fictif".into(),
        version: "1.2.3".into(),
        licence: "MIT".into(),
        description: "Un logiciel inventé pour les essais.".into(),
        icone: icone_fictive,
        logo: None,
    }
}

#[test]
fn sans_logo_l_icone_sert_en_rvba() {
    let identite = fictif();
    let claire = identite.logo_rvba(16, false).expect("l'icône a la bonne taille");
    assert_eq!(claire.len(), 16 * 16 * 4);
    // BGRA (255, 0, 0) devient RVBA (0, 0, 255) : bleu, alpha gardé.
    assert!(claire.chunks(4).all(|p| p == [0, 0, 255, 200]));
    let sombre = identite.logo_rvba(16, true).expect("l'icône a la bonne taille");
    assert!(sombre.chunks(4).all(|p| p == [255, 0, 0, 200]));
}

#[test]
fn le_logo_passe_avant_l_icone() {
    let identite = Identite {
        logo: Some(logo_fictif),
        ..fictif()
    };
    let logo = identite.logo_rvba(32, false).expect("le logo existe à 32 pixels");
    assert!(logo.chunks(4).all(|p| p == [0, 255, 0, 255]));
    // Un logo absent à cette taille ne retombe pas sur l'icône : rien n'est montré.
    assert_eq!(identite.logo_rvba(48, false), None);
}

#[test]
fn une_image_de_la_mauvaise_taille_n_est_pas_rendue() {
    let logo_faux = Identite {
        logo: Some(logo_tronque),
        ..fictif()
    };
    assert_eq!(logo_faux.logo_rvba(16, false), None);
    let icone_fausse = Identite {
        icone: icone_trop_petite,
        ..fictif()
    };
    assert_eq!(icone_fausse.logo_rvba(16, false), None);
    assert_eq!(fictif().logo_rvba(0, false), Some(Vec::new()));
}

#[test]
fn par_defaut_les_pages_communes_ne_montrent_rien_de_facultatif() {
    let c = Communs::default();
    assert_eq!(c.demarrage, None);
    assert_eq!(c.demarrage_bloque, None);
    assert_eq!(c.langue, None);
    assert_eq!(c.notifications, None);
    assert!(!c.notifications_essai);
    assert_eq!(c.journal_detaille, None);
    assert!(!c.journal_effacable);
    assert_eq!(c.mise_a_jour, EtatMiseAJour::Inconnu);
    assert_eq!(c.derniere_recherche, None);
    assert!(c.dossier_installateurs.is_empty());
    assert!(!c.dossier_fixe);
    assert_eq!(c.dossier_automatique, None);
    assert_eq!(c.recherches, None);
    assert!(!c.fichier_licences);
    assert!(c.nouveautes.is_empty());
    assert!(c.credits.is_empty());
}

/// Les clés communes sont un format : le logiciel les compare à ce qu'il reçoit
/// (`Reglages::action`, `Reglages::changer`). Leurs valeurs ne changent pas.
#[test]
fn les_cles_communes_restent() {
    let toutes = [
        (cles::DEMARRAGE, "general.demarrage"),
        (cles::LANGUE, "general.langue"),
        (cles::NOTIFICATIONS, "general.notifications"),
        (cles::NOTIFICATIONS_ESSAYER, "general.notifications_essayer"),
        (cles::JOURNAL_DETAILLE, "general.journal_detaille"),
        (cles::JOURNAL_EFFACER, "journal.effacer"),
        (cles::JOURNAL, "journal"),
        (cles::RETABLIR, "retablir"),
        (cles::DOSSIER, "mise_a_jour.dossier"),
        (cles::INSTALLER, "mise_a_jour.installer"),
        (cles::REDEMARRER, "redemarrer"),
        (cles::RECHERCHES, "mise_a_jour.recherches"),
        (cles::DOSSIER_AUTO, "mise_a_jour.dossier_auto"),
        (cles::LICENCES, "licences"),
    ];
    for (cle, attendue) in toutes {
        assert_eq!(cle, attendue);
        assert!(
            cle.chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c == '.'),
            "{cle} : minuscules, sans espace"
        );
    }
    let mut uniques: Vec<_> = toutes.iter().map(|(c, _)| *c).collect();
    uniques.sort_unstable();
    uniques.dedup();
    assert_eq!(uniques.len(), toutes.len(), "deux clés communes sont égales");
}

#[test]
fn une_page_garde_ses_mots_dans_l_ordre() {
    let page = Page::new("souris", "Souris", '\u{E962}').avec_mots(&["Vitesse", "Boutons"]);
    assert_eq!(page.cle, "souris");
    assert_eq!(page.titre, "Souris");
    assert_eq!(page.icone, '\u{E962}');
    assert_eq!(page.mots, ["Vitesse", "Boutons"]);
    let encore = page.clone().avec_mots(&["Molette"]);
    assert_eq!(encore.mots, ["Vitesse", "Boutons", "Molette"]);
    assert!(Page::new("a", "A", 'a').mots.is_empty());
}

#[test]
fn un_instantane_rend_ce_que_le_logiciel_y_a_mis() {
    #[derive(Debug, PartialEq)]
    struct Brouillon {
        vitesse: u8,
    }
    let pris = Instantane::new(Brouillon { vitesse: 7 });
    assert_eq!(pris.get::<Brouillon>(), Some(&Brouillon { vitesse: 7 }));
    assert_eq!(pris.get::<u8>(), None, "un autre type ne se lit pas");
    // Égal à ses copies, pas à un autre instantané de même contenu.
    assert_eq!(pris.clone(), pris);
    assert_ne!(Instantane::new(Brouillon { vitesse: 7 }), pris);
    // Le journal n'en montre jamais le contenu (aucune donnée de l'utilisateur).
    assert_eq!(format!("{pris:?}"), "Instantane(..)");
}

#[test]
fn les_valeurs_se_comparent() {
    assert_eq!(Valeur::Booleen(true), Valeur::Booleen(true));
    assert_ne!(Valeur::Booleen(true), Valeur::Texte("true".into()));
    assert_ne!(Valeur::Nombre(1.0), Valeur::Nombre(1.5));
    assert_eq!(Valeur::Texte("fr".into()), Valeur::Texte("fr".into()));
}

/// `Reveil::Rien` (ORGANIGRAMME §13, avec le bloc N) : la seconde copie s'arrête sans rien
/// demander. Le défaut reste « ouvrir l'instance en cours » (l'ancien comportement).
#[test]
fn reveil_par_defaut_ouvre_et_rien_ne_demande_rien() {
    assert_eq!(Reveil::default(), Reveil::Parametres);
    assert_ne!(Reveil::Rien, Reveil::Parametres);
    assert_ne!(Reveil::Rien, Reveil::Arguments(Vec::new()));
    let args = Reveil::Arguments(vec!["--choisir=2".into()]);
    assert_eq!(args.clone(), args);
}

#[test]
fn les_defauts_ne_changent_rien() {
    assert_eq!(Niveau::default(), Niveau::Information);
    assert_eq!(EtatMiseAJour::default(), EtatMiseAJour::Inconnu);
    assert!(!FenetrePrincipale::default().clic_gauche);
}

#[test]
fn un_etat_se_compare_champ_par_champ() {
    let pret = Etat {
        genre: GenreEtat::Pret,
        titre: "Fictif".into(),
        ligne: "Prêt".into(),
        detail: None,
    };
    let detail = Etat {
        detail: Some("Une explication inventée.".into()),
        ..pret.clone()
    };
    assert_ne!(pret, detail);
    assert_eq!(pret.clone(), pret);
    assert_ne!(GenreEtat::Attention, GenreEtat::EnCours);
    let credit = Credit {
        nom: "Pièce inventée".into(),
        licence: "MIT".into(),
    };
    assert_eq!(credit.clone(), credit);
}

/// L'icône et la couleur de chaque état, dans les deux modes (bloc G) : vert pour prêt,
/// orange pour en cours et à regarder, rouge pour un problème.
#[test]
fn icone_et_couleur_de_chaque_etat() {
    use crate::theme::{ChoixWindows, icones};
    for sombre in [false, true] {
        let p = Palette::de(&ChoixWindows {
            sombre,
            barre_sombre: sombre,
            accent: None,
            nuances: None,
        });
        assert_eq!(GenreEtat::Pret.icone_et_couleur(&p), (icones::COCHE, p.vert));
        assert_eq!(
            GenreEtat::EnCours.icone_et_couleur(&p),
            (icones::MISE_A_JOUR, p.orange)
        );
        assert_eq!(
            GenreEtat::Attention.icone_et_couleur(&p),
            (icones::ATTENTION, p.orange)
        );
        assert_eq!(
            GenreEtat::Probleme.icone_et_couleur(&p),
            (icones::PROBLEME, p.rouge)
        );
    }
}
