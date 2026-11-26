//! Ce qu'une page du logiciel reçoit pour se dessiner ([`Contexte`]), et ce que les
//! Paramètres font ensuite de ce qu'elle a demandé ([`Parametres::avec_contexte`]).

use std::time::Instant;

use super::Parametres;
use crate::Reglages;
use crate::dessin;
use crate::parametres::modele::{self, Changement, Valeur};
use crate::textes::Textes;
use crate::theme::Palette;
use ic_coeur::parametres::envois;

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
    /// Les « effets d'animation » de Windows : sans eux, le chevron tourne d'un coup.
    animations: bool,
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
    /// refermer). Mettre aussi ces `mots` dans [`modele::Page::avec_mots`], sinon la recherche ne
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
        let duree = if self.animations {
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

impl Parametres {
    /// Fait dessiner le logiciel (une page propre, ses cartes de Général), puis applique ce
    /// qu'il a demandé : changements, redémarrage, sous-page, autre page.
    pub(super) fn avec_contexte(
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
            animations: self.animations,
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
        envois::noter_redemarrage(&mut self.redemarrage, c.redemarrage);
        if let Some(s) = c.sous_page_voulue {
            self.sous_page = s;
        }
        if let Some(a) = c.aller_a {
            self.aller_a(&a);
        }
    }
}
