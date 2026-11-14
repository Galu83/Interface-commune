//! Les crates publiées sans fichier de licence ont leur avis MIT dans `docs/LICENCES.md`, à
//! la version compilée (`Cargo.lock`).

const LICENCES: &str = include_str!("../docs/LICENCES.md");
const VERROU: &str = include_str!("../Cargo.lock");

/// Les crates sans fichier de licence, et leur version quand la liste a été faite.
const SANS_FICHIER: [(&str, &str); 9] = [
    ("accesskit", "0.25.2"),
    ("ecolor", "0.37.0"),
    ("eframe", "0.37.0"),
    ("egui", "0.37.0"),
    ("egui-winit", "0.37.0"),
    ("egui_glow", "0.37.0"),
    ("emath", "0.37.0"),
    ("epaint", "0.37.0"),
    ("profiling", "1.0.18"),
];

/// La version d'une crate dans `Cargo.lock`.
fn version(nom: &str) -> Option<String> {
    let entete = format!("name = \"{nom}\"\n");
    let i = VERROU.find(&entete)? + entete.len();
    let ligne = VERROU[i..].lines().next()?;
    Some(
        ligne
            .strip_prefix("version = \"")?
            .trim_end_matches('"')
            .to_string(),
    )
}

#[test]
fn avis_mit_a_jour() {
    for (nom, v) in SANS_FICHIER {
        let compilee = version(nom).unwrap_or_default();
        assert_eq!(
            compilee, v,
            "{nom} a changé de version : refaire docs/LICENCES.md"
        );
        assert!(
            LICENCES.contains(&format!("## {nom} {v} (MIT)")),
            "{nom} {v} absent de docs/LICENCES.md"
        );
    }
    assert_eq!(
        LICENCES.matches("Permission is hereby granted").count(),
        SANS_FICHIER.len()
    );
}
