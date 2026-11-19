//! Les raccourcis clavier tels qu'on les lit dans le volet, les menus et les Paramètres : **en
//! toutes lettres, comme Windows en français** (« Ctrl+Maj+K », « Alt+Flèche droite »,
//! « Win+Échap »), jamais en symboles (⇧, ⌃, ⌥, ⌘, →).
//!
//! Une seule fonction, [`lisible`], sert partout : elle comprend les écritures courantes d'un
//! raccourci (« ctrl+alt+shift+k » de [`crate::dessin::raccourci_de`], « Ctrl+Alt+→ », « ⌃⇧K »,
//! les noms d'egui comme « Down » ou « PageUp », les noms français) et les rend dans une seule
//! forme : les modificateurs d'abord, dans l'ordre Win, Ctrl, Alt, Maj, puis la touche. Elle ne
//! change rien à la valeur gardée par le logiciel : seule l'affichage passe par elle.

/// Un modificateur, dans l'ordre où Windows l'écrit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Modificateur {
    Win,
    Ctrl,
    Alt,
    Maj,
}

impl Modificateur {
    fn nom(self) -> &'static str {
        match self {
            Modificateur::Win => "Win",
            Modificateur::Ctrl => "Ctrl",
            Modificateur::Alt => "Alt",
            Modificateur::Maj => "Maj",
        }
    }

    /// Le modificateur d'un symbole (⌃, ⌥, ⇧, ⌘, ^).
    fn de_symbole(c: char) -> Option<Modificateur> {
        match c {
            '⌃' | '^' => Some(Modificateur::Ctrl),
            '⌥' => Some(Modificateur::Alt),
            '⇧' => Some(Modificateur::Maj),
            // « Commande » de egui vaut Ctrl sous Windows.
            '⌘' => Some(Modificateur::Ctrl),
            _ => None,
        }
    }

    /// Le modificateur d'un nom (« ctrl », « shift », « maj », « option », « super »…).
    fn de_nom(nom: &str) -> Option<Modificateur> {
        match nom {
            "ctrl" | "ctl" | "control" | "contrôle" | "controle" | "command" | "cmd" | "commande" => {
                Some(Modificateur::Ctrl)
            }
            "alt" | "option" | "opt" => Some(Modificateur::Alt),
            "shift" | "maj" | "majuscule" | "majuscules" => Some(Modificateur::Maj),
            "win" | "windows" | "super" | "meta" | "logo" | "touche windows" => Some(Modificateur::Win),
            _ => None,
        }
    }
}

/// Le nom français d'une touche autre qu'une lettre ou un chiffre (`bas` : en minuscules).
fn touche_nommee(bas: &str) -> Option<&'static str> {
    Some(match bas {
        "escape" | "esc" | "échap" | "echap" | "⎋" => "Échap",
        "enter" | "return" | "entrée" | "entree" | "↵" | "⏎" => "Entrée",
        "tab" | "tabulation" | "⇥" => "Tab",
        "space" | "espace" | "␣" => "Espace",
        "backspace" | "retour arrière" | "retour arriere" | "⌫" => "Retour arrière",
        "delete" | "del" | "suppr" | "supprimer" | "⌦" => "Suppr",
        "insert" | "ins" | "inser" => "Inser",
        "home" | "début" | "debut" | "⇱" => "Début",
        "end" | "fin" | "⇲" => "Fin",
        "pageup" | "page up" | "pgup" | "page précédente" | "page precedente" | "page préc." | "⇞" => {
            "Page précédente"
        }
        "pagedown" | "page down" | "pgdn" | "pgdown" | "page suivante" | "page suiv." | "⇟" => {
            "Page suivante"
        }
        "left" | "arrowleft" | "arrow left" | "flèche gauche" | "fleche gauche" | "←" => "Flèche gauche",
        "right" | "arrowright" | "arrow right" | "flèche droite" | "fleche droite" | "→" => {
            "Flèche droite"
        }
        "up" | "arrowup" | "arrow up" | "flèche haut" | "fleche haut" | "↑" => "Flèche haut",
        "down" | "arrowdown" | "arrow down" | "flèche bas" | "fleche bas" | "↓" => "Flèche bas",
        "plus" => "Plus",
        "minus" | "moins" => "Moins",
        "altgr" | "alt gr" => "Alt Gr",
        "printscreen" | "print screen" | "impr écran" | "impr. écran" | "impr ecran" => "Impr. écran",
        "scrolllock" | "scroll lock" | "arrêt défil" | "arret defil" => "Arrêt défil",
        "pause" => "Pause",
        "capslock" | "caps lock" | "verr. maj" | "verr maj" => "Verr. Maj",
        "numlock" | "num lock" | "verr. num" | "verr num" => "Verr. num",
        "menu" | "contextmenu" | "context menu" => "Menu",
        _ => return None,
    })
}

/// Une touche (ni modificateur ni nom connu) : une lettre en majuscule, F1…F35 tel quel, le
/// reste (chiffre, ponctuation) tel qu'écrit, un mot inconnu avec sa première lettre en
/// majuscule.
fn touche(t: &str) -> String {
    let bas = t.to_lowercase();
    if let Some(nom) = touche_nommee(&bas) {
        return nom.to_string();
    }
    let mut c = t.chars();
    match (c.next(), c.next()) {
        (Some(un), None) => un.to_uppercase().collect(),
        _ if bas.len() > 1
            && bas.starts_with('f')
            && bas[1..].chars().all(|d| d.is_ascii_digit())
            && bas[1..].parse::<u8>().is_ok_and(|n| (1..=35).contains(&n)) =>
        {
            bas.to_uppercase()
        }
        (Some(un), _) => un.to_uppercase().chain(t.chars().skip(1)).collect(),
        (None, _) => String::new(),
    }
}

/// Un raccourci en toutes lettres, comme Windows en français : « Ctrl+Maj+K », « Alt+Flèche
/// droite », « Win+Échap », « F5 ». Les modificateurs d'abord (Win, Ctrl, Alt, Maj), puis la
/// touche. Les symboles (⌃ ⌥ ⇧ ⌘ ← → ↑ ↓ ↵ ⌫…) deviennent des mots ; les noms anglais ou d'egui
/// (« shift », « Down », « PageUp ») aussi. Le résultat est lui-même compris par la fonction :
/// la rappeler ne change rien. Une chaîne vide reste vide.
///
/// ```
/// use interface_commune::touches::lisible;
/// assert_eq!(lisible("ctrl+shift+k"), "Ctrl+Maj+K");
/// assert_eq!(lisible("Ctrl+Alt+→"), "Ctrl+Alt+Flèche droite");
/// assert_eq!(lisible("⌃⇧K"), "Ctrl+Maj+K");
/// ```
pub fn lisible(raccourci: &str) -> String {
    let brut = raccourci.trim();
    // « Ctrl++ » : la touche est le signe plus lui-même.
    let (corps, plus) = match brut.strip_suffix('+') {
        Some(debut) => (debut.trim_end_matches('+'), true),
        None => (brut, false),
    };
    let mut modificateurs: Vec<Modificateur> = Vec::new();
    let mut touches: Vec<String> = Vec::new();
    for morceau in corps.split('+').map(str::trim).filter(|m| !m.is_empty()) {
        // Des symboles collés (« ⌃⇧K ») : on les détache un à un.
        let mut reste = morceau;
        while let Some(m) = reste.chars().next().and_then(Modificateur::de_symbole) {
            modificateurs.push(m);
            reste = &reste[reste.chars().next().map_or(0, char::len_utf8)..];
        }
        let reste = reste.trim();
        if reste.is_empty() {
            continue;
        }
        match Modificateur::de_nom(&reste.to_lowercase()) {
            Some(m) => modificateurs.push(m),
            None => touches.push(touche(reste)),
        }
    }
    if plus {
        touches.push("+".to_string());
    }
    modificateurs.sort();
    modificateurs.dedup();
    modificateurs
        .iter()
        .map(|m| m.nom().to_string())
        .chain(touches)
        .collect::<Vec<_>>()
        .join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_modificateurs_en_toutes_lettres_dans_l_ordre_de_windows() {
        assert_eq!(lisible("ctrl+alt+shift+k"), "Ctrl+Alt+Maj+K");
        assert_eq!(lisible("Shift+Ctrl+K"), "Ctrl+Maj+K");
        assert_eq!(lisible("Ctrl+Maj+K"), "Ctrl+Maj+K");
        assert_eq!(lisible("super+d"), "Win+D");
        assert_eq!(lisible("Windows+Shift+S"), "Win+Maj+S");
        assert_eq!(lisible("Win+Ctrl+Shift+B"), "Win+Ctrl+Maj+B");
        assert_eq!(lisible("option+x"), "Alt+X");
        assert_eq!(lisible("control+x"), "Ctrl+X");
        assert_eq!(lisible("Alt"), "Alt");
        assert_eq!(lisible("Win"), "Win");
        assert_eq!(lisible("Ctrl+Ctrl+K"), "Ctrl+K", "pas de doublon");
    }

    #[test]
    fn les_symboles_deviennent_des_mots() {
        assert_eq!(lisible("⌃⇧K"), "Ctrl+Maj+K");
        assert_eq!(lisible("⌥⇧⌃K"), "Ctrl+Alt+Maj+K");
        assert_eq!(lisible("⇧+⌃+K"), "Ctrl+Maj+K");
        assert_eq!(lisible("⌘K"), "Ctrl+K", "commande vaut Ctrl sous Windows");
        assert_eq!(lisible("Ctrl+Alt+→"), "Ctrl+Alt+Flèche droite");
        assert_eq!(lisible("Ctrl+←"), "Ctrl+Flèche gauche");
        assert_eq!(lisible("Maj+↑"), "Maj+Flèche haut");
        assert_eq!(lisible("Alt+↓"), "Alt+Flèche bas");
        assert_eq!(lisible("Ctrl+⌫"), "Ctrl+Retour arrière");
        assert_eq!(lisible("⇧↵"), "Maj+Entrée");
        for symbole in ['⇧', '⌃', '⌥', '⌘', '→', '←', '↑', '↓', '⏎', '⌫', '⇥', '⎋'] {
            let r = lisible(&format!("⌃{symbole}"));
            assert!(!r.contains(symbole), "{r}");
        }
    }

    #[test]
    fn les_touches_nommees_en_francais() {
        assert_eq!(lisible("Escape"), "Échap");
        assert_eq!(lisible("esc"), "Échap");
        assert_eq!(lisible("Win+Escape"), "Win+Échap");
        assert_eq!(lisible("Enter"), "Entrée");
        assert_eq!(lisible("Ctrl+Space"), "Ctrl+Espace");
        assert_eq!(lisible("Backspace"), "Retour arrière");
        assert_eq!(lisible("Delete"), "Suppr");
        assert_eq!(lisible("Ctrl+Alt+Delete"), "Ctrl+Alt+Suppr");
        assert_eq!(lisible("Home"), "Début");
        assert_eq!(lisible("End"), "Fin");
        assert_eq!(lisible("PageUp"), "Page précédente");
        assert_eq!(lisible("Ctrl+PageDown"), "Ctrl+Page suivante");
        assert_eq!(lisible("Alt+Right"), "Alt+Flèche droite");
        assert_eq!(lisible("ArrowLeft"), "Flèche gauche");
        assert_eq!(lisible("ctrl+down"), "Ctrl+Flèche bas");
        assert_eq!(lisible("Up"), "Flèche haut");
        assert_eq!(lisible("Ctrl+Plus"), "Ctrl+Plus");
        assert_eq!(lisible("Ctrl+Minus"), "Ctrl+Moins");
        assert_eq!(lisible("Tab"), "Tab");
        assert_eq!(lisible("shift+tab"), "Maj+Tab");
    }

    #[test]
    fn les_lettres_les_chiffres_et_les_touches_de_fonction() {
        assert_eq!(lisible("ctrl+k"), "Ctrl+K");
        assert_eq!(lisible("Ctrl+Alt+1"), "Ctrl+Alt+1");
        assert_eq!(lisible("f5"), "F5");
        assert_eq!(lisible("Maj+F10"), "Maj+F10");
        assert_eq!(lisible("alt+f4"), "Alt+F4");
        assert_eq!(lisible("ctrl+é"), "Ctrl+É");
        assert_eq!(lisible("Ctrl+,"), "Ctrl+,");
    }

    #[test]
    fn le_signe_plus_est_une_touche() {
        assert_eq!(lisible("Ctrl++"), "Ctrl++");
        assert_eq!(lisible("ctrl+shift++"), "Ctrl+Maj++");
        assert_eq!(lisible("+"), "+");
    }

    #[test]
    fn vide_espaces_et_inconnu() {
        assert_eq!(lisible(""), "");
        assert_eq!(lisible("   "), "");
        assert_eq!(lisible(" Ctrl + Maj + K "), "Ctrl+Maj+K");
        assert_eq!(
            lisible("Ctrl+volumeup"),
            "Ctrl+Volumeup",
            "un mot inconnu garde ses lettres"
        );
    }

    #[test]
    fn la_forme_lisible_est_stable() {
        for r in [
            "ctrl+alt+shift+k",
            "Ctrl+Alt+→",
            "⌃⇧K",
            "Win+Escape",
            "Alt+PageUp",
            "Ctrl+Alt+Delete",
            "shift+tab",
            "F5",
            "Ctrl++",
            "Ctrl+Retour arrière",
            "Ctrl+Flèche droite",
            "Impr. écran",
            "Alt Gr+E",
        ] {
            let une = lisible(r);
            assert_eq!(lisible(&une), une, "{r}");
        }
    }

    #[test]
    fn ce_que_saisit_le_champ_de_raccourci_se_lit_bien() {
        use egui::{Key, Modifiers};
        let m = Modifiers {
            ctrl: true,
            shift: true,
            ..Modifiers::NONE
        };
        assert_eq!(lisible(&crate::dessin::raccourci_de(m, Key::K)), "Ctrl+Maj+K");
        assert_eq!(
            lisible(&crate::dessin::raccourci_de(Modifiers::ALT, Key::ArrowRight)),
            "Alt+Flèche droite"
        );
        assert_eq!(
            lisible(&crate::dessin::raccourci_de(Modifiers::NONE, Key::F5)),
            "F5"
        );
        assert_eq!(
            lisible(&crate::dessin::raccourci_de(Modifiers::CTRL, Key::Backspace)),
            "Ctrl+Retour arrière"
        );
    }
}
