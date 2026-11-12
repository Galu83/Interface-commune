//! Les polices de Windows (règles : interface Windows 11, points 3 et 4) : Segoe UI Variable
//! (Windows 11), sinon Segoe UI et Segoe UI Semibold (Windows 10) ; Segoe Fluent Icons, sinon
//! Segoe MDL2 Assets. Lues sur le PC, jamais livrées avec un logiciel.
//!
//! Elles sont lues une seule fois, au repos (au démarrage) : jamais au moment d'un clic.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use egui::epaint::text::VariationCoords;
use egui::{FontData, FontDefinitions, FontFamily, FontId, FontTweak};

/// Les fichiers de polices, dans le dossier des polices de Windows.
pub const SEGOE_VARIABLE: &str = "SegUIVar.ttf";
pub const SEGOE: &str = "segoeui.ttf";
pub const SEGOE_SEMIBOLD: &str = "seguisb.ttf";
pub const ICONES_FLUENT: &str = "SegoeIcons.ttf";
pub const ICONES_MDL2: &str = "segmdl2.ttf";

/// Les graisses de Segoe UI Variable : texte courant et titres (Semibold).
const GRAISSE_TEXTE: f32 = 400.0;
const GRAISSE_SEMIBOLD: f32 = 600.0;
/// Les tailles optiques de Segoe UI Variable : « Text » pour le texte, « Display » pour les
/// grands titres.
const OPTIQUE_TEXTE: f32 = 10.5;
const OPTIQUE_TITRE: f32 = 36.0;

/// Le texte courant, à cette taille (points).
pub fn texte(taille: f32) -> FontId {
    FontId::proportional(taille)
}

/// Semibold : titres de section, élément choisi, état.
pub fn semibold(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name("semibold".into()))
}

/// Semibold à la taille optique des grands titres (titre de page, gros titre du volet).
pub fn titre(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name("titre".into()))
}

/// Les icônes (Segoe Fluent Icons), dessinées à leur taille réelle.
pub fn icones(taille: f32) -> FontId {
    FontId::new(taille, FontFamily::Name("icones".into()))
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

fn variable(donnees: &FontData, graisse: f32, optique: f32) -> FontData {
    donnees.clone().tweak(FontTweak {
        coords: VariationCoords::new([(b"wght", graisse), (b"opsz", optique)]),
        ..Default::default()
    })
}

fn charger() -> Result<FontDefinitions, String> {
    let icones = lire(ICONES_FLUENT)
        .or_else(|| lire(ICONES_MDL2))
        .ok_or("Segoe Fluent Icons et Segoe MDL2 Assets introuvables")?;
    let mut d = FontDefinitions::empty();
    let mut poser = |nom: &str, f: FontData| {
        d.font_data.insert(nom.into(), Arc::new(f));
    };
    poser("icones", FontData::from_owned(icones));
    if let Some(v) = lire(SEGOE_VARIABLE) {
        // Un seul fichier, partagé : trois réglages de ses axes.
        let v = FontData::from_owned(v);
        poser("texte", variable(&v, GRAISSE_TEXTE, OPTIQUE_TEXTE));
        poser("semibold", variable(&v, GRAISSE_SEMIBOLD, OPTIQUE_TEXTE));
        poser("titre", variable(&v, GRAISSE_SEMIBOLD, OPTIQUE_TITRE));
    } else {
        let segoe = FontData::from_owned(lire(SEGOE).ok_or("Segoe UI introuvable")?);
        let gras = lire(SEGOE_SEMIBOLD).map_or_else(|| segoe.clone(), FontData::from_owned);
        poser("texte", segoe);
        poser("semibold", gras.clone());
        poser("titre", gras);
    }
    let famille = |premiere: &str, repli: &str| vec![premiere.to_string(), repli.to_string()];
    d.families
        .insert(FontFamily::Proportional, famille("texte", "icones"));
    d.families
        .insert(FontFamily::Monospace, famille("texte", "icones"));
    d.families
        .insert(FontFamily::Name("semibold".into()), famille("semibold", "icones"));
    d.families
        .insert(FontFamily::Name("titre".into()), famille("titre", "icones"));
    d.families
        .insert(FontFamily::Name("icones".into()), famille("icones", "texte"));
    Ok(d)
}

/// Les polices de l'interface. La première demande les lit sur le disque (à faire au repos) ;
/// les suivantes rendent les mêmes, sans rien relire. En erreur (polices absentes), le
/// logiciel garde le menu de secours de Windows.
pub fn polices() -> Result<FontDefinitions, String> {
    static POLICES: OnceLock<Result<FontDefinitions, String>> = OnceLock::new();
    POLICES.get_or_init(charger).clone()
}

/// Pour les essais sans Windows : les polices d'egui ne sont pas compilées (licences), donc
/// des familles vides sous les mêmes noms ; seul le calcul (pas le dessin du texte) y vaut.
/// Sous Windows, les vraies polices.
pub fn polices_ou_vides() -> FontDefinitions {
    polices().unwrap_or_else(|_| {
        let mut d = FontDefinitions::empty();
        for f in [
            FontFamily::Proportional,
            FontFamily::Monospace,
            FontFamily::Name("semibold".into()),
            FontFamily::Name("titre".into()),
            FontFamily::Name("icones".into()),
        ] {
            d.families.insert(f, Vec::new());
        }
        d
    })
}
