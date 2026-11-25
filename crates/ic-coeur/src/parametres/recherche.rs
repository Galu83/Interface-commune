//! « Rechercher un paramètre » : ce qui est tapé est comparé au titre et aux mots de chaque
//! page, et de chaque section repliable ; accents, majuscules, apostrophes et traits d'union
//! sans importance. Tous les mots tapés doivent s'y trouver.

use crate::contrat::Page;

/// Minuscules sans accents, pour comparer ce qui est tapé à ce qui est écrit.
pub fn simplifier(texte: &str) -> String {
    texte
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' | 'ã' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' | 'õ' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            '\u{a0}' | '\u{202f}' | '’' | '\'' | '-' => ' ',
            c => c,
        })
        .collect()
}

/// Les rangs des pages dont le titre ou un mot contient tous les mots tapés. Rien de tapé :
/// rien.
pub fn chercher(pages: &[Page], tape: &str) -> Vec<usize> {
    let mots: Vec<String> = simplifier(tape).split_whitespace().map(str::to_string).collect();
    if mots.is_empty() {
        return Vec::new();
    }
    pages
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let texte = simplifier(&format!("{} {}", p.titre, p.mots.join(" ")));
            mots.iter().all(|m| texte.contains(m.as_str()))
        })
        .map(|(i, _)| i)
        .collect()
}

/// Les mots tapés dans « Rechercher un paramètre » se trouvent-ils dans le titre ou dans les
/// `mots` d'une section repliable ? Tous les mots tapés doivent s'y trouver (accents et
/// majuscules sans importance, [`simplifier`]). Rien de tapé : non.
pub fn section_trouvee(tape: &str, titre: &str, mots: &[&str]) -> bool {
    let tape = simplifier(tape);
    let mut tapes = tape.split_whitespace().peekable();
    if tapes.peek().is_none() {
        return false;
    }
    let texte = simplifier(&format!("{titre} {}", mots.join(" ")));
    tapes.all(|m| texte.contains(m))
}
