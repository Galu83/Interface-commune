//! Le contrôle des polices (règles : interface Windows 11, point 4) : chaque icône et chaque
//! caractère affiché existent dans les polices de Windows. La bibliothèque s'en sert pour ses
//! propres textes ; chaque logiciel s'en sert, dans ses tests, pour les siens :
//!
//! ```no_run
//! use interface_commune::verification;
//! // Les icônes propres au logiciel, et les fichiers d'où vient son texte affiché.
//! let icones = ['\u{E7F4}', '\u{E8A9}'];
//! let sources = [("menu.rs", "… le texte du fichier …")];
//! assert_eq!(verification::icones_absentes(&icones), Ok(Vec::new()));
//! assert_eq!(verification::caracteres_absents(&sources), Ok(Vec::new()));
//! ```
//!
//! Hors de Windows (polices absentes), les contrôles rendent `Err` : le test le dit et saute.

use skrifa::{FontRef, MetadataProvider};

use crate::polices;

/// Un caractère absent d'une police.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absent {
    pub caractere: char,
    /// Le fichier de la police où il manque.
    pub police: &'static str,
    /// D'où vient le caractère (un nom de fichier source, ou « icônes »).
    pub source: String,
}

/// La police contient-elle ce caractère ?
pub fn contient(police: &[u8], c: char) -> bool {
    FontRef::new(police).is_ok_and(|f| f.charmap().map(c).is_some())
}

fn presentes(noms: &[&'static str]) -> Vec<(&'static str, Vec<u8>)> {
    noms.iter()
        .filter_map(|n| Some((*n, polices::lire(n)?)))
        .collect()
}

/// Les icônes qui manquent dans Segoe Fluent Icons ou dans Segoe MDL2 Assets (le repli).
/// `Err` : aucune des deux polices n'est sur ce PC.
pub fn icones_absentes(icones: &[char]) -> Result<Vec<Absent>, String> {
    let polices = presentes(&[polices::ICONES_FLUENT, polices::ICONES_MDL2]);
    if polices.is_empty() {
        return Err("ni Segoe Fluent Icons ni Segoe MDL2 Assets sur ce PC".into());
    }
    let mut absents = Vec::new();
    for (nom, donnees) in &polices {
        for &c in icones {
            if !contient(donnees, c) {
                absents.push(Absent {
                    caractere: c,
                    police: nom,
                    source: "icônes".into(),
                });
            }
        }
    }
    Ok(absents)
}

/// Les caractères de ces textes (nom de la source, texte) qui manquent dans une police de
/// texte de Windows présente sur ce PC (Segoe UI Variable, Segoe UI, Segoe UI Semibold). On
/// passe le texte de tout fichier qui fabrique ce qui s'affiche : chaque caractère est
/// vérifié, commentaires compris. Les caractères de commande (retour à la ligne…) ne
/// comptent pas, ni ceux de la zone privée d'Unicode (les icônes, vérifiées par
/// [`icones_absentes`]). `Err` : aucune police de texte sur ce PC.
pub fn caracteres_absents(sources: &[(&str, &str)]) -> Result<Vec<Absent>, String> {
    let polices = presentes(&[polices::SEGOE_VARIABLE, polices::SEGOE, polices::SEGOE_SEMIBOLD]);
    if polices.is_empty() {
        return Err("ni Segoe UI Variable ni Segoe UI sur ce PC".into());
    }
    let mut absents: Vec<Absent> = Vec::new();
    for (source, texte) in sources {
        for c in texte.chars().filter(|c| !c.is_control() && !zone_privee(*c)) {
            for (nom, donnees) in &polices {
                let deja = absents.iter().any(|a| a.caractere == c && a.police == *nom);
                if !deja && !contient(donnees, c) {
                    absents.push(Absent {
                        caractere: c,
                        police: nom,
                        source: (*source).to_string(),
                    });
                }
            }
        }
    }
    Ok(absents)
}

/// La zone privée d'Unicode, où sont les icônes de Segoe Fluent Icons.
fn zone_privee(c: char) -> bool {
    ('\u{E000}'..='\u{F8FF}').contains(&c)
}
