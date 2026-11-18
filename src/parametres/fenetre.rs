//! La fenêtre des Paramètres (sous Windows) : « Paramètres de <logiciel> », coins arrondis et
//! barre de titre sombre par Windows, aucune animation à nous (Windows anime déjà les
//! fenêtres), taille et place retenues d'une ouverture à l'autre et toujours en entier sur un
//! écran, taille minimale.
//!
//! Trois façons de l'ouvrir :
//! - avec l'icône de la bibliothèque (`horloge`) : dans sa boucle, à côté du volet
//!   ([`ouvrir_dans`]) ;
//! - dans la boucle d'un logiciel qui a la sienne : ses [`AutresFenetres`] dessinées par sa
//!   fenêtre principale ([`ouvrir_dans_boucle`]), ou [`super::Parametres::dessiner`] dans sa
//!   propre fenêtre ;
//! - seuls, comme fenêtre principale d'une boucle ([`ouvrir_seuls`]) : un logiciel sans autre
//!   fenêtre, ou le secours quand la fenêtre du volet n'a pas pu être préparée.

use std::sync::{Arc, Mutex};

use eframe::egui::{self, FontDefinitions, ViewportCommand};

use super::{CLE_FENETRE, Parametres, TAILLE, TAILLE_MIN};
use crate::apparence::{Apparence, couleur};
use crate::autres_fenetres::{AutresFenetres, Cadre, Contenu, Ouverture};
use crate::fenetres::Liens;
use crate::place::{Place, garder_visible, place_valable};
use crate::systeme::{self, hwnd_de};
use crate::{Logiciel, Reglages};

/// La page que les Paramètres ouverts doivent montrer (une notification, `--parametres`…).
pub type PageVoulue = Arc<Mutex<Option<String>>>;

/// L'icône de la fenêtre (barre des tâches, Alt+Tab), en RVBA : l'icône du logiciel.
pub fn icone_fenetre(l: &dyn Reglages, sombre: bool) -> egui::IconData {
    let t = 32;
    let mut px = (l.identite().icone)(t, sombre);
    if px.len() != (t * t * 4) as usize {
        px = vec![0; (t * t * 4) as usize];
    }
    for p in px.chunks_mut(4) {
        p.swap(0, 2);
    }
    egui::IconData {
        rgba: px,
        width: t,
        height: t,
    }
}

/// La place retenue, ramenée sur l'écran le plus proche (il a pu changer).
fn place_retenue(l: &dyn Reglages) -> Option<Place> {
    l.place_parametres().and_then(place_valable).and_then(|p| {
        let travail = systeme::zone_de_travail_pres_de([p[0], p[1], p[0] + p[2], p[1] + p[3]])?;
        Some(garder_visible(
            p,
            travail,
            [TAILLE_MIN[0] as i32, TAILLE_MIN[1] as i32],
        ))
    })
}

/// Comment s'ouvrent les Paramètres : « Paramètres de <logiciel> », leur taille et leur
/// place retenues (sinon centrés sur l'écran de la souris).
pub fn ouverture(l: &dyn Reglages, sombre: bool) -> Ouverture {
    Ouverture::new(&Parametres::titre_fenetre(l), TAILLE)
        .taille_min(TAILLE_MIN)
        .icone(icone_fenetre(l, sombre))
        .place(place_retenue(l))
}

/// Les Paramètres, contenu d'une fenêtre de la boucle.
pub struct ContenuParametres {
    reglages: Arc<dyn Reglages>,
    voulue: PageVoulue,
    parametres: Parametres,
    /// Le thème de l'icône de la fenêtre.
    sombre: Option<bool>,
}

impl ContenuParametres {
    pub fn new(reglages: Arc<dyn Reglages>, voulue: PageVoulue, page: Option<&str>) -> Self {
        let parametres = Parametres::new(&*reglages, page);
        ContenuParametres {
            reglages,
            voulue,
            parametres,
            sombre: None,
        }
    }
}

impl Contenu for ContenuParametres {
    fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>) {
        let ctx = ui.ctx().clone();
        let p = cadre.palette;
        if self.sombre.is_some_and(|s| s != p.sombre) {
            let icone = icone_fenetre(&*self.reglages, p.sombre);
            ctx.send_viewport_cmd(ViewportCommand::Icon(Some(Arc::new(icone))));
        }
        self.sombre = Some(p.sombre);
        if let Some(page) = self.voulue.lock().ok().and_then(|mut v| v.take()) {
            self.parametres.aller_a(&page);
        }
        // Le titre suit la langue.
        cadre.titre(&Parametres::titre_fenetre(&*self.reglages));
        let l = self.reglages.clone();
        self.parametres.dessiner(ui, p, &*l);
    }

    fn peut_fermer(&mut self, imposee: bool) -> bool {
        self.reglages.peut_fermer(imposee)
    }

    fn fermee(&mut self, place: Option<Place>) {
        if let Some(p) = place {
            self.reglages.retenir_place(p);
        }
    }
}

/// La fenêtre des Paramètres ouverte (son HWND), pour s'en servir de propriétaire : un
/// sélecteur ou une boîte qui s'ouvre au-dessus d'elle et la bloque tant qu'elle est ouverte.
/// À appeler sur le fil des fenêtres (celui des méthodes de [`Reglages`]) ; `None` si elle
/// n'est pas ouverte.
pub fn fenetre_parametres(l: &dyn Reglages) -> Option<isize> {
    systeme::fenetre_du_fil(&Parametres::titre_fenetre(l), &[])
}

/// Le sélecteur de dossier de Windows, **au-dessus des Paramètres** (qui sont son
/// propriétaire : il ne passe jamais derrière eux et les bloque pendant qu'il est ouvert) :
/// le dossier choisi, ou `None` si l'utilisateur annule. À appeler sur le fil des fenêtres,
/// par exemple pour `Reglages::action(cles::DOSSIER)`. `depart` : le dossier montré d'abord,
/// s'il est sur un disque de ce PC ; `titre` : le titre de la boîte.
pub fn choisir_dossier(
    l: &dyn Reglages,
    depart: Option<&std::path::Path>,
    titre: &str,
) -> Option<std::path::PathBuf> {
    systeme::choisir_dossier(fenetre_parametres(l), depart, titre)
}

/// Ouvre les Paramètres à cette page parmi les fenêtres d'une boucle ; déjà ouverts, ils
/// vont à cette page et passent devant. `voulue` : partagée avec le logiciel, pour changer
/// de page plus tard.
pub fn ouvrir_dans_boucle(
    autres: &AutresFenetres,
    reglages: &Arc<dyn Reglages>,
    voulue: &PageVoulue,
    page: Option<String>,
) {
    if autres.ouverte(CLE_FENETRE) {
        if page.is_some()
            && let Ok(mut p) = voulue.lock()
        {
            *p = page;
        }
        autres.devant(CLE_FENETRE);
        return;
    }
    if let Ok(mut p) = voulue.lock() {
        *p = None;
    }
    let sombre = crate::apparence::choix_windows().sombre;
    autres.ouvrir(
        CLE_FENETRE,
        ouverture(&**reglages, sombre),
        ContenuParametres::new(reglages.clone(), voulue.clone(), page.as_deref()),
    );
}

/// Ouvre les Paramètres dans la boucle de la bibliothèque (icône près de l'horloge).
pub fn ouvrir_dans(liens: &Arc<Liens>, logiciel: &Arc<dyn Logiciel>, page: Option<String>) {
    let reglages: Arc<dyn Reglages> = logiciel.clone();
    ouvrir_dans_boucle(&liens.autres, &reglages, &liens.page_voulue, page);
}

/// Les Paramètres seuls, fenêtre principale de leur boucle.
struct Seuls {
    contenu: ContenuParametres,
    apparence: Apparence,
    hwnd: Option<isize>,
    images: u32,
    montree: bool,
    place: Option<Place>,
}

impl eframe::App for Seuls {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        couleur(self.apparence.palette.fond).to_normalized_gamma_f32()
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let change = self.apparence.suivre(&ctx);
        if self.hwnd.is_none() {
            self.hwnd = hwnd_de(frame);
            if let Some(h) = self.hwnd {
                // Voilée jusqu'à ce qu'elle soit habillée et dessinée : eframe la montre
                // d'office à sa première image.
                systeme::voiler(h, true);
                // Glissée vers un écran d'une autre échelle : elle suit la souris.
                crate::echelle::suivre_windows(h);
            }
        }
        if let Some(h) = self.hwnd
            && !self.montree
            && let Some(p) = self.place
        {
            // Posée à chaque image cachée, comme les autres fenêtres : née sur l'écran
            // principal, elle change d'échelle en arrivant sur un écran d'une autre échelle
            // (l'écran du haut d'un portable…) ; la fois suivante lui rend sa taille exacte.
            systeme::poser(h, p);
        }
        if let Some(h) = self.hwnd
            && (change || self.images == 0)
        {
            systeme::habiller(h, self.apparence.palette.sombre);
        }
        self.images += 1;
        if !self.montree && self.images >= crate::autres_fenetres::IMAGES_CACHEES {
            // Habillée et dessinée avant d'être montrée : ni éclair blanc, ni fenêtre vide.
            self.montree = true;
            if let Some(h) = self.hwnd {
                systeme::voiler(h, false);
            }
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        } else if !self.montree {
            ctx.request_repaint();
        }
        if ui.input(|i| i.viewport().close_requested()) {
            if self.contenu.peut_fermer(false) {
                if let Some(h) = self.hwnd
                    && let Some(r) = systeme::rect_fenetre(h)
                {
                    self.contenu
                        .fermee(place_valable([r[0], r[1], r[2] - r[0], r[3] - r[1]]));
                }
            } else {
                // Enregistrement refusé : la fenêtre reste ouverte.
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            }
        }
        let p = self.apparence.palette.clone();
        let mut cadre = Cadre::new(&p, self.hwnd, self.images, self.montree);
        self.contenu.dessiner(ui, &mut cadre);
        if let Some(t) = cadre.titre_demande() {
            let t = t.to_string();
            ctx.send_viewport_cmd(ViewportCommand::Title(t));
        }
        if cadre.fermeture_demandee() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

/// Ouvre les Paramètres seuls à cette page (sa clé), comme fenêtre principale d'une boucle,
/// sur le fil qui appelle ; rend la main quand la fenêtre se ferme. `liens` : pour les
/// redessiner et changer de page depuis un autre fil (`Liens::redessiner`,
/// `Liens::page_voulue`).
pub fn ouvrir_seuls(
    reglages: Arc<dyn Reglages>,
    liens: Option<Arc<Liens>>,
    page: Option<String>,
    polices: FontDefinitions,
) -> Result<(), String> {
    let apparence = Apparence::new(None);
    let nom = reglages.identite().nom.clone();
    let o = ouverture(&*reglages, apparence.palette.sombre);
    let place = o.place;
    let mut options = crate::volet::options(o.constructeur());
    options.centered = place.is_none();
    let _imbriquee = crate::sonde::imbriquee("fenêtre des Paramètres (seule)");
    eframe::run_native(
        &nom,
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_fonts(polices);
            apparence.appliquer(&cc.egui_ctx);
            let voulue = match &liens {
                Some(l) => {
                    if let Ok(mut c) = l.contexte.lock() {
                        *c = Some(cc.egui_ctx.clone());
                    }
                    l.page_voulue.clone()
                }
                None => PageVoulue::default(),
            };
            let contenu = ContenuParametres::new(reglages, voulue, page.as_deref());
            Ok(Box::new(Seuls {
                contenu,
                apparence,
                hwnd: None,
                images: 0,
                montree: false,
                place,
            }))
        }),
    )
    .map_err(|e| e.to_string())
}
