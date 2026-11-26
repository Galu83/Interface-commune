//! Les raccourcis clavier tels qu'on les lit dans le volet, les menus et les Paramètres : **en
//! toutes lettres, comme Windows en français** (« Ctrl+Maj+K », « Alt+Flèche droite »,
//! « Win+Échap »), jamais en symboles (⇧, ⌃, ⌥, ⌘, →).
//!
//! Une seule fonction, [`lisible`], sert partout : elle comprend les écritures courantes d'un
//! raccourci (« ctrl+alt+shift+k » du champ de raccourci, « Ctrl+Alt+→ », « ⌃⇧K », les noms
//! anglais de la bibliothèque de dessin comme « Down » ou « PageUp », les noms français) et les
//! rend dans une seule forme : les modificateurs d'abord, dans l'ordre Win, Ctrl, Alt, Maj, puis
//! la touche. Elle ne change rien à la valeur gardée par le logiciel : seul l'affichage passe
//! par elle. Traduire une touche de la bibliothèque de dessin reste au dessin
//! (`interface_commune::dessin::raccourci_de`).

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
/// use ic_coeur::textes::lisible;
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
        // Des symboles collés (« ⌃⇧K ») ou séparés d'une espace (« ⌃ ⇧ K ») : on les détache
        // un à un. Sans sauter l'espace, « ⌃ ⌃ » donnait « Ctrl+⌃ », relu en « Ctrl ».
        let mut reste = morceau;
        while let Some(m) = reste.chars().next().and_then(Modificateur::de_symbole) {
            modificateurs.push(m);
            reste = reste[reste.chars().next().map_or(0, char::len_utf8)..].trim_start();
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
