//! Essais de la page « Mise à jour » : l'icône, sa couleur et le titre selon l'état, la ligne
//! grise, l'explication des recherches, le bouton de la carte et la carte du dossier.

use super::*;
use crate::contrat::{Communs, EtatMiseAJour};
use crate::textes::{ANGLAIS, FRANCAIS, Textes};
use crate::theme::{ChoixWindows, Palette, icones};

/// Recherches actives : pas de redite du dossier, que la carte du dossier montre juste en
/// dessous ; coupées : « Coupé… ».
#[test]
fn pas_de_redite_du_dossier() {
    assert_eq!(explication_recherches(&FRANCAIS, true), "");
    assert_eq!(
        explication_recherches(&FRANCAIS, false),
        FRANCAIS.recherches_coupees
    );
    assert_eq!(
        explication_recherches(&ANGLAIS, false),
        ANGLAIS.recherches_coupees
    );
}

#[test]
fn une_seule_icone_selon_l_etat() {
    let icone = |e: EtatMiseAJour| presentation_mise_a_jour(&e, &FRANCAIS).0;
    assert_eq!(icone(EtatMiseAJour::AJour), icones::COCHE);
    assert_eq!(icone(EtatMiseAJour::Prete("2.0.0".into())), icones::INSTALLER);
    assert_eq!(icone(EtatMiseAJour::Erreur("x".into())), icones::ATTENTION);
    assert_eq!(icone(EtatMiseAJour::Inconnu), icones::MISE_A_JOUR);
    assert_eq!(icone(EtatMiseAJour::Coupee), icones::A_PROPOS);
}

#[test]
fn le_titre_dit_l_etat_dans_les_deux_langues() {
    let titre = |e: EtatMiseAJour, t: &Textes| presentation_mise_a_jour(&e, t).2;
    assert_eq!(titre(EtatMiseAJour::AJour, &FRANCAIS), "Vous êtes à jour");
    assert_eq!(titre(EtatMiseAJour::AJour, &ANGLAIS), "You're up to date");
    assert_eq!(
        titre(EtatMiseAJour::Prete("2.1.0".into()), &FRANCAIS),
        "Version 2.1.0 disponible"
    );
    assert_eq!(
        titre(EtatMiseAJour::Prete("2.1.0".into()), &ANGLAIS),
        "Version 2.1.0 is available"
    );
    assert_eq!(
        titre(EtatMiseAJour::Erreur("Réseau coupé.".into()), &FRANCAIS),
        "La recherche a échoué"
    );
    assert_eq!(
        titre(
            EtatMiseAJour::Information("Mise à jour par le magasin.".into()),
            &FRANCAIS
        ),
        "Mise à jour par le magasin."
    );
}

#[test]
fn les_couleurs_suivent_l_etat() {
    let ton = |e: EtatMiseAJour| presentation_mise_a_jour(&e, &FRANCAIS).1;
    assert_eq!(ton(EtatMiseAJour::AJour), TonMiseAJour::Bon);
    assert_eq!(ton(EtatMiseAJour::Prete("2.0.0".into())), TonMiseAJour::Accent);
    assert_eq!(ton(EtatMiseAJour::Erreur("x".into())), TonMiseAJour::Probleme);
    assert_eq!(ton(EtatMiseAJour::Recherche), TonMiseAJour::Occupe);
    assert_eq!(ton(EtatMiseAJour::Inconnu), TonMiseAJour::Neutre);
}

/// Chaque ton prend sa couleur dans la palette.
#[test]
fn les_tons_prennent_la_couleur_de_la_palette() {
    let p = Palette::de(&ChoixWindows {
        sombre: false,
        barre_sombre: false,
        accent: None,
        nuances: None,
    });
    let couleur = |e: EtatMiseAJour| texte_mise_a_jour(&e, &p, &FRANCAIS).1;
    assert_eq!(couleur(EtatMiseAJour::AJour), p.vert);
    assert_eq!(couleur(EtatMiseAJour::Recherche), p.orange);
    assert_eq!(couleur(EtatMiseAJour::Prete("2.0.0".into())), p.accent);
    assert_eq!(couleur(EtatMiseAJour::Erreur("x".into())), p.rouge);
    assert_eq!(couleur(EtatMiseAJour::Coupee), p.texte_secondaire);
    let (g, _, titre) = texte_mise_a_jour(&EtatMiseAJour::AJour, &p, &FRANCAIS);
    assert_eq!((g, titre.as_str()), (icones::COCHE, "Vous êtes à jour"));
}

#[test]
fn la_ligne_grise_donne_la_version_et_la_derniere_recherche() {
    let a_jour = EtatMiseAJour::AJour;
    assert_eq!(
        ligne_grise_mise_a_jour(&a_jour, "1.0.0", Some("20 nov. 2026, 14:05"), &FRANCAIS),
        "Version actuelle : 1.0.0 · Dernière recherche : 20 nov. 2026, 14:05"
    );
    assert_eq!(
        ligne_grise_mise_a_jour(&a_jour, "1.0.0", None, &ANGLAIS),
        "Current version: 1.0.0"
    );
    let erreur = EtatMiseAJour::Erreur("Réseau coupé.".into());
    assert_eq!(
        ligne_grise_mise_a_jour(&erreur, "1.0.0", None, &FRANCAIS),
        "Réseau coupé. · Version actuelle : 1.0.0"
    );
}

fn communs(mise_a_jour: EtatMiseAJour, recherches: Option<bool>) -> Communs {
    Communs {
        mise_a_jour,
        recherches,
        dossier_installateurs: r"C:\Users\exemple\Téléchargements".into(),
        ..Communs::default()
    }
}

/// « Installer maintenant » quand une version est prête, même recherches coupées ; sinon
/// « Rechercher », sauf recherches coupées.
#[test]
fn le_bouton_de_la_carte() {
    let prete = EtatMiseAJour::Prete("2.0.0".into());
    assert_eq!(
        bouton_mise_a_jour(&communs(prete.clone(), Some(true))),
        Some(BoutonMiseAJour::Installer)
    );
    assert_eq!(
        bouton_mise_a_jour(&communs(prete, Some(false))),
        Some(BoutonMiseAJour::Installer)
    );
    assert_eq!(
        bouton_mise_a_jour(&communs(EtatMiseAJour::AJour, Some(true))),
        Some(BoutonMiseAJour::Rechercher)
    );
    // Pas d'interrupteur des recherches : elles ne sont pas coupées.
    assert_eq!(
        bouton_mise_a_jour(&communs(EtatMiseAJour::AJour, None)),
        Some(BoutonMiseAJour::Rechercher)
    );
    assert_eq!(
        bouton_mise_a_jour(&communs(EtatMiseAJour::AJour, Some(false))),
        None
    );
}

/// La carte du dossier : absente recherches coupées ; « Choisir… » si le dossier n'est pas
/// fixe ; « Automatique » s'il peut être trouvé seul et ne l'est pas encore.
#[test]
fn la_carte_du_dossier() {
    assert_eq!(carte_dossier(&communs(EtatMiseAJour::AJour, Some(false))), None);
    let mut c = communs(EtatMiseAJour::AJour, Some(true));
    assert_eq!(
        carte_dossier(&c),
        Some(CarteDossier {
            choisir: true,
            automatique: false
        })
    );
    c.dossier_automatique = Some(false);
    assert_eq!(
        carte_dossier(&c),
        Some(CarteDossier {
            choisir: true,
            automatique: true
        })
    );
    c.dossier_automatique = Some(true);
    assert_eq!(
        carte_dossier(&c),
        Some(CarteDossier {
            choisir: true,
            automatique: false
        })
    );
    c.dossier_automatique = Some(false);
    c.dossier_fixe = true;
    assert_eq!(
        carte_dossier(&c),
        Some(CarteDossier {
            choisir: false,
            automatique: false
        })
    );
}
