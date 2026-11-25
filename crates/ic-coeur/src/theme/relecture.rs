//! Suivre Windows (C-90) : le clair ou sombre et l'accent sont relus toutes les 2 s, et la
//! palette change dès qu'ils changent ; un thème imposé (captures, essais) l'emporte sur
//! Windows. Le temps écoulé depuis la dernière lecture est donné par l'appelant, jamais lu
//! ici ; lire Windows reste à la bibliothèque (bloc L).

use std::time::Duration;

use super::couleurs::{ChoixWindows, Palette};

/// Le thème est relu dans Windows à cet intervalle.
pub const RELECTURE: Duration = Duration::from_secs(2);

/// Faut-il relire Windows, `depuis` la dernière lecture ?
pub fn faut_relire(depuis: Duration) -> bool {
    depuis >= RELECTURE
}

/// Le choix de Windows, avec le clair (`Some(false)`) ou le sombre (`Some(true)`) imposé s'il
/// y en a un ; l'accent reste celui de Windows.
pub fn impose(mut choix: ChoixWindows, force: Option<bool>) -> ChoixWindows {
    if let Some(f) = force {
        choix.sombre = f;
    }
    choix
}

/// Ce que Windows vient de dire (`lu`), comparé au thème suivi (`actuel`) : le nouveau choix et
/// sa palette s'il a changé, rien sinon (la palette ne se recalcule pas pour rien).
pub fn relu(actuel: &ChoixWindows, lu: ChoixWindows, force: Option<bool>) -> Option<(ChoixWindows, Palette)> {
    let choix = impose(lu, force);
    if &choix == actuel {
        return None;
    }
    let palette = Palette::de(&choix);
    Some((choix, palette))
}
