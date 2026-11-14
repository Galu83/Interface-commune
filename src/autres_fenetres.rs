//! Les autres fenêtres du logiciel, ouvertes et fermées **dans la boucle de fenêtres de la
//! bibliothèque**.
//!
//! winit 0.30 n'accepte qu'une boucle de fenêtres par programme : une deuxième, créée dans un
//! autre fil, échoue (« EventLoop can't be recreated »). La bibliothèque garde donc une seule
//! boucle, toute la vie du logiciel, dont la fenêtre principale est celle du volet (cachée au
//! repos) ; chaque autre fenêtre (les Paramètres, une fenêtre simple, une fenêtre de lecture…)
//! y est une fenêtre de plus d'egui (un « viewport » immédiat), dessinée pendant l'image de la
//! fenêtre principale.
//!
//! Pour un logiciel, depuis n'importe quel fil :
//!
//! ```no_run
//! # fn exemple(fenetres: interface_commune::autres_fenetres::AutresFenetres) {
//! use interface_commune::autres_fenetres::{Cadre, Contenu, Ouverture};
//! struct Bonjour;
//! impl Contenu for Bonjour {
//!     fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
//!         egui::CentralPanel::default().show(ui, |ui| {
//!             if ui.button("Fermer").clicked() {
//!                 cadre.fermer();
//!             }
//!         });
//!     }
//! }
//! fenetres.ouvrir("bonjour", Ouverture::new("Bonjour", [320.0, 160.0]), Bonjour);
//! // … plus tard, depuis n'importe où :
//! fenetres.fermer("bonjour");
//! # }
//! ```
//!
//! Chaque fenêtre s'ouvre **cachée**, se dessine deux fois, puis se montre : jamais une
//! fenêtre vide ni un éclair blanc. Elle suit le thème de Windows (barre de titre sombre,
//! coins arrondis). Elle se ferme par sa croix, Alt+F4, [`Cadre::fermer`] ou
//! [`AutresFenetres::fermer`] ; [`Contenu::fermee`] reçoit alors sa place, à retenir.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use egui::{ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};

use crate::place::Place;
use crate::theme::Palette;

/// Comment ouvrir une fenêtre : son titre, sa taille (points), sa place…
#[derive(Clone, Debug)]
pub struct Ouverture {
    /// Le titre de la barre de titre (aussi celui de la barre des tâches).
    pub titre: String,
    /// La taille intérieure au premier affichage (points).
    pub taille: [f32; 2],
    /// La taille minimale (points), si elle se redimensionne.
    pub taille_min: Option<[f32; 2]>,
    pub redimensionnable: bool,
    /// L'icône de la barre de titre et de la barre des tâches.
    pub icone: Option<Arc<egui::IconData>>,
    /// La place (gauche, haut, largeur, hauteur, en pixels) ; sans elle, la fenêtre est
    /// centrée sur l'écran de la souris.
    pub place: Option<Place>,
    pub toujours_devant: bool,
    pub barre_des_taches: bool,
    /// La barre de titre et le cadre de Windows.
    pub cadre: bool,
    /// Prendre le focus en se montrant.
    pub focus: bool,
}

impl Ouverture {
    /// Une fenêtre ordinaire : barre de titre, redimensionnable, centrée, dans la barre des
    /// tâches, qui prend le focus.
    pub fn new(titre: &str, taille: [f32; 2]) -> Ouverture {
        Ouverture {
            titre: titre.into(),
            taille,
            taille_min: None,
            redimensionnable: true,
            icone: None,
            place: None,
            toujours_devant: false,
            barre_des_taches: true,
            cadre: true,
            focus: true,
        }
    }

    pub fn taille_min(mut self, t: [f32; 2]) -> Ouverture {
        self.taille_min = Some(t);
        self
    }

    pub fn fixe(mut self) -> Ouverture {
        self.redimensionnable = false;
        self
    }

    pub fn icone(mut self, icone: egui::IconData) -> Ouverture {
        self.icone = Some(Arc::new(icone));
        self
    }

    /// La place retenue (pixels) ; `None` : centrée.
    pub fn place(mut self, place: Option<Place>) -> Ouverture {
        self.place = place;
        self
    }

    pub fn toujours_devant(mut self) -> Ouverture {
        self.toujours_devant = true;
        self
    }

    pub fn hors_barre_des_taches(mut self) -> Ouverture {
        self.barre_des_taches = false;
        self
    }

    /// Sans barre de titre ni cadre (une bulle, un bandeau).
    pub fn sans_cadre(mut self) -> Ouverture {
        self.cadre = false;
        self
    }

    /// Se montre sans prendre le focus (une fenêtre de lecture qui ne doit pas voler la
    /// frappe en cours).
    pub fn sans_focus(mut self) -> Ouverture {
        self.focus = false;
        self
    }

    /// Ce qu'egui reçoit : la fenêtre est toujours créée cachée.
    pub fn constructeur(&self) -> ViewportBuilder {
        let mut b = ViewportBuilder::default()
            .with_title(&self.titre)
            .with_inner_size(self.taille)
            .with_resizable(self.redimensionnable)
            .with_decorations(self.cadre)
            .with_taskbar(self.barre_des_taches)
            .with_active(self.focus)
            .with_visible(false);
        if let Some(t) = self.taille_min {
            b = b.with_min_inner_size(t);
        }
        if let Some(i) = &self.icone {
            b = b.with_icon(i.clone());
        }
        if self.toujours_devant {
            b = b.with_always_on_top();
        }
        b
    }
}

/// Ce qu'une fenêtre montre. Ses méthodes sont appelées sur le fil des fenêtres : elles
/// dessinent ce que le logiciel a déjà en mémoire, sans disque, réseau ni attente.
pub trait Contenu: Send + 'static {
    /// Dessine la fenêtre, à chaque image. `ui` n'a ni marge ni fond : un
    /// `egui::CentralPanel` (ou [`crate::dessin::fond`]) les donne.
    fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>);

    /// La fenêtre se ferme ; `place` : sa dernière place (pixels), à retenir.
    fn fermee(&mut self, _place: Option<Place>) {}
}

/// Ce qu'une fenêtre reçoit à chaque image, et ce qu'elle peut demander.
pub struct Cadre<'a> {
    pub palette: &'a Palette,
    /// La fenêtre de Windows (HWND), dès qu'elle est trouvée.
    pub hwnd: Option<isize>,
    /// Le nombre d'images dessinées depuis l'ouverture (1 à la première).
    pub image: u32,
    /// La fenêtre est montrée (les premières images sont dessinées cachées).
    pub montree: bool,
    fermer: bool,
    titre: Option<String>,
}

impl<'a> Cadre<'a> {
    /// Un cadre pour dessiner un contenu ailleurs que dans la boucle (fenêtre principale
    /// d'un logiciel, essais).
    pub fn new(palette: &'a Palette, hwnd: Option<isize>, image: u32, montree: bool) -> Cadre<'a> {
        Cadre {
            palette,
            hwnd,
            image,
            montree,
            fermer: false,
            titre: None,
        }
    }

    /// Le contenu a demandé la fermeture.
    pub fn fermeture_demandee(&self) -> bool {
        self.fermer
    }

    /// Le titre demandé par le contenu, s'il en a demandé un.
    pub fn titre_demande(&self) -> Option<&str> {
        self.titre.as_deref()
    }

    /// Ferme la fenêtre à la fin de cette image.
    pub fn fermer(&mut self) {
        self.fermer = true;
    }

    /// Change le titre de la fenêtre.
    pub fn titre(&mut self, titre: &str) {
        self.titre = Some(titre.into());
    }
}

/// Une fenêtre ouverte.
struct Fenetre {
    ouverture: Ouverture,
    contenu: Box<dyn Contenu>,
    /// Le titre en cours (la fenêtre de Windows est retrouvée par lui).
    titre: String,
    images: u32,
    montree: bool,
    hwnd: Option<isize>,
    /// Le thème avec lequel la barre de titre a été habillée.
    sombre: Option<bool>,
    /// Voilée (`systeme::voiler`) tant qu'elle n'est pas montrée.
    voilee: Option<bool>,
    devant: bool,
    fermer: bool,
}

/// Ce que la fenêtre devient après une image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Apres {
    Garder,
    Fermer,
}

/// Les images dessinées cachées avant de montrer la fenêtre : la première pour la mettre en
/// page, la deuxième pour la dessiner à sa place et à sa taille.
pub const IMAGES_CACHEES: u32 = 2;

impl Fenetre {
    fn image(&mut self, ui: &mut egui::Ui, classe: ViewportClass, palette: &Palette) -> Apres {
        let ctx = ui.ctx().clone();
        if self.fermer || ui.input(|i| i.viewport().close_requested()) {
            return Apres::Fermer;
        }
        self.images = self.images.saturating_add(1);
        let reelle = !matches!(classe, ViewportClass::EmbeddedWindow);
        #[cfg(windows)]
        if reelle {
            self.windows(palette);
        }
        if !self.montree {
            if self.images >= IMAGES_CACHEES || !reelle {
                self.montree = true;
                #[cfg(windows)]
                self.voiler(false);
                ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                if self.ouverture.focus {
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                }
            } else {
                ctx.request_repaint();
            }
        }
        if std::mem::take(&mut self.devant) {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
            #[cfg(windows)]
            if let Some(h) = self.hwnd {
                crate::systeme::premier_plan(h);
            }
        }
        let mut cadre = Cadre {
            palette,
            hwnd: self.hwnd,
            image: self.images,
            montree: self.montree,
            fermer: false,
            titre: None,
        };
        self.contenu.dessiner(ui, &mut cadre);
        if let Some(t) = cadre.titre
            && t != self.titre
        {
            ctx.send_viewport_cmd(ViewportCommand::Title(t.clone()));
            self.titre = t;
        }
        if cadre.fermer {
            Apres::Fermer
        } else {
            Apres::Garder
        }
    }

    /// Sous Windows : la fenêtre retrouvée par son titre, habillée (coins, barre de titre
    /// sombre), posée à sa place ou centrée, avant d'être montrée.
    #[cfg(windows)]
    fn windows(&mut self, palette: &Palette) {
        use crate::systeme;
        if self.hwnd.is_none() {
            self.hwnd = systeme::fenetre_du_fil(&self.titre);
        }
        let Some(h) = self.hwnd else { return };
        if !self.montree {
            // Préparée cachée : voilée dès qu'elle existe (eframe la montrerait d'office).
            self.voiler(true);
        }
        if self.sombre != Some(palette.sombre) {
            self.sombre = Some(palette.sombre);
            systeme::habiller(h, palette.sombre);
        }
        if !self.montree {
            // Posée à chaque image cachée : si Windows change son échelle en l'amenant sur un
            // autre écran, la fois suivante lui rend sa taille.
            match self.ouverture.place {
                Some(p) => systeme::poser(h, p),
                None => {
                    if let Some(t) = systeme::taille_fenetre(h)
                        && let Some(p) = systeme::centree(t)
                    {
                        systeme::deplacer(h, p[0], p[1], false);
                    }
                }
            }
        }
    }

    /// Voile la fenêtre ou lève le voile (une seule fois par changement).
    #[cfg(windows)]
    fn voiler(&mut self, voile: bool) {
        if let Some(h) = self.hwnd
            && self.voilee != Some(voile)
            && crate::systeme::voiler(h, voile)
        {
            self.voilee = Some(voile);
        }
    }

    /// La place de la fenêtre (pixels), pour la retenir.
    fn place(&self) -> Option<Place> {
        #[cfg(windows)]
        {
            let r = crate::systeme::rect_fenetre(self.hwnd?)?;
            crate::place::place_valable([r[0], r[1], r[2] - r[0], r[3] - r[1]])
        }
        #[cfg(not(windows))]
        {
            None
        }
    }
}

type Partagee = Arc<Mutex<Fenetre>>;

#[derive(Default)]
struct Registre {
    fenetres: BTreeMap<String, Partagee>,
}

/// Les autres fenêtres du logiciel. Se clone et s'utilise depuis n'importe quel fil ; la
/// fenêtre principale de la boucle (le volet) les dessine par [`AutresFenetres::montrer`].
#[derive(Clone, Default)]
pub struct AutresFenetres {
    registre: Arc<Mutex<Registre>>,
    contexte: Arc<Mutex<Option<egui::Context>>>,
}

fn verrou<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Un verrou empoisonné (un dessin a paniqué) ne bloque pas les autres fenêtres.
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// L'identifiant egui de la fenêtre de cette clé.
pub fn id_de(cle: &str) -> ViewportId {
    ViewportId::from_hash_of(("interface-commune", cle))
}

impl AutresFenetres {
    /// Le contexte egui de la boucle (fixé par la bibliothèque) : une ouverture ou une
    /// fermeture la réveille.
    pub fn lier(&self, ctx: Option<egui::Context>) {
        *verrou(&self.contexte) = ctx;
        // Les fenêtres demandées avant que la boucle soit prête s'ouvrent maintenant.
        self.reveiller();
    }

    /// Réveille la fenêtre principale, pour qu'elle ouvre, ferme ou redessine.
    pub fn reveiller(&self) {
        if let Some(c) = verrou(&self.contexte).as_ref() {
            c.request_repaint_of(ViewportId::ROOT);
        }
    }

    /// Ouvre la fenêtre de cette clé (une seule à la fois par clé). Déjà ouverte : elle passe
    /// devant, son contenu ne change pas, et la réponse est `false`.
    pub fn ouvrir(&self, cle: &str, ouverture: Ouverture, contenu: impl Contenu) -> bool {
        self.ouvrir_boite(cle, ouverture, Box::new(contenu))
    }

    /// Comme [`AutresFenetres::ouvrir`], pour un contenu déjà en boîte.
    pub fn ouvrir_boite(&self, cle: &str, ouverture: Ouverture, contenu: Box<dyn Contenu>) -> bool {
        let nouvelle = {
            let mut r = verrou(&self.registre);
            if let Some(f) = r.fenetres.get(cle) {
                let mut f = verrou(f);
                if f.fermer {
                    // Fermée mais pas encore partie : elle reste ouverte.
                    f.fermer = false;
                }
                f.devant = true;
                false
            } else {
                let titre = ouverture.titre.clone();
                r.fenetres.insert(
                    cle.into(),
                    Arc::new(Mutex::new(Fenetre {
                        ouverture,
                        contenu,
                        titre,
                        images: 0,
                        montree: false,
                        hwnd: None,
                        sombre: None,
                        voilee: None,
                        devant: false,
                        fermer: false,
                    })),
                );
                true
            }
        };
        self.reveiller();
        nouvelle
    }

    /// Ferme la fenêtre de cette clé (rien si elle n'est pas ouverte).
    pub fn fermer(&self, cle: &str) {
        if let Some(f) = verrou(&self.registre).fenetres.get(cle) {
            verrou(f).fermer = true;
        }
        self.reveiller();
    }

    /// Ferme toutes les fenêtres.
    pub fn tout_fermer(&self) {
        for f in verrou(&self.registre).fenetres.values() {
            verrou(f).fermer = true;
        }
        self.reveiller();
    }

    pub fn ouverte(&self, cle: &str) -> bool {
        verrou(&self.registre)
            .fenetres
            .get(cle)
            .is_some_and(|f| !verrou(f).fermer)
    }

    /// Les clés des fenêtres ouvertes.
    pub fn cles(&self) -> Vec<String> {
        verrou(&self.registre).fenetres.keys().cloned().collect()
    }

    /// Fait passer la fenêtre devant (et la sort de la barre des tâches si elle y est
    /// réduite).
    pub fn devant(&self, cle: &str) {
        if let Some(f) = verrou(&self.registre).fenetres.get(cle) {
            verrou(f).devant = true;
        }
        self.reveiller();
    }

    /// Redessine les fenêtres (l'état du logiciel a changé).
    pub fn redessiner(&self) {
        self.reveiller();
    }

    /// Faut-il une image complète de la fenêtre principale même cachée ? Oui tant qu'une
    /// fenêtre doit s'ouvrir (encore cachée), passer devant ou se fermer : une fenêtre cachée
    /// sans fenêtre visible n'aurait sinon aucune image.
    pub fn a_faire(&self) -> bool {
        verrou(&self.registre).fenetres.values().any(|f| {
            let f = verrou(f);
            !f.montree || f.devant || f.fermer
        })
    }

    /// Dessine les fenêtres ouvertes, à chaque image de la fenêtre principale, et ferme celles
    /// qui le demandent. Aucun verrou n'est tenu pendant le dessin d'un contenu : il peut
    /// ouvrir ou fermer d'autres fenêtres.
    pub fn montrer(&self, ctx: &egui::Context, palette: &Palette) {
        let liste: Vec<(String, Partagee)> = verrou(&self.registre)
            .fenetres
            .iter()
            .map(|(c, f)| (c.clone(), f.clone()))
            .collect();
        for (cle, f) in liste {
            let builder = verrou(&f).ouverture.constructeur();
            let apres = if verrou(&f).fermer {
                Apres::Fermer
            } else {
                ctx.show_viewport_immediate(id_de(&cle), builder, |ui, classe| {
                    verrou(&f).image(ui, classe, palette)
                })
            };
            if apres == Apres::Fermer {
                self.retirer(&cle, &f);
                // La fenêtre principale refait une image : la fenêtre fermée n'y est plus,
                // egui la détruit.
                ctx.request_repaint_of(ViewportId::ROOT);
            }
        }
    }

    fn retirer(&self, cle: &str, f: &Partagee) {
        {
            let mut r = verrou(&self.registre);
            if r.fenetres.get(cle).is_some_and(|g| Arc::ptr_eq(g, f)) {
                r.fenetres.remove(cle);
            }
        }
        let mut f = verrou(f);
        let place = f.place();
        f.contenu.fermee(place);
    }
}

/// Le faux moteur de fenêtres des essais (partagé : egui n'en garde qu'un pour tout le
/// programme).
#[cfg(test)]
pub(crate) mod essais {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        /// Les fenêtres que le faux moteur a dessinées à la dernière image.
        pub(crate) static DESSINEES: RefCell<Vec<ViewportId>> = const { RefCell::new(Vec::new()) };
        /// Les fenêtres dont la croix est cliquée à la prochaine image.
        pub(crate) static CROIX: RefCell<Vec<ViewportId>> = const { RefCell::new(Vec::new()) };
        /// Les ordres que les fenêtres ont donnés (taille, titre…), depuis le début.
        pub(crate) static ORDRES: RefCell<Vec<(ViewportId, ViewportCommand)>> = const { RefCell::new(Vec::new()) };
    }

    /// Un faux moteur de fenêtres : egui croit avoir de vraies fenêtres, rien ne s'ouvre.
    pub(crate) fn contexte() -> egui::Context {
        let ctx = egui::Context::default();
        ctx.set_embed_viewports(false);
        egui::Context::set_immediate_viewport_renderer(|ctx, mut vp| {
            DESSINEES.with(|d| d.borrow_mut().push(vp.ids.this));
            let mut input = egui::RawInput {
                viewport_id: vp.ids.this,
                ..Default::default()
            };
            let mut info = egui::ViewportInfo::default();
            if CROIX.with(|c| c.borrow().contains(&vp.ids.this)) {
                info.events.push(egui::ViewportEvent::Close);
            }
            input.viewports.insert(vp.ids.this, info);
            let mut sortie = ctx.run_ui(input, |ui| (vp.viewport_ui_cb)(ui));
            sortie.textures_delta.clear();
        });
        ctx
    }

    /// Une image de la fenêtre principale ; rend les fenêtres dessinées et celles qu'egui
    /// garde.
    pub(crate) fn image(ctx: &egui::Context, f: &AutresFenetres) -> (Vec<ViewportId>, Vec<ViewportId>) {
        DESSINEES.with(|d| d.borrow_mut().clear());
        let p = Palette::de(&crate::theme::ChoixWindows::default());
        let mut sortie = ctx.run_ui(egui::RawInput::default(), |ui| f.montrer(ui.ctx(), &p));
        sortie.textures_delta.clear();
        ORDRES.with(|o| {
            for (id, v) in &sortie.viewport_output {
                o.borrow_mut().extend(v.commands.iter().map(|c| (*id, c.clone())));
            }
        });
        let dessinees = DESSINEES.with(|d| d.borrow().clone());
        let gardees = sortie
            .viewport_output
            .keys()
            .copied()
            .filter(|i| *i != ViewportId::ROOT)
            .collect();
        (dessinees, gardees)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::essais::{CROIX, contexte, image};
    use super::*;

    #[derive(Clone, Default)]
    struct Trace(Arc<Mutex<Vec<String>>>);

    struct Essai {
        nom: &'static str,
        trace: Trace,
        fermer_a: Option<u32>,
    }

    impl Contenu for Essai {
        fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
            ui.label(self.nom);
            self.trace.0.lock().unwrap().push(format!(
                "{} image {} montrée {}",
                self.nom, cadre.image, cadre.montree
            ));
            if Some(cadre.image) == self.fermer_a {
                cadre.fermer();
            }
        }

        fn fermee(&mut self, _place: Option<Place>) {
            self.trace.0.lock().unwrap().push(format!("{} fermée", self.nom));
        }
    }

    /// Une deuxième fenêtre s'ouvre puis se ferme dans la même boucle, sans en créer une
    /// autre ; elle est dessinée cachée deux fois avant d'être montrée.
    #[test]
    fn deuxieme_fenetre_ouverte_puis_fermee_dans_la_meme_boucle() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        f.lier(Some(ctx.clone()));
        let trace = Trace::default();
        let id = id_de("lecture");
        assert!(f.ouvrir(
            "lecture",
            Ouverture::new("Lecture", [300.0, 200.0]),
            Essai {
                nom: "lecture",
                trace: trace.clone(),
                fermer_a: None,
            },
        ));
        assert!(
            f.a_faire(),
            "à ouvrir : une image même la fenêtre principale cachée"
        );
        let (d, g) = image(&ctx, &f);
        assert_eq!(d, vec![id]);
        assert_eq!(g, vec![id]);
        let _ = image(&ctx, &f);
        assert!(!f.a_faire(), "montrée : plus rien à faire");
        assert!(f.ouverte("lecture"));
        // Une deuxième ouverture de la même clé ne la recrée pas : elle passe devant.
        assert!(!f.ouvrir(
            "lecture",
            Ouverture::new("Lecture", [300.0, 200.0]),
            Essai {
                nom: "autre",
                trace: trace.clone(),
                fermer_a: None,
            },
        ));
        f.fermer("lecture");
        assert!(!f.ouverte("lecture"));
        assert!(f.a_faire());
        let (d, g) = image(&ctx, &f);
        assert!(d.is_empty(), "{d:?}");
        assert!(g.is_empty(), "egui détruit la fenêtre : {g:?}");
        assert!(f.cles().is_empty());
        let t = trace.0.lock().unwrap().clone();
        assert_eq!(
            t,
            vec![
                "lecture image 1 montrée false",
                "lecture image 2 montrée true",
                "lecture fermée",
            ]
        );
    }

    /// Plusieurs fenêtres à la fois ; la croix ou le contenu en ferment une, les autres
    /// restent.
    #[test]
    fn plusieurs_fenetres_et_croix() {
        let ctx = contexte();
        let f = AutresFenetres::default();
        let trace = Trace::default();
        for (cle, fermer_a) in [("a", None), ("b", Some(3)), ("c", None)] {
            f.ouvrir(
                cle,
                Ouverture::new(cle, [200.0, 100.0]),
                Essai {
                    nom: cle,
                    trace: trace.clone(),
                    fermer_a,
                },
            );
        }
        let (d, _) = image(&ctx, &f);
        assert_eq!(d.len(), 3);
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().push(id_de("a")));
        let _ = image(&ctx, &f);
        CROIX.with(|c| c.borrow_mut().clear());
        let (d, g) = image(&ctx, &f);
        assert_eq!(f.cles(), vec!["c".to_string()]);
        assert_eq!(d, vec![id_de("c")]);
        assert_eq!(g, vec![id_de("c")]);
        let t = trace.0.lock().unwrap().clone();
        assert!(t.contains(&"a fermée".to_string()), "{t:?}");
        assert!(t.contains(&"b fermée".to_string()), "{t:?}");
        f.tout_fermer();
        let (_, g) = image(&ctx, &f);
        assert!(g.is_empty());
    }

    /// Un contenu peut ouvrir une autre fenêtre pendant son dessin (aucun verrou tenu).
    #[test]
    fn un_contenu_ouvre_une_autre_fenetre() {
        struct Ouvre(AutresFenetres);
        impl Contenu for Ouvre {
            fn dessiner(&mut self, _: &mut egui::Ui, _: &mut Cadre<'_>) {
                self.0.ouvrir(
                    "fille",
                    Ouverture::new("Fille", [100.0, 100.0]),
                    Essai {
                        nom: "fille",
                        trace: Trace::default(),
                        fermer_a: None,
                    },
                );
            }
        }
        let ctx = contexte();
        let f = AutresFenetres::default();
        f.ouvrir("mere", Ouverture::new("Mère", [100.0, 100.0]), Ouvre(f.clone()));
        let _ = image(&ctx, &f);
        let (d, _) = image(&ctx, &f);
        assert_eq!(d.len(), 2);
    }
}
