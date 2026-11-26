//! Le choix des polices de Windows (règles : interface Windows 11, points 3 et 4 ; C-92), en
//! calcul pur : quels fichiers prendre, dans quel ordre, et ce que devient chacun. Lire les
//! fichiers et les donner à egui reste dans les bords (`interface_commune::polices`, bloc L).
//!
//! - Windows 11 : Segoe UI Variable, un seul fichier, réglé trois fois sur ses axes (graisse et
//!   taille optique) ; Segoe Fluent Icons pour les icônes.
//! - Windows 10 : Segoe UI, et Segoe UI Semibold pour les titres (Segoe UI s'il manque) ;
//!   Segoe MDL2 Assets pour les icônes.
//!
//! Chaque famille retombe sur une autre pour un caractère qu'elle n'a pas : le texte sur les
//! icônes, les icônes sur le texte.

/// Les fichiers de polices, dans le dossier des polices de Windows.
pub const SEGOE_VARIABLE: &str = "SegUIVar.ttf";
pub const SEGOE: &str = "segoeui.ttf";
pub const SEGOE_SEMIBOLD: &str = "seguisb.ttf";
pub const ICONES_FLUENT: &str = "SegoeIcons.ttf";
pub const ICONES_MDL2: &str = "segmdl2.ttf";

/// Les graisses de Segoe UI Variable : texte courant et titres (Semibold).
pub const GRAISSE_TEXTE: f32 = 400.0;
pub const GRAISSE_SEMIBOLD: f32 = 600.0;
/// Les tailles optiques de Segoe UI Variable : « Text » pour le texte, « Display » pour les
/// grands titres.
pub const OPTIQUE_TEXTE: f32 = 10.5;
pub const OPTIQUE_TITRE: f32 = 36.0;

/// Les noms des polices posées (les clés que le dessin connaît).
pub const TEXTE: &str = "texte";
pub const SEMIBOLD: &str = "semibold";
pub const TITRE: &str = "titre";
pub const ICONES: &str = "icones";

/// Une police posée : un fichier tel quel, ou Segoe UI Variable réglée sur ses axes.
#[derive(Debug, Clone, PartialEq)]
pub enum Police<T> {
    Telle(T),
    Reglee { donnees: T, graisse: f32, optique: f32 },
}

/// Une famille de polices : celle du texte courant, à chasse fixe, ou nommée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Famille {
    Proportionnelle,
    ChasseFixe,
    Nommee(&'static str),
}

/// Chaque famille, avec sa police puis celle où elle retombe.
pub const FAMILLES: [(Famille, [&str; 2]); 5] = [
    (Famille::Proportionnelle, [TEXTE, ICONES]),
    (Famille::ChasseFixe, [TEXTE, ICONES]),
    (Famille::Nommee(SEMIBOLD), [SEMIBOLD, ICONES]),
    (Famille::Nommee(TITRE), [TITRE, ICONES]),
    (Famille::Nommee(ICONES), [ICONES, TEXTE]),
];

/// Les polices à poser, chacune sous son nom, d'après les fichiers que `lire` trouve (dans
/// cet ordre : les icônes, puis le texte). `Err` : de quoi écrire le texte ou les icônes
/// manque ; le logiciel garde alors le menu de secours de Windows.
pub fn plan<T: Clone>(
    mut lire: impl FnMut(&'static str) -> Option<T>,
) -> Result<Vec<(&'static str, Police<T>)>, &'static str> {
    let icones = lire(ICONES_FLUENT)
        .or_else(|| lire(ICONES_MDL2))
        .ok_or("Segoe Fluent Icons et Segoe MDL2 Assets introuvables")?;
    let mut poses = vec![(ICONES, Police::Telle(icones))];
    if let Some(v) = lire(SEGOE_VARIABLE) {
        // Un seul fichier, partagé : trois réglages de ses axes.
        let reglee = |graisse, optique| Police::Reglee {
            donnees: v.clone(),
            graisse,
            optique,
        };
        poses.push((TEXTE, reglee(GRAISSE_TEXTE, OPTIQUE_TEXTE)));
        poses.push((SEMIBOLD, reglee(GRAISSE_SEMIBOLD, OPTIQUE_TEXTE)));
        poses.push((TITRE, reglee(GRAISSE_SEMIBOLD, OPTIQUE_TITRE)));
    } else {
        let segoe = lire(SEGOE).ok_or("Segoe UI introuvable")?;
        let gras = lire(SEGOE_SEMIBOLD).unwrap_or_else(|| segoe.clone());
        poses.push((TEXTE, Police::Telle(segoe)));
        poses.push((SEMIBOLD, Police::Telle(gras.clone())));
        poses.push((TITRE, Police::Telle(gras)));
    }
    Ok(poses)
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Le plan de faux fichiers : leur nom tient lieu de contenu.
    type Plan = Result<Vec<(&'static str, Police<&'static str>)>, &'static str>;

    /// Un faux dossier des polices : `lire` rend le nom du fichier s'il y est, et note l'ordre
    /// des lectures.
    fn plan_avec(presents: &[&'static str]) -> (Plan, Vec<&'static str>) {
        let mut lus = Vec::new();
        let r = plan(|nom| {
            lus.push(nom);
            presents.contains(&nom).then_some(nom)
        });
        (r, lus)
    }

    fn reglee(graisse: f32, optique: f32) -> Police<&'static str> {
        Police::Reglee {
            donnees: SEGOE_VARIABLE,
            graisse,
            optique,
        }
    }

    /// Windows 11 : Segoe UI Variable réglée trois fois, Segoe Fluent Icons.
    #[test]
    fn windows_11() {
        let (r, lus) = plan_avec(&[SEGOE_VARIABLE, SEGOE, SEGOE_SEMIBOLD, ICONES_FLUENT, ICONES_MDL2]);
        assert_eq!(
            r,
            Ok(vec![
                (ICONES, Police::Telle(ICONES_FLUENT)),
                (TEXTE, reglee(400.0, 10.5)),
                (SEMIBOLD, reglee(600.0, 10.5)),
                (TITRE, reglee(600.0, 36.0)),
            ])
        );
        // Ni Segoe UI ni Segoe MDL2 Assets ne sont lus pour rien.
        assert_eq!(lus, [ICONES_FLUENT, SEGOE_VARIABLE]);
    }

    /// Windows 10 (C-92) : Segoe UI, Segoe UI Semibold pour les titres, Segoe MDL2 Assets.
    #[test]
    fn windows_10() {
        let (r, lus) = plan_avec(&[SEGOE, SEGOE_SEMIBOLD, ICONES_MDL2]);
        assert_eq!(
            r,
            Ok(vec![
                (ICONES, Police::Telle(ICONES_MDL2)),
                (TEXTE, Police::Telle(SEGOE)),
                (SEMIBOLD, Police::Telle(SEGOE_SEMIBOLD)),
                (TITRE, Police::Telle(SEGOE_SEMIBOLD)),
            ])
        );
        assert_eq!(
            lus,
            [ICONES_FLUENT, ICONES_MDL2, SEGOE_VARIABLE, SEGOE, SEGOE_SEMIBOLD]
        );
    }

    /// Sans Segoe UI Semibold, les titres prennent Segoe UI.
    #[test]
    fn sans_semibold_les_titres_prennent_segoe() {
        let (r, _) = plan_avec(&[SEGOE, ICONES_MDL2]);
        assert_eq!(
            r,
            Ok(vec![
                (ICONES, Police::Telle(ICONES_MDL2)),
                (TEXTE, Police::Telle(SEGOE)),
                (SEMIBOLD, Police::Telle(SEGOE)),
                (TITRE, Police::Telle(SEGOE)),
            ])
        );
    }

    /// Sans police d'icônes ni de texte : une erreur, et rien n'est lu après le manque.
    #[test]
    fn ce_qui_manque_est_dit() {
        let (r, lus) = plan_avec(&[SEGOE_VARIABLE, SEGOE]);
        assert_eq!(r, Err("Segoe Fluent Icons et Segoe MDL2 Assets introuvables"));
        assert_eq!(lus, [ICONES_FLUENT, ICONES_MDL2]);
        let (r, _) = plan_avec(&[ICONES_FLUENT, SEGOE_SEMIBOLD]);
        assert_eq!(r, Err("Segoe UI introuvable"));
    }

    /// Chaque famille a une police posée et un repli posé, différent d'elle.
    #[test]
    fn chaque_famille_retombe_sur_une_police_posee() {
        let (Ok(poses), _) = plan_avec(&[SEGOE, ICONES_MDL2]) else {
            unreachable!("Windows 10 : des polices");
        };
        let noms: Vec<_> = poses.iter().map(|(n, _)| *n).collect();
        for (_, [premiere, repli]) in FAMILLES {
            assert!(noms.contains(&premiere) && noms.contains(&repli));
            assert_ne!(premiere, repli);
        }
        for nom in [SEMIBOLD, TITRE, ICONES] {
            assert!(FAMILLES.iter().any(|(f, _)| *f == Famille::Nommee(nom)), "{nom}");
        }
    }
}
