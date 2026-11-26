//! Ce que les valeurs du registre de Windows veulent dire pour le thème (bloc L, la décision ;
//! la lecture du registre reste dans les bords, `interface_commune::systeme::choix_windows`).
//!
//! Les quatre valeurs lues, toutes sous `HKEY_CURRENT_USER` :
//! - `…\Themes\Personalize\AppsUseLightTheme` (DWORD) : 0, les applications sont sombres ;
//! - `…\Themes\Personalize\SystemUsesLightTheme` (DWORD) : 1, la barre des tâches est claire ;
//! - `…\Explorer\Accent\AccentPalette` (octets) : 8 couleurs RGBA, de la plus claire à la plus
//!   foncée (la 8e ne sert pas) ;
//! - `…\DWM\AccentColor` (DWORD) : l'accent, écrit `0xAABBGGRR`.
//!
//! Une valeur absente (Windows plus ancien, réglage jamais fait) a toujours un sens sûr : clair
//! pour les applications, sombre pour la barre des tâches (le réglage de Windows 11 à
//! l'installation), l'accent par défaut.

use super::couleurs::{ChoixWindows, Rgb};

/// Les octets de `AccentPalette` qui servent : 7 couleurs de 4 octets.
const OCTETS_DES_NUANCES: usize = 7 * 4;

/// Le choix de Windows, d'après les valeurs lues (`None` : valeur absente ou illisible).
pub fn choix_du_registre(
    applications_claires: Option<u32>,
    barre_claire: Option<u32>,
    palette_d_accent: Option<&[u8]>,
    couleur_d_accent: Option<u32>,
) -> ChoixWindows {
    let nuances = palette_d_accent
        .and_then(|o| o.get(..OCTETS_DES_NUANCES))
        .map(|o| std::array::from_fn(|i| Rgb(o[i * 4], o[i * 4 + 1], o[i * 4 + 2])));
    let accent = couleur_d_accent.map(|v| Rgb(v as u8, (v >> 8) as u8, (v >> 16) as u8));
    ChoixWindows {
        sombre: applications_claires == Some(0),
        barre_sombre: barre_claire != Some(1),
        accent,
        nuances,
    }
}

#[cfg(test)]
mod essais {
    use super::*;

    /// Rien de lu : le clair, la barre sombre, l'accent par défaut.
    #[test]
    fn rien_de_lu_donne_le_clair_et_la_barre_sombre() {
        assert_eq!(
            choix_du_registre(None, None, None, None),
            ChoixWindows {
                sombre: false,
                barre_sombre: true,
                accent: None,
                nuances: None,
            }
        );
    }

    /// 0 veut dire sombre, 1 veut dire clair ; toute autre valeur garde le sens sûr.
    #[test]
    fn les_deux_modes() {
        assert!(choix_du_registre(Some(0), None, None, None).sombre);
        assert!(!choix_du_registre(Some(1), None, None, None).sombre);
        assert!(!choix_du_registre(Some(7), None, None, None).sombre);
        assert!(!choix_du_registre(None, Some(1), None, None).barre_sombre);
        assert!(choix_du_registre(None, Some(0), None, None).barre_sombre);
        assert!(choix_du_registre(None, Some(7), None, None).barre_sombre);
    }

    /// `AccentColor` est écrit `0xAABBGGRR` : le rouge dans l'octet du bas, l'alpha ignoré.
    #[test]
    fn l_accent_se_lit_a_l_envers() {
        let c = choix_du_registre(None, None, None, Some(0xFF_D7_78_00));
        assert_eq!(c.accent, Some(Rgb(0x00, 0x78, 0xD7)));
        let c = choix_du_registre(None, None, None, Some(0x00_33_22_11));
        assert_eq!(c.accent, Some(Rgb(0x11, 0x22, 0x33)));
    }

    /// Les 7 premières couleurs RGBA de la palette, dans l'ordre ; l'alpha et la 8e ignorés.
    #[test]
    fn la_palette_donne_sept_nuances() {
        let octets: Vec<u8> = (0..32u8).collect();
        let Some(n) = choix_du_registre(None, None, Some(&octets), None).nuances else {
            unreachable!("32 octets donnent une palette");
        };
        assert_eq!(n[0], Rgb(0, 1, 2));
        assert_eq!(n[1], Rgb(4, 5, 6));
        assert_eq!(n[6], Rgb(24, 25, 26));
        // 28 octets suffisent ; un de moins, et la palette est ignorée.
        assert!(
            choix_du_registre(None, None, Some(&octets[..28]), None)
                .nuances
                .is_some()
        );
        assert_eq!(
            choix_du_registre(None, None, Some(&octets[..27]), None).nuances,
            None
        );
        assert_eq!(choix_du_registre(None, None, Some(&[]), None).nuances, None);
    }
}
