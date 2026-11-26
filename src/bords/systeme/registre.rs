//! Le thème de Windows lu dans le registre : le mode des applications, celui de la barre des
//! tâches, l'accent et sa palette. Ce que les valeurs veulent dire se décide dans le cœur
//! ([`ic_coeur::theme::choix_du_registre`]). Chaque `unsafe` est commenté.

use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, RRF_RT_REG_BINARY, RRF_RT_REG_DWORD, RegGetValueW,
};
use windows::core::{PCWSTR, w};

use crate::theme::ChoixWindows;

fn lire_dword(cle: PCWSTR, valeur: PCWSTR) -> Option<u32> {
    let mut v = 0u32;
    let mut taille = 4u32;
    // SAFETY: tampon local de 4 octets annoncé comme tel ; RRF_RT_REG_DWORD garantit le type.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            cle,
            valeur,
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut v).cast()),
            Some(&mut taille),
        )
    };
    r.is_ok().then_some(v)
}

fn lire_octets(cle: PCWSTR, valeur: PCWSTR) -> Option<Vec<u8>> {
    let mut tampon = [0u8; 64];
    let mut taille = tampon.len() as u32;
    // SAFETY: tampon local dont la taille est passée ; Windows n'écrit pas au-delà.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            cle,
            valeur,
            RRF_RT_REG_BINARY,
            None,
            Some(tampon.as_mut_ptr().cast()),
            Some(&mut taille),
        )
    };
    r.is_ok()
        .then(|| tampon[..(taille as usize).min(tampon.len())].to_vec())
}

/// Mode sombre ou clair, barre des tâches, accent : ce que Windows a choisi.
pub fn choix_windows() -> ChoixWindows {
    let theme = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    // AccentPalette : 8 couleurs RGBA, de la plus claire à la plus foncée.
    let palette = lire_octets(
        w!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent"),
        w!("AccentPalette"),
    );
    ic_coeur::theme::choix_du_registre(
        lire_dword(theme, w!("AppsUseLightTheme")),
        lire_dword(theme, w!("SystemUsesLightTheme")),
        palette.as_deref(),
        // AccentColor : 0xAABBGGRR.
        lire_dword(w!("Software\\Microsoft\\Windows\\DWM"), w!("AccentColor")),
    )
}
