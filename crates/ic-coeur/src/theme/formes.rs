//! Dessiner une forme simple sans fichier image : la part de chaque pixel.

/// La part du pixel (`x`, `y`) dans une forme (0 à 255), en échantillonnant 4×4 points par
/// pixel (bords lissés) : pour dessiner l'icône d'un logiciel sans fichier image. `s` :
/// pixels par unité de la grille du dessin ; `forme` dit si un point de la grille est dans
/// la forme.
pub fn couverture(x: usize, y: usize, s: f32, forme: impl Fn(f32, f32) -> bool) -> u8 {
    let mut n = 0u32;
    for sy in 0..4 {
        for sx in 0..4 {
            let fx = (x as f32 + (sx as f32 + 0.5) / 4.0) / s;
            let fy = (y as f32 + (sy as f32 + 0.5) / 4.0) / s;
            n += u32::from(forme(fx, fy));
        }
    }
    (n * 255 / 16) as u8
}
