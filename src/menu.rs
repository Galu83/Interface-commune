//! Le petit menu de l'icône (clic droit), en calcul pur (règles : interface Windows 11,
//! point 10) : toujours dans cet ordre, avec ces mots ([`crate::textes`]) et ces icônes dans
//! tous les logiciels :
//! « Ouvrir <logiciel> » en gras (E8A7), seulement s'il a une fenêtre principale
//! (`Logiciel::fenetre_principale`), puis les une ou deux actions propres au logiciel, un trait, puis « Paramètres » (E713),
//! « Rechercher une mise à jour » (E895), les lignes communes de plus (« Ouvrir le
//! journal »…), et « Quitter » en dernier.

use crate::icones;
use crate::textes::Textes;

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
    /// « Ouvrir <logiciel> » : la fenêtre principale. En gras, toujours la première ligne.
    Ouvrir,
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

    /// La ligne commune « Ouvrir le journal » (clé `journal`), dans la langue de `t`.
    pub fn ouvrir_le_journal(t: &Textes) -> Action {
        Action::new(crate::cles::JOURNAL, icones::JOURNAL, t.ouvrir_le_journal)
    }
}

/// Les actions propres au logiciel : deux au plus (règles : « une ou deux »).
pub const ACTIONS_PROPRES_MAX: usize = 2;
/// Les lignes communes de plus : deux au plus.
pub const LIGNES_EN_PLUS_MAX: usize = 2;

/// Les lignes du petit menu, dans l'ordre des règles ; `None` : un trait de séparation
/// (seulement entre les actions propres et les lignes communes). Au-delà de deux, les
/// actions propres et les lignes de plus sont ignorées.
pub fn lignes(t: &Textes, propres: &[Action], en_plus: &[Action]) -> Vec<Option<Ligne>> {
    lignes_avec(t, None, propres, en_plus)
}

/// Les lignes du petit menu, comme [`lignes`] ; avec `ouvrir` (le nom du logiciel, quand il a
/// une fenêtre principale), « Ouvrir <logiciel> » vient en première ligne, avant les actions
/// propres, et le trait suit ce premier groupe.
pub fn lignes_avec(
    t: &Textes,
    ouvrir: Option<&str>,
    propres: &[Action],
    en_plus: &[Action],
) -> Vec<Option<Ligne>> {
    let ligne = |a: &Action| {
        Some(Ligne {
            icone: a.icone,
            texte: a.texte.clone(),
            quoi: Choix::Action(a.cle.clone()),
        })
    };
    let mut l: Vec<Option<Ligne>> = ouvrir
        .map(|nom| Ligne {
            icone: icones::OUVRIR,
            texte: crate::textes::remplir(t.ouvrir_le_logiciel, &[nom]),
            quoi: Choix::Ouvrir,
        })
        .into_iter()
        .map(Some)
        .collect();
    l.extend(propres.iter().take(ACTIONS_PROPRES_MAX).map(ligne));
    if !l.is_empty() {
        l.push(None);
    }
    l.push(Some(Ligne {
        icone: icones::REGLAGES,
        texte: t.parametres.into(),
        quoi: Choix::Parametres,
    }));
    l.push(Some(Ligne {
        icone: icones::MISE_A_JOUR,
        texte: t.rechercher_mise_a_jour.into(),
        quoi: Choix::RechercherMiseAJour,
    }));
    l.extend(en_plus.iter().take(LIGNES_EN_PLUS_MAX).map(ligne));
    l.push(Some(Ligne {
        icone: icones::QUITTER,
        texte: t.quitter.into(),
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
        let fr = &crate::textes::FRANCAIS;
        let l = lignes(fr, &propres, &[Action::ouvrir_le_journal(fr)]);
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
            textes(&lignes(fr, &[], &[])),
            ["Paramètres", "Rechercher une mise à jour", "Quitter"]
        );
        // Trois actions propres : deux gardées.
        let trois = [
            Action::new("a", '\u{E710}', "A"),
            Action::new("b", '\u{E710}', "B"),
            Action::new("c", '\u{E710}', "C"),
        ];
        assert_eq!(textes(&lignes(fr, &trois, &[]))[..3], ["A", "B", "—"]);
        // En anglais : les mêmes lignes, les mots de la langue.
        let en = &crate::textes::ANGLAIS;
        assert_eq!(
            textes(&lignes(en, &[], &[Action::ouvrir_le_journal(en)])),
            ["Settings", "Check for updates", "Open the log", "Quit"]
        );
    }

    /// Avec une fenêtre principale : « Ouvrir <logiciel> » en première ligne (E8A7), avant
    /// les actions propres, puis le trait ; sans action propre, le trait suit « Ouvrir ».
    /// Sans fenêtre principale, le menu ne change pas.
    #[test]
    fn ouvrir_le_logiciel_en_tete() {
        let fr = &crate::textes::FRANCAIS;
        let propres = [Action::new("relire", '\u{E72C}', "Relire")];
        let l = lignes_avec(fr, Some("Exemple"), &propres, &[]);
        assert_eq!(
            textes(&l),
            [
                "Ouvrir Exemple",
                "Relire",
                "—",
                "Paramètres",
                "Rechercher une mise à jour",
                "Quitter"
            ]
        );
        let premiere = l[0].clone().unwrap();
        assert_eq!(premiere.quoi, Choix::Ouvrir);
        assert_eq!(premiere.icone, '\u{E8A7}');
        assert_eq!(
            textes(&lignes_avec(fr, Some("Exemple"), &[], &[]))[..2],
            ["Ouvrir Exemple", "—"]
        );
        assert_eq!(
            textes(&lignes_avec(&crate::textes::ANGLAIS, Some("Exemple"), &[], &[]))[0],
            "Open Exemple"
        );
        assert_eq!(lignes_avec(fr, None, &propres, &[]), lignes(fr, &propres, &[]));
    }
}
