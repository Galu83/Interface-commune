//! La fenêtre de lecture, dessinée sans fenêtre : un texte lu à voix haute, le mot lu
//! surligné et suivi ; un clic sur un mot le rend au logiciel (reprendre la lecture là).
//! Reprise de Vocalis (`interface/src/simples.rs`).

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use egui::epaint::text::{LayoutJob, TextFormat, TextWrapping};
use egui::text_selection::LabelSelectionState;
use egui::{Align, Color32, FontId, Galley, Sense, Stroke, Ui, vec2};

use crate::apparence::couleur;
use crate::theme::{Palette, Rgb};

/// Ce que montre la fenêtre de lecture.
#[derive(Debug, Clone, PartialEq)]
pub struct Lecture {
    pub texte: String,
    /// Chaque mot : [premier caractère, fin] (en caractères, pas en octets).
    pub mots: Vec<[u32; 2]>,
    /// Le mot lu (son rang dans `mots`) : surligné, et la page défile jusqu'à lui.
    pub mot: Option<usize>,
    /// Taille du texte (points).
    pub taille: f32,
    /// Couleur et opacité (0 à 1) du surlignage.
    pub surlignage: Rgb,
    pub opacite: f32,
}

impl Default for Lecture {
    fn default() -> Self {
        Lecture {
            texte: String::new(),
            mots: Vec::new(),
            mot: None,
            taille: 14.0,
            surlignage: Rgb(255, 214, 10),
            opacite: 0.4,
        }
    }
}

impl Lecture {
    /// Un texte et ses mots, découpés aux espaces.
    pub fn de(texte: &str) -> Lecture {
        Lecture {
            texte: texte.into(),
            mots: mots(texte),
            ..Default::default()
        }
    }

    /// Le mot dont la fin est la première à ne pas être avant ce caractère (un clic entre
    /// deux mots donne le suivant).
    pub fn mot_au_caractere(&self, car: usize) -> Option<usize> {
        self.mots.iter().position(|m| car as u32 <= m[1])
    }

    fn surlignage(&self) -> Color32 {
        let Rgb(r, g, b) = self.surlignage;
        let a = (self.opacite + 0.2).clamp(0.15, 1.0);
        Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0) as u8)
    }
}

/// Les mots d'un texte, découpés aux espaces : [premier caractère, fin] de chacun.
pub fn mots(texte: &str) -> Vec<[u32; 2]> {
    let mut v = Vec::new();
    let mut debut: Option<u32> = None;
    let mut n = 0u32;
    for c in texte.chars() {
        if c.is_whitespace() {
            if let Some(d) = debut.take() {
                v.push([d, n]);
            }
        } else if debut.is_none() {
            debut = Some(n);
        }
        n += 1;
    }
    if let Some(d) = debut {
        v.push([d, n]);
    }
    v
}

struct Paragraphe {
    /// Premier caractère du paragraphe, et sa fin.
    debut: usize,
    fin: usize,
    galley: Option<Arc<Galley>>,
    surligne: bool,
}

/// Ce que la fenêtre de lecture garde d'une image à l'autre : la mise en page de chaque
/// paragraphe (refaite seulement pour ceux dont le surlignage change), le mot suivi.
#[derive(Default)]
pub struct EtatLecture {
    /// Empreinte du texte, largeur, taille, thème de la dernière mise en page.
    clef: (u64, i32, i32, bool),
    /// Début de chaque caractère en octets (un de plus que le nombre de caractères).
    octets: Vec<usize>,
    paragraphes: Vec<Paragraphe>,
    mot_dessine: Option<usize>,
    /// Le mot lu à la dernière image : s'il change, la page défile jusqu'à lui.
    mot_suivi: Option<usize>,
    /// Où est le mot lu à l'écran à la dernière image.
    pub mot_vu: Option<egui::Rect>,
    /// La zone visible du texte à la dernière image.
    pub zone_vue: Option<egui::Rect>,
}

fn empreinte(texte: &str) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    texte.hash(&mut h);
    h.finish()
}

impl EtatLecture {
    fn preparer(&mut self, l: &Lecture, largeur: f32, sombre: bool) {
        let clef = (
            empreinte(&l.texte),
            (largeur * 4.0) as i32,
            (l.taille * 4.0) as i32,
            sombre,
        );
        if clef == self.clef && !(self.paragraphes.is_empty() && !l.texte.is_empty()) {
            return;
        }
        if clef.0 != self.clef.0 || self.octets.is_empty() {
            self.octets = l
                .texte
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(l.texte.len()))
                .collect();
        }
        self.clef = clef;
        self.mot_dessine = None;
        self.paragraphes.clear();
        let mut debut = 0usize;
        for ligne in l.texte.split('\n') {
            let n = ligne.chars().count();
            self.paragraphes.push(Paragraphe {
                debut,
                fin: debut + n,
                galley: None,
                surligne: false,
            });
            debut += n + 1;
        }
    }

    fn tranche<'a>(&self, l: &'a Lecture, a: usize, b: usize) -> &'a str {
        let o = |c: usize| self.octets.get(c).copied().unwrap_or(l.texte.len());
        l.texte.get(o(a)..o(b)).unwrap_or("")
    }

    fn mise_en_page(
        &self,
        ui: &Ui,
        p: &Palette,
        l: &Lecture,
        par: &Paragraphe,
        largeur: f32,
        mot: Option<[u32; 2]>,
    ) -> Arc<Galley> {
        let normal = TextFormat {
            font_id: FontId::proportional(l.taille),
            color: couleur(p.texte),
            ..Default::default()
        };
        let mut job = LayoutJob {
            wrap: TextWrapping::wrap_at_width(largeur),
            ..Default::default()
        };
        match mot {
            Some([a, b]) if (a as usize) >= par.debut && (b as usize) <= par.fin && a < b => {
                let (a, b) = (a as usize, b as usize);
                job.append(self.tranche(l, par.debut, a), 0.0, normal.clone());
                let mut vif = normal.clone();
                vif.background = l.surlignage();
                job.append(self.tranche(l, a, b), 0.0, vif);
                job.append(self.tranche(l, b, par.fin), 0.0, normal);
            }
            _ => job.append(self.tranche(l, par.debut, par.fin), 0.0, normal),
        }
        ui.painter().layout_job(job)
    }
}

/// Dessine le texte lu dans ce `Ui`, le mot lu surligné (la page le suit quand il change) ;
/// rend le mot cliqué. Glisser sélectionne le texte (Ctrl+C le copie).
pub fn dessiner_lecture(ui: &mut Ui, p: &Palette, l: &Lecture, e: &mut EtatLecture) -> Option<usize> {
    let marge = 20.0;
    let largeur = (ui.available_width() - 2.0 * marge).max(120.0);
    e.preparer(l, largeur, p.sombre);
    let mot = l.mot.and_then(|i| l.mots.get(i).copied());
    let change = e.mot_dessine != l.mot;
    let contient =
        |par: &Paragraphe| mot.is_some_and(|[a, b]| a as usize >= par.debut && b as usize <= par.fin);
    for i in 0..e.paragraphes.len() {
        let veut = contient(&e.paragraphes[i]);
        let par = &e.paragraphes[i];
        if par.galley.is_none() || (change && (veut || par.surligne)) {
            let g = e.mise_en_page(ui, p, l, par, largeur, if veut { mot } else { None });
            let par = &mut e.paragraphes[i];
            par.galley = Some(g);
            par.surligne = veut;
        }
    }
    e.mot_dessine = l.mot;
    let suivre = e.mot_suivi != l.mot;
    e.mot_suivi = l.mot;
    e.mot_vu = None;
    let mut clic = None;
    let mut zone = None;
    let mut mot_vu = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(marge - 2.0);
            for par in &e.paragraphes {
                let Some(g) = &par.galley else { continue };
                let (rect, r) = ui.allocate_exact_size(
                    vec2(largeur, g.size().y.max(l.taille * 1.3)),
                    Sense::click_and_drag(),
                );
                let rect = rect.translate(vec2(marge, 0.0));
                if ui.is_rect_visible(rect) {
                    LabelSelectionState::label_text_selection(
                        ui,
                        &r,
                        rect.min,
                        g.clone(),
                        couleur(p.texte),
                        Stroke::NONE,
                    );
                }
                if par.surligne
                    && let Some([a, _]) = mot
                {
                    let dans = (a as usize).saturating_sub(par.debut);
                    let c = g
                        .pos_from_cursor(egui::text::CCursor::new(dans))
                        .translate(rect.min.to_vec2());
                    mot_vu = Some(c);
                    if suivre {
                        ui.scroll_to_rect(c, Some(Align::Center));
                    }
                }
                // Un double-clic ne reprend qu'une fois (au premier clic).
                if r.clicked()
                    && !r.double_clicked()
                    && let Some(pos) = r.interact_pointer_pos()
                {
                    let dans: usize = g.cursor_from_pos(pos - rect.min).index.into();
                    clic = l.mot_au_caractere(dans + par.debut);
                }
            }
            zone = Some(ui.clip_rect());
            ui.add_space(marge);
        });
    e.mot_vu = mot_vu;
    e.zone_vue = zone;
    clic
}

#[cfg(test)]
mod tests {
    // Dans un essai, un échec doit arrêter l'essai : `unwrap` et `expect` y sont voulus.
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use crate::dessin::toutes_faites::essais::{cliquer, palette, une_image};

    #[test]
    fn mots_et_clic_entre_deux_mots() {
        let l = Lecture::de("Un  été\nchaud");
        assert_eq!(l.mots, vec![[0, 2], [4, 7], [8, 13]]);
        assert_eq!(l.mot_au_caractere(3), Some(1));
        assert_eq!(l.mot_au_caractere(9), Some(2));
        assert_eq!(l.mot_au_caractere(20), None);
    }

    fn surligne(e: &EtatLecture) -> Vec<String> {
        e.paragraphes
            .iter()
            .filter_map(|p| p.galley.as_ref())
            .flat_map(|g| {
                g.job
                    .sections
                    .iter()
                    .filter(|s| s.format.background != Color32::TRANSPARENT)
                    .map(|s| g.job.text[s.byte_range.start.0..s.byte_range.end.0].to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Le mot lu est surligné (et seulement lui), la page le suit ; un clic sur un mot le
    /// rend.
    #[test]
    fn mot_surligne_suivi_et_clique() {
        let ctx = egui::Context::default();
        let p = palette();
        let paragraphe = "Le soleil se lève sur la colline et les oiseaux chantent.";
        let texte = vec![paragraphe; 80].join("\n");
        let mut l = Lecture::de(&texte);
        let mut e = EtatLecture::default();
        let mut cliques = Vec::new();
        l.mot = Some(1);
        for _ in 0..2 {
            une_image(&ctx, vec![], |ui| {
                cliques.extend(dessiner_lecture(ui, &p, &l, &mut e));
            });
        }
        assert_eq!(surligne(&e), vec!["soleil"]);
        // Un mot loin dans le texte : la page défile jusqu'à lui.
        let loin = l.mots.len() - 2;
        l.mot = Some(loin);
        for _ in 0..20 {
            une_image(&ctx, vec![], |ui| {
                cliques.extend(dessiner_lecture(ui, &p, &l, &mut e));
            });
        }
        assert_eq!(surligne(&e), vec!["oiseaux"]);
        let (vu, zone) = (e.mot_vu.expect("vu"), e.zone_vue.expect("zone"));
        assert!(zone.contains_rect(vu), "{vu:?} dans {zone:?}");
        cliquer(&ctx, vu.center() + vec2(2.0, 0.0), &mut |ui: &mut Ui| {
            cliques.extend(dessiner_lecture(ui, &p, &l, &mut e));
        });
        assert_eq!(cliques, vec![loin]);
    }
}
