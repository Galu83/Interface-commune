//! Les images des icônes (bloc M) : l'[`Image`] donnée par le logiciel, l'icône de Windows
//! qu'on en fait, et l'icône du logiciel à la taille des petites icônes (l'icône seule).

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS, DeleteObject,
    HGDIOBJ,
};
use windows::Win32::UI::HiDpi::{GetDpiForSystem, GetSystemMetricsForDpi};
use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO, SM_CXSMICON};

use crate::Logiciel;

/// Une image d'icône : `taille` × `taille` pixels en BGRA (alpha non prémultiplié), lignes
/// de haut en bas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub taille: u32,
    pub bgra: Vec<u8>,
}

impl Image {
    /// Depuis des pixels RVBA (l'ordre de la plupart des bibliothèques d'images).
    pub fn depuis_rvba(taille: u32, mut rvba: Vec<u8>) -> Image {
        for p in rvba.as_chunks_mut::<4>().0 {
            p.swap(0, 2);
        }
        Image { taille, bgra: rvba }
    }
}

/// Fait une icône de Windows depuis des pixels BGRA (`taille`² × 4 octets).
pub(crate) fn icone_windows(taille: i32, px: &[u8]) -> Option<HICON> {
    if taille <= 0 || px.len() != (taille as usize) * (taille as usize) * 4 {
        return None;
    }
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: taille,
            // Négatif : lignes de haut en bas.
            biHeight: -taille,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    // SAFETY: `info` décrit une image 32 bits de `taille`² pixels ; Windows alloue `bits`
    // (taille × taille × 4 octets), où l'on copie exactement `px` (taille vérifiée plus
    // haut). Les deux images sont détruites après CreateIconIndirect, qui en fait sa copie.
    unsafe {
        let couleur = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        if bits.is_null() {
            let _ = DeleteObject(HGDIOBJ(couleur.0));
            return None;
        }
        std::ptr::copy_nonoverlapping(px.as_ptr(), bits.cast::<u8>(), px.len());
        let masque_octets = vec![0u8; (taille as usize).div_ceil(16) * 2 * taille as usize];
        let masque = CreateBitmap(taille, taille, 1, 1, Some(masque_octets.as_ptr().cast()));
        let ii = ICONINFO {
            fIcon: true.into(),
            hbmMask: masque,
            hbmColor: couleur,
            ..Default::default()
        };
        let icone = CreateIconIndirect(&ii).ok();
        let _ = DeleteObject(HGDIOBJ(couleur.0));
        let _ = DeleteObject(HGDIOBJ(masque.0));
        icone
    }
}

/// Fabrique l'icône du logiciel à la taille des petites icônes.
pub(crate) fn fabriquer_icone(logiciel: &dyn Logiciel, barre_sombre: bool) -> Option<HICON> {
    // SAFETY: simples lectures de réglages de Windows.
    let taille = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem()) }.clamp(16, 64);
    let px = logiciel
        .icone_etat(taille as u32, barre_sombre)
        .unwrap_or_else(|| (logiciel.identite().icone)(taille as u32, barre_sombre));
    icone_windows(taille, &px)
}
