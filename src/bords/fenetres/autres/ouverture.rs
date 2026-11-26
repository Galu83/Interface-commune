//! Comment ouvrir une autre fenêtre ([`Ouverture`]), ce qu'elle dessine ([`Contenu`]) et ce
//! qu'elle reçoit à chaque image ([`Cadre`]).

use std::sync::Arc;

use egui::ViewportBuilder;

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
    /// Sans `place`, la bibliothèque centre la fenêtre sur l'écran de la souris à chaque
    /// image cachée ; `false` : le contenu la pose lui-même (près d'une zone, par exemple).
    pub centree: bool,
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
            centree: true,
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

    /// Le contenu pose lui-même la fenêtre (`systeme::poser` sur le `hwnd` du `Cadre`) : la
    /// bibliothèque ne la centre pas avant de la montrer.
    pub fn pose_par_le_contenu(mut self) -> Ouverture {
        self.centree = false;
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
    /// `egui::CentralPanel` les donne.
    fn dessiner(&mut self, ui: &mut egui::Ui, cadre: &mut Cadre<'_>);

    /// La fenêtre va se fermer : le contenu enregistre ce qui attend, et peut **refuser**
    /// (`false`) si l'enregistrement a échoué : la fenêtre reste ouverte, et le contenu montre
    /// pourquoi. Toujours appelé, quelle que soit la façon de fermer, même fenêtre réduite.
    ///
    /// `imposee` : la fermeture ne se discute pas ([`AutresFenetres::fermer`](super::AutresFenetres::fermer) ou
    /// [`AutresFenetres::tout_fermer`](super::AutresFenetres::tout_fermer) : le logiciel se ferme, la session se termine) ; la
    /// réponse est alors ignorée, mais le contenu a pu enregistrer. Pour la croix, Alt+F4 ou
    /// la barre des tâches, `imposee` est `false` et le refus garde la fenêtre.
    fn peut_fermer(&mut self, _imposee: bool) -> bool {
        true
    }

    /// Le contenu est prêt à être vu (sa taille est connue, ce qu'il attend est arrivé) ?
    /// Tant que non, la fenêtre reste cachée, même après ses deux images ; on la redessine
    /// jusqu'à ce que oui. Par défaut : toujours prêt.
    fn prete(&self) -> bool {
        true
    }

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
    pub(super) fermer: bool,
    pub(super) titre: Option<String>,
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
