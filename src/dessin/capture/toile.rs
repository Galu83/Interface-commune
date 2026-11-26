//! Le petit rendu logiciel des captures : les triangles d'egui posés sur une toile de pixels,
//! avec ses textures (l'atlas des polices, les images). Rien ne passe par OpenGL ni par
//! l'écran : le résultat ne dépend ni de la carte graphique ni de l'échelle de l'écran.

use std::collections::HashMap;

use egui::epaint::Vertex;
use egui::{Color32, TextureId};

use super::Image;

/// Une texture d'egui (l'atlas des polices, une image) : pixels prémultipliés.
pub(super) struct Texture {
    largeur: usize,
    hauteur: usize,
    pixels: Vec<Color32>,
}

pub(super) fn poser(
    textures: &mut HashMap<TextureId, Texture>,
    id: TextureId,
    delta: &egui::epaint::ImageDelta,
) {
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
pub(super) struct Toile {
    largeur: u32,
    hauteur: u32,
    px: Vec<[f32; 4]>,
}

impl Toile {
    pub(super) fn new(largeur: u32, hauteur: u32, fond: Color32) -> Toile {
        let [r, g, b, _] = fond.to_array();
        Toile {
            largeur,
            hauteur,
            px: vec![[f32::from(r), f32::from(g), f32::from(b), 255.0]; (largeur * hauteur) as usize],
        }
    }

    /// Dessine les triangles d'un maillage, limités à `clip` (points).
    pub(super) fn mesh(
        &mut self,
        sommets: &[Vertex],
        indices: &[u32],
        t: &Texture,
        clip: egui::Rect,
        ppp: f32,
    ) {
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

    pub(super) fn image(self) -> Image {
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
}
