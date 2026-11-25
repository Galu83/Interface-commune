//! Les pages des Paramètres : leurs clés, leur ordre, la page demandée sur la ligne de commande
//! (`--parametres <page>`), et les pages qui arrivent après l'ouverture (la page montrée reste
//! si elle existe encore, sinon Général).

use crate::contrat::Page;
use crate::textes::Textes;
use crate::theme::icones;

/// Les clés des pages communes.
pub const ACCUEIL: &str = "accueil";
pub const GENERAL: &str = "general";
pub const MISE_A_JOUR: &str = "mise_a_jour";
pub const A_PROPOS: &str = "a_propos";

/// Les trois pages communes, toujours en bas, dans cet ordre, avec ces noms (dans la langue
/// de `t`) et ces icônes.
pub fn pages_communes(t: &Textes) -> [Page; 3] {
    [
        Page::new(GENERAL, t.general, icones::REGLAGES).avec_mots(&[
            t.demarrer_avec_windows,
            t.langue,
            t.notifications,
            t.ouvrir_le_journal,
            t.retablir,
        ]),
        Page::new(MISE_A_JOUR, t.mise_a_jour, icones::MISE_A_JOUR).avec_mots(&[
            t.rechercher_mise_a_jour,
            t.dossier_installateurs,
            t.installer_maintenant,
            t.nouveautes,
        ]),
        Page::new(A_PROPOS, t.a_propos, icones::A_PROPOS).avec_mots(&[
            t.version.trim_end_matches("{}").trim(),
            t.licence,
            t.credits,
        ]),
    ]
}

/// La page « Accueil », facultative, en premier.
pub fn page_accueil(t: &Textes) -> Page {
    Page::new(ACCUEIL, t.accueil, icones::ACCUEIL)
}

/// Toute la navigation : l'Accueil s'il est voulu, les pages propres au logiciel (sans
/// doublon avec une clé commune), puis Général, Mise à jour, À propos.
pub fn navigation(t: &Textes, accueil: bool, propres: &[Page]) -> Vec<Page> {
    let communes = pages_communes(t);
    let reservee = |c: &str| c == ACCUEIL || communes.iter().any(|p| p.cle == c);
    let mut v = Vec::new();
    if accueil {
        v.push(page_accueil(t));
    }
    for p in propres {
        if !reservee(&p.cle) && !v.iter().any(|q: &Page| q.cle == p.cle) {
            v.push(p.clone());
        }
    }
    v.extend(communes);
    v
}

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
    cle.and_then(|c| rang_de_cle(pages, c)).unwrap_or(0)
}

/// Le rang de la page de cette clé, si elle existe (aller à une page : rien ne bouge sinon).
pub fn rang_de_cle(pages: &[Page], cle: &str) -> Option<usize> {
    pages.iter().position(|p| p.cle == cle)
}

/// Avec quoi la navigation a été faite : l'Accueil et les clés des pages propres. Des pages
/// qui changent (envoyées après l'ouverture) la font refaire ; seules les clés comptent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Suivi {
    accueil: bool,
    cles: Vec<String>,
}

impl Suivi {
    /// La navigation vient d'être faite avec ces pages.
    pub fn de(accueil: bool, propres: &[Page]) -> Suivi {
        Suivi {
            accueil,
            cles: propres.iter().map(|p| p.cle.clone()).collect(),
        }
    }

    /// Les mêmes pages qu'à la dernière fois (mêmes clés, dans le même ordre) ?
    pub fn memes_pages(&self, accueil: bool, propres: &[Page]) -> bool {
        self.accueil == accueil
            && self.cles.len() == propres.len()
            && self.cles.iter().zip(propres).all(|(a, p)| *a == p.cle)
    }
}

/// Ce que devient la page montrée quand la navigation est refaite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSuivie {
    /// Elle existe encore, à ce rang : la sous-page et la recherche restent.
    Gardee(usize),
    /// Elle a disparu : Général (ce rang), sans sous-page ni recherche.
    Perdue(usize),
}

/// La page de clé `cle` dans la nouvelle navigation `pages`.
pub fn page_suivie(pages: &[Page], cle: &str) -> PageSuivie {
    match rang_de_cle(pages, cle) {
        Some(i) => PageSuivie::Gardee(i),
        None => PageSuivie::Perdue(rang_de_page(pages, Some(GENERAL))),
    }
}
