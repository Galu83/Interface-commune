//! Les autres fenêtres du logiciel dans la boucle de la bibliothèque ([`AutresFenetres`]) :
//! chacune est une fenêtre de plus d'egui, dessinée avec la fenêtre principale ; le détail et un
//! exemple : [`crate::autres_fenetres`].
//!
//! - `ouverture` : comment ouvrir une fenêtre, ce qu'elle dessine, ce qu'elle reçoit ;
//! - `delais` : les attentes avant une création (création ratée, affichage changé,
//!   destruction récente) ;
//! - `essais` : les essais, sur le faux moteur egui (`super::faux_moteur`).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::bords::verrou::verrou;

use egui::{ViewportBuilder, ViewportClass, ViewportCommand, ViewportId};

use crate::place::Place;
use crate::theme::Palette;

mod delais;
mod ouverture;

#[cfg(test)]
mod essais;

pub use delais::{
    APRES_DESTRUCTION, CREATIONS_MAX, DESTRUCTION_MAX, REPRISE, affichage_change, attente_apres_destruction,
    attente_avant_creation, depuis_le_changement,
};
pub use ouverture::{Cadre, Contenu, Ouverture};

use delais::{Destructions, NOUVEL_ESSAI, SUIVI_DE_LA_REPRISE};

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
    /// Le contenu a déjà été consulté sur cette fermeture (`Contenu::peut_fermer`).
    consulte: bool,
    /// Les créations ratées de suite (eframe n'a pas pu créer la fenêtre de Windows ou son
    /// contexte OpenGL) : voir [`CREATIONS_MAX`].
    ratees: u32,
    /// Pas de nouvel essai de création avant cet instant (après une création ratée).
    pas_avant: Option<std::time::Instant>,
}

/// Ce que la fenêtre devient après une image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Apres {
    Garder,
    Fermer,
}

/// Mode d'essai : les fenêtres sont créées et dessinées comme d'habitude, mais jamais montrées
/// ni mises au premier plan ([`garder_cachees`]).
// ÉTAT GLOBAL : posé par `garder_cachees()`, fonction publique des essais sans poignée vers
// la boucle que la bibliothèque crée elle-même.
static GARDER_CACHEES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Pour les essais seulement : `true`, les autres fenêtres (les Paramètres…) sont créées et
/// dessinées en entier (egui, OpenGL), mais **jamais montrées** : rien n'apparaît à l'écran et
/// le clavier reste à l'application en cours. `false` revient au fonctionnement normal.
pub fn garder_cachees(oui: bool) {
    GARDER_CACHEES.store(oui, std::sync::atomic::Ordering::Release);
}

/// Les images dessinées cachées avant de montrer la fenêtre : la première pour la mettre en
/// page, la deuxième pour la dessiner à sa place et à sa taille.
pub const IMAGES_CACHEES: u32 = 2;

impl Fenetre {
    /// Combien de temps la création attend encore avant son nouvel essai (`None` : elle peut
    /// être tentée). Seule une fenêtre jamais dessinée attend : une fenêtre déjà créée est
    /// dessinée à chaque image, egui détruirait celle qui ne l'est pas.
    fn attente(
        &self,
        maintenant: std::time::Instant,
        destruction: Option<std::time::Duration>,
    ) -> Option<std::time::Duration> {
        if self.images > 0 || self.montree {
            return None;
        }
        let essai = self.pas_avant.and_then(|t| t.checked_duration_since(maintenant));
        attente_avant_creation(depuis_le_changement(), essai, destruction)
    }

    fn image(&mut self, ui: &mut egui::Ui, classe: ViewportClass, palette: &Palette) -> Apres {
        let ctx = ui.ctx().clone();
        if self.fermer {
            return Apres::Fermer;
        }
        if ui.input(|i| i.viewport().close_requested()) {
            // La croix, Alt+F4, la barre des tâches : le contenu enregistre, ou refuse.
            if self.contenu.peut_fermer(false) {
                self.consulte = true;
                return Apres::Fermer;
            }
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        }
        self.images = self.images.saturating_add(1);
        let reelle = !matches!(classe, ViewportClass::EmbeddedWindow);
        #[cfg(windows)]
        if reelle {
            self.windows(palette);
        }
        if !self.montree {
            if reelle
                && (self.images < IMAGES_CACHEES
                    || !self.contenu.prete()
                    || GARDER_CACHEES.load(std::sync::atomic::Ordering::Acquire))
            {
                ctx.request_repaint();
            } else {
                self.montree = true;
                #[cfg(windows)]
                self.voiler(false);
                #[cfg(windows)]
                let sans_focus = !self.ouverture.focus && self.hwnd.is_some();
                #[cfg(not(windows))]
                let sans_focus = false;
                if sans_focus {
                    // Jamais le premier plan : `Visible(true)` activerait la fenêtre.
                    #[cfg(windows)]
                    if let Some(h) = self.hwnd {
                        crate::systeme::montrer_sans_activer(h);
                    }
                } else {
                    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    if self.ouverture.focus {
                        ctx.send_viewport_cmd(ViewportCommand::Focus);
                    }
                }
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
            let mut prises = verrou(&PRISES);
            self.hwnd = systeme::fenetre_du_fil(&self.titre, &prises);
            if let Some(h) = self.hwnd {
                prises.push(h);
                // Glissée vers un écran d'une autre échelle : elle suit la souris
                // (`crate::echelle`).
                crate::echelle::suivre_windows(h);
            }
        }
        let Some(h) = self.hwnd else { return };
        // winit réécrit les styles de la fenêtre à chaque changement de ses réglages (visibilité,
        // boutons…) : ceux qu'il ne connaît pas sont donc reposés à chaque image (sans effet si
        // déjà faits), avant de la montrer.
        if !self.ouverture.focus {
            systeme::sans_focus(h, !self.ouverture.barre_des_taches);
        }
        if !self.ouverture.cadre {
            systeme::sans_cadre(h);
        }
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
                None if self.ouverture.centree => {
                    if let Some(t) = systeme::taille_fenetre(h)
                        && let Some(p) = systeme::centree(t)
                    {
                        systeme::deplacer(h, p[0], p[1], false);
                    }
                }
                None => {}
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

/// Les fenêtres de Windows déjà attribuées à une fenêtre du logiciel (deux fenêtres du même titre
/// ne se confondent pas).
// ÉTAT GLOBAL : les fenêtres de Windows sont à tout le processus ; deux boucles (recréation
// après panique) ne doivent pas se prendre la même.
static PRISES: Mutex<Vec<isize>> = Mutex::new(Vec::new());

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
    /// La fenêtre principale de la boucle (le volet, cachée au repos), 0 tant qu'elle n'est pas
    /// prête : une demande venue d'un autre fil la réveille.
    racine: Arc<std::sync::atomic::AtomicIsize>,
    /// Les fenêtres retirées qui n'ont pas encore disparu (les créations attendent).
    destructions: Arc<Mutex<Destructions>>,
    /// L'heure du réveil déjà programmé de la fenêtre principale ([`AutresFenetres::reveiller_dans`]).
    reveil_a: Arc<Mutex<Option<std::time::Instant>>>,
    /// Les réveils programmés qui ont sonné (essais).
    #[cfg(test)]
    reveils: Arc<std::sync::atomic::AtomicU32>,
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

    /// La fenêtre principale de la boucle (0 : aucune), fixée par le volet.
    pub(crate) fn racine(&self, h: isize) {
        self.racine.store(h, std::sync::atomic::Ordering::Release);
    }

    /// Réveille la fenêtre principale, pour qu'elle ouvre, ferme ou redessine. Cachée au repos,
    /// elle ne répond pas à une simple demande de dessin venue d'un autre fil : elle est
    /// montrée voilée (rien n'apparaît), dessine, puis se recache.
    ///
    /// Pendant un réveil de Windows (`bords::reprise`), elle n'est pas montrée : le réveil
    /// est remis à plus tard.
    pub fn reveiller(&self) {
        if let Some(c) = verrou(&self.contexte).as_ref() {
            c.request_repaint_of(ViewportId::ROOT);
        }
        if let Some(d) = crate::bords::reprise::attente_avant_dessin() {
            self.reveiller_dans(d);
            return;
        }
        #[cfg(windows)]
        crate::systeme::reveiller(self.racine.load(std::sync::atomic::Ordering::Acquire));
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
                        consulte: false,
                        ratees: 0,
                        pas_avant: None,
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
        self.prochaine_image().is_some()
    }

    /// Dans combien de temps la fenêtre principale doit faire une image complète, même
    /// cachée (`None` : rien à faire). `Some(0)` : tout de suite ; plus : une fenêtre dont la
    /// création a raté attend son nouvel essai (`NOUVEL_ESSAI`) au lieu de le refaire à
    /// chaque image.
    pub fn prochaine_image(&self) -> Option<std::time::Duration> {
        let maintenant = std::time::Instant::now();
        let destruction = self.attente_destruction();
        verrou(&self.registre)
            .fenetres
            .values()
            .filter_map(|f| {
                let f = verrou(f);
                if f.fermer {
                    return Some(std::time::Duration::ZERO);
                }
                if let Some(d) = f.attente(maintenant, destruction) {
                    return Some(d);
                }
                (!f.montree || f.devant).then_some(std::time::Duration::ZERO)
            })
            .min()
    }

    /// L'attente due aux destructions en cours, maintenant.
    fn attente_destruction(&self) -> Option<std::time::Duration> {
        #[cfg(windows)]
        let existe = crate::systeme::fenetre_existe;
        #[cfg(not(windows))]
        let existe = |_: isize| false;
        verrou(&self.destructions).attente(std::time::Instant::now(), existe)
    }

    /// Réveille la fenêtre principale dans `d`, **par [`AutresFenetres::reveiller`]** et pas
    /// par une minuterie de repeinture d'egui.
    ///
    /// Une fenêtre ne peut être créée que dans une image que winit fait dans son
    /// contexte (un message de la fenêtre) ; eframe peint les fenêtres cachées hors de ce
    /// contexte (minuterie, toutes les 100 ms) : une création tentée là ne trouve pas la
    /// boucle de winit, eframe rend la main sans dessiner et egui panique (« the user callback
    /// was never called »). Le réveil de `reveiller` passe par un message de la fenêtre : il
    /// est dans le bon contexte. Un seul réveil programmé à la fois (le plus proche) : s'il
    /// reste quelque chose à attendre, l'image qu'il provoque en reprogramme un.
    pub fn reveiller_dans(&self, d: std::time::Duration) {
        let heure = std::time::Instant::now() + d;
        {
            let mut a = verrou(&self.reveil_a);
            if a.is_some_and(|t| t <= heure) {
                return;
            }
            *a = Some(heure);
        }
        let moi = self.clone();
        let _ = std::thread::Builder::new()
            .name("reveil-des-fenetres".into())
            .spawn(move || {
                std::thread::sleep(d);
                {
                    let mut a = verrou(&moi.reveil_a);
                    if *a == Some(heure) {
                        *a = None;
                    }
                }
                #[cfg(test)]
                moi.reveils.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                moi.reveiller();
            });
    }

    /// Essais : les fenêtres n'attendent plus leur nouvel essai de création.
    #[cfg(test)]
    fn sans_attente(&self) {
        for f in verrou(&self.registre).fenetres.values() {
            verrou(f).pas_avant = None;
        }
    }

    /// Dessine les fenêtres ouvertes, à chaque image de la fenêtre principale, et ferme celles
    /// qui le demandent. Aucun verrou n'est tenu pendant le dessin d'un contenu : il peut
    /// ouvrir ou fermer d'autres fenêtres.
    pub fn montrer(&self, ctx: &egui::Context, palette: &Palette) {
        // Une fenêtre retirée qui n'a pas encore disparu : constatée dès qu'elle disparaît, sans
        // attendre un clic.
        let destruction = self.attente_destruction();
        if let Some(d) = destruction {
            self.reveiller_dans(d);
        }
        let liste: Vec<(String, Partagee)> = verrou(&self.registre)
            .fenetres
            .iter()
            .map(|(c, f)| (c.clone(), f.clone()))
            .collect();
        for (cle, f) in liste {
            let builder = verrou(&f).ouverture.constructeur();
            let apres = if verrou(&f).fermer {
                Apres::Fermer
            } else if let Some(d) = verrou(&f).attente(std::time::Instant::now(), destruction) {
                // Création différée (nouvel essai, destruction en cours, changement
                // d'affichage) : `show_viewport_immediate` n'est PAS appelée pour cette
                // fenêtre dans cette image (une fenêtre jamais dessinée n'existe pas encore
                // pour egui : rien n'est détruit), et un réveil par message la reprend.
                self.reveiller_dans(d);
                continue;
            } else {
                let premiere = {
                    let g = verrou(&f);
                    g.images == 0 && !g.montree
                };
                let debut = std::time::Instant::now();
                // La création de la fenêtre se passe dans cet appel : le chien de garde écrit
                // au journal celle qui ne finit pas.
                let _creation = premiere.then(|| {
                    let titre = verrou(&f).titre.clone();
                    crate::sonde::creation(&titre)
                });
                let apres = self.image_ou_essai(ctx, &cle, &f, builder, palette);
                // Le premier dessin après un changement d'affichage, mesuré (la reprise s'est
                // bien passée, ou non).
                if premiere
                    && let Some(depuis) = depuis_le_changement()
                    && depuis < SUIVI_DE_LA_REPRISE
                {
                    let g = verrou(&f);
                    if g.images > 0 {
                        crate::sonde::noter(format!(
                            "Fenêtre « {} » : premier dessin après le changement d'affichage de Windows : {} ms (le changement date de {} ms).",
                            g.titre,
                            debut.elapsed().as_millis(),
                            depuis.as_millis()
                        ));
                    }
                }
                apres
            };
            if apres == Apres::Fermer {
                self.retirer(&cle, &f);
                // La fenêtre principale refait une image : la fenêtre fermée n'y est plus,
                // egui la détruit.
                ctx.request_repaint_of(ViewportId::ROOT);
            }
        }
    }

    /// Une image de la fenêtre ; si eframe n'a pas pu la créer, un nouvel essai plus tard.
    ///
    /// Quand eframe ne peut pas créer la fenêtre de Windows ou son contexte OpenGL (Windows
    /// refait l'affichage : un écran retiré ou rendu, une carte graphique qui se relance), il
    /// rend la main sans dessiner, et egui panique (« the user callback was never called »).
    /// Sans ce garde-fou, la panique emportait toute la boucle des fenêtres (le volet et le
    /// petit menu étaient recréés, à froid, au milieu du changement d'affichage). Ici, la
    /// fenêtre attend [`NOUVEL_ESSAI`] et réessaie ; après [`CREATIONS_MAX`] échecs de suite,
    /// elle est fermée. Une panique du contenu lui-même (son dessin a commencé) suit son cours.
    fn image_ou_essai(
        &self,
        ctx: &egui::Context,
        cle: &str,
        f: &Partagee,
        builder: ViewportBuilder,
        palette: &Palette,
    ) -> Apres {
        let dessinee = std::cell::Cell::new(false);
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.show_viewport_immediate(id_de(cle), builder, |ui, classe| {
                dessinee.set(true);
                verrou(f).image(ui, classe, palette)
            })
        }));
        match r {
            Ok(apres) => {
                let mut g = verrou(f);
                g.ratees = 0;
                g.pas_avant = None;
                apres
            }
            Err(p) if dessinee.get() => std::panic::resume_unwind(p),
            Err(_) => {
                // Panique rattrapée : son lieu ne doit pas être pris, plus tard, pour celui
                // d'une autre panique (la sonde le garde pour le journal).
                let _ = crate::sonde::lieu_de_la_derniere_panique();
                let (n, titre) = {
                    let mut g = verrou(f);
                    g.ratees = g.ratees.saturating_add(1);
                    g.pas_avant = Some(std::time::Instant::now() + NOUVEL_ESSAI);
                    (g.ratees, g.titre.clone())
                };
                if n >= CREATIONS_MAX {
                    crate::sonde::noter(format!(
                        "Fenêtre « {titre} » : Windows n'a pas pu la créer ({n} essais de suite) ; fermée."
                    ));
                    Apres::Fermer
                } else {
                    crate::sonde::noter(format!(
                        "Fenêtre « {titre} » : Windows n'a pas pu la créer (essai {n} sur {CREATIONS_MAX}) ; nouvel essai dans {} ms.",
                        NOUVEL_ESSAI.as_millis()
                    ));
                    self.reveiller_dans(NOUVEL_ESSAI);
                    Apres::Garder
                }
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
        if !f.consulte {
            // Fermée par le logiciel ou par son contenu : le refus ne compte pas.
            let _ = f.contenu.peut_fermer(true);
        }
        if let Some(h) = f.hwnd {
            verrou(&PRISES).retain(|x| *x != h);
            // egui détruit la fenêtre à la fin de l'image : les créations suivantes attendent.
            verrou(&self.destructions).retiree(h, std::time::Instant::now());
        }
        let place = f.place();
        f.contenu.fermee(place);
    }
}
