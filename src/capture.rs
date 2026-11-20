//! Des captures du volet, du petit menu et des Paramètres **pour la documentation**, sans
//! ouvrir aucune fenêtre : egui met la page en forme, puis un petit rendu logiciel
//! (triangles, polices d'egui) dessine l'image, enregistrée en PNG. Rien ne passe par
//! OpenGL ni par l'écran : cela se lance partout, pendant les essais, et le résultat ne
//! dépend ni de la carte graphique ni de l'échelle de l'écran.
//!
//! Le logiciel donne ce qu'il montre : pour la documentation, un logiciel qui parle de
//! données inventées (règle n° 1 : jamais celles du propriétaire). Les polices Segoe de
//! Windows sont nécessaires (sans elles, [`Erreur`] : une capture sans texte ne servirait à
//! rien).
//!
//! ```no_run
//! # fn exemple(logiciel: &dyn interface_commune::Logiciel) -> Result<(), String> {
//! use interface_commune::capture::{self, Options};
//! // Tout : le volet, le petit menu et chaque page des Paramètres, en clair et en sombre.
//! let faites = capture::toutes(logiciel, std::path::Path::new("docs/captures"), 1.0)?;
//! // Ou une seule image :
//! let page = capture::parametres(logiciel, Some("general"), Options::default())?;
//! std::fs::write("general.png", page.png()?).map_err(|e| e.to_string())?;
//! # let _ = faites;
//! # Ok(())
//! # }
//! ```
//!
//! Limite : un rendu simplifié (une texture échantillonnée au plus proche, sans correction
//! de gamma) : les images sont fidèles à la mise en page et aux couleurs, pas au pixel près
//! à ce qu'affiche la carte graphique.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::epaint::{Primitive, Vertex};
use egui::{Color32, TextureId};

use crate::apparence::{Apparence, couleur};
use crate::mesures::PARAMETRES_TAILLE;
use crate::parametres::{Parametres, modele};
use crate::surface;
use crate::theme::Palette;
use crate::theme::Surface;
use crate::{Logiciel, Reglages, polices};

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

/// Une texture d'egui (l'atlas des polices, une image) : pixels prémultipliés.
struct Texture {
    largeur: usize,
    hauteur: usize,
    pixels: Vec<Color32>,
}

fn poser(textures: &mut HashMap<TextureId, Texture>, id: TextureId, delta: &egui::epaint::ImageDelta) {
    let egui::ImageData::Color(image) = &delta.image;
    let [l, h] = image.size;
    match delta.pos {
        None => {
            textures.insert(
                id,
                Texture {
                    largeur: l,
                    hauteur: h,
                    pixels: image.pixels.clone(),
                },
            );
        }
        Some([x0, y0]) => {
            let Some(t) = textures.get_mut(&id) else { return };
            for y in 0..h {
                for x in 0..l {
                    let (tx, ty) = (x0 + x, y0 + y);
                    if tx < t.largeur
                        && ty < t.hauteur
                        && let Some(p) = image.pixels.get(y * l + x)
                    {
                        t.pixels[ty * t.largeur + tx] = *p;
                    }
                }
            }
        }
    }
}

/// L'image en construction : pixels prémultipliés, d'abord tous de la couleur du fond.
struct Toile {
    largeur: u32,
    hauteur: u32,
    px: Vec<[f32; 4]>,
}

impl Toile {
    fn new(largeur: u32, hauteur: u32, fond: Color32) -> Toile {
        let [r, g, b, _] = fond.to_array();
        Toile {
            largeur,
            hauteur,
            px: vec![[f32::from(r), f32::from(g), f32::from(b), 255.0]; (largeur * hauteur) as usize],
        }
    }

    /// Dessine les triangles d'un maillage, limités à `clip` (points).
    fn mesh(&mut self, sommets: &[Vertex], indices: &[u32], t: &Texture, clip: egui::Rect, ppp: f32) {
        let (cx0, cy0) = (
            (clip.min.x * ppp).floor().max(0.0),
            (clip.min.y * ppp).floor().max(0.0),
        );
        let cx1 = (clip.max.x * ppp).ceil().min(self.largeur as f32);
        let cy1 = (clip.max.y * ppp).ceil().min(self.hauteur as f32);
        for tri in indices.as_chunks::<3>().0 {
            let (Some(a), Some(b), Some(c)) = (
                sommets.get(tri[0] as usize),
                sommets.get(tri[1] as usize),
                sommets.get(tri[2] as usize),
            ) else {
                continue;
            };
            self.triangle([a, b, c], t, ppp, [cx0, cy0, cx1, cy1]);
        }
    }

    fn triangle(&mut self, [a, b, c]: [&Vertex; 3], t: &Texture, ppp: f32, clip: [f32; 4]) {
        let p = |v: &Vertex| (f64::from(v.pos.x * ppp), f64::from(v.pos.y * ppp));
        let (pa, mut pb, mut pc) = (p(a), p(b), p(c));
        let (a, mut b, mut c) = (a, b, c);
        let aire = |p: (f64, f64), q: (f64, f64), r: (f64, f64)| {
            (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0)
        };
        let mut total = aire(pa, pb, pc);
        if total == 0.0 || !total.is_finite() {
            return;
        }
        if total < 0.0 {
            // Toujours dans le même sens : la règle « haut-gauche » en dépend.
            std::mem::swap(&mut pb, &mut pc);
            std::mem::swap(&mut b, &mut c);
            total = -total;
        }
        let x0 = (pa.0.min(pb.0).min(pc.0).floor() as f32).max(clip[0]) as i64;
        let y0 = (pa.1.min(pb.1).min(pc.1).floor() as f32).max(clip[1]) as i64;
        let x1 = (pa.0.max(pb.0).max(pc.0).ceil() as f32).min(clip[2]) as i64;
        let y1 = (pa.1.max(pb.1).max(pc.1).ceil() as f32).min(clip[3]) as i64;
        // Un pixel est à ce triangle si son centre est dedans ; sur une arête, seulement si
        // c'est une arête « haut » ou « gauche » : deux triangles voisins ne dessinent jamais
        // deux fois le même pixel (ce qui doublerait un fond translucide).
        let compte = |w: f64, de: (f64, f64), vers: (f64, f64)| {
            w > 0.0 || (w == 0.0 && ((vers.1 == de.1 && vers.0 < de.0) || vers.1 > de.1))
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let q = (x as f64 + 0.5, y as f64 + 0.5);
                let (wa, wb, wc) = (aire(pb, pc, q), aire(pc, pa, q), aire(pa, pb, q));
                if !(compte(wa, pb, pc) && compte(wb, pc, pa) && compte(wc, pa, pb)) {
                    continue;
                }
                let (ka, kb, kc) = ((wa / total) as f32, (wb / total) as f32, (wc / total) as f32);
                let u = ka * a.uv.x + kb * b.uv.x + kc * c.uv.x;
                let v = ka * a.uv.y + kb * b.uv.y + kc * c.uv.y;
                let tx = ((u * t.largeur as f32) as isize).clamp(0, t.largeur as isize - 1) as usize;
                let ty = ((v * t.hauteur as f32) as isize).clamp(0, t.hauteur as isize - 1) as usize;
                let Some(texel) = t.pixels.get(ty * t.largeur + tx) else {
                    continue;
                };
                let mut src = [0.0f32; 4];
                let (ca, cb, cc) = (a.color.to_array(), b.color.to_array(), c.color.to_array());
                let tex = texel.to_array();
                for i in 0..4 {
                    let sommet = ka * f32::from(ca[i]) + kb * f32::from(cb[i]) + kc * f32::from(cc[i]);
                    src[i] = sommet * f32::from(tex[i]) / 255.0;
                }
                if src[3] <= 0.0 {
                    continue;
                }
                let i = (y as u32 * self.largeur + x as u32) as usize;
                let Some(dst) = self.px.get_mut(i) else { continue };
                let reste = 1.0 - src[3] / 255.0;
                for k in 0..4 {
                    dst[k] = src[k] + dst[k] * reste;
                }
            }
        }
    }

    fn image(self) -> Image {
        let mut rvba = Vec::with_capacity(self.px.len() * 4);
        for [r, g, b, _] in self.px {
            // Le fond est opaque : la couleur prémultipliée est la couleur.
            rvba.extend([r, g, b].map(|c| c.round().clamp(0.0, 255.0) as u8));
            rvba.push(255);
        }
        Image {
            largeur: self.largeur,
            hauteur: self.hauteur,
            rvba,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;

    fn faux_texel() -> Texture {
        Texture {
            largeur: 1,
            hauteur: 1,
            pixels: vec![Color32::WHITE],
        }
    }

    fn sommet(x: f32, y: f32, c: Color32) -> Vertex {
        Vertex {
            pos: egui::pos2(x, y),
            uv: egui::pos2(0.0, 0.0),
            color: c,
        }
    }

    /// Un rectangle translucide, fait de deux triangles, est blanchi une seule fois partout
    /// (même sur la diagonale commune), et ne dépasse pas.
    #[test]
    fn deux_triangles_ne_se_superposent_pas() {
        let mut t = Toile::new(8, 8, Color32::BLACK);
        // Blanc à moitié transparent, prémultiplié.
        let c = Color32::from_rgba_premultiplied(128, 128, 128, 128);
        let s = [
            sommet(2.0, 2.0, c),
            sommet(6.0, 2.0, c),
            sommet(6.0, 6.0, c),
            sommet(2.0, 6.0, c),
        ];
        t.mesh(
            &s,
            &[0, 1, 2, 0, 2, 3],
            &faux_texel(),
            egui::Rect::EVERYTHING,
            1.0,
        );
        let image = t.image();
        for y in 2..6 {
            for x in 2..6 {
                assert_eq!(image.pixel(x, y), Some([128, 128, 128, 255]), "({x}, {y})");
            }
        }
        assert_eq!(image.pixel(1, 3), Some([0, 0, 0, 255]));
        assert_eq!(image.pixel(6, 3), Some([0, 0, 0, 255]));
        assert_eq!(image.pixel(3, 6), Some([0, 0, 0, 255]));
        assert_eq!(image.pixel(8, 0), None);
    }

    /// Un maillage limité par son rectangle de découpe ne dessine pas au-delà.
    #[test]
    fn decoupe_respectee() {
        let mut t = Toile::new(6, 6, Color32::BLACK);
        let b = Color32::WHITE;
        let s = [
            sommet(0.0, 0.0, b),
            sommet(6.0, 0.0, b),
            sommet(6.0, 6.0, b),
            sommet(0.0, 6.0, b),
        ];
        let clip = egui::Rect::from_min_max(egui::pos2(1.0, 1.0), egui::pos2(3.0, 3.0));
        t.mesh(&s, &[0, 1, 2, 0, 2, 3], &faux_texel(), clip, 1.0);
        let image = t.image();
        assert_eq!(image.pixel(1, 1), Some([255, 255, 255, 255]));
        assert_eq!(image.pixel(2, 2), Some([255, 255, 255, 255]));
        assert_eq!(image.pixel(3, 3), Some([0, 0, 0, 255]));
        assert_eq!(image.pixel(0, 0), Some([0, 0, 0, 255]));
    }

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
