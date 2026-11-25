//! Les flèches du clavier dans le volet (C-33), en calcul pur (bloc G) : flèche bas passe à
//! la ligne suivante comme Tab, flèche haut à la précédente comme Maj+Tab. Une flèche avec
//! un modificateur (Maj, Ctrl, Alt…) ne change pas. Le dessin traduit les touches reçues avant
//! que la boucle ne les lise.

/// Une flèche verticale du clavier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fleche {
    Haut,
    Bas,
}

/// La touche Tab qui remplace une flèche : `arriere` pour Maj+Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tab {
    pub arriere: bool,
}

/// Ce que devient une flèche : Tab (bas) ou Maj+Tab (haut) ; rien si une touche de
/// modification est tenue.
pub fn fleche_en_tab(fleche: Fleche, modificateur: bool) -> Option<Tab> {
    if modificateur {
        return None;
    }
    Some(Tab {
        arriere: fleche == Fleche::Haut,
    })
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Flèche bas = Tab, flèche haut = Maj+Tab ; avec un modificateur, rien ne change (C-33).
    #[test]
    fn fleches_comme_tab() {
        assert_eq!(fleche_en_tab(Fleche::Bas, false), Some(Tab { arriere: false }));
        assert_eq!(fleche_en_tab(Fleche::Haut, false), Some(Tab { arriere: true }));
        assert_eq!(fleche_en_tab(Fleche::Bas, true), None);
        assert_eq!(fleche_en_tab(Fleche::Haut, true), None);
    }
}
