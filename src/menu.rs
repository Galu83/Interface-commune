//! Le petit menu de l'icône (clic droit), en calcul pur (règles : interface Windows 11,
//! point 10) : toujours dans cet ordre, avec ces mots et ces icônes dans tous les logiciels :
//! les une ou deux actions propres au logiciel, un trait, puis « Paramètres » (E713),
//! « Rechercher une mise à jour » (E895), les lignes communes de plus (« Ouvrir le
//! journal »…), et « Quitter » en dernier.

use crate::icones;

/// Une ligne du petit menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ligne {
    pub icone: char,
    pub texte: String,
    pub quoi: Choix,
}

/// Ce qu'une ligne du petit menu demande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choix {
    /// Une action propre au logiciel, ou une ligne commune de plus : sa clé, rendue au
    /// logiciel (`Logiciel::action`).
    Action(String),
    Parametres,
    RechercherMiseAJour,
    Quitter,
}

/// Une action propre au logiciel (petit menu) ou une ligne commune de plus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// La clé rendue au logiciel quand l'action est choisie.
    pub cle: String,
    pub icone: char,
    pub texte: String,
}

impl Action {
    pub fn new(cle: &str, icone: char, texte: &str) -> Action {
        Action {
            cle: cle.into(),
            icone,
            texte: texte.into(),
        }
    }

    /// La ligne commune « Ouvrir le journal » (clé `journal`).
    pub fn ouvrir_le_journal() -> Action {
        Action::new("journal", icones::JOURNAL, "Ouvrir le journal")
    }
}

/// Les actions propres au logiciel : deux au plus (règles : « une ou deux »).
pub const ACTIONS_PROPRES_MAX: usize = 2;
/// Les lignes communes de plus : deux au plus.
pub const LIGNES_EN_PLUS_MAX: usize = 2;

/// Les textes communs, les mêmes dans tous les logiciels.
pub const PARAMETRES: &str = "Paramètres";
pub const RECHERCHER: &str = "Rechercher une mise à jour";
pub const QUITTER: &str = "Quitter";

/// Les lignes du petit menu, dans l'ordre des règles ; `None` : un trait de séparation
/// (seulement entre les actions propres et les lignes communes). Au-delà de deux, les
/// actions propres et les lignes de plus sont ignorées.
pub fn lignes(propres: &[Action], en_plus: &[Action]) -> Vec<Option<Ligne>> {
    let ligne = |a: &Action| {
        Some(Ligne {
            icone: a.icone,
            texte: a.texte.clone(),
            quoi: Choix::Action(a.cle.clone()),
        })
    };
    let mut l: Vec<Option<Ligne>> = propres.iter().take(ACTIONS_PROPRES_MAX).map(ligne).collect();
    if !l.is_empty() {
        l.push(None);
    }
    l.push(Some(Ligne {
        icone: icones::REGLAGES,
        texte: PARAMETRES.into(),
        quoi: Choix::Parametres,
    }));
    l.push(Some(Ligne {
        icone: icones::MISE_A_JOUR,
        texte: RECHERCHER.into(),
        quoi: Choix::RechercherMiseAJour,
    }));
    l.extend(en_plus.iter().take(LIGNES_EN_PLUS_MAX).map(ligne));
    l.push(Some(Ligne {
        icone: icones::QUITTER,
        texte: QUITTER.into(),
        quoi: Choix::Quitter,
    }));
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textes(l: &[Option<Ligne>]) -> Vec<&str> {
        l.iter()
            .map(|l| l.as_ref().map_or("—", |l| l.texte.as_str()))
            .collect()
    }

    /// L'ordre et les icônes de la fiche : actions propres, trait, Paramètres (E713),
    /// Rechercher une mise à jour (E895), lignes de plus, Quitter.
    #[test]
    fn ordre_du_petit_menu() {
        let propres = [Action::new("relire", '\u{E72C}', "Relire les écrans")];
        let l = lignes(&propres, &[Action::ouvrir_le_journal()]);
        assert_eq!(
            textes(&l),
            [
                "Relire les écrans",
                "—",
                "Paramètres",
                "Rechercher une mise à jour",
                "Ouvrir le journal",
                "Quitter"
            ]
        );
        let icones_: Vec<char> = l.iter().flatten().map(|l| l.icone).collect();
        assert_eq!(icones_[1], '\u{E713}');
        assert_eq!(icones_[2], '\u{E895}');
        assert_eq!(l.last().cloned().flatten().map(|l| l.quoi), Some(Choix::Quitter));
        // Sans action propre : pas de trait en tête.
        assert_eq!(
            textes(&lignes(&[], &[])),
            ["Paramètres", "Rechercher une mise à jour", "Quitter"]
        );
        // Trois actions propres : deux gardées.
        let trois = [
            Action::new("a", '\u{E710}', "A"),
            Action::new("b", '\u{E710}', "B"),
            Action::new("c", '\u{E710}', "C"),
        ];
        assert_eq!(textes(&lignes(&trois, &[]))[..3], ["A", "B", "—"]);
    }
}
