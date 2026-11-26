//! La seule fabrique de fenêtre cachée de la bibliothèque : la fenêtre de l'icône unique
//! (`horloge`), celle des icônes d'une zone (`zone`) et celles des essais. Une fenêtre de style
//! simple, demandée de taille nulle (Windows lui donne sa taille minimale), jamais montrée, sur le fil qui l'appelle : elle reçoit les messages
//! de Windows (gestes sur l'icône, thème, fin de session, réveil).
//!
//! Pas `HWND_MESSAGE` : une fenêtre « de messages seulement » ne reçoit pas les messages
//! diffusés à toutes les fenêtres, dont « TaskbarCreated » (l'Explorateur a redémarré, il faut
//! remettre l'icône) et le changement de thème.

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, RegisterClassW, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPED,
};
use windows::core::{HSTRING, PCWSTR};

/// La procédure qui reçoit les messages de la fenêtre.
pub(crate) type Procedure = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// Que faire si la classe de fenêtre est déjà enregistrée dans le programme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClasseDejaLa {
    /// Une erreur : la fenêtre doit être la seule de sa classe (l'icône unique ; une deuxième
    /// instance la retrouve par sa classe).
    Refuser,
    /// La reprendre : plusieurs fenêtres de la même classe sont permises (une deuxième zone
    /// d'icônes, les essais).
    Reprendre,
}

/// Crée une fenêtre cachée de cette classe et de ce titre, dont `procedure` reçoit les
/// messages. La procédure doit vivre tout le programme (une fonction) et rendre à
/// `DefWindowProcW` ce qu'elle ne traite pas. La fenêtre est détruite par l'appelant
/// (`DestroyWindow`), sur le même fil.
pub(crate) fn fenetre_cachee(
    classe: &HSTRING,
    titre: &HSTRING,
    procedure: Procedure,
    deja_la: ClasseDejaLa,
) -> Result<HWND, String> {
    // SAFETY: module de ce programme ; classe décrite par une structure locale dont le nom
    // vit pendant les appels (Windows le copie) ; la procédure est une fonction, qui vit tout
    // le programme ; fenêtre de style simple et de taille nulle, jamais montrée.
    unsafe {
        let instance = GetModuleHandleW(PCWSTR::null()).map_err(|e| e.to_string())?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(classe.as_ptr()),
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 && deja_la == ClasseDejaLa::Refuser {
            return Err("classe de fenêtre refusée".into());
        }
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            classe,
            titre,
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .map_err(|e| e.to_string())
    }
}
