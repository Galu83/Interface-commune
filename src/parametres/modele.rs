//! Le squelette des Paramètres en calcul pur, sans fenêtre (tout est testé) : l'ordre des
//! pages et leurs clés, la page demandée sur la ligne de commande (`--parametres <page>`), la
//! recherche d'un paramètre, la bulle « Annuler » après un changement (règles : interface
//! Windows 11, point 7).

use std::time::{Duration, Instant};

use crate::icones;

/// Une page de la navigation.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// La clé de la page (`--parametres <clé>`) : minuscules, sans espace.
    pub cle: String,
    pub titre: String,
    pub icone: char,
    /// Des mots de plus pour « Rechercher un paramètre » (les titres de ses cartes…).
    pub mots: Vec<String>,
}

impl Page {
    pub fn new(cle: &str, titre: &str, icone: char) -> Page {
        Page {
            cle: cle.into(),
            titre: titre.into(),
            icone,
            mots: Vec::new(),
        }
    }

    /// Ajoute des mots pour la recherche.
    pub fn avec_mots(mut self, mots: &[&str]) -> Page {
        self.mots.extend(mots.iter().map(|m| (*m).to_string()));
        self
    }
}

/// Les clés des pages communes.
pub const ACCUEIL: &str = "accueil";
pub const GENERAL: &str = "general";
pub const MISE_A_JOUR: &str = "mise_a_jour";
pub const A_PROPOS: &str = "a_propos";

/// Les trois pages communes, toujours en bas, dans cet ordre, avec ces noms et ces icônes.
pub fn pages_communes() -> [Page; 3] {
    [
        Page::new(GENERAL, "Général", icones::REGLAGES).avec_mots(&[
            "Démarrer avec Windows",
            "Langue",
            "Notifications",
            "Ouvrir le journal",
            "Rétablir les réglages par défaut",
        ]),
        Page::new(MISE_A_JOUR, "Mise à jour", icones::MISE_A_JOUR).avec_mots(&[
            "Rechercher une mise à jour",
            "Dossier des installateurs",
            "Installer maintenant",
            "Nouveautés",
        ]),
        Page::new(A_PROPOS, "À propos", icones::A_PROPOS).avec_mots(&["Version", "Licence", "Crédits"]),
    ]
}

/// La page « Accueil », facultative, en premier.
pub fn page_accueil() -> Page {
    Page::new(ACCUEIL, "Accueil", icones::ACCUEIL)
}

/// Toute la navigation : l'Accueil s'il est voulu, les pages propres au logiciel (sans
/// doublon avec une clé commune), puis Général, Mise à jour, À propos.
pub fn navigation(accueil: bool, propres: &[Page]) -> Vec<Page> {
    let communes = pages_communes();
    let reservee = |c: &str| c == ACCUEIL || communes.iter().any(|p| p.cle == c);
    let mut v = Vec::new();
    if accueil {
        v.push(page_accueil());
    }
    for p in propres {
        if !reservee(&p.cle) && !v.iter().any(|q: &Page| q.cle == p.cle) {
            v.push(p.clone());
        }
    }
    v.extend(communes);
    v
}

/// Le champ « Rechercher un paramètre » est montré à partir de ce nombre de pages (règles :
/// plus de six).
pub const PAGES_POUR_RECHERCHE: usize = 7;

/// La ligne de commande demande-t-elle les Paramètres ? `None` : non ; `Some(None)` : oui,
/// sans page ; `Some(Some(clé))` : à cette page (`--parametres <page>`, ou
/// `--parametres=<page>`). Une clé est bornée à 40 caractères simples ; une autre valeur
/// compte comme « sans page ».
pub fn page_demandee<S: AsRef<str>>(args: &[S]) -> Option<Option<String>> {
    let valable = |c: &str| {
        let c = c.trim().to_lowercase();
        (!c.is_empty()
            && c.len() <= 40
            && c.chars()
                .all(|x| x.is_ascii_alphanumeric() || x == '_' || x == '-'))
        .then(|| c.replace('-', "_"))
    };
    let mut i = args.iter().map(AsRef::as_ref);
    while let Some(a) = i.next() {
        if let Some(v) = a.strip_prefix("--parametres=") {
            return Some(valable(v));
        }
        if a == "--parametres" {
            return Some(i.next().filter(|v| !v.starts_with("--")).and_then(valable));
        }
    }
    None
}

/// Le rang de la page de cette clé dans la navigation ; une clé inconnue : la première page.
pub fn rang_de_page(pages: &[Page], cle: Option<&str>) -> usize {
    cle.and_then(|c| pages.iter().position(|p| p.cle == c))
        .unwrap_or(0)
}

/// Minuscules sans accents, pour comparer ce qui est tapé à ce qui est écrit.
pub fn simplifier(texte: &str) -> String {
    texte
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'â' | 'ä' | 'á' | 'ã' => 'a',
            'ç' => 'c',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' | 'í' => 'i',
            'ô' | 'ö' | 'ó' | 'õ' => 'o',
            'ù' | 'û' | 'ü' | 'ú' => 'u',
            '\u{a0}' | '\u{202f}' | '’' | '\'' | '-' => ' ',
            c => c,
        })
        .collect()
}

/// Les rangs des pages dont le titre ou un mot contient tous les mots tapés. Rien de tapé :
/// rien.
pub fn chercher(pages: &[Page], tape: &str) -> Vec<usize> {
    let mots: Vec<String> = simplifier(tape).split_whitespace().map(str::to_string).collect();
    if mots.is_empty() {
        return Vec::new();
    }
    pages
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let texte = simplifier(&format!("{} {}", p.titre, p.mots.join(" ")));
            mots.iter().all(|m| texte.contains(m.as_str()))
        })
        .map(|(i, _)| i)
        .collect()
}

/// La valeur d'un réglage, telle que la bulle « Annuler » la rend au logiciel.
#[derive(Debug, Clone, PartialEq)]
pub enum Valeur {
    Booleen(bool),
    Nombre(f64),
    Texte(String),
}

/// La bulle « Annuler » reste ce temps après un changement.
pub const DUREE_ANNULER: Duration = Duration::from_secs(6);

/// Le dernier changement, que la bulle « Annuler » peut défaire.
#[derive(Debug, Clone, PartialEq)]
pub struct Annulable {
    pub cle: String,
    /// Le nom du réglage, montré dans la bulle (« Thème changé »).
    pub titre: String,
    pub avant: Valeur,
    quand: Instant,
}

/// La bulle « Annuler » : chaque changement s'applique et s'enregistre tout de suite (le
/// logiciel s'en charge) ; la bulle garde la valeur d'avant quelques secondes.
#[derive(Debug, Default)]
pub struct Annulation {
    annulable: Option<Annulable>,
}

impl Annulation {
    /// L'utilisateur vient de changer `cle` de `avant` à `apres`. Plusieurs changements de
    /// suite du même réglage (un curseur, une frappe) ne font qu'un : « Annuler » revient à
    /// la valeur d'avant le premier.
    pub fn changer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: &Valeur, maintenant: Instant) {
        let suite = self
            .annulable
            .as_ref()
            .is_some_and(|a| a.cle == cle && maintenant.saturating_duration_since(a.quand) < DUREE_ANNULER);
        if suite {
            if let Some(a) = &mut self.annulable {
                a.quand = maintenant;
                if a.avant == *apres {
                    // Revenu à la valeur de départ : plus rien à annuler.
                    self.annulable = None;
                }
            }
        } else if avant != *apres {
            self.annulable = Some(Annulable {
                cle: cle.to_string(),
                titre: titre.to_string(),
                avant,
                quand: maintenant,
            });
        }
    }

    /// Ce que la bulle propose, tant qu'elle est montrée.
    pub fn a_annuler(&self, maintenant: Instant) -> Option<&Annulable> {
        self.annulable
            .as_ref()
            .filter(|a| maintenant.saturating_duration_since(a.quand) < DUREE_ANNULER)
    }

    /// Le temps qu'il reste à la bulle (pour redessiner quand elle disparaît).
    pub fn reste(&self, maintenant: Instant) -> Option<Duration> {
        self.a_annuler(maintenant)
            .map(|a| DUREE_ANNULER.saturating_sub(maintenant.saturating_duration_since(a.quand)))
    }

    /// « Annuler » est cliqué : rend la clé et la valeur d'avant, à remettre par le logiciel.
    /// Rien si la bulle a déjà disparu.
    pub fn annuler(&mut self, maintenant: Instant) -> Option<(String, Valeur)> {
        let a = self.annulable.take()?;
        (maintenant.saturating_duration_since(a.quand) < DUREE_ANNULER).then_some((a.cle, a.avant))
    }

    /// La bulle est fermée sans annuler.
    pub fn oublier(&mut self) {
        self.annulable = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn propres() -> Vec<Page> {
        vec![
            Page::new("ecrans", "Écrans", '\u{E7F4}').avec_mots(&["Luminosité"]),
            // Une clé réservée : ignorée.
            Page::new("general", "Mon général", '\u{E713}'),
            Page::new("dispositions", "Dispositions", '\u{E8A9}'),
        ]
    }

    /// L'Accueil en premier, les pages propres, puis toujours Général, Mise à jour, À propos,
    /// avec ces noms et ces icônes (E713, E895, E946).
    #[test]
    fn ordre_des_pages() {
        let n = navigation(true, &propres());
        let cles: Vec<&str> = n.iter().map(|p| p.cle.as_str()).collect();
        assert_eq!(
            cles,
            [
                "accueil",
                "ecrans",
                "dispositions",
                "general",
                "mise_a_jour",
                "a_propos"
            ]
        );
        let fin: Vec<(&str, char)> = n[3..].iter().map(|p| (p.titre.as_str(), p.icone)).collect();
        assert_eq!(
            fin,
            [
                ("Général", '\u{E713}'),
                ("Mise à jour", '\u{E895}'),
                ("À propos", '\u{E946}')
            ]
        );
        assert_eq!(navigation(false, &[]).len(), 3);
    }

    #[test]
    fn ligne_de_commande() {
        assert_eq!(page_demandee(&["prog"]), None);
        assert_eq!(page_demandee(&["prog", "--parametres"]), Some(None));
        assert_eq!(
            page_demandee(&["prog", "--parametres", "Mise-a-jour"]),
            Some(Some("mise_a_jour".into()))
        );
        assert_eq!(
            page_demandee(&["--parametres=ecrans"]),
            Some(Some("ecrans".into()))
        );
        assert_eq!(page_demandee(&["--parametres", "--autre"]), Some(None));
        assert_eq!(page_demandee(&["--parametres", "../../x"]), Some(None));
        let long = "a".repeat(41);
        assert_eq!(page_demandee(&["--parametres", long.as_str()]), Some(None));
        let n = navigation(false, &propres());
        assert_eq!(rang_de_page(&n, Some("a_propos")), 4);
        assert_eq!(rang_de_page(&n, Some("inconnue")), 0);
        assert_eq!(rang_de_page(&n, None), 0);
    }

    #[test]
    fn recherche_sans_accents() {
        let n = navigation(true, &propres());
        assert_eq!(chercher(&n, "luminosite"), vec![1]);
        assert_eq!(chercher(&n, "DÉMARRER windows"), vec![3]);
        assert_eq!(chercher(&n, "  "), Vec::<usize>::new());
        assert_eq!(chercher(&n, "introuvable"), Vec::<usize>::new());
    }

    /// Effet immédiat et « Annuler » : la bulle garde la valeur d'avant, disparaît après
    /// quelques secondes ; des changements de suite du même réglage ne font qu'un.
    #[test]
    fn effet_immediat_et_annuler() {
        let t = Instant::now();
        let mut a = Annulation::default();
        let b = Valeur::Booleen;
        a.changer("theme", "Thème", b(false), &b(true), t);
        assert_eq!(a.a_annuler(t).map(|x| x.titre.as_str()), Some("Thème"));
        assert!(a.reste(t + Duration::from_secs(1)).is_some());
        assert_eq!(
            a.annuler(t + Duration::from_secs(1)),
            Some(("theme".into(), b(false)))
        );
        assert_eq!(a.annuler(t + Duration::from_secs(1)), None);
        // Un curseur : 1 → 2 → 3, « Annuler » revient à 1.
        let n = Valeur::Nombre;
        a.changer("vitesse", "Vitesse", n(1.0), &n(2.0), t);
        a.changer(
            "vitesse",
            "Vitesse",
            n(2.0),
            &n(3.0),
            t + Duration::from_millis(300),
        );
        assert_eq!(a.a_annuler(t).map(|x| x.avant.clone()), Some(n(1.0)));
        // Revenu à la valeur de départ : plus rien à annuler.
        a.changer(
            "vitesse",
            "Vitesse",
            n(3.0),
            &n(1.0),
            t + Duration::from_millis(600),
        );
        assert!(a.a_annuler(t + Duration::from_millis(600)).is_none());
        // La bulle disparaît après DUREE_ANNULER ; trop tard pour annuler.
        a.changer(
            "nom",
            "Nom",
            Valeur::Texte("a".into()),
            &Valeur::Texte("b".into()),
            t,
        );
        assert!(a.a_annuler(t + DUREE_ANNULER).is_none());
        assert_eq!(a.annuler(t + DUREE_ANNULER), None);
        // Un changement qui ne change rien n'ouvre pas de bulle.
        a.changer("x", "X", b(true), &b(true), t);
        assert!(a.a_annuler(t).is_none());
    }
}
