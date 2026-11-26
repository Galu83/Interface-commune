//! Le volet et le petit menu mis en page par egui seul, sans aucune fenêtre (rien ne s'ouvre
//! chez le propriétaire pendant les essais), avec un logiciel inventé, et les captures pour la
//! documentation. Les Paramètres sont dans `tests/sans_fenetre_parametres.rs` ; ce qu'ils
//! partagent, dans `tests/sans_fenetre/commun.rs`.

// Dans un essai, un verrou empoisonné doit faire échouer l'essai.
#![allow(clippy::expect_used)]

// Le logiciel inventé et les images sans fenêtre, partagés avec `sans_fenetre_parametres.rs` :
// chaque fichier d'essais n'en prend qu'une partie, le reste n'y sert pas.
#[allow(dead_code)]
#[path = "sans_fenetre/commun.rs"]
mod commun;

use commun::*;

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use interface_commune::surface::{self, LARGEUR, LARGEUR_MAX, LARGEUR_MENU};
use interface_commune::theme::Surface;
use interface_commune::{
    Communs, Etat, GenreEtat, Identite, Logiciel, Palette, Reglages, Valeur, apparence, polices,
};

/// Le volet et le petit menu tiennent dans leur largeur à 100, 125, 150 et 200 %, et le petit
/// menu dessine ses lignes dans l'ordre des règles.
#[test]
fn volet_et_petit_menu_sans_fenetre() {
    let demo = Demo::new();
    for ppp in [1.0, 1.25, 1.5, 2.0] {
        let mut taille = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 1200.0), ppp, |ui, p| {
            taille = surface::dessiner(ui, p, Surface::Volet, &demo).1;
        });
        assert!((LARGEUR..=LARGEUR_MAX).contains(&taille.x), "{ppp} : {taille:?}");
        if polices::polices().is_ok() {
            for (t, r) in &textes {
                assert!(
                    r.max.x <= taille.x + 0.5,
                    "{ppp} : « {t} » dépasse ({r:?}, {taille:?})"
                );
            }
            // La version est petite, à droite du gros titre, sur la même ligne ; elle n'est pas
            // répétée ailleurs.
            let (_, nom) = textes
                .iter()
                .find(|(t, _)| t == "Bureau du salon")
                .expect("le titre");
            let (_, version) = textes.iter().find(|(t, _)| t == "1.2.3").expect("la version");
            assert!(
                version.min.x > nom.max.x,
                "{ppp} : à droite du titre ({version:?} {nom:?})"
            );
            assert!(version.height() < nom.height(), "{ppp} : plus petite");
            assert!(
                version.center().y > nom.center().y && version.max.y <= nom.max.y + 1.0,
                "{ppp} : posée sur la ligne de base ({version:?} {nom:?})"
            );
            assert!(!textes.iter().any(|(t, _)| t.contains("Démo 1.2.3")), "{ppp}");
            // La roue des Paramètres est en haut à droite, sur la ligne du titre, au centre de
            // sa hauteur ; rien en dessous des réglages (plus de ligne du bas).
            let roue = textes
                .iter()
                .find(|(t, _)| t == interface_commune::icones::REGLAGES.to_string().as_str())
                .map(|(_, r)| *r)
                .expect("la roue");
            assert!(
                roue.center().x > taille.x - 36.0,
                "{ppp} : à droite ({roue:?} {taille:?})"
            );
            assert!(
                (roue.center().y - nom.center().y).abs() < 6.0,
                "{ppp} : sur la ligne du titre ({roue:?} {nom:?})"
            );
            let dernier = textes.iter().map(|(_, r)| r.max.y).fold(0.0, f32::max);
            assert!(
                roue.max.y < dernier - 100.0,
                "{ppp} : pas en bas ({roue:?} {dernier})"
            );
        }
        let mut taille = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 1200.0), ppp, |ui, p| {
            taille = surface::dessiner(ui, p, Surface::Menu, &demo).1;
        });
        assert!(
            (LARGEUR_MENU..=LARGEUR_MAX).contains(&taille.x),
            "{ppp} : {taille:?}"
        );
        if polices::polices().is_ok() {
            let ordre: Vec<&str> = textes
                .iter()
                .map(|(t, _)| t.as_str())
                .filter(|t| t.chars().any(|c| c.is_alphabetic()))
                .collect();
            assert_eq!(
                ordre,
                [
                    "Relire les écrans",
                    "Paramètres",
                    "Rechercher une mise à jour",
                    "Ouvrir le journal",
                    "Quitter"
                ]
            );
        }
    }
}

/// Le petit menu a les mesures de Windows 11 ([`interface_commune::mesures`]) : lignes de 32
/// points séparées de 2, pour tous les logiciels.
#[test]
fn petit_menu_aux_mesures_communes() {
    use interface_commune::mesures::{MENU_ECART, MENU_LIGNE};
    if polices::polices().is_err() {
        return;
    }
    let demo = Demo::new();
    let textes = image(egui::vec2(800.0, 1200.0), 1.0, |ui, p| {
        surface::dessiner(ui, p, Surface::Menu, &demo);
    });
    let y = |t: &str| {
        textes
            .iter()
            .find(|(x, _)| x == t)
            .map(|(_, r)| r.center().y)
            .expect(t)
    };
    let pas = y("Rechercher une mise à jour") - y("Paramètres");
    assert!((pas - MENU_LIGNE - MENU_ECART).abs() < 0.6, "pas : {pas}");
}

/// Les captures pour la documentation : le volet, le petit menu et chaque page des Paramètres,
/// en clair et en sombre, en PNG, sans aucune fenêtre. Le dossier peut être choisi par
/// `CAPTURES_VERS` (pour les regarder à la main) ; sinon un dossier temporaire.
#[test]
fn captures_pour_la_documentation() {
    use interface_commune::capture::{self, Options};
    if polices::polices().is_err() {
        // Sans les polices de Windows, une capture serait sans texte : elle le refuse.
        assert!(capture::volet(&Demo::new(), Options::default()).is_err());
        return;
    }
    let demo = Demo::new();
    let clair = capture::volet(&demo, Options::default()).expect("volet");
    let sombre = capture::volet(
        &demo,
        Options {
            sombre: true,
            ..Options::default()
        },
    )
    .expect("volet sombre");
    assert_eq!((clair.largeur, clair.hauteur), (sombre.largeur, sombre.hauteur));
    assert!(
        (LARGEUR as u32..=LARGEUR_MAX as u32).contains(&clair.largeur),
        "{}",
        clair.largeur
    );
    assert_ne!(clair.rvba, sombre.rvba, "le thème change l'image");
    // Le texte est dessiné : plus d'une couleur dans l'image.
    let couleurs: std::collections::HashSet<[u8; 4]> =
        clair.rvba.as_chunks::<4>().0.iter().copied().collect();
    assert!(couleurs.len() > 8, "{} couleurs", couleurs.len());
    // L'échelle 2 donne deux fois plus de pixels.
    let net = capture::volet(
        &demo,
        Options {
            echelle: 2.0,
            ..Options::default()
        },
    )
    .expect("volet net");
    assert_eq!(net.largeur, clair.largeur * 2);

    let dossier = std::env::var_os("CAPTURES_VERS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!("interface-commune-captures-{}", std::process::id()))
        });
    let faites = capture::toutes(&demo, &dossier, 1.0).expect("captures");
    // Volet + petit menu + Accueil, 2 pages propres, Général, Mise à jour, À propos : 8 × 2 thèmes.
    assert_eq!(faites.len(), 2 * (2 + 6), "{faites:?}");
    for f in &faites {
        let octets = std::fs::read(f).expect("fichier");
        assert_eq!(&octets[..8], b"\x89PNG\r\n\x1a\n", "{f:?}");
    }
    assert!(
        faites
            .iter()
            .any(|f| f.ends_with("parametres-mise-a-jour-sombre.png"))
    );
    if std::env::var_os("CAPTURES_VERS").is_none() {
        let _ = std::fs::remove_dir_all(&dossier);
    }
}

/// Un logiciel aux nombreux appareils, dont les images sont des textures chargées dans le
/// contexte egui du volet (comme celles d'un logiciel de batteries).
struct Appareils {
    identite: Identite,
    appels: std::sync::atomic::AtomicUsize,
    texture: Mutex<Option<egui::TextureHandle>>,
    /// Le nombre d'appareils montrés (il peut changer entre deux images).
    nombre: std::sync::atomic::AtomicUsize,
    /// Un pied fixe (« Rechercher les appareils ») sous la liste.
    pied: AtomicBool,
}

impl Appareils {
    fn avec(nombre: usize) -> Self {
        Appareils {
            identite: identite(),
            appels: Default::default(),
            texture: Mutex::default(),
            nombre: nombre.into(),
            pied: AtomicBool::new(false),
        }
    }
}

impl Reglages for Appareils {
    fn identite(&self) -> &Identite {
        &self.identite
    }

    fn action(&self, _: &str) {}

    fn rechercher_mise_a_jour(&self) {}

    fn communs(&self) -> Communs {
        Communs::default()
    }

    fn changer(&self, _: &str, _: Valeur) {}
}

impl Logiciel for Appareils {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Démo".into(),
            ligne: "30 appareils connectés".into(),
            detail: None,
        }
    }

    fn briques_avec_contexte(&self, ctx: &egui::Context) -> Vec<interface_commune::briques::Brique> {
        use interface_commune::briques::{Brique, ImageAppareil, PlusAppareil};
        self.appels.fetch_add(1, Ordering::AcqRel);
        let mut t = self.texture.lock().expect("verrou");
        let texture = t
            .get_or_insert_with(|| {
                ctx.load_texture(
                    "appareil",
                    egui::ColorImage::new([16, 16], vec![egui::Color32::WHITE; 256]),
                    egui::TextureOptions::LINEAR,
                )
            })
            .id();
        (0..self.nombre.load(Ordering::Acquire))
            .map(|i| Brique::Appareil {
                cle: None,
                glyphe: '\u{E7F4}',
                nom: format!("Appareil {i}"),
                pourcentage: Some(0.5),
                sous_ligne: Some("Vu à l'instant".into()),
                plus: PlusAppareil {
                    image: Some(ImageAppareil {
                        texture,
                        cote: 16.0,
                        fond: None,
                    }),
                    ..Default::default()
                },
            })
            .collect()
    }

    fn briques_du_pied(&self) -> Vec<interface_commune::briques::Brique> {
        if self.pied.load(Ordering::Acquire) {
            vec![interface_commune::briques::Brique::Action {
                cle: "rechercher".into(),
                glyphe: '\u{E72C}',
                texte: PIED.into(),
            }]
        } else {
            Vec::new()
        }
    }

    fn quitter(&self) {}

    fn noter(&self, _: String) {}
}

/// Le texte du pied fixe des essais.
const PIED: &str = "Rechercher les appareils";

/// Le volet reçoit le contexte egui pour ses briques (une fois par image) ; sans hauteur
/// maximale il est aussi haut que son contenu (comme avant) ; avec, il ne la dépasse pas et
/// garde en haut le titre, sa version et la roue dentée, les briques défilant dessous.
#[test]
fn volet_borne_et_briques_avec_contexte() {
    let l = Appareils::avec(30);
    let mut libre = egui::Vec2::ZERO;
    image(egui::vec2(800.0, 4000.0), 1.0, |ui, p| {
        libre = surface::dessiner(ui, p, Surface::Volet, &l).1;
    });
    assert_eq!(l.appels.load(Ordering::Acquire), 3, "une fois par image");
    assert!(
        l.texture.lock().expect("verrou").is_some(),
        "texture chargée dans le contexte"
    );
    assert!(libre.y > 1000.0, "30 appareils : {libre:?}");
    for max in [400.0, 600.0] {
        let mut borne = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 4000.0), 1.5, |ui, p| {
            borne = surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(max)).1;
        });
        assert!(borne.y <= max, "{max} : {borne:?}");
        assert!(borne.y >= max - 60.0, "la place est remplie : {borne:?}");
        assert_eq!(borne.x, libre.x, "même largeur");
        if polices::polices().is_ok() {
            let (_, version) = textes
                .iter()
                .find(|(t, _)| t == "1.2.3")
                .expect("la version reste en haut, avec le titre");
            assert!(version.max.y <= borne.y, "{version:?} {borne:?}");
            let haut = textes
                .iter()
                .filter(|(t, _)| t.starts_with("Appareil "))
                .map(|(_, r)| r.min.y)
                .fold(f32::MAX, f32::min);
            assert!(haut < max, "les premiers appareils sont visibles");
        }
    }
    // Une hauteur qui suffit : rien ne change.
    let mut large = egui::Vec2::ZERO;
    image(egui::vec2(800.0, 4000.0), 1.0, |ui, p| {
        large = surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(libre.y + 100.0)).1;
    });
    assert!((large.y - libre.y).abs() <= 1.0, "{large:?} {libre:?}");
    // Le petit menu n'est jamais borné.
    let mut menu = egui::Vec2::ZERO;
    image(egui::vec2(800.0, 4000.0), 1.0, |ui, p| {
        menu = surface::dessiner_borne(ui, p, Surface::Menu, &l, Some(10.0)).1;
    });
    assert!(menu.y > 10.0);
}

/// Une fenêtre qui suit la taille mesurée, comme celle du volet : à chaque image elle a la
/// taille rendue à l'image d'avant. Rend la taille voulue après `images` images.
fn fenetre_qui_suit(
    ctx: &egui::Context,
    p: &Palette,
    depart: egui::Vec2,
    images: usize,
    dessin: impl FnMut(&mut egui::Ui, &Palette) -> egui::Vec2,
) -> egui::Vec2 {
    fenetre_qui_suit_et_formes(ctx, p, depart, images, dessin).0
}

/// Comme [`fenetre_qui_suit`], avec aussi les formes de la dernière image.
fn fenetre_qui_suit_et_formes(
    ctx: &egui::Context,
    p: &Palette,
    depart: egui::Vec2,
    images: usize,
    mut dessin: impl FnMut(&mut egui::Ui, &Palette) -> egui::Vec2,
) -> (egui::Vec2, Vec<egui::epaint::ClippedShape>) {
    let mut fenetre = depart;
    let mut voulue = depart;
    let mut formes = Vec::new();
    for _ in 0..images {
        let mut entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, fenetre)),
            ..Default::default()
        };
        entree
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(1.0);
        let mut s = ctx.run_ui(entree, |ui| voulue = dessin(ui, p));
        s.textures_delta.clear();
        formes = s.shapes;
        fenetre = voulue;
    }
    (voulue, formes)
}

/// Les textes (avec leur place) et les rectangles dessinés (place, remplissage) d'une image.
fn textes_et_rects(formes: &[egui::epaint::ClippedShape]) -> (Vec<(String, egui::Rect)>, Vec<egui::Rect>) {
    fn parcourir(s: &egui::Shape, textes: &mut Vec<(String, egui::Rect)>, rects: &mut Vec<egui::Rect>) {
        match s {
            egui::Shape::Vec(v) => v.iter().for_each(|s| parcourir(s, textes, rects)),
            egui::Shape::Text(t) => textes.push((
                t.galley.text().to_string(),
                egui::Rect::from_min_size(t.pos, t.galley.size()),
            )),
            egui::Shape::Rect(r) if r.fill.a() > 0 => rects.push(r.rect),
            _ => {}
        }
    }
    let (mut textes, mut rects) = (Vec::new(), Vec::new());
    for c in formes {
        parcourir(&c.shape, &mut textes, &mut rects);
    }
    (textes, rects)
}

/// Une barre de défilement : un rectangle étroit et haut, collé au bord droit du volet.
fn barres(rects: &[egui::Rect], largeur: f32) -> usize {
    rects
        .iter()
        .filter(|r| r.width() <= 12.0 && r.height() >= 20.0 && r.max.x >= largeur - 14.0)
        .count()
}

/// Le pied du volet (« Rechercher les appareils ») reste fixe, visible, en bas : seule la
/// liste défile, avec une barre visible quand elle dépasse ; la fenêtre regrandit jusqu'à la
/// hauteur maximale et le pied suit la liste quand elle est courte.
#[test]
fn volet_borne_pied_fixe_et_barre_visible() {
    if polices::polices().is_err() {
        return;
    }
    let l = Appareils::avec(2);
    l.pied.store(true, Ordering::Release);
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    let a = apparence::Apparence::de(Default::default(), Some(false));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let mesure = |depart: egui::Vec2| {
        let (t, formes) = fenetre_qui_suit_et_formes(&ctx, &p, depart, 8, |ui, p| {
            surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(600.0)).1
        });
        let (textes, rects) = textes_et_rects(&formes);
        let pied = textes
            .iter()
            .find(|(t, _)| t == PIED)
            .map(|(_, r)| *r)
            .expect("le pied est dessiné");
        let dernier = textes
            .iter()
            .filter(|(t, _)| t.starts_with("Appareil "))
            .map(|(_, r)| r.max.y)
            .fold(0.0, f32::max);
        (t, pied, dernier, barres(&rects, t.x))
    };
    // Deux appareils : le pied suit la liste, pas de barre.
    let (petit, pied, dernier, barre) = mesure(egui::vec2(400.0, 800.0));
    assert!(
        pied.min.y > dernier && pied.max.y <= petit.y,
        "{pied:?} {dernier} {petit:?}"
    );
    assert_eq!(barre, 0, "pas de barre quand tout tient {petit:?} {pied:?}");
    // Six appareils (la fenêtre regrandit), puis huit : le pied reste visible dans la fenêtre.
    l.nombre.store(6, Ordering::Release);
    let (six, pied6, dernier6, barre6) = mesure(petit);
    assert!(six.y > petit.y + 100.0 && six.y <= 600.0, "{petit:?} {six:?}");
    assert!(
        pied6.min.y > dernier6 && pied6.max.y <= six.y,
        "{pied6:?} {six:?}"
    );
    assert_eq!(barre6, 0);
    // Trente appareils : la fenêtre est à la borne, le pied est fixe tout en bas, la barre se voit.
    l.nombre.store(30, Ordering::Release);
    let (plein, pied30, _, barre30) = mesure(six);
    assert!(plein.y <= 600.0 && plein.y >= 600.0 - 60.0, "{plein:?}");
    assert!(
        pied30.max.y <= plein.y && pied30.max.y >= plein.y - 30.0,
        "{pied30:?} {plein:?}"
    );
    assert!(barre30 >= 1, "la barre de défilement doit se voir ({barre30})");
    // Huit appareils, comme sur le portable : même chose, jamais coupé.
    l.nombre.store(8, Ordering::Release);
    let (huit, pied8, _, _) = mesure(plein);
    assert!(huit.y <= 600.0 && pied8.max.y <= huit.y, "{pied8:?} {huit:?}");
    // Sans pied, rien ne change : plus de texte du pied.
    l.pied.store(false, Ordering::Release);
    let (sans, formes) = fenetre_qui_suit_et_formes(&ctx, &p, huit, 8, |ui, p| {
        surface::dessiner_borne(ui, p, Surface::Volet, &l, Some(600.0)).1
    });
    assert!(!textes_et_rects(&formes).0.iter().any(|(t, _)| t == PIED));
    assert!(sans.y >= huit.y - 60.0 - 1.0);
}

/// Le contenu grandit après l'ouverture (des appareils déconnectés apparaissent) : la fenêtre,
/// qui avait la taille du petit contenu, doit grandir avec lui, jusqu'à la hauteur maximale
/// (elle ne reste pas bloquée à sa taille d'avant, avec une barre de défilement), et rapetisser
/// quand le contenu rapetisse. Le petit menu et le volet sans borne font de même.
#[test]
fn volet_borne_grandit_avec_son_contenu() {
    let l = Appareils::avec(2);
    let ctx = egui::Context::default();
    ctx.set_fonts(polices::polices_ou_vides());
    let a = apparence::Apparence::de(Default::default(), Some(false));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let mesure = |max: Option<f32>, depart: egui::Vec2| {
        fenetre_qui_suit(&ctx, &p, depart, 6, |ui, p| {
            surface::dessiner_borne(ui, p, Surface::Volet, &l, max).1
        })
    };
    // Deux appareils : le volet tient largement sous la borne.
    let petit = mesure(Some(600.0), egui::vec2(400.0, 800.0));
    assert!(petit.y < 300.0, "{petit:?}");
    // Six appareils (toujours sous la borne) : la fenêtre grandit avec le contenu.
    l.nombre.store(6, Ordering::Release);
    let grand = mesure(Some(600.0), petit);
    let libre = mesure(None, egui::vec2(petit.x, 4000.0));
    assert!(
        grand.y > petit.y + 100.0,
        "n'a pas grandi : {petit:?} -> {grand:?}"
    );
    assert!(
        grand.y <= 600.0 && (grand.y - libre.y).abs() <= 1.0,
        "doit valoir le contenu {libre:?} : {grand:?}"
    );
    // Trente appareils : elle grandit jusqu'à la borne, et pas au-delà.
    l.nombre.store(30, Ordering::Release);
    let plein = mesure(Some(600.0), grand);
    assert!(plein.y <= 600.0 && plein.y >= 600.0 - 60.0, "{plein:?}");
    // Le contenu rapetisse : la fenêtre aussi.
    l.nombre.store(2, Ordering::Release);
    let retour = mesure(Some(600.0), plein);
    assert!((retour.y - petit.y).abs() <= 1.0, "{petit:?} {retour:?}");
    // Sans borne et petit menu : la fenêtre grandit aussi (le contenu déborde, il est mesuré).
    l.nombre.store(6, Ordering::Release);
    let sans = mesure(None, petit);
    assert!((sans.y - libre.y).abs() <= 1.0, "{sans:?} {libre:?}");
    let menu = fenetre_qui_suit(&ctx, &p, egui::vec2(200.0, 40.0), 6, |ui, p| {
        surface::dessiner_borne(ui, p, Surface::Menu, &l, Some(10.0)).1
    });
    assert!(menu.y > 100.0, "{menu:?}");
}

/// Les raccourcis des briques s'affichent en toutes lettres, comme Windows en français :
/// jamais de symbole (⇧, ⌃, ⌘, →), quelle que soit l'écriture donnée par le logiciel.
#[test]
fn raccourcis_en_toutes_lettres() {
    use interface_commune::briques::{self, Brique};
    let liste = vec![
        Brique::Choix {
            cle: "a".into(),
            glyphe: None,
            texte: "Premier".into(),
            coche: false,
            raccourci: Some("ctrl+shift+k".into()),
        },
        Brique::Choix {
            cle: "b".into(),
            glyphe: None,
            texte: "Deuxième".into(),
            coche: false,
            raccourci: Some("⌃⇧→".into()),
        },
        Brique::Raccourcis(vec![
            ("Écran suivant".into(), "Win+Escape".into()),
            ("Écran précédent".into(), "Ctrl+Alt+←".into()),
        ]),
    ];
    let textes = image(egui::vec2(400.0, 400.0), 1.0, |ui, p| {
        briques::dessiner(ui, p, &liste);
    });
    if polices::polices().is_err() {
        return;
    }
    let vus: Vec<&str> = textes.iter().map(|(t, _)| t.as_str()).collect();
    for attendu in [
        "Ctrl+Maj+K",
        "Ctrl+Maj+Flèche droite",
        "Win+Échap",
        "Ctrl+Alt+Flèche gauche",
    ] {
        assert!(vus.contains(&attendu), "« {attendu} » absent de {vus:?}");
    }
    for t in vus {
        assert!(
            !t.contains(['⇧', '⌃', '⌥', '⌘', '→', '←']),
            "symbole dans « {t} »"
        );
    }
}

/// Un nom de logiciel long, avec sa version, ne passe jamais sous la roue des Paramètres : le
/// titre se coupe par « … », la version reste entière, et la roue garde sa place à droite.
#[test]
fn titre_long_ne_chevauche_pas_la_roue() {
    if polices::polices().is_err() {
        return;
    }
    let mut demo = Demo::new();
    demo.identite.version = "12.34.56".into();
    let long = "Un nom de logiciel vraiment beaucoup trop long pour tenir sur une seule ligne du volet";
    *demo.titre.lock().expect("verrou") = Some(long.into());
    let roue_texte = interface_commune::icones::REGLAGES.to_string();
    for ppp in [1.0, 1.25, 1.5, 2.0] {
        let mut taille = egui::Vec2::ZERO;
        let textes = image(egui::vec2(800.0, 1200.0), ppp, |ui, p| {
            taille = surface::dessiner(ui, p, Surface::Volet, &demo).1;
        });
        let roue = textes
            .iter()
            .find(|(t, _)| *t == roue_texte)
            .map(|(_, r)| *r)
            .expect("la roue");
        let titre = textes
            .iter()
            .find(|(t, _)| t.starts_with("Un nom de logiciel"))
            .map(|(t, r)| (t.clone(), *r))
            .expect("le titre");
        let (_, version) = textes.iter().find(|(t, _)| t == "12.34.56").expect("la version");
        // Le bouton fait 36 de large, centré sur l'icône : son bord gauche.
        let bord = roue.center().x - 18.0;
        assert!(
            titre.1.max.x <= bord - 8.0 + 0.5,
            "{ppp} : titre sous la roue ({:?} {roue:?})",
            titre.1
        );
        assert!(
            version.max.x <= bord - 8.0 + 0.5,
            "{ppp} : version sous la roue ({version:?} {roue:?})"
        );
        assert!(
            version.min.x >= titre.1.max.x,
            "{ppp} : version à droite du titre"
        );
        assert!(
            roue.center().x > taille.x - 36.0,
            "{ppp} : la roue reste à droite"
        );
        assert!((LARGEUR..=LARGEUR_MAX).contains(&taille.x), "{ppp} : {taille:?}");
    }
}

/// Les choix illustrés : l'illustration a une largeur fixe, donc le texte et la sous-ligne
/// commencent au même endroit pour 1 ou 4 cases ; l'actif montre la coche et son étiquette ;
/// les pastilles montrent le nom et la mention.
#[test]
fn choix_illustres_alignes() {
    use interface_commune::briques::{self, Brique, Case, Pastille};
    if polices::polices().is_err() {
        return;
    }
    let c = |l| Case {
        lettre: Some(l),
        logo: None,
        couleur: Some(interface_commune::apparence::couleurs_pastilles()[0]),
        ..Default::default()
    };
    let choix = |cle: &str, texte: &str, n: usize, actif: bool| Brique::ChoixIllustre {
        cle: cle.into(),
        texte: texte.into(),
        sous_ligne: Some(format!("sous {texte}")),
        illustration: (0..n).map(|_| c('A')).collect(),
        raccourci: Some("ctrl+alt+shift+1".into()),
        actif,
        etiquette_active: Some("Affiché".into()),
    };
    let liste = vec![
        Brique::Pastilles {
            titre: Some("Ordinateurs".into()),
            pastilles: vec![Pastille {
                cle: None,
                lettre: 'F',
                logo: None,
                couleur: interface_commune::apparence::couleurs_pastilles()[1],
                nom: "Fixe".into(),
                mention: Some("ce PC".into()),
                glyphe: None,
                absent: false,
            }],
        },
        choix("un", "Un", 1, true),
        choix("quatre", "Quatre", 4, false),
    ];
    let textes = image(egui::vec2(400.0, 400.0), 1.0, |ui, p| {
        briques::dessiner(ui, p, &liste);
    });
    let x = |t: &str| {
        textes
            .iter()
            .find(|(s, _)| s == t)
            .unwrap_or_else(|| panic!("« {t} » absent de {textes:?}"))
            .1
            .min
            .x
    };
    assert!((x("Un") - x("Quatre")).abs() < 0.5, "texte décalé");
    assert!(
        (x("sous Un") - x("sous Quatre")).abs() < 0.5,
        "sous-ligne décalée"
    );
    assert!((x("Un") - x("sous Un")).abs() < 0.5);
    for attendu in ["Affiché", "Ctrl+Alt+Maj+1", "Fixe", "· ce PC", "Ordinateurs"] {
        x(attendu);
    }
}
