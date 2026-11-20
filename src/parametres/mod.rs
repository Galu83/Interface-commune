//! Les Paramètres à la façon de Windows 11 (règles : interface Windows 11, point 7), le même
//! squelette dans tous les logiciels, dessiné comme ceux de PowerPeek (le modèle) :
//!
//! - barre de titre « Paramètres de <logiciel> » ;
//! - en haut de la navigation : le vrai logo du logiciel (32 points), son nom (16 points,
//!   semibold) et juste dessous « Version x.y.z » (12 points) ;
//! - la navigation à gauche, dont la largeur suit ses textes (250 points au moins, comme
//!   PowerPeek) : lignes de 38 points, icônes, petite barre d'accent sur la page choisie ;
//!   l'Accueil facultatif, les pages propres, puis, séparées par 6 points, toujours Général,
//!   Mise à jour et À propos (juste après, jamais collées au bas de la fenêtre) ;
//! - « Rechercher un paramètre » dès sept pages ;
//! - une carte par réglage, effet immédiat, bulle « Annuler », bandeau de redémarrage ;
//! - « Conserver ces modifications ? » (15 s, retour automatique) pour un réglage qui peut
//!   couper l'écran, la souris, le clavier ou la liaison ([`Contexte::changer_a_confirmer`]) ;
//! - une sous-page s'ouvre par un chevron « › », avec le chemin « Page › Sous-page » ;
//! - la fenêtre retient sa taille et sa place (sous Windows, `fenetre`).
//!
//! Tous les mots viennent de [`crate::textes`] (français ou anglais) ou du logiciel. Le
//! dessin ne dépend pas de la fenêtre : [`Parametres::dessiner`] se teste sans rien ouvrir,
//! et un logiciel qui a sa propre boucle de fenêtres peut le dessiner dans la sienne.

#[cfg(windows)]
pub mod fenetre;
pub mod modele;

use std::time::Instant;

use egui::{Align2, CornerRadius, FontId, RichText, Sense, pos2, vec2};

use crate::apparence::couleur;
use crate::dessin::{
    self, Genre, bouton, carte, carte_info, carte_large, interrupteur_ecrit, titre_page, titre_section,
};
use crate::polices::semibold;
use crate::textes::{Textes, remplir};
use crate::theme::{Palette, TAILLE_PETIT};
use crate::{Communs, EtatMiseAJour, Reglages, cles, icones};
use modele::{Annulation, Changement, Confirmation, PAGES_POUR_RECHERCHE, Page, Retour, Valeur};

/// La clé de la fenêtre des Paramètres parmi les autres fenêtres de la boucle
/// ([`crate::autres_fenetres`]) : réservée.
pub const CLE_FENETRE: &str = "parametres";

// Les mesures, écrites une seule fois dans `crate::mesures`.
pub use crate::mesures::{
    ESPACE_COMMUNES, NAVIGATION_LARGEUR_MAX as LARGEUR_NAVIGATION_MAX,
    NAVIGATION_LARGEUR_MIN as LARGEUR_NAVIGATION_MIN, NAVIGATION_LIGNE as LIGNE_NAVIGATION,
    PARAMETRES_TAILLE as TAILLE, PARAMETRES_TAILLE_MIN as TAILLE_MIN, RAYON_ALERTE, TAILLE_LOGO, TAILLE_NOM,
};
use crate::mesures::{NAVIGATION_DROITE, NAVIGATION_MARGES as MARGE_NAVIGATION, NAVIGATION_TEXTE_X};

/// Ce qu'une page reçoit pour se dessiner : la palette, et de quoi signaler un changement,
/// un redémarrage nécessaire ou une sous-page.
pub struct Contexte<'a> {
    pub palette: &'a Palette,
    /// Les mots communs, dans la langue du logiciel.
    pub textes: &'a Textes,
    /// La sous-page ouverte dans cette page (son titre), s'il y en a une.
    pub sous_page: Option<String>,
    sous_page_voulue: Option<Option<String>>,
    changements: Vec<(String, String, Valeur, Valeur)>,
    groupes: Vec<(String, Vec<Changement>)>,
    redemarrage: Option<String>,
    aller_a: Option<String>,
    brouillon: Option<String>,
    a_confirmer: Vec<(String, String, Valeur, Valeur)>,
    groupes_a_confirmer: Vec<(String, Vec<Changement>)>,
    brouillon_a_confirmer: Option<String>,
    /// La clé de la page qui se dessine (une section repliable se retient par page).
    page: String,
    /// Le numéro de cette ouverture des Paramètres : les sections repartent repliées.
    session: u64,
    /// Ce qui était tapé dans « Rechercher un paramètre » quand on est arrivé sur cette page.
    recherche: String,
}

impl Contexte<'_> {
    /// Ce qui était tapé dans « Rechercher un paramètre » quand on est arrivé sur cette page
    /// en la choisissant dans les résultats ; vide si on y est venu autrement. Oublié dès
    /// qu'on change de page à la main.
    pub fn recherche(&self) -> &str {
        &self.recherche
    }

    /// Une section repliable, comme l'« Expander » de Windows 11 : une carte (`titre`,
    /// `explication`) qui, cliquée, montre ce que dessine `contenu` (la page `Contexte` lui est
    /// rendue pour signaler un changement). Repliée à chaque ouverture des Paramètres ; l'état
    /// est gardé par page et par titre. Quand on arrive par « Rechercher un paramètre » et que
    /// les mots tapés sont dans le titre ou dans `mots`, elle s'ouvre (une fois ; on peut la
    /// refermer). Mettre aussi ces `mots` dans [`Page::avec_mots`], sinon la recherche ne
    /// trouve pas la page. Pour une sous-page, voir `dessin::carte_chevron`.
    pub fn section_repliable(
        &mut self,
        ui: &mut egui::Ui,
        titre: &str,
        explication: &str,
        mots: &[&str],
        contenu: impl FnOnce(&mut egui::Ui, &mut Contexte<'_>),
    ) {
        self.section(ui, None, titre, explication, mots, contenu);
    }

    /// Comme [`Contexte::section_repliable`], avec une icône (`glyphe`) à gauche du titre.
    pub fn section_repliable_avec_icone(
        &mut self,
        ui: &mut egui::Ui,
        glyphe: char,
        titre: &str,
        explication: &str,
        mots: &[&str],
        contenu: impl FnOnce(&mut egui::Ui, &mut Contexte<'_>),
    ) {
        self.section(ui, Some(glyphe), titre, explication, mots, contenu);
    }

    fn section(
        &mut self,
        ui: &mut egui::Ui,
        glyphe: Option<char>,
        titre: &str,
        explication: &str,
        mots: &[&str],
        contenu: impl FnOnce(&mut egui::Ui, &mut Contexte<'_>),
    ) {
        let id = egui::Id::unique(("section_repliable", self.session, &self.page, titre));
        let id_recherche = id.with("recherche");
        let ctx = ui.ctx().clone();
        let mut ouvert = ctx.memory(|m| m.data.get_temp::<bool>(id)).unwrap_or(false);
        // La recherche ouvre la section une seule fois par texte tapé : après, la main
        // de l'utilisateur l'emporte.
        let deja = ctx.memory(|m| m.data.get_temp::<String>(id_recherche));
        if deja.as_deref() != Some(self.recherche.as_str()) {
            ctx.memory_mut(|m| m.data.insert_temp(id_recherche, self.recherche.clone()));
            if modele::section_trouvee(&self.recherche, titre, mots) {
                ouvert = true;
            }
        }
        // Rotation du chevron : la durée et la courbe de Windows ; aucune animation si
        // Windows les a coupées.
        let duree = if effets_d_animation() {
            crate::theme::FERMETURE_FONDU_MS as f32 / 1000.0
        } else {
            0.0
        };
        let part = ctx.animate_bool_with_time_and_easing(
            id.with("rotation"),
            ouvert,
            duree,
            crate::theme::ralenti_a_l_arrivee,
        );
        let r = dessin::entete_section(ui, self.palette, id, glyphe, titre, explication, part, ouvert);
        if r.clicked() {
            ouvert = !ouvert;
        }
        ctx.memory_mut(|m| m.data.insert_temp(id, ouvert));
        if ouvert {
            ui.add_space(4.0);
            contenu(ui, self);
        }
    }

    /// Le brouillon entier du logiciel ([`Reglages::instantane`]) vient d'être modifié par
    /// l'utilisateur : la bulle « Annuler » (titre `titre`) le remet, d'un coup, tel qu'il
    /// était au début de cette image ([`Reglages::remettre`]). À appeler quand le contenu d'une
    /// page change le brouillon ; plusieurs appels de suite ne font qu'une bulle.
    pub fn brouillon_modifie(&mut self, titre: &str) {
        self.brouillon = Some(titre.into());
    }

    /// Un réglage vient d'être changé par l'utilisateur : la bibliothèque le rend au
    /// logiciel (`Reglages::changer`, effet immédiat) et ouvre la bulle « Annuler ».
    pub fn changer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        self.changements.push((cle.into(), titre.into(), avant, apres));
    }

    /// Plusieurs réglages changés d'un seul geste (clé, avant, après) : tous rendus au
    /// logiciel, et une seule bulle « Annuler » (titre `titre`) qui les remet tous.
    pub fn changer_groupe(&mut self, titre: &str, changements: Vec<Changement>) {
        self.groupes.push((titre.into(), changements));
    }

    /// Comme [`Contexte::changer`], pour un réglage qui peut couper l'écran, la souris, le
    /// clavier ou la liaison : il est rendu au logiciel tout de suite, puis une carte demande
    /// « Conserver ces modifications ? » (« Conserver » ou « Revenir », [`modele::DUREE_CONSERVER`]
    /// pour répondre). Sans réponse, comme avec « Revenir » ou la fermeture des Paramètres,
    /// la valeur d'avant revient au logiciel par [`Reglages::changer`], le chemin d'« Annuler ».
    /// Pas de bulle « Annuler » pour ce changement. Pendant la confirmation, un nouveau
    /// changement à confirmer garde la valeur d'avant le **premier**.
    pub fn changer_a_confirmer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        self.a_confirmer.push((cle.into(), titre.into(), avant, apres));
    }

    /// Comme [`Contexte::changer_groupe`], à confirmer d'un seul coup
    /// ([`Contexte::changer_a_confirmer`]).
    pub fn changer_groupe_a_confirmer(&mut self, titre: &str, changements: Vec<Changement>) {
        self.groupes_a_confirmer.push((titre.into(), changements));
    }

    /// Comme [`Contexte::brouillon_modifie`], à confirmer ([`Contexte::changer_a_confirmer`]) :
    /// « Revenir » remet le brouillon entier d'avant ([`Reglages::remettre`]).
    pub fn brouillon_a_confirmer(&mut self, titre: &str) {
        self.brouillon_a_confirmer = Some(titre.into());
    }

    /// Ce changement ne prend effet qu'au redémarrage : le bandeau le dit en haut de la page.
    pub fn redemarrage_necessaire(&mut self, raison: &str) {
        self.redemarrage = Some(raison.into());
    }

    /// Ouvre une sous-page de la page (`None` : revient à la page).
    pub fn ouvrir_sous_page(&mut self, titre: Option<&str>) {
        self.sous_page_voulue = Some(titre.map(str::to_string));
    }

    /// Va à une autre page (sa clé).
    pub fn aller_a(&mut self, cle: &str) {
        self.aller_a = Some(cle.into());
    }
}

/// « Effets d'animation » de Windows ; hors Windows, toujours oui.
fn effets_d_animation() -> bool {
    #[cfg(windows)]
    {
        crate::systeme::effets_d_animation()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Le logo dessiné, gardé entre deux images (une texture par taille et par thème).
struct Logo {
    pixels: u32,
    sombre: bool,
    texture: Option<egui::TextureHandle>,
}

/// Ce que la fenêtre des Paramètres retient entre deux images.
pub struct Parametres {
    pub pages: Vec<Page>,
    pub page: usize,
    pub sous_page: Option<String>,
    pub recherche: String,
    pub annulation: Annulation,
    /// « Conserver ces modifications ? » : ce qui attend la réponse.
    pub confirmation: Confirmation,
    /// Le focus est déjà posé sur « Revenir » (le bouton par défaut) pour cette carte.
    focus_conserver: bool,
    /// Le redémarrage demandé par un réglage (la raison), jusqu'au redémarrage.
    pub redemarrage: Option<String>,
    /// Le brouillon entier du logiciel au début de l'image en cours (pour « Annuler »).
    avant_image: Option<modele::Instantane>,
    /// « Rétablir les réglages par défaut » attend sa confirmation.
    pub confirmer_retablir: bool,
    /// « Effacer le journal » attend sa confirmation.
    pub confirmer_effacer: bool,
    /// Les textes avec lesquels la navigation a été faite (une autre langue la refait).
    textes: Textes,
    logo: Option<Logo>,
    /// Le numéro de cette ouverture (les sections repliables repartent repliées).
    session: u64,
    /// Ce qui était tapé dans la recherche quand la page montrée a été choisie.
    recherche_page: String,
}

/// Compte les ouvertures des Paramètres, pour que chacune reparte avec des sections repliées.
static OUVERTURES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Parametres {
    pub fn new(l: &dyn Reglages, page: Option<&str>) -> Parametres {
        let textes = l.textes().clone();
        let pages = modele::navigation(&textes, l.accueil(), &l.pages());
        let page = modele::rang_de_page(&pages, page);
        Parametres {
            pages,
            page,
            sous_page: None,
            recherche: String::new(),
            annulation: Annulation::default(),
            confirmation: Confirmation::default(),
            focus_conserver: false,
            redemarrage: None,
            avant_image: None,
            confirmer_retablir: false,
            confirmer_effacer: false,
            textes,
            logo: None,
            session: OUVERTURES.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            recherche_page: String::new(),
        }
    }

    /// Va à la page de cette clé, si elle existe.
    pub fn aller_a(&mut self, cle: &str) {
        if let Some(i) = self.pages.iter().position(|p| p.cle == cle) {
            self.page = i;
            self.sous_page = None;
            self.recherche_page.clear();
        }
    }

    /// Va à la page de cette clé comme si l'utilisateur avait tapé `tape` dans « Rechercher un
    /// paramètre » puis choisi cette page dans les résultats : les sections repliables qui
    /// contiennent ces mots s'ouvrent ([`Contexte::recherche`]).
    pub fn aller_a_par_recherche(&mut self, cle: &str, tape: &str) {
        self.aller_a(cle);
        self.recherche = tape.into();
        self.recherche_page = tape.trim().to_string();
    }

    /// La clé de la page montrée.
    pub fn cle(&self) -> &str {
        self.pages.get(self.page).map_or("", |p| p.cle.as_str())
    }

    /// Un changement : rendu au logiciel tout de suite, et la bulle « Annuler ».
    pub fn changer(&mut self, l: &dyn Reglages, cle: &str, titre: &str, avant: Valeur, apres: Valeur) {
        self.annulation.changer(cle, titre, avant, &apres, Instant::now());
        l.changer(cle, apres);
    }

    /// Un groupe de changements : tous rendus au logiciel, une seule bulle « Annuler ».
    pub fn changer_groupe(&mut self, l: &dyn Reglages, titre: &str, changements: Vec<Changement>) {
        for (cle, avant, apres) in &changements {
            if avant != apres {
                l.changer(cle, apres.clone());
            }
        }
        self.annulation.changer_groupe(titre, changements, Instant::now());
    }

    /// Rend au logiciel ce que « Revenir » ramène : le brouillon d'avant, puis chaque
    /// valeur d'avant, par le même chemin qu'« Annuler ».
    fn rendre(l: &dyn Reglages, retour: Retour) {
        if let Some(avant) = &retour.brouillon {
            l.remettre(avant);
        }
        for (cle, avant) in retour.valeurs {
            l.changer(&cle, avant);
        }
    }

    /// Un changement **à confirmer** : rendu au logiciel tout de suite, puis la carte
    /// « Conserver ces modifications ? » ([`Contexte::changer_a_confirmer`]).
    pub fn changer_a_confirmer(
        &mut self,
        l: &dyn Reglages,
        cle: &str,
        titre: &str,
        avant: Valeur,
        apres: Valeur,
    ) {
        self.tic(l);
        if avant == apres {
            return;
        }
        self.annulation.oublier();
        self.confirmation
            .changer(cle, titre, avant, &apres, Instant::now());
        l.changer(cle, apres);
    }

    /// Un groupe de changements **à confirmer**, d'un seul coup.
    pub fn changer_groupe_a_confirmer(
        &mut self,
        l: &dyn Reglages,
        titre: &str,
        changements: Vec<Changement>,
    ) {
        self.tic(l);
        if changements.iter().all(|(_, avant, apres)| avant == apres) {
            return;
        }
        self.annulation.oublier();
        for (cle, avant, apres) in &changements {
            if avant != apres {
                l.changer(cle, apres.clone());
            }
        }
        self.confirmation
            .changer_groupe(titre, changements, Instant::now());
    }

    /// Un brouillon entier **à confirmer** : `avant` est son état avant ce changement (le
    /// logiciel l'a déjà modifié).
    pub fn brouillon_a_confirmer(&mut self, l: &dyn Reglages, titre: &str, avant: modele::Instantane) {
        self.tic(l);
        self.annulation.oublier();
        self.confirmation.changer_brouillon(titre, avant, Instant::now());
    }

    /// « Conserver » : les changements restent.
    pub fn conserver(&mut self) {
        self.confirmation.conserver(Instant::now());
    }

    /// « Revenir » : tout revient comme avant le premier changement à confirmer. Appelé aussi
    /// par la fermeture des Paramètres ([`Parametres::fermer`]).
    pub fn revenir(&mut self, l: &dyn Reglages) {
        Self::rendre(l, self.confirmation.revenir());
        self.annulation.oublier();
    }

    /// À appeler à chaque image (la bibliothèque le fait) : le délai de « Conserver » est-il
    /// écoulé ? Alors tout revient comme avant. Rend `true` si cela vient d'arriver.
    pub fn tic(&mut self, l: &dyn Reglages) -> bool {
        self.tic_a(l, Instant::now())
    }

    /// Comme [`Parametres::tic`], à une heure donnée (pour les essais : pas d'attente de 15 s).
    pub fn tic_a(&mut self, l: &dyn Reglages, maintenant: Instant) -> bool {
        match self.confirmation.echu(maintenant) {
            Some(retour) => {
                Self::rendre(l, retour);
                self.annulation.oublier();
                true
            }
            None => false,
        }
    }

    /// Les Paramètres se ferment : ce qui attendait « Conserver » revient comme avant
    /// ([`Parametres::revenir`]), avant que le logiciel enregistre ([`Reglages::peut_fermer`]).
    pub fn fermer(&mut self, l: &dyn Reglages) {
        self.revenir(l);
    }

    /// Le titre de la fenêtre : « Paramètres de <logiciel> », dans la langue du logiciel.
    pub fn titre_fenetre(l: &dyn Reglages) -> String {
        remplir(l.textes().parametres_de, &[&l.identite().nom])
    }

    /// La langue a changé : la navigation est refaite, sur la même page.
    fn suivre_la_langue(&mut self, l: &dyn Reglages) {
        if *l.textes() == self.textes {
            return;
        }
        let cle = self.cle().to_string();
        self.textes = l.textes().clone();
        self.pages = modele::navigation(&self.textes, l.accueil(), &l.pages());
        self.page = modele::rang_de_page(&self.pages, Some(&cle));
    }

    /// La largeur de la navigation (points) : ce qu'il faut à l'en-tête et aux noms des
    /// pages pour tenir sur une ligne, entre [`LARGEUR_NAVIGATION_MIN`] et
    /// [`LARGEUR_NAVIGATION_MAX`].
    pub fn largeur_navigation(&self, ui: &egui::Ui, l: &dyn Reglages) -> f32 {
        let mesure = |t: &str, f: FontId| {
            ui.painter()
                .layout_no_wrap(t.to_string(), f, egui::Color32::WHITE)
                .size()
                .x
        };
        let id = l.identite();
        let nom = mesure(&id.nom, semibold(TAILLE_NOM));
        let version = mesure(
            &remplir(self.textes.version, &[&id.version]),
            FontId::proportional(TAILLE_PETIT),
        );
        let entete = 8.0 + TAILLE_LOGO + 4.0 + ui.spacing().item_spacing.x + nom.max(version);
        let lignes = self
            .pages
            .iter()
            .map(|p| NAVIGATION_TEXTE_X + mesure(&p.titre, FontId::proportional(14.0)) + NAVIGATION_DROITE)
            .fold(0.0, f32::max);
        let marges = f32::from(MARGE_NAVIGATION[0]) + f32::from(MARGE_NAVIGATION[1]);
        (entete.max(lignes) + marges)
            .ceil()
            .clamp(LARGEUR_NAVIGATION_MIN, LARGEUR_NAVIGATION_MAX)
    }

    /// Toute la fenêtre : navigation et page.
    pub fn dessiner(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        self.suivre_la_langue(l);
        self.tic(l);
        let largeur = self.largeur_navigation(ui, l);
        let [g, d, h, b] = MARGE_NAVIGATION;
        egui::Panel::left("navigation")
            .exact_size(largeur)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(couleur(p.fond))
                    .inner_margin(egui::Margin {
                        left: g,
                        right: d,
                        top: h,
                        bottom: b,
                    }),
            )
            .show(ui, |ui| self.navigation(ui, p, l));
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(couleur(p.fond))
                    .inner_margin(egui::Margin {
                        left: 18,
                        right: 4,
                        top: 22,
                        bottom: 0,
                    }),
            )
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .inner_margin(egui::Margin {
                                left: 0,
                                right: 24,
                                top: 0,
                                bottom: 24,
                            })
                            .show(ui, |ui| {
                                ui.set_max_width(ui.available_width().max(300.0));
                                self.page_montree(ui, p, l);
                            });
                    });
            });
        // « Conserver ces modifications ? », devant tout le reste.
        self.carte_conserver(ui, p, l);
    }

    /// Le logo à sa taille réelle (pixels de l'écran), sans flou.
    fn logo(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let ppp = ui.ctx().pixels_per_point();
        let px = (TAILLE_LOGO * ppp).round().clamp(16.0, 256.0) as u32;
        let a_refaire = self
            .logo
            .as_ref()
            .is_none_or(|g| g.pixels != px || (g.sombre != p.sombre && l.identite().logo.is_none()));
        if a_refaire {
            let texture = l.identite().logo_rvba(px, p.sombre).map(|rvba| {
                let image = egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &rvba);
                ui.ctx()
                    .load_texture("logo du logiciel", image, egui::TextureOptions::NEAREST)
            });
            self.logo = Some(Logo {
                pixels: px,
                sombre: p.sombre,
                texture,
            });
        }
        let (rect, _) = ui.allocate_exact_size(egui::Vec2::splat(px as f32 / ppp), Sense::hover());
        let coin = pos2((rect.min.x * ppp).round() / ppp, (rect.min.y * ppp).round() / ppp);
        if let Some(t) = self.logo.as_ref().and_then(|g| g.texture.as_ref()) {
            let uv = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            ui.painter().image(
                t.id(),
                egui::Rect::from_min_size(coin, rect.size()),
                uv,
                egui::Color32::WHITE,
            );
        }
    }

    fn navigation(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let id = l.identite();
        ui.add_space(4.0);
        // L'en-tête : le logo, puis le nom et « Version x.y.z » l'un sous l'autre.
        let version = remplir(self.textes.version, &[&id.version]);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            self.logo(ui, p, l);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(&id.nom).font(semibold(TAILLE_NOM)));
                ui.label(
                    RichText::new(version)
                        .size(TAILLE_PETIT)
                        .color(couleur(p.texte_secondaire)),
                );
            });
        });
        ui.add_space(18.0);
        let trouvees = if self.pages.len() >= PAGES_POUR_RECHERCHE {
            ui.add(
                egui::TextEdit::singleline(&mut self.recherche)
                    .hint_text(self.textes.rechercher_un_parametre)
                    .min_size(vec2(0.0, 32.0))
                    .desired_width(f32::INFINITY)
                    .char_limit(60),
            );
            ui.add_space(6.0);
            let t = modele::chercher(&self.pages, &self.recherche);
            (!self.recherche.trim().is_empty()).then_some(t)
        } else {
            None
        };
        let communes = self.pages.len().saturating_sub(3);
        for i in 0..self.pages.len() {
            if trouvees.as_ref().is_some_and(|t| !t.contains(&i)) {
                continue;
            }
            if i == communes && i > 0 && trouvees.is_none() {
                // Les trois pages communes, juste après les pages propres, séparées d'elles.
                ui.add_space(ESPACE_COMMUNES);
            }
            let (g, nom) = (self.pages[i].icone, self.pages[i].titre.clone());
            let choisie = self.page == i;
            let (rect, r) =
                ui.allocate_exact_size(vec2(ui.available_width(), LIGNE_NAVIGATION), Sense::click());
            if choisie || r.hovered() || r.has_focus() {
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(5), couleur(p.survol));
            }
            if choisie {
                let barre =
                    egui::Rect::from_center_size(pos2(rect.left() + 1.5, rect.center().y), vec2(3.0, 16.0));
                ui.painter()
                    .rect_filled(barre, CornerRadius::same(2), couleur(p.accent));
            }
            dessin::icone(
                ui,
                pos2(rect.left() + 16.0 + 8.0, rect.center().y),
                g,
                16.0,
                p.texte,
            );
            ui.painter().text(
                pos2(rect.left() + NAVIGATION_TEXTE_X, rect.center().y),
                Align2::LEFT_CENTER,
                &nom,
                FontId::proportional(14.0),
                couleur(p.texte),
            );
            if l.alerte(&self.pages[i].cle) {
                // Le point d'alerte de PowerPeek : à droite de la ligne, couleur d'accent.
                ui.painter().circle_filled(
                    pos2(rect.right() - 16.0, rect.center().y),
                    RAYON_ALERTE,
                    couleur(p.accent),
                );
            }
            r.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, choisie, &nom));
            dessin::focus(ui, p, &r, 5);
            if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                self.page = i;
                self.sous_page = None;
                self.recherche_page = self.recherche.trim().to_string();
                self.confirmer_retablir = false;
                self.confirmer_effacer = false;
            }
            ui.add_space(2.0);
        }
    }

    /// Le chemin en haut de la page (« Page › Sous-page ») et le titre.
    fn entete(&mut self, ui: &mut egui::Ui, p: &Palette, titre: &str) {
        match self.sous_page.clone() {
            None => titre_page(ui, p, titre, ""),
            Some(s) => {
                ui.horizontal(|ui| {
                    if dessin::bouton_icone(ui, p, icones::RETOUR, self.textes.retour).clicked() {
                        self.sous_page = None;
                    }
                    ui.label(
                        RichText::new(format!("{titre} › {s}"))
                            .font(crate::polices::titre(crate::theme::TAILLE_TITRE)),
                    );
                });
                ui.add_space(8.0);
            }
        }
    }

    fn page_montree(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let Some(page) = self.pages.get(self.page).cloned() else {
            return;
        };
        // Le bandeau de redémarrage, en haut de la page.
        if let Some(r) = self.redemarrage.clone() {
            if dessin::bandeau(
                ui,
                p,
                p.orange,
                icones::RETABLIR,
                &r,
                self.textes.redemarrer_maintenant,
            ) {
                l.action(cles::REDEMARRER);
            }
            ui.add_space(8.0);
        }
        self.entete(ui, p, &page.titre);
        self.avant_image = l.instantane();
        self.bulle_annuler(ui, p, l);
        let communs = l.communs();
        let t = self.textes.clone();
        match page.cle.as_str() {
            modele::GENERAL => self.page_general(ui, p, l, &communs),
            modele::MISE_A_JOUR => {
                let mut changes = Vec::new();
                page_mise_a_jour(ui, p, &t, l, &communs, |cle, titre, avant, apres| {
                    changes.push((cle.to_string(), titre.to_string(), avant, apres));
                });
                for (cle, titre, avant, apres) in changes {
                    self.changer(l, &cle, &titre, avant, apres);
                }
                self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_mise_a_jour(ui, c));
            }
            modele::A_PROPOS => self.page_a_propos(ui, p, l, &communs),
            cle => self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_page(cle, ui, c)),
        }
        // Le bas de la fenêtre, sous toutes les pages.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_pied(ui, c));
    }

    /// Fait dessiner le logiciel (une page propre, ses cartes de Général), puis applique ce
    /// qu'il a demandé : changements, redémarrage, sous-page, autre page.
    fn avec_contexte(
        &mut self,
        ui: &mut egui::Ui,
        p: &Palette,
        l: &dyn Reglages,
        dessin: impl FnOnce(&dyn Reglages, &mut egui::Ui, &mut Contexte<'_>),
    ) {
        let t = self.textes.clone();
        let mut c = Contexte {
            palette: p,
            textes: &t,
            sous_page: self.sous_page.clone(),
            sous_page_voulue: None,
            changements: Vec::new(),
            groupes: Vec::new(),
            redemarrage: None,
            aller_a: None,
            brouillon: None,
            a_confirmer: Vec::new(),
            groupes_a_confirmer: Vec::new(),
            brouillon_a_confirmer: None,
            page: self.cle().to_string(),
            session: self.session,
            recherche: self.recherche_page.clone(),
        };
        dessin(l, ui, &mut c);
        for (cle, titre, avant, apres) in c.changements {
            self.changer(l, &cle, &titre, avant, apres);
        }
        for (titre, changements) in c.groupes {
            self.changer_groupe(l, &titre, changements);
        }
        if let Some(titre) = c.brouillon
            && let Some(avant) = self.avant_image.clone()
        {
            self.annulation.changer_brouillon(&titre, avant, Instant::now());
        }
        for (cle, titre, avant, apres) in c.a_confirmer {
            self.changer_a_confirmer(l, &cle, &titre, avant, apres);
        }
        for (titre, changements) in c.groupes_a_confirmer {
            self.changer_groupe_a_confirmer(l, &titre, changements);
        }
        if let Some(titre) = c.brouillon_a_confirmer
            && let Some(avant) = self.avant_image.clone()
        {
            self.brouillon_a_confirmer(l, &titre, avant);
        }
        if let Some(r) = c.redemarrage {
            self.redemarrage = Some(r);
        }
        if let Some(s) = c.sous_page_voulue {
            self.sous_page = s;
        }
        if let Some(a) = c.aller_a {
            self.aller_a(&a);
        }
    }

    /// La bulle « Annuler », quelques secondes après un changement.
    fn bulle_annuler(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let maintenant = Instant::now();
        let Some(a) = self.annulation.a_annuler(maintenant).cloned() else {
            return;
        };
        if let Some(reste) = self.annulation.reste(maintenant) {
            ui.ctx().request_repaint_after(reste);
        }
        let mut annuler = false;
        let mut fermer = false;
        let t = &self.textes;
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(remplir(t.modifie, &[&a.titre]));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    fermer = dessin::bouton_icone(ui, p, icones::FERMER, t.fermer).clicked();
                    annuler = bouton(ui, p, Some(icones::ANNULER), t.annuler, Genre::Normal).clicked();
                });
            });
        });
        ui.add_space(8.0);
        if annuler {
            if let Some(avant) = self.annulation.annuler_brouillon(Instant::now()) {
                l.remettre(&avant);
            }
            for (cle, avant) in self.annulation.annuler(Instant::now()) {
                l.changer(&cle, avant);
            }
        } else if fermer {
            self.annulation.oublier();
        }
    }

    /// La carte « Conserver ces modifications ? », devant les Paramètres (le reste ne répond
    /// plus tant qu'elle est là), avec le compte à rebours. « Revenir » est le bouton par
    /// défaut (Entrée) ; Échap = « Revenir ».
    fn carte_conserver(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages) {
        let maintenant = Instant::now();
        let Some(titre) = self.confirmation.en_attente(maintenant).map(|a| a.titre.clone()) else {
            self.focus_conserver = false;
            return;
        };
        let secondes = self.confirmation.secondes(maintenant).unwrap_or(0);
        let ctx = ui.ctx().clone();
        if let Some(d) = self.confirmation.prochain_changement(maintenant) {
            ctx.request_repaint_after(d);
        }
        let t = self.textes.clone();
        let (mut garder, mut revenir) = (false, false);
        let reste = self
            .confirmation
            .reste(maintenant)
            .map_or(0.0, |r| r.as_secs_f32() / modele::DUREE_CONSERVER.as_secs_f32());
        let donner_le_focus = !self.focus_conserver;
        let cadre = dessin::cadre_carte(p)
            .fill(couleur(p.fond_menu))
            .inner_margin(egui::Margin::same(20))
            .shadow(egui::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: egui::Color32::from_black_alpha(if p.sombre { 130 } else { 60 }),
            });
        let id = egui::Id::unique("interface_commune_conserver");
        // Sans fondu : une boîte de dialogue de Windows apparaît d'un coup, tout de suite lisible.
        let modale = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).fade_in(false))
            .backdrop_color(egui::Color32::from_black_alpha(if p.sombre { 130 } else { 90 }))
            .frame(cadre)
            .show(&ctx, |ui| {
                ui.set_width(380.0);
                ui.label(RichText::new(t.conserver_question).font(semibold(18.0)));
                ui.add_space(8.0);
                ui.label(remplir(t.modifie, &[&titre]));
                ui.add_space(2.0);
                ui.label(
                    RichText::new(remplir(t.conserver_retour, &[&secondes.to_string()]))
                        .color(couleur(p.texte_secondaire)),
                );
                ui.add_space(10.0);
                // Le temps qui reste, en barre fine : elle se vide jusqu'au retour automatique.
                let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(2), couleur(p.survol));
                let plein = egui::Rect::from_min_size(rect.min, vec2(rect.width() * reste, rect.height()));
                ui.painter()
                    .rect_filled(plein, CornerRadius::same(2), couleur(p.accent));
                ui.add_space(16.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // De droite à gauche : « Revenir » (par défaut), puis « Conserver ».
                    let r = bouton(ui, p, None, t.revenir, Genre::Accent);
                    if donner_le_focus {
                        r.request_focus();
                    }
                    revenir = r.clicked();
                    garder = bouton(ui, p, None, t.conserver, Genre::Normal).clicked();
                });
            });
        self.focus_conserver = true;
        // Échap = « Revenir » (la carte est devant : rien d'autre ne le prend).
        let echap = modale.is_top_modal
            && !modale.any_popup_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if garder {
            self.conserver();
        } else if revenir || echap {
            self.revenir(l);
        }
    }

    fn page_general(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let t = self.textes.clone();
        if let Some(actif) = c.demarrage {
            let mut v = actif;
            let bloque = c.demarrage_bloque.as_deref();
            carte(
                ui,
                p,
                icones::DEMARRAGE,
                t.demarrer_avec_windows,
                bloque.unwrap_or(t.demarrer_explication),
                |ui| {
                    ui.add_enabled_ui(bloque.is_none(), |ui| {
                        interrupteur_ecrit(ui, p, &t, &mut v);
                    });
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::DEMARRAGE,
                    t.demarrer_avec_windows,
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        if let Some((langue, langues)) = &c.langue {
            let mut choix = langue.clone();
            carte(ui, p, icones::LANGUE, t.langue, t.langue_explication, |ui| {
                egui::ComboBox::from_id_salt("langue")
                    .selected_text(choix.clone())
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        for x in langues {
                            ui.selectable_value(&mut choix, x.clone(), x);
                        }
                    });
            });
            if choix != *langue {
                self.changer(
                    l,
                    cles::LANGUE,
                    t.langue,
                    Valeur::Texte(langue.clone()),
                    Valeur::Texte(choix),
                );
            }
        }
        if let Some(actif) = c.notifications {
            let mut v = actif;
            carte(
                ui,
                p,
                icones::NOTIFICATIONS,
                t.notifications,
                t.notifications_explication,
                |ui| {
                    interrupteur_ecrit(ui, p, &t, &mut v);
                    if c.notifications_essai && bouton(ui, p, None, t.essayer, Genre::Normal).clicked() {
                        l.action(cles::NOTIFICATIONS_ESSAYER);
                    }
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::NOTIFICATIONS,
                    t.notifications,
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        // Les cartes propres au logiciel, avant le journal.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_general(ui, c));
        if self.confirmer_effacer {
            carte_large(
                ui,
                p,
                icones::JOURNAL,
                t.effacer_journal_question,
                t.effacer_journal_explication,
                260.0,
                |ui| {
                    if bouton(ui, p, None, t.non, Genre::Normal).clicked() {
                        self.confirmer_effacer = false;
                    }
                    if bouton(ui, p, None, t.oui_effacer, Genre::Danger).clicked() {
                        self.confirmer_effacer = false;
                        l.action(cles::JOURNAL_EFFACER);
                    }
                },
            );
        } else {
            carte(ui, p, icones::JOURNAL, t.journal, t.journal_explication, |ui| {
                if bouton(ui, p, None, t.ouvrir_le_journal, Genre::Normal).clicked() {
                    l.action(cles::JOURNAL);
                }
                if c.journal_effacable && bouton(ui, p, None, t.effacer_le_journal, Genre::Danger).clicked() {
                    self.confirmer_effacer = true;
                }
            });
        }
        if let Some(actif) = c.journal_detaille {
            let mut v = actif;
            carte(
                ui,
                p,
                icones::JOURNAL,
                t.journal_detaille,
                t.journal_detaille_explication,
                |ui| {
                    interrupteur_ecrit(ui, p, &t, &mut v);
                },
            );
            if v != actif {
                self.changer(
                    l,
                    cles::JOURNAL_DETAILLE,
                    t.journal_detaille,
                    Valeur::Booleen(actif),
                    Valeur::Booleen(v),
                );
            }
        }
        titre_section(ui, t.remise_a_zero);
        if self.confirmer_retablir {
            let explication = l.retablir_explication();
            carte_large(
                ui,
                p,
                icones::RETABLIR,
                t.retablir_question,
                explication.as_deref().unwrap_or(t.retablir_question_explication),
                260.0,
                |ui| {
                    if bouton(ui, p, None, t.non, Genre::Normal).clicked() {
                        self.confirmer_retablir = false;
                    }
                    if bouton(ui, p, None, t.oui_retablir, Genre::Danger).clicked() {
                        self.confirmer_retablir = false;
                        l.action(cles::RETABLIR);
                    }
                },
            );
        } else {
            carte(
                ui,
                p,
                icones::RETABLIR,
                t.retablir,
                t.retablir_explication,
                |ui| {
                    if bouton(ui, p, None, t.retablir_bouton, Genre::Danger).clicked() {
                        self.confirmer_retablir = true;
                    }
                },
            );
        }
    }

    fn page_a_propos(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let id = l.identite();
        let t = self.textes.clone();
        // Le logo, le nom et la version, puis ce que fait le logiciel.
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.set_min_height(48.0);
                self.logo(ui, p, l);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("{} {}", id.nom, id.version)).font(semibold(14.0)));
                    if !id.description.is_empty() {
                        ui.add(
                            egui::Label::new(
                                RichText::new(dessin::insecables(&id.description))
                                    .size(TAILLE_PETIT)
                                    .color(couleur(p.texte_secondaire)),
                            )
                            .wrap(),
                        );
                    }
                });
            });
        });
        let sous_licence = remplir(t.sous_licence, &[&id.nom, &id.licence]);
        if c.fichier_licences {
            carte(ui, p, icones::LICENCE, t.licence, &sous_licence, |ui| {
                if bouton(ui, p, None, t.voir_licences, Genre::Normal).clicked() {
                    l.action(cles::LICENCES);
                }
            });
        } else {
            carte_info(ui, p, icones::LICENCE, t.licence, &sous_licence);
        }
        // Les cartes propres au logiciel, avant les crédits.
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_a_propos(ui, c));
        if !c.credits.is_empty() {
            titre_section(ui, t.credits);
            dessin::cadre_carte(p).show(ui, |ui| {
                ui.set_width(ui.available_width());
                for cr in &c.credits {
                    // Un nom long passe à la ligne au lieu de déborder.
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&cr.nom);
                        ui.label(RichText::new(&cr.licence).color(couleur(p.texte_secondaire)));
                    });
                }
            });
        }
    }
}

/// La couleur d'une icône d'état de la mise à jour, avant de choisir dans la palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonMiseAJour {
    Neutre,
    Bon,
    Occupe,
    Accent,
    Probleme,
}

/// Ce que dit la page « Mise à jour » d'un état, à la façon de Windows Update : une seule
/// icône (coche = à jour, flèche = mise à jour disponible, triangle = problème), sa couleur
/// et le titre.
pub fn presentation_mise_a_jour(e: &EtatMiseAJour, t: &Textes) -> (char, TonMiseAJour, String) {
    use TonMiseAJour::*;
    match e {
        EtatMiseAJour::Inconnu => (icones::MISE_A_JOUR, Neutre, t.maj_pas_cherche.into()),
        EtatMiseAJour::AJour => (icones::COCHE, Bon, t.maj_a_jour.into()),
        EtatMiseAJour::Recherche => (icones::MISE_A_JOUR, Occupe, t.maj_recherche.into()),
        EtatMiseAJour::Prete(v) => (icones::INSTALLER, Accent, remplir(t.maj_prete, &[v])),
        EtatMiseAJour::Erreur(_) => (icones::ATTENTION, Probleme, t.maj_erreur.into()),
        EtatMiseAJour::Coupee => (icones::A_PROPOS, Neutre, t.maj_coupee.into()),
        EtatMiseAJour::Information(e) => (icones::A_PROPOS, Neutre, e.clone()),
    }
}

/// La ligne grise sous le titre : la raison d'une erreur, la version installée, puis la
/// dernière recherche, séparées par « · ».
pub fn ligne_grise_mise_a_jour(
    e: &EtatMiseAJour,
    version: &str,
    derniere: Option<&str>,
    t: &Textes,
) -> String {
    let mut morceaux: Vec<String> = Vec::new();
    if let EtatMiseAJour::Erreur(raison) = e {
        morceaux.push(raison.clone());
    }
    morceaux.push(remplir(t.version_actuelle, &[version]));
    if let Some(d) = derniere {
        morceaux.push(remplir(t.derniere_recherche, &[d]));
    }
    morceaux.join(" · ")
}

/// Le texte de l'état de la mise à jour, et sa couleur.
pub fn texte_mise_a_jour(e: &EtatMiseAJour, p: &Palette, t: &Textes) -> (char, crate::Rgb, String) {
    let (glyphe, ton, texte) = presentation_mise_a_jour(e, t);
    let c = match ton {
        TonMiseAJour::Neutre => p.texte_secondaire,
        TonMiseAJour::Bon => p.vert,
        TonMiseAJour::Occupe => p.orange,
        TonMiseAJour::Accent => p.accent,
        TonMiseAJour::Probleme => p.rouge,
    };
    (glyphe, c, texte)
}

fn page_mise_a_jour(
    ui: &mut egui::Ui,
    p: &Palette,
    t: &Textes,
    l: &dyn Reglages,
    c: &Communs,
    mut changer: impl FnMut(&str, &str, Valeur, Valeur),
) {
    let (g, couleur_etat, titre) = texte_mise_a_jour(&c.mise_a_jour, p, t);
    let gris = ligne_grise_mise_a_jour(
        &c.mise_a_jour,
        &l.identite().version,
        c.derniere_recherche.as_deref(),
        t,
    );
    dessin::cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        // Le bouton d'abord (à droite) : le titre et la ligne grise prennent la place qui reste
        // et passent à la ligne, sans jamais passer sous le bouton.
        // Une hauteur fixe : une disposition de droite à gauche prendrait toute celle de la page.
        let taille = egui::vec2(ui.available_width(), 56.0);
        ui.allocate_ui_with_layout(taille, egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let EtatMiseAJour::Prete(_) = c.mise_a_jour {
                if bouton(
                    ui,
                    p,
                    Some(icones::INSTALLER),
                    t.installer_maintenant,
                    Genre::Accent,
                )
                .clicked()
                {
                    l.action(cles::INSTALLER);
                }
            } else if c.recherches != Some(false)
                && bouton(
                    ui,
                    p,
                    Some(icones::MISE_A_JOUR),
                    t.rechercher_mise_a_jour,
                    Genre::Accent,
                )
                .clicked()
            {
                l.rechercher_mise_a_jour();
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(40.0, 44.0), egui::Sense::hover());
                dessin::icone(ui, r.center(), g, 30.0, couleur_etat);
                ui.vertical(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(dessin::insecables(&titre))
                                .font(semibold(18.0))
                                .color(couleur(p.texte)),
                        )
                        .wrap(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(gris)
                                .size(TAILLE_PETIT)
                                .color(couleur(p.texte_secondaire)),
                        )
                        .wrap(),
                    );
                });
            });
        });
    });
    if !c.nouveautes.is_empty() {
        titre_section(ui, t.nouveautes);
        dessin::cadre_carte(p).show(ui, |ui| {
            ui.set_width(ui.available_width());
            for n in &c.nouveautes {
                ui.add(egui::Label::new(format!("• {n}")).wrap());
            }
        });
    }
    titre_section(ui, t.reglages);
    if let Some(actif) = c.recherches {
        let mut v = actif;
        let explication = match (actif, c.dossier_automatique) {
            (false, _) => t.recherches_coupees,
            (true, Some(true)) => t.recherches_automatique,
            _ => t.recherches_dossier,
        };
        carte(
            ui,
            p,
            icones::MISE_A_JOUR,
            t.chercher_mises_a_jour,
            explication,
            |ui| {
                interrupteur_ecrit(ui, p, t, &mut v);
            },
        );
        if v != actif {
            changer(
                cles::RECHERCHES,
                t.chercher_mises_a_jour,
                Valeur::Booleen(actif),
                Valeur::Booleen(v),
            );
        }
    }
    if c.recherches == Some(false) {
        return;
    }
    let automatique = !c.dossier_fixe && c.dossier_automatique == Some(false);
    carte_large(
        ui,
        p,
        icones::DOSSIER,
        t.dossier_installateurs,
        &c.dossier_installateurs,
        if automatique { 240.0 } else { 120.0 },
        |ui| {
            if !c.dossier_fixe && bouton(ui, p, None, t.choisir, Genre::Normal).clicked() {
                l.action(cles::DOSSIER);
            }
            if automatique && bouton(ui, p, None, t.automatique, Genre::Normal).clicked() {
                l.action(cles::DOSSIER_AUTO);
            }
        },
    );
}

#[cfg(test)]
mod essais_mise_a_jour {
    use super::*;
    use crate::textes::{ANGLAIS, FRANCAIS};

    #[test]
    fn une_seule_icone_selon_l_etat() {
        let icone = |e: EtatMiseAJour| presentation_mise_a_jour(&e, &FRANCAIS).0;
        assert_eq!(icone(EtatMiseAJour::AJour), icones::COCHE);
        assert_eq!(icone(EtatMiseAJour::Prete("2.0.0".into())), icones::INSTALLER);
        assert_eq!(icone(EtatMiseAJour::Erreur("x".into())), icones::ATTENTION);
    }

    #[test]
    fn le_titre_dit_l_etat_dans_les_deux_langues() {
        let titre = |e: EtatMiseAJour, t: &Textes| presentation_mise_a_jour(&e, t).2;
        assert_eq!(titre(EtatMiseAJour::AJour, &FRANCAIS), "Vous êtes à jour");
        assert_eq!(titre(EtatMiseAJour::AJour, &ANGLAIS), "You're up to date");
        assert_eq!(
            titre(EtatMiseAJour::Prete("2.1.0".into()), &FRANCAIS),
            "Version 2.1.0 disponible"
        );
        assert_eq!(
            titre(EtatMiseAJour::Prete("2.1.0".into()), &ANGLAIS),
            "Version 2.1.0 is available"
        );
        assert_eq!(
            titre(EtatMiseAJour::Erreur("Réseau coupé.".into()), &FRANCAIS),
            "La recherche a échoué"
        );
    }

    #[test]
    fn les_couleurs_suivent_l_etat() {
        let ton = |e: EtatMiseAJour| presentation_mise_a_jour(&e, &FRANCAIS).1;
        assert_eq!(ton(EtatMiseAJour::AJour), TonMiseAJour::Bon);
        assert_eq!(ton(EtatMiseAJour::Prete("2.0.0".into())), TonMiseAJour::Accent);
        assert_eq!(ton(EtatMiseAJour::Erreur("x".into())), TonMiseAJour::Probleme);
    }

    #[test]
    fn la_ligne_grise_donne_la_version_et_la_derniere_recherche() {
        let a_jour = EtatMiseAJour::AJour;
        assert_eq!(
            ligne_grise_mise_a_jour(&a_jour, "1.0.0", Some("20 nov. 2026, 14:05"), &FRANCAIS),
            "Version actuelle : 1.0.0 · Dernière recherche : 20 nov. 2026, 14:05"
        );
        assert_eq!(
            ligne_grise_mise_a_jour(&a_jour, "1.0.0", None, &ANGLAIS),
            "Current version: 1.0.0"
        );
        let erreur = EtatMiseAJour::Erreur("Réseau coupé.".into());
        assert_eq!(
            ligne_grise_mise_a_jour(&erreur, "1.0.0", None, &FRANCAIS),
            "Réseau coupé. · Version actuelle : 1.0.0"
        );
    }
}
