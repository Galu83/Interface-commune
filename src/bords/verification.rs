//! Le contrôle des polices, côté bords (bloc O) : lire les polices de Windows sur le disque
//! (`crate::polices`) et dire, pour chaque icône et chaque caractère affiché, s'il y est.
//! Repris tel quel à `interface_commune::verification` (`src/verification.rs`), qui garde la
//! documentation pour les logiciels.
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

/// Les logos (glyphes de pastille ou de menu) que la police d'icônes de ce PC n'a pas : à la
/// place de chacun, la **lettre** s'affiche. Le logiciel les vérifie dans ses tests comme ses
/// propres icônes ([`icones_absentes`] contrôle Segoe Fluent Icons et Segoe MDL2 Assets ;
/// celle-ci seulement la police réellement chargée, celle qui sert au dessin). `Err` : aucune
/// police d'icônes sur ce PC.
pub fn logos_absents(logos: &[char]) -> Result<Vec<char>, String> {
    if polices::polices().is_err() {
        return Err("ni Segoe Fluent Icons ni Segoe MDL2 Assets sur ce PC".into());
    }
    Ok(logos
        .iter()
        .copied()
        .filter(|&c| !polices::icone_disponible(c))
        .collect())
}

#[cfg(test)]
mod essais {
    use super::*;
    use crate::pastilles::{Marque, marque};

    /// Un glyphe absent de la police d'icônes (Windows 10 : Segoe MDL2 Assets n'a pas tout)
    /// redonne la lettre : jamais un carré vide.
    #[test]
    fn glyphe_absent_redonne_la_lettre() {
        let sans_police = |_: char| false;
        let avec_police = |_: char| true;
        assert_eq!(
            marque(Some('\u{E977}'), Some('F'), sans_police),
            Some(Marque::Lettre('F'))
        );
        assert_eq!(
            marque(Some('\u{E977}'), Some('F'), avec_police),
            Some(Marque::Logo('\u{E977}'))
        );
        assert_eq!(marque(None, Some('F'), avec_police), Some(Marque::Lettre('F')));
        // Pas de lettre ni de glyphe disponible : rien, jamais un carré.
        assert_eq!(marque(Some('\u{E977}'), None, sans_police), None);
        // Seul le glyphe qui manque retombe sur la lettre.
        let seulement_fixe = |c: char| c == '\u{E977}';
        assert_eq!(
            marque(Some('\u{EC59}'), Some('B'), seulement_fixe),
            Some(Marque::Lettre('B'))
        );
    }

    /// Sur un PC sans Segoe Fluent Icons ni MDL2 Assets, rien n'est « disponible ».
    #[test]
    fn la_police_chargee_repond_comme_le_controle() {
        let logos = [
            '\u{E977}', '\u{E7F8}', '\u{E7F4}', '\u{E70A}', '\u{EC59}', '\u{E7FC}',
        ];
        match logos_absents(&logos) {
            Ok(absents) => {
                // Ce qui manque à la police chargée redonne la lettre ; le reste est dessiné.
                for c in logos {
                    assert_eq!(!absents.contains(&c), polices::icone_disponible(c));
                }
            }
            Err(e) => {
                eprintln!("{e} : test sauté.");
                assert!(logos.iter().all(|&c| !polices::icone_disponible(c)));
            }
        }
        // Un caractère qui n'existe dans aucune police d'icônes : la lettre.
        assert!(!polices::icone_disponible('\u{F8FE}'));
    }
}
