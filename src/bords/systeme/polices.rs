//! Les polices de Windows (règles : interface Windows 11, points 3 et 4 ; C-92) : Segoe UI
//! Variable (Windows 11), sinon Segoe UI et Segoe UI Semibold (Windows 10) ; Segoe Fluent
//! Icons, sinon Segoe MDL2 Assets. Lues sur le PC, jamais livrées avec un logiciel. Quel
//! fichier prendre et ce que devient chacun se décide dans le cœur
//! ([`ic_coeur::theme::polices`]) ; ici, lire les fichiers et les donner à egui.
//!
//! Elles sont lues une seule fois, au repos (au démarrage) : jamais au moment d'un clic.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use egui::epaint::text::VariationCoords;
use egui::{FontData, FontDefinitions, FontFamily, FontId, FontTweak};

use ic_coeur::theme::polices::{self as plan, Famille, Police};

use crate::bords::verrou::verrou;

/// Les fichiers de polices, dans le dossier des polices de Windows.
pub use ic_coeur::theme::polices::{ICONES_FLUENT, ICONES_MDL2, SEGOE, SEGOE_SEMIBOLD, SEGOE_VARIABLE};

/// Le texte courant, à cette taille (points).
pub fn texte(taille: f32) -> FontId {
    FontId::proportional(taille)
}

/// Semibold : titres de section, élément choisi, état.
pub fn semibold(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name(plan::SEMIBOLD.into()))
}

/// Semibold à la taille optique des grands titres (titre de page, gros titre du volet).
pub fn titre(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name(plan::TITRE.into()))
}

/// Les icônes (Segoe Fluent Icons), dessinées à leur taille réelle.
pub fn icones(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name(plan::ICONES.into()))
}

/// Le dossier des polices de Windows (retrouvé par la variable d'environnement).
pub fn dossier() -> Option<PathBuf> {
    std::env::var_os("WINDIR")
        .or_else(|| std::env::var_os("SystemRoot"))
        .map(|d| PathBuf::from(d).join("Fonts"))
}

/// Lit une police de Windows par son nom de fichier.
pub fn lire(nom: &str) -> Option<Vec<u8>> {
    std::fs::read(dossier()?.join(nom)).ok()
}

/// La famille d'egui d'une famille du cœur.
fn famille(f: Famille) -> FontFamily {
    match f {
        Famille::Proportionnelle => FontFamily::Proportional,
        Famille::ChasseFixe => FontFamily::Monospace,
        Famille::Nommee(nom) => FontFamily::Name(nom.into()),
    }
}

/// Une police du plan, pour egui : telle quelle, ou réglée sur ses axes (graisse, taille
/// optique).
fn pour_egui(p: Police<FontData>) -> FontData {
    match p {
        Police::Telle(f) => f,
        Police::Reglee {
            donnees,
            graisse,
            optique,
        } => donnees.tweak(FontTweak {
            coords: VariationCoords::new([(b"wght", graisse), (b"opsz", optique)]),
            ..Default::default()
        }),
    }
}

fn charger() -> Result<FontDefinitions, String> {
    let poses = plan::plan(|nom| lire(nom).map(FontData::from_owned))?;
    let mut d = FontDefinitions::empty();
    for (nom, p) in poses {
        d.font_data.insert(nom.into(), Arc::new(pour_egui(p)));
    }
    for (f, noms) in plan::FAMILLES {
        d.families
            .insert(famille(f), noms.iter().map(|n| (*n).to_string()).collect());
    }
    Ok(d)
}

/// Les polices de l'interface. La première demande les lit sur le disque (à faire au repos) ;
/// les suivantes rendent les mêmes, sans rien relire. En erreur (polices absentes), le
/// logiciel garde le menu de secours de Windows.
pub fn polices() -> Result<FontDefinitions, String> {
    // ÉTAT GLOBAL : les polices lues une fois sur le disque (lecture coûteuse), partagées par
    // toutes les fenêtres et tous les fils.
    static POLICES: OnceLock<Result<FontDefinitions, String>> = OnceLock::new();
    POLICES.get_or_init(charger).clone()
}

/// La police d'icônes **déjà chargée** a-t-elle ce caractère ? Rien n'est relu sur le disque :
/// on interroge la police que `polices()` a lue au démarrage (Segoe Fluent Icons, sinon Segoe
/// MDL2 Assets), et chaque réponse est retenue. Sans police (hors Windows) : `false`. Sert à
/// retomber sur une lettre quand un glyphe manque, jamais à dessiner un carré vide.
pub fn icone_disponible(c: char) -> bool {
    // ÉTAT GLOBAL : les réponses retenues pour la police déjà chargée (même raison que
    // `POLICES`).
    static REPONSES: OnceLock<Mutex<HashMap<char, bool>>> = OnceLock::new();
    let table = REPONSES.get_or_init(Mutex::default);
    if let Some(r) = verrou(table).get(&c) {
        return *r;
    }
    let present = polices()
        .ok()
        .and_then(|d| d.font_data.get(plan::ICONES).cloned())
        .is_some_and(|f| crate::bords::verification::contient(AsRef::<[u8]>::as_ref(&*f.font), c));
    verrou(table).insert(c, present);
    present
}

/// Pour les essais sans Windows : les polices d'egui ne sont pas compilées (licences), donc
/// des familles vides sous les mêmes noms ; seul le calcul (pas le dessin du texte) y vaut.
/// Sous Windows, les vraies polices.
pub fn polices_ou_vides() -> FontDefinitions {
    polices().unwrap_or_else(|_| {
        let mut d = FontDefinitions::empty();
        for (f, _) in plan::FAMILLES {
            d.families.insert(famille(f), Vec::new());
        }
        d
    })
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Les polices se lisent dans le dossier `Fonts` de Windows, retrouvé sans chemin écrit
    /// en dur.
    #[test]
    fn le_dossier_est_celui_de_windows() {
        let Some(d) = dossier() else {
            eprintln!("ni WINDIR ni SystemRoot : essai sauté.");
            return;
        };
        assert_eq!(d.file_name().and_then(|n| n.to_str()), Some("Fonts"));
    }

    /// Les cinq familles existent toujours, polices lues ou non ; les familles nommées sont
    /// celles que `semibold`, `titre` et `icones` demandent.
    #[test]
    fn les_familles_sont_toujours_la() {
        let d = polices_ou_vides();
        for f in [
            texte(14.0).family,
            semibold(14.0).family,
            titre(28.0).family,
            icones(16.0).family,
            FontFamily::Monospace,
        ] {
            assert!(d.families.contains_key(&f), "{f:?}");
        }
        assert_eq!(d.families.len(), 5);
    }

    /// Sur ce PC, chaque famille commence par une police posée, et retombe sur une autre.
    #[test]
    fn chaque_famille_a_ses_polices() {
        let Ok(d) = polices() else {
            eprintln!("polices de Windows absentes : essai sauté.");
            return;
        };
        for (f, noms) in &d.families {
            assert_eq!(noms.len(), 2, "{f:?}");
            for n in noms {
                assert!(d.font_data.contains_key(n), "{f:?} : {n}");
            }
        }
        for nom in [plan::TEXTE, plan::SEMIBOLD, plan::TITRE, plan::ICONES] {
            assert!(d.font_data.contains_key(nom), "{nom}");
        }
    }
}
