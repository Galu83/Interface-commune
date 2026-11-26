//! Les images de référence (C-115 : le rendu ne change pas sans décision écrite).
//!
//! Chaque surface est rendue sans fenêtre (`capture`, rendu logiciel) puis comparée à son
//! image dans `tests/references/images/`. Les noms ne portent pas de version : une image ne
//! change que si le rendu change.
//!
//! - **Comparer** (par défaut) : un écart fait échouer l'essai ; l'image rendue et une image des
//!   écarts (en rouge) sont écrites dans `captures/references-ecarts/` pour qu'on les regarde.
//! - **Refaire** (`REFERENCES=refaire cargo test --test references`) : réécrit toutes les
//!   images et l'empreinte des polices. Seulement après une décision écrite de changer le
//!   rendu (méthode, refonte : « même rendu sauf décision écrite »).
//!
//! Les images dépendent des polices Segoe de Windows : leur empreinte est gardée dans
//! `tests/references/polices.txt`. Sur un poste dont les polices diffèrent (une autre version
//! de Windows, le serveur de la CI), la comparaison est sautée avec un message, et les images
//! rendues vont dans `captures/references-ici/` : la comparaison n'a de sens qu'avec les
//! mêmes polices.

#![cfg(windows)]
// Dans un essai, une image illisible doit faire échouer l'essai.
#![allow(clippy::expect_used, clippy::panic)]

#[path = "references/logiciel.rs"]
mod logiciel;

use std::path::{Path, PathBuf};

use interface_commune::capture::{self, Image, Options};
use interface_commune::dessin::{self, Genre};
use interface_commune::parametres::Parametres;
use interface_commune::toutes_faites::{
    self, Bouton, Champ, Choix, EtatLecture, EtatSimple, Lecture, Simple, Ton,
};
use interface_commune::{
    EtatMiseAJour, Logiciel, Reglages, Rgb, Valeur, couleurs, nuancier, polices, seuils, textes,
};

use logiciel::Fictif;

/// Un écart plus petit que ceci, sur chaque canal, ne compte pas (arrondis du rendu).
const ECART_PERMIS: u8 = 6;
/// La part des pixels qui peuvent dépasser [`ECART_PERMIS`] : 1 sur 5 000.
const PIXELS_PERMIS: usize = 5_000;

fn dossier_des_references() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("references")
}

fn dossier_des_captures(nom: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("captures").join(nom)
}

/// L'empreinte des polices que lit `polices` : nom, taille et somme (FNV-1a) de chaque fichier.
fn empreinte_des_polices() -> String {
    let mut lignes = Vec::new();
    for nom in [
        polices::SEGOE_VARIABLE,
        polices::SEGOE,
        polices::SEGOE_SEMIBOLD,
        polices::ICONES_FLUENT,
        polices::ICONES_MDL2,
    ] {
        match polices::lire(nom) {
            Some(octets) => {
                let mut somme: u64 = 0xcbf2_9ce4_8422_2325;
                for o in &octets {
                    somme ^= u64::from(*o);
                    somme = somme.wrapping_mul(0x0100_0000_01b3);
                }
                lignes.push(format!("{nom} {} {somme:016x}", octets.len()));
            }
            None => lignes.push(format!("{nom} absente")),
        }
    }
    lignes.join("\n") + "\n"
}

fn clair() -> Options {
    Options::default()
}

fn sombre() -> Options {
    Options {
        sombre: true,
        ..Options::default()
    }
}

fn a_150(o: Options) -> Options {
    Options { echelle: 1.5, ..o }
}

fn theme(o: Options) -> &'static str {
    if o.sombre { "sombre" } else { "clair" }
}

/// Toutes les images, avec leur nom de fichier (sans `.png`).
fn toutes_les_images() -> Vec<(String, Image)> {
    let fictif = Fictif::new();
    let mut images = Vec::new();
    let mut ajouter = |nom: String, image: Result<Image, String>| {
        images.push((nom.clone(), image.unwrap_or_else(|e| panic!("{nom} : {e}"))));
    };
    // Le volet et le petit menu.
    for o in [clair(), sombre()] {
        ajouter(format!("volet-{}", theme(o)), capture::volet(&fictif, o));
        ajouter(
            format!("petit-menu-{}", theme(o)),
            capture::petit_menu(&fictif, o),
        );
    }
    ajouter("volet-clair-150".into(), capture::volet(&fictif, a_150(clair())));
    ajouter(
        "volet-borne-clair".into(),
        capture::volet_borne(&fictif, 320.0, clair()),
    );
    // Chaque page des Paramètres.
    let reglages: &dyn Reglages = &fictif;
    let pages = interface_commune::parametres::modele::navigation(
        reglages.textes(),
        reglages.accueil(),
        &reglages.pages(),
    );
    for o in [clair(), sombre()] {
        for p in &pages {
            let nom = format!("parametres-{}-{}", p.cle.replace('_', "-"), theme(o));
            ajouter(nom, capture::parametres(reglages, Some(&p.cle), o));
        }
    }
    ajouter(
        "parametres-general-clair-150".into(),
        capture::parametres(reglages, Some("general"), a_150(clair())),
    );
    // La page Mise à jour, dans chacun de ses états.
    for (nom, e) in [
        ("inconnu", EtatMiseAJour::Inconnu),
        ("recherche", EtatMiseAJour::Recherche),
        ("prete", EtatMiseAJour::Prete("2.5.0".into())),
        (
            "erreur",
            EtatMiseAJour::Erreur("Le serveur ne répond pas.".into()),
        ),
        ("coupee", EtatMiseAJour::Coupee),
        (
            "information",
            EtatMiseAJour::Information("Une information inventée.".into()),
        ),
    ] {
        let l = Fictif::avec_mise_a_jour(e);
        let cle = interface_commune::parametres::modele::MISE_A_JOUR;
        ajouter(
            format!("mise-a-jour-{nom}-clair"),
            capture::parametres(&l, Some(cle), clair()),
        );
    }
    // La carte « Conserver ces modifications ? ».
    let mut s = Parametres::new(reglages, Some("general"));
    s.changer_a_confirmer(
        reglages,
        "general.demarrage",
        "Démarrage",
        Valeur::Booleen(true),
        Valeur::Booleen(false),
    );
    ajouter(
        "parametres-conserver-clair".into(),
        capture::parametres_de(&mut s, reglages, clair()),
    );
    // Les fenêtres toutes faites et les contrôles.
    for o in [clair(), sombre()] {
        ajouter(
            format!("version-prete-{}", theme(o)),
            fenetre_simple(&version_prete(&fictif), o),
        );
        ajouter(
            format!("fenetre-simple-{}", theme(o)),
            fenetre_simple(&simple_complete(), o),
        );
        ajouter(format!("fenetre-lecture-{}", theme(o)), fenetre_lecture(o));
        ajouter(
            format!("controles-{}", theme(o)),
            capture::dessin(egui::vec2(480.0, 900.0), o, controles),
        );
        ajouter(
            format!("cases-debranchees-{}", theme(o)),
            capture::dessin(egui::vec2(360.0, 200.0), o, cases_debranchees),
        );
    }
    images
}

/// Des choix illustrés avec un écran débranché : dans la rangée de cases égales, dans le plan
/// des écrans, et dans un plan si petit que les cases n'y sont que leur couleur.
fn cases_debranchees(ui: &mut egui::Ui, p: &interface_commune::Palette) {
    use interface_commune::briques::{self, Brique, Case};
    let case = |lettre, rang: usize, place: Option<[i32; 4]>, debranche: bool| Case {
        lettre: Some(lettre),
        couleur: Some(couleurs::palette()[rang]),
        place,
        debranche,
        ..Default::default()
    };
    let choix = |cle: &str, texte: &str, cases: Vec<Case>| Brique::ChoixIllustre {
        cle: cle.into(),
        texte: texte.into(),
        sous_ligne: Some("L'écran de droite est débranché".into()),
        illustration: cases,
        raccourci: None,
        actif: false,
        etiquette_active: None,
    };
    let briques = vec![
        choix(
            "rangee",
            "Rangée",
            vec![case('A', 0, None, false), case('B', 3, None, true)],
        ),
        choix(
            "plan",
            "Plan",
            vec![
                case('A', 0, Some([0, 0, 2560, 1440]), false),
                case('B', 3, Some([2560, 0, 2560, 1440]), true),
                case('B', 3, Some([2560, 1440, 1920, 1200]), false),
            ],
        ),
        choix(
            "petit",
            "Plan, couleurs seules",
            vec![
                case('A', 0, Some([0, 0, 3840, 2160]), false),
                case('B', 3, Some([3840, 0, 800, 480]), true),
            ],
        ),
    ];
    briques::dessiner(ui, p, &briques);
}

fn version_prete(l: &Fictif) -> Simple {
    let r: &dyn Reglages = l;
    Simple::version_prete(
        r.identite(),
        &textes::FRANCAIS,
        "2.5.0",
        &["Une nouveauté inventée.".into(), "Une autre nouveauté.".into()],
    )
}

fn simple_complete() -> Simple {
    Simple {
        titre: "Copier les réglages".into(),
        sous_titre: "Une fenêtre inventée, avec tout ce qu'elle sait montrer.".into(),
        statut: "Copie en cours…".into(),
        ton: Ton::EnCours,
        barre: Some(0.6),
        detail: "Trois fichiers sur cinq.".into(),
        champ: Some(Champ {
            cle: "nom".into(),
            libelle: "Nom de la copie".into(),
            valeur: "Salon".into(),
        }),
        choix: Some(Choix {
            cle: "ou".into(),
            libelle: "Où la ranger".into(),
            valeur: "ici".into(),
            options: vec![
                ("ici".into(), "Ici".into()),
                ("ailleurs".into(), "Ailleurs".into()),
            ],
        }),
        boutons: vec![
            Bouton::new("annuler", "Annuler"),
            Bouton::new("copier", "Copier").accent(),
        ],
        ..Default::default()
    }
}

fn fenetre_simple(s: &Simple, o: Options) -> Result<Image, String> {
    let mut e = EtatSimple::default();
    capture::dessin(egui::vec2(s.largeur_fenetre(), 440.0), o, |ui, p| {
        toutes_faites::dessiner_simple(ui, p, &textes::FRANCAIS, s, &mut e);
    })
}

/// La fenêtre de lecture (bloc I) : un texte inventé de deux paragraphes, le mot lu surligné
/// dans le premier.
fn fenetre_lecture(o: Options) -> Result<Image, String> {
    let mut l = Lecture::de(
        "Une page inventée, lue à voix haute : le mot lu est surligné et la page le suit.\n\n\
         Un clic sur un mot reprend la lecture là, au mot choisi.",
    );
    l.mot = Some(5);
    let mut e = EtatLecture::default();
    capture::dessin(egui::vec2(480.0, 240.0), o, |ui, p| {
        toutes_faites::dessiner_lecture(ui, p, &l, &mut e);
    })
}

/// Une planche des contrôles de `dessin`, de `nuancier`, de `couleurs` et de `seuils`.
fn controles(ui: &mut egui::Ui, p: &interface_commune::Palette) {
    let t = &textes::FRANCAIS;
    dessin::titre_page(
        ui,
        p,
        "Contrôles",
        "Chaque contrôle commun, tel qu'il se dessine.",
    );
    dessin::titre_section(ui, "Boutons");
    ui.horizontal(|ui| {
        dessin::bouton(ui, p, None, "Normal", Genre::Normal);
        dessin::bouton(ui, p, Some('\u{E896}'), "Accent", Genre::Accent);
        dessin::bouton(ui, p, None, "Danger", Genre::Danger);
        dessin::bouton_icone(ui, p, '\u{E713}', "Réglages");
    });
    dessin::titre_section(ui, "Interrupteurs");
    ui.horizontal(|ui| {
        dessin::interrupteur_ecrit(ui, p, t, &mut true);
        dessin::interrupteur_ecrit(ui, p, t, &mut false);
    });
    dessin::titre_section(ui, "Cartes");
    dessin::carte_info(ui, p, '\u{E946}', "Une carte d'information", "Son explication.");
    dessin::carte_chevron(
        ui,
        p,
        '\u{E713}',
        "Une carte qui mène ailleurs",
        "Son explication.",
    );
    dessin::bandeau(ui, p, p.orange, '\u{E7BA}', "Un bandeau inventé.", "Agir");
    dessin::titre_section(ui, "États");
    dessin::etat(ui, p.vert, '\u{E73E}', "Prêt");
    dessin::etat(ui, p.orange, '\u{E7BA}', "Attention");
    dessin::etat(ui, p.rouge, '\u{E783}', "Problème");
    dessin::titre_section(ui, "Couleurs");
    ui.horizontal(|ui| {
        let palette = couleurs::palette();
        couleurs::choisir_couleur(ui, p, &palette, Some(2), "Couleur");
        nuancier::bouton_nuancier(
            ui,
            p,
            Rgb(0x2E, 0xA0, 0x43),
            "Couleur d'un bon niveau",
            &[palette.to_vec()],
        );
    });
    dessin::titre_section(ui, "Seuils");
    let zones = [
        seuils::Zone {
            nom: "Bas",
            couleur: p.rouge,
        },
        seuils::Zone {
            nom: "Moyen",
            couleur: p.orange,
        },
        seuils::Zone {
            nom: "Bon",
            couleur: p.vert,
        },
    ];
    seuils::barre_seuils(
        ui,
        p,
        "seuils",
        &zones,
        &mut [20, 60],
        &seuils::Options::default(),
    );
}

/// L'image de référence lue sur le disque.
fn lire(chemin: &Path) -> Option<Image> {
    let octets = std::fs::read(chemin).ok()?;
    let i = image::load_from_memory_with_format(&octets, image::ImageFormat::Png)
        .ok()?
        .to_rgba8();
    Some(Image {
        largeur: i.width(),
        hauteur: i.height(),
        rvba: i.into_raw(),
    })
}

/// `None` si les deux images sont pareilles (aux arrondis près), sinon l'écart et son image.
fn ecart(attendue: &Image, rendue: &Image) -> Option<(String, Option<Image>)> {
    if (attendue.largeur, attendue.hauteur) != (rendue.largeur, rendue.hauteur) {
        return Some((
            format!(
                "taille {}×{} au lieu de {}×{}",
                rendue.largeur, rendue.hauteur, attendue.largeur, attendue.hauteur
            ),
            None,
        ));
    }
    let mut image = rendue.clone();
    let mut differents = 0;
    for (i, (a, r)) in attendue.rvba.chunks(4).zip(rendue.rvba.chunks(4)).enumerate() {
        let loin = a.iter().zip(r).any(|(x, y)| x.abs_diff(*y) > ECART_PERMIS);
        let px = &mut image.rvba[i * 4..i * 4 + 4];
        if loin {
            differents += 1;
            px.copy_from_slice(&[255, 0, 0, 255]);
        } else {
            // Le reste en gris pâle, pour voir où sont les écarts.
            let g = (u16::from(px[0]) + u16::from(px[1]) + u16::from(px[2])) / 6 + 128;
            px.copy_from_slice(&[g as u8, g as u8, g as u8, 255]);
        }
    }
    let permis = (attendue.rvba.len() / 4) / PIXELS_PERMIS;
    (differents > permis).then(|| {
        (
            format!("{differents} pixels diffèrent (au plus {permis})"),
            Some(image),
        )
    })
}

#[test]
fn le_rendu_reste_celui_des_references() {
    if polices::polices().is_err() {
        eprintln!("Polices Segoe absentes : pas d'images de référence ici.");
        return;
    }
    let dossier = dossier_des_references();
    let images_dossier = dossier.join("images");
    let empreinte = empreinte_des_polices();
    let images = toutes_les_images();

    if std::env::var("REFERENCES").as_deref() == Ok("refaire") {
        let _ = std::fs::remove_dir_all(&images_dossier);
        for (nom, image) in &images {
            image
                .enregistrer(&images_dossier.join(format!("{nom}.png")))
                .expect("écriture");
        }
        std::fs::write(dossier.join("polices.txt"), &empreinte).expect("écriture");
        eprintln!("{} images de référence refaites.", images.len());
        return;
    }

    let attendue = std::fs::read_to_string(dossier.join("polices.txt")).unwrap_or_default();
    if attendue.replace("\r\n", "\n") != empreinte {
        let ici = dossier_des_captures("references-ici");
        for (nom, image) in &images {
            let _ = image.enregistrer(&ici.join(format!("{nom}.png")));
        }
        eprintln!(
            "Polices différentes de celles des références : comparaison sautée (images rendues dans {}).\nAttendu :\n{attendue}Ici :\n{empreinte}",
            ici.display()
        );
        return;
    }

    let ecarts_dossier = dossier_des_captures("references-ecarts");
    let _ = std::fs::remove_dir_all(&ecarts_dossier);
    let mut fautes = Vec::new();
    for (nom, rendue) in &images {
        let Some(reference) = lire(&images_dossier.join(format!("{nom}.png"))) else {
            fautes.push(format!("{nom} : pas d'image de référence"));
            continue;
        };
        if let Some((pourquoi, image_ecart)) = ecart(&reference, rendue) {
            let _ = rendue.enregistrer(&ecarts_dossier.join(format!("{nom}.png")));
            if let Some(i) = image_ecart {
                let _ = i.enregistrer(&ecarts_dossier.join(format!("{nom}-ecarts.png")));
            }
            fautes.push(format!("{nom} : {pourquoi}"));
        }
    }
    // Une image de référence sans surface : une surface a disparu de l'essai.
    let noms: Vec<String> = images.iter().map(|(n, _)| format!("{n}.png")).collect();
    for f in std::fs::read_dir(&images_dossier)
        .expect("dossier des références")
        .flatten()
    {
        let nom = f.file_name().to_string_lossy().into_owned();
        if !noms.contains(&nom) {
            fautes.push(format!("{nom} : plus rendue par l'essai"));
        }
    }
    assert!(
        fautes.is_empty(),
        "Le rendu a changé ({} écarts, images dans {}) :\n{}",
        fautes.len(),
        ecarts_dossier.display(),
        fautes.join("\n")
    );
}

/// Deux rendus de suite donnent la même image : sans cela, les références ne voudraient rien dire.
#[test]
fn le_rendu_est_stable() {
    if polices::polices().is_err() {
        return;
    }
    let fictif = Fictif::new();
    let a = capture::volet(&fictif, clair()).expect("volet");
    let b = capture::volet(&fictif, clair()).expect("volet");
    assert_eq!(a, b);
    let _: &dyn Logiciel = &fictif;
}
