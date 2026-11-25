//! Le menu de choix au clavier, en calcul pur (bloc F) : la ligne active, les flèches, Début,
//! Fin, Entrée, Échap, les lignes grisées sautées ; la ligne active à l'ouverture ; le nom
//! d'une ligne pour le lecteur d'écran. Le dessin vit dans `interface_commune::menu_choix`.

/// Les touches que le menu comprend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Touche {
    Haut,
    Bas,
    Debut,
    Fin,
    Entree,
    Echap,
}

/// Ce que fait une touche.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effet {
    /// Rien ne change.
    Rien,
    /// Cette ligne devient la ligne active.
    Active(usize),
    /// Cette ligne est choisie : le menu se ferme.
    Choisir(usize),
    /// Le menu se ferme sans rien choisir.
    Fermer,
}

fn libre(grisees: &[bool], i: usize) -> bool {
    grisees.get(i) == Some(&false)
}

fn premiere(grisees: &[bool]) -> Option<usize> {
    (0..grisees.len()).find(|&i| libre(grisees, i))
}

fn derniere(grisees: &[bool]) -> Option<usize> {
    (0..grisees.len()).rev().find(|&i| libre(grisees, i))
}

/// Ce que fait `touche` quand `active` est la ligne active et que `grisees[i]` dit si la
/// ligne `i` est grisée. Les flèches sautent les lignes grisées et font le tour (comme les
/// menus de Windows) ; sans ligne active, Bas va à la première et Haut à la dernière ; Début
/// et Fin vont à la première et à la dernière ligne libres ; Entrée choisit la ligne active
/// si elle est libre ; Échap ferme. Une ligne active hors liste ou grisée compte pour aucune.
pub fn suivre(grisees: &[bool], active: Option<usize>, touche: Touche) -> Effet {
    let n = grisees.len();
    let active = active.filter(|&i| i < n);
    let vers = |i: Option<usize>| i.map_or(Effet::Rien, Effet::Active);
    match touche {
        Touche::Echap => Effet::Fermer,
        Touche::Entree => active
            .filter(|&i| libre(grisees, i))
            .map_or(Effet::Rien, Effet::Choisir),
        Touche::Debut => vers(premiere(grisees)),
        Touche::Fin => vers(derniere(grisees)),
        Touche::Bas => match active {
            None => vers(premiere(grisees)),
            Some(i) => vers((1..=n).map(|d| (i + d) % n).find(|&j| libre(grisees, j))),
        },
        Touche::Haut => match active {
            None => vers(derniere(grisees)),
            Some(i) => vers((1..=n).map(|d| (i + n - d) % n).find(|&j| libre(grisees, j))),
        },
    }
}

/// La ligne active à l'ouverture : la première ligne cochée et libre, sinon la première ligne
/// libre, sinon aucune. `cochees[i]` et `grisees[i]` disent l'état de la ligne `i`.
pub fn initiale(cochees: &[bool], grisees: &[bool]) -> Option<usize> {
    let libre = |i: usize| grisees.get(i) != Some(&true);
    (0..cochees.len())
        .find(|&i| cochees[i] && libre(i))
        .or_else(|| (0..cochees.len()).find(|&i| libre(i)))
}

/// Le nom d'une ligne pour le lecteur d'écran : le texte, le sous-texte (s'il n'est pas vide),
/// puis `mot_coche` (« coché ») si elle est choisie. La couleur n'y est pour rien.
pub fn nom_accessible(texte: &str, sous_texte: Option<&str>, coche: bool, mot_coche: &str) -> String {
    let mut parties = vec![texte];
    if let Some(s) = sous_texte.filter(|s| !s.is_empty()) {
        parties.push(s);
    }
    if coche {
        parties.push(mot_coche);
    }
    parties.join(", ")
}

#[cfg(test)]
mod essais {
    use super::*;

    fn libres(n: usize) -> Vec<bool> {
        vec![false; n]
    }

    #[test]
    fn bas_et_haut_font_le_tour() {
        let g = libres(3);
        assert_eq!(suivre(&g, Some(0), Touche::Bas), Effet::Active(1));
        assert_eq!(suivre(&g, Some(2), Touche::Bas), Effet::Active(0));
        assert_eq!(suivre(&g, Some(0), Touche::Haut), Effet::Active(2));
        assert_eq!(suivre(&g, Some(2), Touche::Haut), Effet::Active(1));
    }

    #[test]
    fn sans_ligne_active_bas_va_a_la_premiere_et_haut_a_la_derniere() {
        let g = libres(4);
        assert_eq!(suivre(&g, None, Touche::Bas), Effet::Active(0));
        assert_eq!(suivre(&g, None, Touche::Haut), Effet::Active(3));
    }

    #[test]
    fn les_lignes_grisees_sont_sautees() {
        let g = [true, false, true, false, true];
        assert_eq!(suivre(&g, None, Touche::Bas), Effet::Active(1));
        assert_eq!(suivre(&g, Some(1), Touche::Bas), Effet::Active(3));
        assert_eq!(suivre(&g, Some(3), Touche::Bas), Effet::Active(1), "fait le tour");
        assert_eq!(suivre(&g, Some(1), Touche::Haut), Effet::Active(3));
        assert_eq!(suivre(&g, Some(3), Touche::Haut), Effet::Active(1));
        assert_eq!(
            suivre(&g, Some(0), Touche::Bas),
            Effet::Active(1),
            "active grisée : on avance"
        );
    }

    #[test]
    fn debut_et_fin_vont_aux_lignes_libres_des_extremites() {
        let g = [true, false, false, true];
        assert_eq!(suivre(&g, Some(2), Touche::Debut), Effet::Active(1));
        assert_eq!(suivre(&g, Some(1), Touche::Fin), Effet::Active(2));
    }

    #[test]
    fn entree_choisit_la_ligne_active_si_elle_est_libre() {
        let g = [false, true, false];
        assert_eq!(suivre(&g, Some(2), Touche::Entree), Effet::Choisir(2));
        assert_eq!(suivre(&g, Some(1), Touche::Entree), Effet::Rien, "grisée");
        assert_eq!(suivre(&g, None, Touche::Entree), Effet::Rien);
        assert_eq!(suivre(&g, Some(9), Touche::Entree), Effet::Rien, "hors liste");
    }

    #[test]
    fn echap_ferme_toujours() {
        assert_eq!(suivre(&[], None, Touche::Echap), Effet::Fermer);
        assert_eq!(suivre(&libres(2), Some(1), Touche::Echap), Effet::Fermer);
    }

    #[test]
    fn tout_grise_ou_vide_rien_ne_bouge() {
        for g in [vec![], vec![true, true]] {
            for t in [
                Touche::Haut,
                Touche::Bas,
                Touche::Debut,
                Touche::Fin,
                Touche::Entree,
            ] {
                assert_eq!(suivre(&g, Some(0), t), Effet::Rien, "{g:?} {t:?}");
                assert_eq!(suivre(&g, None, t), Effet::Rien, "{g:?} {t:?}");
            }
        }
    }

    #[test]
    fn la_ligne_initiale_est_la_cochee_sinon_la_premiere_libre() {
        let mut cochees = [false, false, false];
        let mut grisees = [false, false, false];
        assert_eq!(initiale(&cochees, &grisees), Some(0));
        cochees[1] = true;
        assert_eq!(initiale(&cochees, &grisees), Some(1));
        grisees[1] = true;
        assert_eq!(initiale(&cochees, &grisees), Some(0), "la cochée est grisée");
        grisees[0] = true;
        assert_eq!(initiale(&cochees, &grisees), Some(2));
        grisees[2] = true;
        assert_eq!(initiale(&cochees, &grisees), None);
        assert_eq!(initiale(&[], &[]), None);
    }

    #[test]
    fn le_nom_pour_le_lecteur_d_ecran() {
        assert_eq!(nom_accessible("Fixe", None, false, "coché"), "Fixe");
        assert_eq!(nom_accessible("Fixe", Some(""), false, "coché"), "Fixe");
        assert_eq!(
            nom_accessible("Fixe", Some("Ce PC"), false, "coché"),
            "Fixe, Ce PC"
        );
        assert_eq!(
            nom_accessible("Fixe", Some("Ce PC"), true, "coché"),
            "Fixe, Ce PC, coché"
        );
    }
}
