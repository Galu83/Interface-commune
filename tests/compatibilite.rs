//! Les filets de compatibilité (C-114) : ce que chacun des quatre logiciels prend de la
//! bibliothèque, relevé dans leur code le 24 nov. 2026 (KVMSoft, PowerPeek, Vocalis, Chat PDC),
//! complété le 25 nov. avec les usages de KVMSoft et de PowerPeek sur la rev e077572.
//!
//! Chaque module reprend :
//! - tous les chemins que le logiciel importe ou écrit en entier (`use … as _` : le chemin doit
//!   exister) ;
//! - la signature de chaque fonction libre qu'il appelle (un pointeur de fonction typé) ;
//! - les méthodes des traits qu'il remplace, avec leur signature exacte.
//!
//! Si un de ces essais ne compile plus, le logiciel ne compilera plus non plus en changeant
//! seulement son `rev` : le changement d'API doit alors être écrit dans `MIGRATION.md` et
//! dans le plan de la refonte, avec la façon de mettre le logiciel à jour.
//!
//! Le relevé se refait à chaque nouvel import dans un logiciel ; ces modules ne vérifient que
//! les noms et les types, pas le comportement (les autres essais s'en chargent).
#![cfg(windows)]
#![allow(unused_imports)]
// Les signatures vérifiées sont recopiées telles quelles : longues, sans alias exprès.
#![allow(clippy::type_complexity)]

use interface_commune::Identite;

/// Une identité inventée, la même pour les quatre.
fn identite() -> Identite {
    Identite {
        nom: "Imité".into(),
        version: "0.0.1".into(),
        licence: "MIT".into(),
        description: "Un logiciel inventé pour les essais de compatibilité.".into(),
        icone: |_, _| Vec::new(),
        logo: None,
    }
}

/// Ce que KVMSoft prend de la bibliothèque.
mod kvmsoft {
    use interface_commune as ic;
    #[rustfmt::skip]
    use interface_commune::{Communs as _, Credit as _, Etat as _, EtatMiseAJour as _, GenreEtat as _, Identite as _, Instantane as _, Logiciel as _, Page as _, Palette as _, Reglages as _, Rgb as _, Valeur as _, autres_fenetres::Ouverture as _, briques as _, briques::Brique as _, briques::Case as _, briques::Pastille as _, capture as _, capture::Options as _, cles as _, couleurs as _, dessin as _, dessin::Genre as _, dessin::carte as _, dessin::carte_large as _, horloge as _, horloge::AuClic as _, horloge::Horloge as _, horloge::MessageWindows as _, horloge::Niveau as _, horloge::Poste as _, horloge::Reveil as _, menu::Action as _, menu_choix as _, menu_choix::Ligne as _, menu_choix::MenuChoix as _, menu_choix::Resultat as _, mesures::ILLUSTRE_CASES_MAX as _, mesures::PARAMETRES_TAILLE as _, parametres::Contexte as _, parametres::Parametres as _, parametres::modele as _, parametres::modele::A_PROPOS as _, parametres::modele::GENERAL as _, parametres::modele::MISE_A_JOUR as _, parametres::modele::PAGES_POUR_RECHERCHE as _, pastilles as _, pastilles::Marque as _, pastilles::marque as _, place::Place as _, sonde as _, sonde::Delais as _, sonde::Inscription as _, sonde::battement as _, sonde::brancher_le_journal as _, sonde::etape as _, sonde::inscrire as _, sonde::regler as _, textes::FRANCAIS as _, theme as _, theme::ARRONDI_CARTE as _, theme::ARRONDI_CONTROLE as _, theme::Rgb as _, theme::TAILLE_EXPLICATION as _, theme::TAILLE_PETIT as _, theme::TAILLE_SECTION as _, theme::TAILLE_TEXTE as _, theme::TAILLE_TITRE as _, theme::TAILLE_TITRE_VOLET as _, toutes_faites as _, toutes_faites::Bouton as _, toutes_faites::Champ as _, toutes_faites::EtatSimple as _, toutes_faites::FenetreSimple as _, toutes_faites::PLUS_TARD as _, toutes_faites::Reponse as _, toutes_faites::Simple as _, toutes_faites::Ton as _, toutes_faites::ouvrir_simple as _, zone::FinDeSession as _};

    /// Un logiciel inventé qui remplace, dans les traits, les mêmes méthodes que KVMSoft.
    struct Imite(ic::Identite);

    impl ic::Reglages for Imite {
        fn action(&self, _cle: &str) {}
        fn changer(&self, _cle: &str, _valeur: ic::Valeur) {}
        fn communs(&self) -> ic::Communs {
            ic::Communs::default()
        }
        fn dessiner_general(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_mise_a_jour(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_page(&self, _cle: &str, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_pied(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn identite(&self) -> &ic::Identite {
            &self.0
        }
        fn instantane(&self) -> Option<ic::Instantane> {
            None
        }
        fn pages(&self) -> Vec<ic::Page> {
            Vec::new()
        }
        fn peut_fermer(&self, _imposee: bool) -> bool {
            true
        }
        fn place_parametres(&self) -> Option<ic::place::Place> {
            None
        }
        fn rechercher_mise_a_jour(&self) {}
        fn remettre(&self, _instantane: &ic::Instantane) {}
        fn retablir_explication(&self) -> Option<String> {
            None
        }
        fn retenir_place(&self, _place: ic::place::Place) {}
    }

    impl ic::Logiciel for Imite {
        fn actions_menu(&self) -> Vec<ic::menu::Action> {
            Vec::new()
        }
        fn briques(&self) -> Vec<ic::briques::Brique> {
            Vec::new()
        }
        fn etat(&self) -> ic::Etat {
            etat()
        }
        fn icone_etat(&self, _taille: u32, _barre_sombre: bool) -> Option<Vec<u8>> {
            None
        }
        fn noter(&self, _ligne: String) {}
        fn quitter(&self) {}
    }

    fn etat() -> ic::Etat {
        ic::Etat {
            genre: ic::GenreEtat::Pret,
            titre: String::new(),
            ligne: String::new(),
            detail: None,
        }
    }

    #[test]
    fn les_noms_et_les_signatures_restent() {
        let _ = ic::Communs::default;
        let _ = ic::Page::new;
        let _ = ic::Palette::de;
        let _: fn() -> ic::theme::ChoixWindows = ic::apparence::choix_windows;
        let _: fn(ic::Rgb) -> egui::Color32 = ic::apparence::couleur;
        let _: fn() -> [ic::Rgb; 10] = ic::apparence::couleurs_pastilles;
        let _ = ic::cles::NOTIFICATIONS_ESSAYER;
        let _: fn(&mut egui::Ui, &ic::Palette, &[ic::Rgb], Option<usize>, &str) -> Option<usize> =
            ic::couleurs::choisir_couleur;
        let _: fn() -> [ic::Rgb; 10] = ic::couleurs::palette;
        let _: fn(&mut egui::Ui, &ic::Palette, &ic::briques::Pastille) -> egui::Response =
            ic::couleurs::pastille_de_carte;
        let _ = ic::dessin::Genre::Accent;
        let _: fn(&mut egui::Ui, &ic::Palette, Option<char>, &str, ic::dessin::Genre) -> egui::Response =
            ic::dessin::bouton;
        let _: fn(&mut egui::Ui, &ic::Palette, char, &str) -> egui::Response = ic::dessin::bouton_icone;
        let _: fn(&ic::Palette) -> egui::Frame = ic::dessin::cadre_carte;
        let _: fn(&mut egui::Ui, &ic::Palette, char, &str, &str) -> bool = ic::dessin::carte_chevron;
        let _: fn(&mut egui::Ui, &ic::Palette, char, &str, &str) = ic::dessin::carte_info;
        let _: fn(&mut egui::Ui, ic::Rgb, char, &str) = ic::dessin::etat;
        let _: fn(&egui::Ui, &ic::Palette, &egui::Response, u8) = ic::dessin::focus;
        let _: fn(&egui::Ui, egui::Pos2, char, f32, ic::Rgb) = ic::dessin::icone;
        let _: fn(&str) -> String = ic::dessin::insecables;
        let _: fn(&mut egui::Ui, &ic::Palette, &ic::Textes, &mut bool) -> egui::Response =
            ic::dessin::interrupteur_ecrit;
        let _: fn(&mut egui::Ui, &str) = ic::dessin::titre_section;
        let _ = ic::horloge::Horloge::new;
        let _: fn(&str, &ic::horloge::Reveil) -> Result<ic::horloge::Unique, bool> =
            ic::horloge::instance_unique_avec;
        let _: fn() = ic::horloge::reglage_dpi;
        let _ = ic::parametres::Parametres::new;
        let _: fn(&dyn ic::Reglages, Option<&std::path::Path>, &str) -> Option<std::path::PathBuf> =
            ic::parametres::fenetre::choisir_dossier;
        let _: fn(&dyn ic::Reglages, bool) -> egui::IconData = ic::parametres::fenetre::icone_fenetre;
        let _: fn(&[ic::Page], &str) -> Vec<usize> = ic::parametres::modele::chercher;
        let _: fn(&ic::Textes, bool, &[ic::Page]) -> Vec<ic::Page> = ic::parametres::modele::navigation;
        let _: fn(&[ic::Page], Option<&str>) -> usize = ic::parametres::modele::rang_de_page;
        let _: fn(&str, &str, &[&str]) -> bool = ic::parametres::modele::section_trouvee;
        let _: fn(ic::Rgb) -> ic::Rgb = ic::pastilles::couleur_lettre;
        let _: fn(ic::place::Place) -> Option<ic::place::Place> = ic::place::place_valable;
        let _: fn(char) -> bool = ic::polices::icone_disponible;
        let _: fn(f32) -> egui::FontId = ic::polices::semibold;
        let _: fn(&'static str) -> ic::sonde::Imbriquee = ic::sonde::imbriquee;
        let _: fn(&std::path::Path) -> bool = ic::systeme::disque_local;
        let _: fn(
            &mut egui::Ui,
            &ic::Palette,
            &ic::Textes,
            &ic::toutes_faites::Simple,
            &mut ic::toutes_faites::EtatSimple,
        ) -> Option<ic::toutes_faites::Clic> = ic::toutes_faites::dessiner_simple;
        let _: fn(&[char]) -> Result<Vec<char>, String> = ic::verification::logos_absents;
        // Ajoutés le 25 nov. 2026 (KVMSoft sur la rev e077572).
        let _: fn(Option<char>, Option<char>, fn(char) -> bool) -> Option<ic::pastilles::Marque> =
            ic::pastilles::marque;
        let _: fn(
            &ic::autres_fenetres::AutresFenetres,
            &str,
            ic::autres_fenetres::Ouverture,
            &ic::Textes,
            ic::toutes_faites::Simple,
            fn(ic::toutes_faites::Reponse),
        ) -> Option<ic::toutes_faites::FenetreSimple> = ic::toutes_faites::ouvrir_simple;
        let _: fn(&str, [f32; 2]) -> ic::autres_fenetres::Ouverture = ic::autres_fenetres::Ouverture::new;
        let _: fn(u32, usize, isize) -> Option<ic::zone::FinDeSession> = ic::zone::FinDeSession::de;
        // Les tests de KVMSoft qui n'attendent jamais : une horloge à soi, une sonde réglée.
        let _: fn(&str) -> ic::horloge::Horloge = ic::horloge::Horloge::new;
        let _: fn(&'static str, Option<Box<dyn Fn() + Send + Sync>>) -> ic::sonde::Inscription =
            ic::sonde::inscrire;
        let _: fn() = ic::sonde::battement;
        let _: fn(&'static str) = ic::sonde::etape;
        let _: fn(fn(String)) = ic::sonde::brancher_le_journal;
        let _: fn(ic::sonde::Delais) = ic::sonde::regler;
        let _ = ic::sonde::Delais {
            pas: std::time::Duration::ZERO,
            blocage: std::time::Duration::ZERO,
        };
        // Les mesures du thème que KVMSoft reprend pour ses propres dessins.
        let _: [u8; 2] = [ic::theme::ARRONDI_CARTE, ic::theme::ARRONDI_CONTROLE];
        let _: [f32; 6] = [
            ic::theme::TAILLE_TITRE,
            ic::theme::TAILLE_TITRE_VOLET,
            ic::theme::TAILLE_SECTION,
            ic::theme::TAILLE_TEXTE,
            ic::theme::TAILLE_PETIT,
            ic::theme::TAILLE_EXPLICATION,
        ];
        let logiciel = Imite(super::identite());
        let _: &dyn ic::Logiciel = &logiciel;
    }

    /// Promesse de format : KVMSoft envoie à un autre PC le rang (0 à 9) de la couleur d'un
    /// appareil, pas la couleur. Le même rang doit donner la même couleur partout, quelle que
    /// soit la version de la bibliothèque de chaque côté : ces dix couleurs ne changent pas
    /// d'ordre ni de valeur. Une couleur en plus s'ajoute derrière (rang 10, 11…).
    #[test]
    fn les_rangs_des_couleurs_restent() {
        const RANGS: [u32; 10] = [
            0x7B57C9, // 0 violet
            0xB03A9C, // 1 magenta
            0x1D7F86, // 2 sarcelle foncé
            0x94604F, // 3 brun
            0x667A94, // 4 ardoise
            0x857A2A, // 5 olive
            0x3865F3, // 6 indigo
            0xC0416B, // 7 framboise
            0x8E679E, // 8 prune
            0x7C6D61, // 9 taupe
        ];
        let palette = ic::couleurs::palette();
        for (rang, (couleur, attendue)) in palette.iter().zip(RANGS).enumerate() {
            let lue = u32::from(couleur.0) << 16 | u32::from(couleur.1) << 8 | u32::from(couleur.2);
            assert_eq!(
                lue, attendue,
                "le rang {rang} a changé de couleur : promesse de format rompue"
            );
        }
        assert_eq!(
            palette,
            ic::apparence::couleurs_pastilles(),
            "une seule liste de couleurs"
        );
    }
}

/// Ce que PowerPeek prend de la bibliothèque.
mod powerpeek {
    use interface_commune as ic;
    #[rustfmt::skip]
    use interface_commune::{Communs as _, Credit as _, Etat as _, EtatMiseAJour as _, GenreEtat as _, Identite as _, Logiciel as _, Page as _, Palette as _, Reglages as _, Rgb as _, Textes as _, Valeur as _, apparence::Apparence as _, autres_fenetres::AutresFenetres as _, autres_fenetres::Ouverture as _, briques as _, briques::Brique as _, briques::ImageAppareil as _, briques::PlusAppareil as _, capture as _, capture::Options as _, cles as _, dessin as _, echelle as _, echelle::suivre_windows as _, fenetres::Demande as _, fenetres::Fin as _, fenetres::Liens as _, fenetres::Ouvert as _, fenetres::Supervise as _, fenetres::superviser_les_fenetres as _, horloge::Niveau as _, icones::INSTALLER as _, menu::Action as _, parametres as _, parametres::Contexte as _, parametres::Parametres as _, parametres::fenetre::PageVoulue as _, parametres::fenetre::choisir_dossier as _, parametres::fenetre::ouverture as _, parametres::fenetre::ouvrir_dans_boucle as _, parametres::modele as _, place::Place as _, place::PlacementVolet as _, polices::polices as _, seuils as _, sonde as _, sonde::Delais as _, sonde::Inscription as _, sonde::battement as _, sonde::brancher_le_journal as _, sonde::etape as _, sonde::inscrire as _, sonde::regler as _, seuils::Options as _, seuils::ReponseSeuils as _, seuils::Zone as _, textes as _, theme::ChoixWindows as _, theme::Surface as _, zone as _, zone::Bouton as _, zone::Evenement as _, zone::FenetresCommunes as _, zone::Image as _, zone::LigneMenu as _, zone::Message as _, zone::SuiteDuGeste as _, zone::Zone as _};

    /// Un logiciel inventé qui remplace, dans les traits, les mêmes méthodes que PowerPeek.
    struct Imite(ic::Identite);

    impl ic::Reglages for Imite {
        fn action(&self, _cle: &str) {}
        fn changer(&self, _cle: &str, _valeur: ic::Valeur) {}
        fn communs(&self) -> ic::Communs {
            ic::Communs::default()
        }
        fn dessiner_a_propos(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_mise_a_jour(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_page(&self, _cle: &str, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_pied(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn identite(&self) -> &ic::Identite {
            &self.0
        }
        fn pages(&self) -> Vec<ic::Page> {
            Vec::new()
        }
        fn peut_fermer(&self, _imposee: bool) -> bool {
            true
        }
        fn place_parametres(&self) -> Option<ic::place::Place> {
            None
        }
        fn rechercher_mise_a_jour(&self) {}
        fn retablir_explication(&self) -> Option<String> {
            None
        }
        fn retenir_place(&self, _place: ic::place::Place) {}
        fn textes(&self) -> &ic::Textes {
            &ic::textes::FRANCAIS
        }
    }

    impl ic::Logiciel for Imite {
        fn actions_menu(&self) -> Vec<ic::menu::Action> {
            Vec::new()
        }
        fn briques_avec_contexte(&self, _ctx: &egui::Context) -> Vec<ic::briques::Brique> {
            Vec::new()
        }
        fn briques_du_pied(&self) -> Vec<ic::briques::Brique> {
            Vec::new()
        }
        fn etat(&self) -> ic::Etat {
            etat()
        }
        fn noter(&self, _ligne: String) {}
        fn placement_volet(&self) -> ic::place::PlacementVolet {
            ic::place::PlacementVolet::default()
        }
        fn quitter(&self) {}
    }

    fn etat() -> ic::Etat {
        ic::Etat {
            genre: ic::GenreEtat::Pret,
            titre: String::new(),
            ligne: String::new(),
            detail: None,
        }
    }

    #[test]
    fn les_noms_et_les_signatures_restent() {
        let _ = <Imite as ic::Logiciel>::placement_volet;
        let _ = ic::Palette::de;
        let _ = <Imite as ic::Reglages>::action;
        let _ = <Imite as ic::Reglages>::peut_fermer;
        let _ = ic::apparence::Apparence::new;
        let _: fn(ic::Rgb) -> egui::Color32 = ic::apparence::couleur;
        let _: fn(
            &ic::fenetres::Liens,
            &ic::fenetres::Demandes,
            ic::theme::Surface,
            [i32; 4],
            Option<[i32; 2]>,
            std::time::Instant,
        ) -> ic::fenetres::Aiguillage = ic::fenetres::envoyer_le_clic;
        let _: fn(&ic::Textes, &[ic::menu::Action], &[ic::menu::Action]) -> Vec<Option<ic::menu::Ligne>> =
            ic::menu::lignes;
        let _: fn(&mut egui::Ui, &ic::Palette, ic::Rgb, &str, &[Vec<ic::Rgb>]) -> Option<ic::Rgb> =
            ic::nuancier::bouton_nuancier;
        let _: fn() -> Result<egui::FontDefinitions, String> = ic::polices::polices;
        let _: fn() -> egui::FontDefinitions = ic::polices::polices_ou_vides;
        let _: fn(f32) -> egui::FontId = ic::polices::semibold;
        let _: fn(
            &mut egui::Ui,
            &ic::Palette,
            &str,
            &[ic::seuils::Zone<'_>],
            &mut [u8],
            &ic::seuils::Options<'_>,
        ) -> ic::seuils::ReponseSeuils = ic::seuils::barre_seuils;
        let _: fn(
            &mut egui::Ui,
            &ic::Palette,
            ic::theme::Surface,
            &dyn ic::Logiciel,
            Option<f32>,
        ) -> (Option<ic::surface::Choisi>, egui::Vec2) = ic::surface::dessiner_borne;
        let _ = ic::theme::Surface::Menu;
        let _ = ic::theme::Surface::Volet;
        let _: fn(&str, &str, &str) -> u128 = ic::zone::guid_stable;
        // Ajoutés le 25 nov. 2026 (PowerPeek en refonte, sur la rev e077572).
        let _ = ic::Page::new;
        let _: ic::fenetres::Ouvert = Default::default();
        let _ = ic::fenetres::Ouvert::Parametres;
        let _ = [
            ic::zone::Bouton::Gauche,
            ic::zone::Bouton::Droit,
            ic::zone::Bouton::Milieu,
        ];
        let _: fn(&dyn ic::Reglages, bool) -> ic::autres_fenetres::Ouverture =
            ic::parametres::fenetre::ouverture;
        let _: fn(&dyn ic::Reglages, Option<&std::path::Path>, &str) -> Option<std::path::PathBuf> =
            ic::parametres::fenetre::choisir_dossier;
        let _: fn(&dyn ic::Reglages) -> String = ic::parametres::Parametres::titre_fenetre;
        // La boucle du logiciel : les fenêtres de la bibliothèque, ses Paramètres, son échelle.
        let _: fn(&ic::autres_fenetres::AutresFenetres, Option<egui::Context>) =
            ic::autres_fenetres::AutresFenetres::lier;
        let _: fn(&ic::autres_fenetres::AutresFenetres, &egui::Context, &ic::Palette) =
            ic::autres_fenetres::AutresFenetres::montrer;
        let _: fn(
            &ic::autres_fenetres::AutresFenetres,
            &std::sync::Arc<dyn ic::Reglages>,
            &ic::parametres::fenetre::PageVoulue,
            Option<String>,
        ) = ic::parametres::fenetre::ouvrir_dans_boucle;
        let _: ic::parametres::fenetre::PageVoulue = std::sync::Arc::new(std::sync::Mutex::new(None));
        let _: fn(&mut ic::parametres::Parametres, &mut egui::Ui, &ic::Palette, &dyn ic::Reglages) =
            ic::parametres::Parametres::dessiner;
        let _: fn(isize) -> bool = ic::echelle::suivre_windows;
        // La supervision de sa boucle : le trait (ses quatre méthodes) et la fonction.
        let _: fn(&mut FaussesFenetres) -> Result<(), String> =
            <FaussesFenetres as ic::fenetres::Supervise>::boucle;
        let _: fn(&mut FaussesFenetres, &ic::fenetres::Evenement) =
            <FaussesFenetres as ic::fenetres::Supervise>::perdues;
        let _: fn(&mut FaussesFenetres) -> bool =
            <FaussesFenetres as ic::fenetres::Supervise>::attendre_un_clic;
        let _: fn(&FaussesFenetres) -> bool = <FaussesFenetres as ic::fenetres::Supervise>::quitter;
        let _: fn(&mut FaussesFenetres, fn() -> std::time::Instant) -> ic::fenetres::Fin =
            ic::fenetres::superviser_les_fenetres;
        let _ = [
            ic::fenetres::Fin::Quitter,
            ic::fenetres::Fin::Impossible(String::new()),
        ];
        let _: fn(&ic::fenetres::Evenement) -> String = ic::fenetres::Evenement::phrase;
        // La sonde des fils, branchée sur le journal de PowerPeek.
        let _: fn(&'static str, Option<Box<dyn Fn() + Send + Sync>>) -> ic::sonde::Inscription =
            ic::sonde::inscrire;
        let _: fn() = ic::sonde::battement;
        let _: fn(&'static str) = ic::sonde::etape;
        let _: fn(fn(String)) = ic::sonde::brancher_le_journal;
        let _: fn(ic::sonde::Delais) = ic::sonde::regler;
        let logiciel = Imite(super::identite());
        let _: &dyn ic::Logiciel = &logiciel;
    }

    /// PowerPeek prend egui en direct (ses pages propres, ses dessins) : la version d'egui fait
    /// partie de l'API de la bibliothèque. Un changement de version d'egui (ou d'eframe) se
    /// traite comme un changement d'API : écrit dans `MIGRATION.md`, avec la migration de chaque
    /// logiciel, et ce garde mis à jour en même temps.
    #[test]
    fn la_version_d_egui_reste() {
        const ATTENDUE: &str = "0.37.";
        let verrou = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"));
        let lignes: Vec<&str> = verrou.lines().collect();
        for nom in ["egui", "eframe"] {
            let entete = format!("name = \"{nom}\"");
            let versions: Vec<&str> = lignes
                .windows(2)
                .filter(|l| l[0] == entete)
                .filter_map(|l| l[1].strip_prefix("version = \""))
                .map(|v| v.trim_end_matches('"'))
                .collect();
            assert!(!versions.is_empty(), "{nom} introuvable dans Cargo.lock");
            for v in versions {
                assert!(
                    v.starts_with(ATTENDUE),
                    "{nom} {v} au lieu de {ATTENDUE}x : changement d'egui = changement d'API, à migrer"
                );
            }
        }
    }

    // -----------------------------------------------------------------------------------
    // La boucle du logiciel (façon 2 du bloc J), vue du dehors et sans fenêtre : PowerPeek
    // tient sa propre boucle eframe, la fait superviser par la bibliothèque et y ouvre les
    // Paramètres. Ici, la boucle est jouée d'avance et l'horloge est inventée.

    /// Une boucle de fenêtres jouée d'avance : chaque tour rend le résultat suivant de
    /// `tours` ; quand il n'y en a plus, le logiciel se ferme. `clics` : les réponses
    /// d'`attendre_un_clic` (faux quand il n'y en a plus).
    struct FaussesFenetres {
        tours: std::collections::VecDeque<Result<(), String>>,
        clics: std::collections::VecDeque<bool>,
        recues: Vec<ic::fenetres::Evenement>,
        fin: bool,
    }

    impl FaussesFenetres {
        fn new(tours: Vec<Result<(), String>>, clics: Vec<bool>) -> Self {
            Self {
                tours: tours.into(),
                clics: clics.into(),
                recues: Vec::new(),
                fin: false,
            }
        }

        fn reprises(&self) -> Vec<(usize, ic::fenetres::Reprise)> {
            self.recues.iter().map(|e| (e.nombre, e.reprise)).collect()
        }
    }

    impl ic::fenetres::Supervise for FaussesFenetres {
        fn boucle(&mut self) -> Result<(), String> {
            match self.tours.pop_front() {
                Some(r) => r,
                None => {
                    self.fin = true;
                    Ok(())
                }
            }
        }
        fn perdues(&mut self, e: &ic::fenetres::Evenement) {
            assert!(!e.phrase().is_empty());
            self.recues.push(e.clone());
        }
        fn attendre_un_clic(&mut self) -> bool {
            self.clics.pop_front().unwrap_or(false)
        }
        fn quitter(&self) -> bool {
            self.fin
        }
    }

    /// C-50 et C-52 vus du dehors : jusqu'à 3 pertes en une minute, les fenêtres sont
    /// recréées tout de suite ; la 4e attend le clic suivant ; pas de clic, le logiciel se
    /// ferme. Une boucle jamais créée arrête tout ; un échec passager ne compte pas.
    #[test]
    fn la_supervision_vue_du_dehors() {
        use ic::fenetres::{Fin, Perte, Reprise::*, superviser_les_fenetres};
        let t0 = std::time::Instant::now();

        // Une rafale : l'horloge ne bouge pas.
        let mut f = FaussesFenetres::new(vec![Ok(()); 4], vec![]);
        assert_eq!(superviser_les_fenetres(&mut f, || t0), Fin::Quitter);
        assert_eq!(
            f.reprises(),
            [
                (1, Maintenant),
                (2, Maintenant),
                (3, Maintenant),
                (4, AuClicSuivant)
            ]
        );
        assert!(f.recues.iter().all(|e| e.perte == Perte::Fermee));

        // Les mêmes pertes, une par minute et une seconde : jamais de rafale.
        let heure = std::cell::Cell::new(t0);
        let horloge = || {
            let t = heure.get();
            heure.set(t + std::time::Duration::from_secs(61));
            t
        };
        let mut f = FaussesFenetres::new(vec![Ok(()); 5], vec![]);
        assert_eq!(superviser_les_fenetres(&mut f, horloge), Fin::Quitter);
        assert_eq!(f.reprises(), [(1, Maintenant); 5]);

        // Rien n'a jamais marché : le menu de Windows pour de bon, sans perte annoncée.
        let mut f = FaussesFenetres::new(vec![Err("pas d'OpenGL".into())], vec![]);
        assert_eq!(
            superviser_les_fenetres(&mut f, || t0),
            Fin::Impossible("pas d'OpenGL".into())
        );
        assert!(f.recues.is_empty());

        // Un échec passager après un premier tour : nouvel essai au clic suivant, sans
        // compter une perte de plus ; puis le logiciel se ferme pendant un tour.
        let mut f = FaussesFenetres::new(vec![Ok(()), Err("écrans en veille".into())], vec![true]);
        assert_eq!(superviser_les_fenetres(&mut f, || t0), Fin::Quitter);
        assert_eq!(f.reprises(), [(1, Maintenant), (1, AuClicSuivant)]);
        assert_eq!(f.recues[1].perte, Perte::Echec("écrans en veille".into()));
    }

    /// C-47 vu du dehors : les Paramètres s'ouvrent parmi les fenêtres de la boucle du
    /// logiciel, une seule fois ; redemandés, ils changent de page. Aucune boucle egui ne
    /// tourne : rien n'est dessiné, aucune fenêtre n'apparaît.
    #[test]
    fn les_parametres_s_ouvrent_dans_la_boucle_sans_fenetre() {
        use ic::parametres::fenetre::{PageVoulue, ouvrir_dans_boucle};
        let autres = ic::autres_fenetres::AutresFenetres::default();
        let reglages: std::sync::Arc<dyn ic::Reglages> = std::sync::Arc::new(Imite(super::identite()));
        let voulue = PageVoulue::default();

        ouvrir_dans_boucle(&autres, &reglages, &voulue, Some("general".into()));
        assert_eq!(autres.cles(), [ic::parametres::CLE_FENETRE]);
        assert!(autres.ouverte(ic::parametres::CLE_FENETRE));
        assert_eq!(*voulue.lock().unwrap(), None);

        ouvrir_dans_boucle(&autres, &reglages, &voulue, Some("apparence".into()));
        assert_eq!(autres.cles().len(), 1);
        assert_eq!(voulue.lock().unwrap().as_deref(), Some("apparence"));
    }
}

/// Ce que Vocalis prend de la bibliothèque.
mod vocalis {
    use interface_commune as ic;
    #[rustfmt::skip]
    use interface_commune::{Communs as _, Etat as _, EtatMiseAJour as _, GenreEtat as _, Identite as _, Logiciel as _, Page as _, Palette as _, Reglages as _, Rgb as _, Valeur as _, apparence as _, apparence::Apparence as _, autres_fenetres as _, autres_fenetres::AutresFenetres as _, autres_fenetres::Cadre as _, autres_fenetres::Contenu as _, autres_fenetres::IMAGES_CACHEES as _, autres_fenetres::Ouverture as _, briques::Brique as _, capture as _, capture::Options as _, cles as _, dessin as _, horloge as _, horloge::AuClic as _, horloge::Horloge as _, horloge::MessageWindows as _, horloge::Niveau as _, horloge::Poste as _, menu as _, parametres::Contexte as _, parametres::Parametres as _, parametres::modele::DUREE_ANNULER as _, parametres::modele::Instantane as _, parametres::modele::page_demandee as _, place::Place as _, polices as _, surface as _, systeme as _, textes::FRANCAIS as _, theme::Surface as _, verification as _, zone::FinDeSession as _};

    /// Un logiciel inventé qui remplace, dans les traits, les mêmes méthodes que Vocalis.
    struct Imite(ic::Identite);

    impl ic::Reglages for Imite {
        fn accueil(&self) -> bool {
            false
        }
        fn action(&self, _cle: &str) {}
        fn changer(&self, _cle: &str, _valeur: ic::Valeur) {}
        fn communs(&self) -> ic::Communs {
            ic::Communs::default()
        }
        fn dessiner_general(&self, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn dessiner_page(&self, _cle: &str, _ui: &mut egui::Ui, _page: &mut ic::parametres::Contexte) {}
        fn identite(&self) -> &ic::Identite {
            &self.0
        }
        fn instantane(&self) -> Option<ic::Instantane> {
            None
        }
        fn pages(&self) -> Vec<ic::Page> {
            Vec::new()
        }
        fn place_parametres(&self) -> Option<ic::place::Place> {
            None
        }
        fn rechercher_mise_a_jour(&self) {}
        fn remettre(&self, _instantane: &ic::Instantane) {}
        fn retenir_place(&self, _place: ic::place::Place) {}
    }

    impl ic::Logiciel for Imite {
        fn actions_menu(&self) -> Vec<ic::menu::Action> {
            Vec::new()
        }
        fn briques(&self) -> Vec<ic::briques::Brique> {
            Vec::new()
        }
        fn etat(&self) -> ic::Etat {
            etat()
        }
        fn lignes_en_plus(&self) -> Vec<ic::menu::Action> {
            Vec::new()
        }
        fn noter(&self, _ligne: String) {}
        fn quitter(&self) {}
    }

    /// Une fenêtre à soi, comme celles de Vocalis.
    struct Fenetre;

    impl ic::autres_fenetres::Contenu for Fenetre {
        fn dessiner(&mut self, _ui: &mut egui::Ui, _cadre: &mut ic::autres_fenetres::Cadre<'_>) {}
        fn fermee(&mut self, _place: Option<ic::place::Place>) {}
        fn prete(&self) -> bool {
            true
        }
    }

    fn etat() -> ic::Etat {
        ic::Etat {
            genre: ic::GenreEtat::Pret,
            titre: String::new(),
            ligne: String::new(),
            detail: None,
        }
    }

    #[test]
    fn les_noms_et_les_signatures_restent() {
        let _ = ic::apparence::Apparence::de;
        let _: fn(ic::Rgb) -> egui::Color32 = ic::apparence::couleur;
        let _: fn(&str) -> Result<ic::horloge::Unique, bool> = ic::horloge::instance_unique;
        let _ = ic::parametres::modele::Instantane::new::<u32>;
        let _: fn() -> egui::FontDefinitions = ic::polices::polices_ou_vides;
        let _: fn() -> ic::theme::ChoixWindows = ic::systeme::choix_windows;
        let _: fn(i32, i32) -> Option<([i32; 4], [i32; 4])> = ic::systeme::ecran_autour;
        let _: fn() -> Option<[i32; 2]> = ic::systeme::position_souris;
        let logiciel = Imite(super::identite());
        let _: &dyn ic::Logiciel = &logiciel;
        let _: Box<dyn ic::autres_fenetres::Contenu> = Box::new(Fenetre);
    }
}

/// Ce que Chat PDC prend de la bibliothèque.
mod chat_pdc {
    use interface_commune as ic;
    #[rustfmt::skip]
    use interface_commune::{Communs as _, Credit as _, Etat as _, EtatMiseAJour as _, FenetrePrincipale as _, GenreEtat as _, Identite as _, Logiciel as _, Reglages as _, Valeur as _, cles as _, clics as _, clics::Geste as _, clics::Suite as _, horloge as _, horloge::AuClic as _, horloge::Horloge as _, horloge::Niveau as _, horloge::Poste as _, horloge::Reveil as _, menu as _, menu::Action as _, menu::Choix as _, parametres::modele::GENERAL as _, parametres::modele::MISE_A_JOUR as _, textes::FRANCAIS as _};

    /// Un logiciel inventé qui remplace, dans les traits, les mêmes méthodes que Chat PDC.
    struct Imite(ic::Identite);

    impl ic::Reglages for Imite {
        fn action(&self, _cle: &str) {}
        fn changer(&self, _cle: &str, _valeur: ic::Valeur) {}
        fn communs(&self) -> ic::Communs {
            ic::Communs::default()
        }
        fn identite(&self) -> &ic::Identite {
            &self.0
        }
        fn rechercher_mise_a_jour(&self) {}
    }

    impl ic::Logiciel for Imite {
        fn actions_menu(&self) -> Vec<ic::menu::Action> {
            Vec::new()
        }
        fn etat(&self) -> ic::Etat {
            etat()
        }
        fn fenetre_principale(&self) -> Option<ic::FenetrePrincipale> {
            Some(ic::FenetrePrincipale::default())
        }
        fn icone_etat(&self, _taille: u32, _barre_sombre: bool) -> Option<Vec<u8>> {
            None
        }
        fn noter(&self, _ligne: String) {}
        fn ouvrir_fenetre_principale(&self) {}
        fn quitter(&self) {}
    }

    fn etat() -> ic::Etat {
        ic::Etat {
            genre: ic::GenreEtat::Pret,
            titre: String::new(),
            ligne: String::new(),
            detail: None,
        }
    }

    #[test]
    fn les_noms_et_les_signatures_restent() {
        let _: fn(ic::clics::Geste, Option<ic::FenetrePrincipale>) -> ic::clics::Suite = ic::clics::suite;
        let _: fn(
            &ic::Textes,
            Option<&str>,
            &[ic::menu::Action],
            &[ic::menu::Action],
        ) -> Vec<Option<ic::menu::Ligne>> = ic::menu::lignes_avec;
        let _: fn(&dyn ic::Reglages, Option<&std::path::Path>, &str) -> Option<std::path::PathBuf> =
            ic::parametres::fenetre::choisir_dossier;
        let _: fn([i32; 2]) -> Option<[i32; 2]> = ic::systeme::centree;
        let _: fn() -> ic::theme::ChoixWindows = ic::systeme::choix_windows;
        let _: fn(isize, bool) = ic::systeme::habiller;
        let _: fn(isize) = ic::systeme::premier_plan;
        let logiciel = Imite(super::identite());
        let _: &dyn ic::Logiciel = &logiciel;
    }
}
