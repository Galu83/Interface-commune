//! La barre à seuils, en calcul pur (bloc F) : où une limite peut aller entre ses voisines, et
//! ce que demandent les touches. Le dessin vit dans `interface_commune::seuils`.

/// Les bornes entre lesquelles la limite `i` peut aller, ses voisines étant à leur place et
/// `ecart` le vide minimal entre deux limites ; la barre va de 0 à `max`.
pub fn bornes(seuils: &[u8], i: usize, ecart: u8, max: u8) -> (u8, u8) {
    let bas = if i == 0 {
        0
    } else {
        seuils[i - 1].saturating_add(ecart).min(max)
    };
    let haut = if i + 1 == seuils.len() {
        max
    } else {
        seuils[i + 1].saturating_sub(ecart)
    };
    (bas, haut.max(bas))
}

/// Met la limite `i` à `voulu`, ramené entre ses voisines. Rend vrai si elle a bougé.
pub fn deplacer(seuils: &mut [u8], i: usize, voulu: i32, ecart: u8, max: u8) -> bool {
    let (bas, haut) = bornes(seuils, i, ecart, max);
    let v = voulu.clamp(i32::from(bas), i32::from(haut)) as u8;
    let change = seuils[i] != v;
    seuils[i] = v;
    change
}

/// Une borne demandée au clavier : Début (la plus basse) ou Fin (la plus haute).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Borne {
    Debut,
    Fin,
}

/// Ce que les touches d'une image demandent pour la limite `i` : la borne demandée, prise entre
/// ses voisines ; sinon `depart` (sa valeur, ou celle où la souris l'a glissée) plus `pas`
/// (négatif vers la gauche). Le résultat passe ensuite par [`deplacer`].
pub fn voulu_au_clavier(
    seuils: &[u8],
    i: usize,
    depart: i32,
    pas: i32,
    borne: Option<Borne>,
    ecart: u8,
    max: u8,
) -> i32 {
    let (bas, haut) = bornes(seuils, i, ecart, max);
    match borne {
        Some(Borne::Debut) => i32::from(bas),
        Some(Borne::Fin) => i32::from(haut),
        None => depart + pas,
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    #[test]
    fn une_limite_reste_entre_ses_voisines() {
        let mut s = [20u8, 50];
        assert!(deplacer(&mut s, 0, 90, 1, 100));
        assert_eq!(s, [49, 50]);
        assert!(
            !deplacer(&mut s, 1, 0, 1, 100),
            "la deuxième ne passe pas la première"
        );
        assert_eq!(s, [49, 50]);
        assert!(deplacer(&mut s, 0, -30, 1, 100));
        assert_eq!(s, [0, 50]);
        assert!(deplacer(&mut s, 1, 400, 1, 100));
        assert_eq!(s, [0, 100]);
        assert!(!deplacer(&mut s, 1, 100, 1, 100), "rien n'a bougé");
    }

    #[test]
    fn deux_limites_peuvent_se_toucher() {
        let mut s = [20u8, 50];
        assert!(deplacer(&mut s, 0, 80, 0, 100));
        assert_eq!(s, [50, 50]);
    }

    #[test]
    fn les_bornes_tiennent_compte_des_voisines_et_de_l_ecart() {
        let s = [20u8, 50, 80];
        assert_eq!(bornes(&s, 0, 1, 100), (0, 49));
        assert_eq!(bornes(&s, 1, 5, 100), (25, 75));
        assert_eq!(bornes(&s, 2, 1, 100), (51, 100));
        // Des voisines trop proches : la borne haute ne passe jamais sous la basse.
        assert_eq!(bornes(&[50, 50, 50], 1, 5, 100), (55, 55));
    }

    #[test]
    fn le_clavier_avance_d_un_pas_ou_va_a_une_borne() {
        let s = [20u8, 50];
        assert_eq!(voulu_au_clavier(&s, 0, 20, 1, None, 1, 100), 21);
        assert_eq!(voulu_au_clavier(&s, 0, 20, -10, None, 1, 100), 10);
        assert_eq!(
            voulu_au_clavier(&s, 0, 35, 1, None, 1, 100),
            36,
            "glissée puis une flèche"
        );
        assert_eq!(voulu_au_clavier(&s, 0, 35, 0, Some(Borne::Debut), 1, 100), 0);
        assert_eq!(voulu_au_clavier(&s, 0, 20, 0, Some(Borne::Fin), 1, 100), 49);
        assert_eq!(voulu_au_clavier(&s, 1, 50, 0, Some(Borne::Fin), 1, 100), 100);
    }
}
