//! Les étages de la refonte (`docs/ORGANIGRAMME.md`, §2 principe 3, et §10), vérifiés en
//! lisant les fichiers du dépôt :
//! - le **cœur** (`crates/ic-coeur`) ne dépend de rien : ni egui, ni eframe, ni Windows, ni de
//!   `interface-commune` (le sens des dépendances va du dessin et des bords vers le cœur,
//!   jamais l'inverse) ; son code n'appelle ni egui, ni Windows, ni le disque, le réseau,
//!   l'horloge ou le système, et n'a aucun `unsafe` ;
//! - le **dessin** (`src/dessin.rs` aujourd'hui, `src/dessin/` ensuite) n'appelle jamais
//!   Windows : ni `windows::`, ni `unsafe`, ni eframe ;
//! - `unsafe` n'est permis que dans les **bords** (`src/bords/`) et, en attendant qu'ils y
//!   passent, dans les deux modules qui appellent encore Windows aujourd'hui (`horloge`, `zone`).
//!
//! Le compilateur tient déjà le cœur (une crate sans dépendance) ; cet essai le redit, avec
//! un message clair, et tient seul la frontière entre le dessin et les bords, qui vivent dans
//! la même crate.
// Dans un essai, un échec doit arrêter l'essai : `expect` et `panic!` y sont voulus.
#![allow(clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

const RACINE: &str = env!("CARGO_MANIFEST_DIR");
const VERROU: &str = include_str!("../Cargo.lock");

/// Les crates que le cœur ne prend jamais, même pour ses seuls essais.
const INTERDITES_AU_COEUR: [&str; 12] = [
    "egui",
    "eframe",
    "epaint",
    "emath",
    "ecolor",
    "egui_glow",
    "winit",
    "windows",
    "windows-sys",
    "raw-window-handle",
    "skrifa",
    "interface-commune",
];

/// Le texte d'un fichier du dépôt, chemin relatif à sa racine.
fn lire(relatif: &str) -> String {
    let chemin = Path::new(RACINE).join(relatif);
    fs::read_to_string(&chemin).unwrap_or_else(|e| panic!("{relatif} illisible : {e}"))
}

/// Tous les fichiers `.rs` sous un dossier (aucun s'il n'existe pas).
fn sources(dossier: &Path) -> Vec<PathBuf> {
    let mut trouves = Vec::new();
    let Ok(entrees) = fs::read_dir(dossier) else {
        return trouves;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        if chemin.is_dir() {
            trouves.extend(sources(&chemin));
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            trouves.push(chemin);
        }
    }
    trouves
}

/// Le code sans ses commentaires (`//…` jusqu'au bout de la ligne, `/* … */`). Un `//` dans une
/// chaîne coupe le reste de la ligne : l'essai voit alors moins, jamais plus.
fn sans_commentaires(texte: &str) -> String {
    let mut code = String::new();
    let mut reste = texte;
    while let Some(i) = reste.find("/*") {
        code.push_str(&reste[..i]);
        reste = reste[i..].find("*/").map_or("", |j| &reste[i + j + 2..]);
    }
    code.push_str(reste);
    code.lines()
        .map(|l| l.find("//").map_or(l, |i| &l[..i]))
        .collect::<Vec<_>>()
        .join("\n")
}

fn est_identifiant(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// `motif` est-il dans `code` comme un mot : rien d'un identifiant juste avant, ni juste après
/// si le motif finit par une lettre (`unsafe` trouve `unsafe {`, pas `unsafe_code`).
fn contient(code: &str, motif: &str) -> bool {
    let fin_en_mot = motif.chars().last().is_some_and(est_identifiant);
    code.match_indices(motif).any(|(i, _)| {
        let avant = code[..i].chars().last().is_none_or(|c| !est_identifiant(c));
        let apres = code[i + motif.len()..].chars().next();
        avant && (!fin_en_mot || apres.is_none_or(|c| !est_identifiant(c)))
    })
}

/// Les fichiers de `fichiers` qui contiennent un des `motifs`, avec le motif trouvé.
fn fautes(fichiers: &[PathBuf], motifs: &[&str]) -> Vec<String> {
    let mut trouvees = Vec::new();
    for chemin in fichiers {
        let texte = fs::read_to_string(chemin).unwrap_or_else(|e| panic!("{chemin:?} : {e}"));
        let code = sans_commentaires(&texte);
        for motif in motifs.iter().filter(|m| contient(&code, m)) {
            let relatif = chemin.strip_prefix(RACINE).unwrap_or(chemin);
            trouvees.push(format!("{} : « {motif} »", relatif.display()));
        }
    }
    trouvees
}

/// Les crates dont dépend `paquet` dans `Cargo.lock` (le graphe résolu par Cargo).
fn dependances_verrouillees(paquet: &str) -> Vec<String> {
    let entete = format!("name = \"{paquet}\"");
    let bloc = VERROU
        .split("[[package]]")
        .find(|b| b.lines().any(|l| l.trim() == entete))
        .unwrap_or_else(|| panic!("{paquet} absent de Cargo.lock"));
    let Some(debut) = bloc.find("dependencies = [") else {
        return Vec::new();
    };
    let liste = &bloc[debut..];
    let liste = &liste[..liste.find(']').unwrap_or(liste.len())];
    liste
        .split('"')
        .skip(1)
        .step_by(2)
        .filter_map(|d| d.split_whitespace().next())
        .map(str::to_string)
        .collect()
}

#[test]
fn le_coeur_ne_declare_aucune_dependance() {
    let manifeste = lire("crates/ic-coeur/Cargo.toml");
    let mut section = String::new();
    let mut fautes = Vec::new();
    for ligne in manifeste.lines().map(str::trim) {
        if ligne.starts_with('[') {
            section = ligne.to_string();
        } else if section.contains("dependencies") && !ligne.is_empty() && !ligne.starts_with('#') {
            let nom = ligne
                .split(['=', '.', ' '])
                .next()
                .unwrap_or("")
                .trim_matches('"');
            let essais = section == "[dev-dependencies]";
            if !essais || INTERDITES_AU_COEUR.contains(&nom) {
                fautes.push(format!("{section} {ligne}"));
            }
        }
    }
    assert!(
        fautes.is_empty(),
        "le cœur (crates/ic-coeur) ne dépend de rien (ORGANIGRAMME.md, §10) : {fautes:?}"
    );
}

#[test]
fn le_graphe_resolu_va_du_dessin_vers_le_coeur() {
    let du_coeur = dependances_verrouillees("ic-coeur");
    let interdites: Vec<_> = du_coeur
        .iter()
        .filter(|d| INTERDITES_AU_COEUR.contains(&d.as_str()))
        .collect();
    assert!(
        interdites.is_empty(),
        "Cargo.lock : ic-coeur dépend de {interdites:?}"
    );
    let de_la_racine = dependances_verrouillees("interface-commune");
    assert!(
        de_la_racine.iter().any(|d| d == "ic-coeur"),
        "Cargo.lock : interface-commune doit prendre ic-coeur (le sens des étages)"
    );
}

#[test]
fn le_coeur_est_pur() {
    let lib = sans_commentaires(&lire("crates/ic-coeur/src/lib.rs"));
    assert!(
        lib.contains("#![forbid(unsafe_code)]"),
        "ic-coeur : #![forbid(unsafe_code)] manque"
    );
    let fichiers = sources(&Path::new(RACINE).join("crates/ic-coeur/src"));
    assert!(!fichiers.is_empty(), "ic-coeur : aucune source trouvée");
    let motifs = [
        "egui",
        "eframe",
        "epaint",
        "winit",
        "windows::",
        "windows_sys",
        "raw_window_handle",
        "interface_commune",
        "unsafe",
        "extern",
        "#[link",
        "std::fs",
        "std::net",
        "std::env",
        "std::process",
        "std::thread",
        "Instant::now",
        "SystemTime",
    ];
    let trouvees = fautes(&fichiers, &motifs);
    assert!(
        trouvees.is_empty(),
        "le cœur reçoit des faits et rend des décisions, sans egui, Windows, disque, réseau, \
         horloge ni système (ORGANIGRAMME.md, §10) : {trouvees:?}"
    );
}

#[test]
fn le_dessin_n_appelle_pas_windows() {
    let racine = Path::new(RACINE);
    let mut fichiers = sources(&racine.join("src/dessin"));
    let module = racine.join("src/dessin.rs");
    if module.is_file() {
        fichiers.push(module);
    }
    assert!(
        !fichiers.is_empty(),
        "ni src/dessin.rs ni src/dessin/ : l'essai ne verrait rien"
    );
    let motifs = [
        "windows::",
        "windows_sys",
        "unsafe",
        "eframe",
        "winit",
        "raw_window_handle",
    ];
    let trouvees = fautes(&fichiers, &motifs);
    assert!(
        trouvees.is_empty(),
        "le dessin, c'est egui seul, jamais Windows (ORGANIGRAMME.md, §10) : {trouvees:?}"
    );
}

/// Les modules qui appellent Windows aujourd'hui, hors de `src/bords/`. Chacun sort de cette
/// liste quand son bloc (M, J) le range dans les bords ; la liste finit vide. `echelle` et
/// `systeme` en sont sortis au bloc L, `horloge` et `zone` au bloc M.
const UNSAFE_EN_ATTENDANT: [&str; 0] = [];

#[test]
fn unsafe_seulement_dans_les_bords() {
    let src = Path::new(RACINE).join("src");
    let bords = src.join("bords");
    let fichiers: Vec<_> = sources(&src)
        .into_iter()
        .filter(|f| !f.starts_with(&bords))
        .filter(|f| {
            let relatif = f
                .strip_prefix(&src)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/");
            !UNSAFE_EN_ATTENDANT.contains(&relatif.as_str())
        })
        .collect();
    let trouvees = fautes(&fichiers, &["unsafe"]);
    assert!(
        trouvees.is_empty(),
        "`unsafe` n'est permis que dans src/bords/ (ORGANIGRAMME.md, §2) : {trouvees:?}"
    );
}

#[test]
fn le_motif_est_un_mot() {
    assert!(contient("unsafe { f() }", "unsafe"));
    assert!(!contient("#![forbid(unsafe_code)]", "unsafe"));
    assert!(contient("use windows::Win32;", "windows::"));
    assert!(!contient("theme::choix_windows::lire", "windows::"));
    assert!(!contient(
        &sans_commentaires("// egui\n/* eframe */ let a = 1;"),
        "egui"
    ));
    assert!(!contient(&sans_commentaires("/* eframe */ let a = 1;"), "eframe"));
}
