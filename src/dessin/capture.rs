//! Les captures sans fenêtre (bloc I de la refonte, `docs/ORGANIGRAMME.md` §3.2) : egui met
//! la page en forme, puis le rendu logiciel de [`toile`] la dessine en pixels. Rien n'ouvre de
//! fenêtre ni ne lit l'écran ; seuls les PNG demandés sont écrits sur le disque. Ce qu'on
//! capture vient du dessin déjà rangé : le volet et le petit menu ([`super::surface`]), les
//! Paramètres ([`super::parametres`]) et tout dessin passé par [`dessin`] (les fenêtres toutes
//! faites, les contrôles). Repris par [`crate::capture`], à son chemin d'aujourd'hui, avec la
//! documentation pour les logiciels.

mod toile;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::TextureId;
use egui::epaint::Primitive;

use super::parametres::Parametres;
use super::surface;
use crate::apparence::{Apparence, couleur};
use crate::mesures::PARAMETRES_TAILLE;
use crate::parametres::modele;
use crate::theme::Palette;
use crate::theme::Surface;
use crate::{Logiciel, Reglages, polices};
use toile::{Texture, Toile, poser};

/// Ce qui empêche une capture.
pub type Erreur = String;

/// Comment faire une capture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// Thème sombre ou clair.
    pub sombre: bool,
    /// Pixels par point (1 : taille normale ; 2 : deux fois plus de pixels, plus net).
    pub echelle: f32,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            sombre: false,
            echelle: 1.0,
        }
    }
}

/// Une image : pixels RVBA, opaques, lignes de haut en bas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub largeur: u32,
    pub hauteur: u32,
    pub rvba: Vec<u8>,
}

impl Image {
    /// L'image en PNG.
    pub fn png(&self) -> Result<Vec<u8>, Erreur> {
        use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
        let mut sortie = Vec::new();
        PngEncoder::new(&mut sortie)
            .write_image(&self.rvba, self.largeur, self.hauteur, ExtendedColorType::Rgba8)
            .map_err(|e| e.to_string())?;
        Ok(sortie)
    }

    /// Enregistre le PNG (le dossier est créé s'il manque).
    pub fn enregistrer(&self, chemin: &Path) -> Result<(), Erreur> {
        if let Some(d) = chemin.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        std::fs::write(chemin, self.png()?).map_err(|e| e.to_string())
    }

    /// Le pixel (x, y) : rouge, vert, bleu, opacité.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.largeur || y >= self.hauteur {
            return None;
        }
        let i = ((y * self.largeur + x) * 4) as usize;
        self.rvba.get(i..i + 4)?.try_into().ok()
    }
}

/// Une image de ce que dessine `dessin`, dans une fenêtre de `taille` points, sur le fond des
/// fenêtres : pour la documentation d'un élément (un menu, une carte) hors d'une page des
/// Paramètres. `dessin` est appelé à chaque passe (egui en fait plusieurs pour poser les
/// tailles), l'image est celle de la dernière : garder l'état à l'extérieur du rappel.
pub fn dessin(
    taille: egui::Vec2,
    o: Options,
    dessin: impl FnMut(&mut egui::Ui, &Palette),
) -> Result<Image, Erreur> {
    capturer(taille, o, false, dessin)
}

/// Le volet du logiciel, tel qu'il s'ouvre.
pub fn volet(l: &dyn Logiciel, o: Options) -> Result<Image, Erreur> {
    surface_du_logiciel(l, Surface::Volet, o)
}

/// Le volet du logiciel avec une hauteur maximale (`PlacementVolet::hauteur_max`, en points,
/// marges comprises) : aussi haut que son contenu, sans dépasser `hauteur_max` ; au-delà, les
/// briques défilent sous le titre et l'état.
pub fn volet_borne(l: &dyn Logiciel, hauteur_max: f32, o: Options) -> Result<Image, Erreur> {
    let mut taille = egui::Vec2::ZERO;
    // Mesuré dans une fenêtre bien plus petite que le contenu : la taille voulue ne dépend pas
    // de la fenêtre du moment.
    let _ = capturer(egui::vec2(400.0, 120.0), o, true, |ui, p| {
        taille = surface::dessiner_borne(ui, p, Surface::Volet, l, Some(hauteur_max)).1;
    })?;
    if taille.x < 1.0 || taille.y < 1.0 {
        return Err("la surface n'a rien dessiné".into());
    }
    capturer(taille, o, true, |ui, p| {
        surface::dessiner_borne(ui, p, Surface::Volet, l, Some(hauteur_max));
    })
}

/// Le petit menu du logiciel, tel qu'il s'ouvre.
pub fn petit_menu(l: &dyn Logiciel, o: Options) -> Result<Image, Erreur> {
    surface_du_logiciel(l, Surface::Menu, o)
}

/// Les Paramètres à cette page (sa clé ; `None` : la première), à leur taille d'ouverture.
pub fn parametres(l: &dyn Reglages, page: Option<&str>, o: Options) -> Result<Image, Erreur> {
    let mut s = Parametres::new(l, page);
    parametres_de(&mut s, l, o)
}

/// Des Paramètres déjà en route, tels qu'ils sont : par exemple avec la carte « Conserver
/// ces modifications ? » ouverte ([`Parametres::changer_a_confirmer`]), pour la documentation.
pub fn parametres_de(s: &mut Parametres, l: &dyn Reglages, o: Options) -> Result<Image, Erreur> {
    capturer(PARAMETRES_TAILLE.into(), o, false, |ui, p| s.dessiner(ui, p, l))
}

/// Tout : `volet-clair.png`, `petit-menu-sombre.png`, `parametres-<page>-clair.png`… pour le
/// volet, le petit menu et chaque page des Paramètres, en clair et en sombre, dans `dossier`.
/// Rend les fichiers faits.
pub fn toutes(l: &dyn Logiciel, dossier: &Path, echelle: f32) -> Result<Vec<PathBuf>, Erreur> {
    let reglages: &dyn Reglages = l;
    let pages = modele::navigation(reglages.textes(), reglages.accueil(), &reglages.pages());
    let mut faites = Vec::new();
    for (sombre, theme) in [(false, "clair"), (true, "sombre")] {
        let o = Options { sombre, echelle };
        let mut a_faire: Vec<(String, Image)> = vec![
            (format!("volet-{theme}"), volet(l, o)?),
            (format!("petit-menu-{theme}"), petit_menu(l, o)?),
        ];
        for p in &pages {
            a_faire.push((
                format!("parametres-{}-{theme}", p.cle.replace('_', "-")),
                parametres(reglages, Some(&p.cle), o)?,
            ));
        }
        for (nom, image) in a_faire {
            let f = dossier.join(format!("{nom}.png"));
            image.enregistrer(&f)?;
            faites.push(f);
        }
    }
    Ok(faites)
}

/// Le volet ou le petit menu : la taille voulue vient du contenu (une première passe la
/// mesure), puis l'image est faite exactement à cette taille.
fn surface_du_logiciel(l: &dyn Logiciel, s: Surface, o: Options) -> Result<Image, Erreur> {
    let mut taille = egui::Vec2::ZERO;
    let _ = capturer(egui::vec2(900.0, 1400.0), o, true, |ui, p| {
        taille = surface::dessiner(ui, p, s, l).1;
    })?;
    if taille.x < 1.0 || taille.y < 1.0 {
        return Err("la surface n'a rien dessiné".into());
    }
    capturer(taille, o, true, |ui, p| {
        surface::dessiner(ui, p, s, l);
    })
}

/// Met en page puis dessine : `dessin` est appelé à chaque passe (egui en fait plusieurs
/// pour poser les tailles), l'image est celle de la dernière. `menu` : le fond est celui des
/// menus, sinon celui des fenêtres.
fn capturer(
    taille: egui::Vec2,
    o: Options,
    menu: bool,
    mut dessin: impl FnMut(&mut egui::Ui, &Palette),
) -> Result<Image, Erreur> {
    let polices = polices::polices()?;
    let ctx = egui::Context::default();
    ctx.set_fonts(polices);
    let a = Apparence::de(Default::default(), Some(o.sombre));
    a.appliquer(&ctx);
    let p = a.palette.clone();
    let fond = couleur(if menu { p.fond_menu } else { p.fond });
    let echelle = o.echelle.clamp(0.5, 4.0);
    let mut textures: HashMap<TextureId, Texture> = HashMap::new();
    let mut dernier = None;
    for _ in 0..4 {
        let mut entree = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, taille)),
            ..Default::default()
        };
        entree
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(echelle);
        let mut sortie = ctx.run_ui(entree, |ui| {
            // Le fond de la surface, comme celui de la fenêtre.
            egui::Frame::new().fill(fond).show(ui, |ui| {
                ui.set_min_size(taille);
                dessin(ui, &p);
            });
        });
        for (id, deltas) in &sortie.textures_delta.set {
            for delta in deltas {
                poser(&mut textures, *id, delta);
            }
        }
        sortie.textures_delta.clear();
        dernier = Some((std::mem::take(&mut sortie.shapes), sortie.pixels_per_point));
    }
    let (formes, ppp) = dernier.ok_or("aucune image")?;
    let primitives = ctx.tessellate(formes, ppp);
    let largeur = (taille.x * ppp).round().max(1.0) as u32;
    let hauteur = (taille.y * ppp).round().max(1.0) as u32;
    let mut toile = Toile::new(largeur, hauteur, fond);
    for prim in &primitives {
        if let Primitive::Mesh(m) = &prim.primitive
            && let Some(t) = textures.get(&m.texture_id)
        {
            toile.mesh(&m.vertices, &m.indices, t, prim.clip_rect, ppp);
        }
    }
    Ok(toile.image())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    /// Le PNG fait l'aller-retour : mêmes dimensions, mêmes pixels.
    #[test]
    fn png_aller_retour() {
        let image = Image {
            largeur: 3,
            hauteur: 2,
            rvba: [10, 20, 30, 255].repeat(6),
        };
        let octets = image.png().unwrap();
        assert_eq!(&octets[..8], b"\x89PNG\r\n\x1a\n");
        let lue = image::load_from_memory(&octets).unwrap().to_rgba8();
        assert_eq!((lue.width(), lue.height()), (3, 2));
        assert_eq!(lue.as_raw(), &image.rvba);
    }
}
