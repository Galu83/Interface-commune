//! Le réveil du fil des fenêtres : un événement Windows qu'on attend en répondant aux messages
//! envoyés au fil. Chaque `unsafe` est commenté.

use std::time::Duration;

use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_FAILED};
use windows::Win32::System::Threading::{CreateEventW, SetEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    MSG, MsgWaitForMultipleObjects, PM_NOREMOVE, PM_QS_SENDMESSAGE, PeekMessageW, QS_SENDMESSAGE,
};
use windows::core::PCWSTR;

/// Le réveil du fil des fenêtres (un événement Windows). Entre deux fenêtres, ce fil attend
/// une demande ; il doit pourtant répondre aux messages que Windows envoie à la fenêtre
/// cachée que winit lui laisse (« Winit Thread Event Target ») : sinon Windows le croit bloqué,
/// et à la fermeture de session il arrête le logiciel de force (« s'est bloquée »).
pub struct Reveil(isize);

impl Reveil {
    pub fn new() -> Self {
        // SAFETY: événement anonyme à remise automatique, fermé par `Drop`.
        let h = unsafe { CreateEventW(None, false, false, PCWSTR::null()) };
        Reveil(h.map_or(0, |h| h.0 as isize))
    }

    /// Lève le fil qui attend (une demande vient d'être envoyée).
    pub fn sonner(&self) {
        if self.0 != 0 {
            // SAFETY: événement créé par `new`, ouvert jusqu'à `Drop`.
            let _ = unsafe { SetEvent(HANDLE(self.0 as *mut _)) };
        }
    }

    /// Attend que le réveil sonne (au plus `ms`), en traitant au passage les messages
    /// *envoyés* aux fenêtres de ce fil (fin de session, « répond-il ? » de Windows). Les
    /// messages *postés* restent dans la file pour la prochaine fenêtre (winit les traite).
    pub fn attendre(&self, ms: u32) {
        let poignee = [HANDLE(self.0 as *mut _)];
        let (poignees, ms): (Option<&[HANDLE]>, u32) = if self.0 != 0 {
            (Some(&poignee), ms)
        } else {
            // Sans événement : on regarde la file souvent.
            (None, ms.min(50))
        };
        let mut m = MSG::default();
        // SAFETY: événement valide (ou aucun) ; `PeekMessageW` sur un MSG local, limité aux
        // messages envoyés, n'en retire aucun de la file.
        let resultat = unsafe {
            let r = MsgWaitForMultipleObjects(poignees, false, ms, QS_SENDMESSAGE);
            let _ = PeekMessageW(&mut m, None, 0, 0, PM_NOREMOVE | PM_QS_SENDMESSAGE);
            r
        };
        if resultat == WAIT_FAILED {
            // L'attente a échoué (poignée refusée) : une courte pause, pour ne jamais
            // tourner en boucle à 100 % du processeur.
            std::thread::sleep(Duration::from_millis(u64::from(ms.min(50))));
        }
    }
}

impl Default for Reveil {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Reveil {
    fn drop(&mut self) {
        if self.0 != 0 {
            // SAFETY: événement créé par `new`, fermé une seule fois.
            let _ = unsafe { CloseHandle(HANDLE(self.0 as *mut _)) };
        }
    }
}
