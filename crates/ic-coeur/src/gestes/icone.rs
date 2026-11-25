//! L'icône près de l'horloge, en calcul pur : son info-bulle (raccourcie, copiée dans le
//! tampon fixe de Windows sans couper un caractère en deux, C-07) et le délai contre le
//! clignotement quand elle change trop vite. Les appels à Windows restent dans la
//! bibliothèque (`interface_commune::zone` et `horloge`).

use std::time::Duration;

use super::temps::Horodatage;

/// Caractères d'une info-bulle gardés par Windows (127 et le zéro final).
pub const INFOBULLE_MAX: usize = 127;

/// Le texte raccourci à `max` caractères, « … » compris.
pub fn raccourcir(texte: &str, max: usize) -> String {
    if texte.chars().count() <= max {
        texte.to_string()
    } else {
        texte.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    }
}

/// Copie un texte dans un tampon fixe de Windows (UTF-16), coupé pour garder son zéro final.
/// La coupe tombe toujours entre deux caractères : un caractère hors du plan de base (un
/// emoji, deux unités UTF-16) qui ne tient plus en entier n'est pas copié à moitié, Windows
/// montrerait sinon un carré. Un tampon vide reste vide.
pub fn copier(dest: &mut [u16], texte: &str) {
    let Some(place) = dest.len().checked_sub(1) else {
        return;
    };
    let mut n = 0;
    for c in texte.chars() {
        let mut unites = [0u16; 2];
        let unites = c.encode_utf16(&mut unites);
        let fin = n + unites.len();
        if fin > place {
            break;
        }
        if let Some(d) = dest.get_mut(n..fin) {
            d.copy_from_slice(unites);
        }
        n = fin;
    }
    if let Some(d) = dest.get_mut(n) {
        *d = 0;
    }
}

/// Au plus un changement d'icône par seconde (contre le clignotement).
pub const DELAI_ICONE: Duration = Duration::from_secs(1);

/// Le temps à attendre avant de changer l'icône, changée pour la dernière fois à `derniere` ;
/// `None` : tout de suite.
pub fn attente_icone<T: Horodatage>(derniere: Option<T>, maintenant: T) -> Option<Duration> {
    let passe = maintenant.depuis(derniere?)?;
    DELAI_ICONE.checked_sub(passe).filter(|d| !d.is_zero())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_bulle_raccourcie() {
        assert_eq!(raccourcir("Souris", 10), "Souris");
        assert_eq!(raccourcir("Souris MX Master", 8), "Souris …");
        assert_eq!(
            raccourcir(&"é".repeat(200), INFOBULLE_MAX).chars().count(),
            INFOBULLE_MAX
        );
    }

    /// Le texte lu dans un tampon, jusqu'au zéro final.
    fn lu(tampon: &[u16]) -> String {
        let fin = tampon.iter().position(|&u| u == 0).unwrap_or(tampon.len());
        String::from_utf16(&tampon[..fin]).unwrap()
    }

    /// Copié en entier s'il tient, coupé sinon, toujours avec son zéro final ; un caractère de
    /// deux unités n'est jamais coupé en deux (C-07).
    #[test]
    fn copie_dans_le_tampon_de_windows() {
        let mut t = [0xFFFFu16; 8];
        copier(&mut t, "Volume");
        assert_eq!(lu(&t), "Volume");
        assert_eq!(t[6], 0);
        let mut t = [0xFFFFu16; 4];
        copier(&mut t, "Batterie");
        assert_eq!(lu(&t), "Bat");
        assert_eq!(t[3], 0);
        // « 🔋 » prend deux unités : avec deux places libres il tient, avec une il n'est
        // pas copié du tout (jamais une moitié seule).
        let mut t = [0xFFFFu16; 5];
        copier(&mut t, "ab🔋");
        assert_eq!(lu(&t), "ab🔋");
        let mut t = [0xFFFFu16; 4];
        copier(&mut t, "ab🔋");
        assert_eq!(lu(&t), "ab");
        assert_eq!(t[2], 0);
        let mut t = [0xFFFFu16; 4];
        copier(&mut t, "abc🔋");
        assert_eq!(lu(&t), "abc");
        // Une info-bulle de 200 emojis dans le vrai tampon de Windows (128 unités).
        let mut t = [0xFFFFu16; INFOBULLE_MAX + 1];
        copier(&mut t, &"🔋".repeat(200));
        assert_eq!(lu(&t), "🔋".repeat(63));
        let mut vide: [u16; 0] = [];
        copier(&mut vide, "rien");
    }

    /// Au plus un changement d'icône par seconde : le suivant attend la fin du délai.
    #[test]
    fn icone_contre_le_clignotement() {
        let t = Duration::from_secs(1_000);
        assert_eq!(attente_icone(None, t), None);
        assert_eq!(
            attente_icone(Some(t), t + Duration::from_millis(300)),
            Some(Duration::from_millis(700))
        );
        assert_eq!(attente_icone(Some(t), t + DELAI_ICONE), None);
        assert_eq!(attente_icone(Some(t), t + Duration::from_secs(5)), None);
    }
}
