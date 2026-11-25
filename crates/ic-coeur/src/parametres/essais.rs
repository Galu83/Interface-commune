//! Essais des pages des Paramètres : l'ordre de la navigation, la ligne de commande, la
//! recherche, les lignes montrées, la largeur de la colonne (C-80) et la page gardée quand la
//! navigation est refaite.

use super::*;
use crate::contrat::Page;
use crate::textes::FRANCAIS;

fn propres() -> Vec<Page> {
    vec![
        Page::new("ecrans", "Écrans", '\u{E7F4}').avec_mots(&["Luminosité"]),
        // Une clé réservée : ignorée.
        Page::new("general", "Mon général", '\u{E713}'),
        Page::new("dispositions", "Dispositions", '\u{E8A9}'),
    ]
}

/// Quatre pages propres, huit en tout : de quoi montrer « Rechercher un paramètre ».
fn beaucoup() -> Vec<Page> {
    let mut v: Vec<Page> = ["un", "deux", "trois"]
        .iter()
        .map(|c| Page::new(c, c, '\u{E713}'))
        .collect();
    v.push(Page::new("ecrans", "Écrans", '\u{E7F4}').avec_mots(&["Luminosité"]));
    v
}

/// L'Accueil en premier, les pages propres, puis toujours Général, Mise à jour, À propos,
/// avec ces noms et ces icônes (E713, E895, E946).
#[test]
fn ordre_des_pages() {
    let n = navigation(&FRANCAIS, true, &propres());
    let cles: Vec<&str> = n.iter().map(|p| p.cle.as_str()).collect();
    assert_eq!(
        cles,
        [
            "accueil",
            "ecrans",
            "dispositions",
            "general",
            "mise_a_jour",
            "a_propos"
        ]
    );
    let fin: Vec<(&str, char)> = n[3..].iter().map(|p| (p.titre.as_str(), p.icone)).collect();
    assert_eq!(
        fin,
        [
            ("Général", '\u{E713}'),
            ("Mise à jour", '\u{E895}'),
            ("À propos", '\u{E946}')
        ]
    );
    assert_eq!(navigation(&FRANCAIS, false, &[]).len(), 3);
}

#[test]
fn ligne_de_commande() {
    assert_eq!(page_demandee(&["prog"]), None);
    assert_eq!(page_demandee(&["prog", "--parametres"]), Some(None));
    assert_eq!(
        page_demandee(&["prog", "--parametres", "Mise-a-jour"]),
        Some(Some("mise_a_jour".into()))
    );
    assert_eq!(
        page_demandee(&["--parametres=ecrans"]),
        Some(Some("ecrans".into()))
    );
    assert_eq!(page_demandee(&["--parametres", "--autre"]), Some(None));
    assert_eq!(page_demandee(&["--parametres", "../../x"]), Some(None));
    let long = "a".repeat(41);
    assert_eq!(page_demandee(&["--parametres", long.as_str()]), Some(None));
    let n = navigation(&FRANCAIS, false, &propres());
    assert_eq!(rang_de_page(&n, Some("a_propos")), 4);
    assert_eq!(rang_de_page(&n, Some("inconnue")), 0);
    assert_eq!(rang_de_page(&n, None), 0);
    assert_eq!(rang_de_cle(&n, "dispositions"), Some(1));
    assert_eq!(rang_de_cle(&n, "inconnue"), None);
}

#[test]
fn recherche_sans_accents() {
    let n = navigation(&FRANCAIS, true, &propres());
    assert_eq!(chercher(&n, "luminosite"), vec![1]);
    assert_eq!(chercher(&n, "DÉMARRER windows"), vec![3]);
    assert_eq!(chercher(&n, "  "), Vec::<usize>::new());
    assert_eq!(chercher(&n, "introuvable"), Vec::<usize>::new());
    assert_eq!(simplifier("Écran Évolué"), "ecran evolue");
}

/// La recherche ouvre une section : accents, plusieurs mots, rien de tapé.
#[test]
fn section_ouverte_par_la_recherche() {
    let mots = ["Luminosité", "Contraste"];
    assert!(section_trouvee("ecran", "Écrans", &mots));
    assert!(section_trouvee("LUMINOSITE", "Écrans", &mots));
    assert!(section_trouvee("contraste ecrans", "Écrans", &mots));
    assert!(section_trouvee("  contr ", "Écrans", &mots));
    assert!(!section_trouvee("contraste volume", "Écrans", &mots));
    assert!(!section_trouvee("", "Écrans", &mots));
    assert!(!section_trouvee("   ", "Écrans", &mots));
    assert!(!section_trouvee("son", "Écrans", &[]));
    assert!(section_trouvee("avancé", "Réglages avancés", &[]));
}

/// Hors recherche : toutes les lignes, un espace avant Général (la première des trois pages
/// communes) ; sans page propre, pas d'espace en tête.
#[test]
fn lignes_et_espace_avant_les_pages_communes() {
    let n = navigation(&FRANCAIS, true, &propres());
    let l = lignes_de_navigation(&n, "");
    assert_eq!(l.iter().map(|x| x.rang).collect::<Vec<_>>(), [0, 1, 2, 3, 4, 5]);
    let espaces: Vec<usize> = l.iter().filter(|x| x.espace_avant).map(|x| x.rang).collect();
    assert_eq!(espaces, [3]);
    let seules = navigation(&FRANCAIS, false, &[]);
    assert!(lignes_de_navigation(&seules, "").iter().all(|x| !x.espace_avant));
}

/// Le champ de recherche n'apparaît qu'à partir de sept pages ; avant, un texte tapé ne
/// filtre rien. Pendant une recherche : les pages trouvées seules, sans espace.
#[test]
fn recherche_montree_a_partir_de_sept_pages() {
    let peu = navigation(&FRANCAIS, true, &propres());
    assert!(!recherche_montree(&peu));
    assert_eq!(lignes_de_navigation(&peu, "luminosite").len(), peu.len());
    let n = navigation(&FRANCAIS, true, &beaucoup());
    assert_eq!(n.len(), PAGES_POUR_RECHERCHE + 1);
    assert!(recherche_montree(&n));
    let l = lignes_de_navigation(&n, "luminosite");
    assert_eq!(
        l,
        [LigneNavigation {
            rang: 4,
            espace_avant: false
        }]
    );
    // Des espaces seuls : pas une recherche.
    assert_eq!(lignes_de_navigation(&n, "   ").len(), n.len());
    assert!(lignes_de_navigation(&n, "introuvable").is_empty());
}

/// C-80 : la largeur suit l'en-tête et le plus long nom de page, marges comprises, arrondie
/// au point du dessus, bornée entre 250 et 340 points.
#[test]
fn largeur_de_la_colonne() {
    // En-tête : 8 + 32 + 4 + 8 + 100 = 152 ; ligne : 46 + 120,3 + 32 = 198,3 ; marges 16.
    assert_eq!(largeur_navigation(100.0, 80.0, 8.0, [120.3, 60.0]), 250.0);
    // 46 + 200,2 + 32 + 16 = 294,2 → 295.
    assert_eq!(largeur_navigation(100.0, 80.0, 8.0, [200.2]), 295.0);
    // L'en-tête l'emporte : 8 + 32 + 4 + 8 + 230 + 16 = 298 (la version, plus longue).
    assert_eq!(largeur_navigation(100.0, 230.0, 8.0, [10.0]), 298.0);
    assert_eq!(largeur_navigation(100.0, 80.0, 8.0, [500.0]), 340.0);
    assert_eq!(largeur_navigation(0.0, 0.0, 0.0, []), 250.0);
}

/// La navigation refaite : les mêmes clés ne la refont pas ; une page partie renvoie à
/// Général, une page gardée garde son nouveau rang.
#[test]
fn page_gardee_quand_les_pages_changent() {
    let avant = propres();
    let suivi = Suivi::de(true, &avant);
    assert!(suivi.memes_pages(true, &avant));
    assert!(!suivi.memes_pages(false, &avant));
    // Un titre change, pas une clé : la même navigation.
    let mut renommees = avant.clone();
    renommees[0].titre = "Moniteurs".into();
    assert!(suivi.memes_pages(true, &renommees));
    assert!(!suivi.memes_pages(true, &avant[..2]));
    let mut inverse = avant.clone();
    inverse.swap(0, 2);
    assert!(!suivi.memes_pages(true, &inverse));
    assert!(Suivi::default().memes_pages(false, &[]));
    // « Écrans » disparaît : Général ; « Dispositions » passe au rang 1.
    let apres = navigation(&FRANCAIS, true, &avant[2..]);
    assert_eq!(page_suivie(&apres, "ecrans"), PageSuivie::Perdue(2));
    assert_eq!(page_suivie(&apres, "dispositions"), PageSuivie::Gardee(1));
    assert_eq!(page_suivie(&apres, A_PROPOS), PageSuivie::Gardee(4));
}
