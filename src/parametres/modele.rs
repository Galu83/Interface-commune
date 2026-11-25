//! Le squelette des Paramètres en calcul pur, sans fenêtre (tout est testé) : l'ordre des
//! pages et leurs clés, la page demandée sur la ligne de commande (`--parametres <page>`), la
//! recherche d'un paramètre, la bulle « Annuler » après un changement, et « Conserver ces
//! modifications ? » (retour automatique) pour ce qui peut couper l'écran, la souris, le
//! clavier ou la liaison (règles : interface Windows 11, point 7).

use std::time::{Duration, Instant};

use crate::icones;
use crate::textes::Textes;

// Le contrat (bloc A), repris du cœur à ce chemin.
pub use ic_coeur::contrat::{Instantane, Page, Valeur};

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

/// Les mots tapés dans « Rechercher un paramètre » se trouvent-ils dans le titre ou dans les
/// `mots` d'une section repliable ? Tous les mots tapés doivent s'y trouver (accents et
/// majuscules sans importance, [`simplifier`]). Rien de tapé : non.
pub fn section_trouvee(tape: &str, titre: &str, mots: &[&str]) -> bool {
    let tape = simplifier(tape);
    let mut tapes = tape.split_whitespace().peekable();
    if tapes.peek().is_none() {
        return false;
    }
    let texte = simplifier(&format!("{titre} {}", mots.join(" ")));
    tapes.all(|m| texte.contains(m))
}

/// Un changement d'un groupe : la clé, la valeur d'avant, celle d'après.
pub type Changement = (String, Valeur, Valeur);

/// La bulle « Annuler » reste ce temps après un changement.
pub const DUREE_ANNULER: Duration = Duration::from_secs(6);

/// Le dernier changement, que la bulle « Annuler » peut défaire.
#[derive(Debug, Clone, PartialEq)]
pub struct Annulable {
    pub cle: String,
    /// Le nom du réglage, montré dans la bulle (« Thème changé »).
    pub titre: String,
    pub avant: Valeur,
    /// Les autres réglages d'un groupe de changements (clé, valeur d'avant), défaits avec lui.
    pub autres: Vec<(String, Valeur)>,
    /// Un brouillon entier : son état d'avant (la clé et `avant` ne servent alors pas).
    pub instantane: Option<Instantane>,
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
                autres: Vec::new(),
                instantane: None,
                quand: maintenant,
            });
        }
    }

    /// Un brouillon entier vient d'être changé : `avant` est son état au début de l'image.
    /// Plusieurs changements de suite (une frappe) ne font qu'un : « Annuler » revient à l'état
    /// d'avant le premier.
    pub fn changer_brouillon(&mut self, titre: &str, avant: Instantane, maintenant: Instant) {
        if let Some(a) = &mut self.annulable
            && a.instantane.is_some()
            && maintenant.saturating_duration_since(a.quand) < DUREE_ANNULER
        {
            a.quand = maintenant;
            return;
        }
        self.annulable = Some(Annulable {
            cle: String::new(),
            titre: titre.to_string(),
            avant: Valeur::Booleen(false),
            autres: Vec::new(),
            instantane: Some(avant),
            quand: maintenant,
        });
    }

    /// « Annuler » est cliqué sur un brouillon entier : son état d'avant, à rendre au logiciel
    /// ([`crate::Reglages::remettre`]). Rien si la bulle n'est pas celle d'un brouillon, ou a
    /// déjà disparu.
    pub fn annuler_brouillon(&mut self, maintenant: Instant) -> Option<Instantane> {
        let est_brouillon = self.a_annuler(maintenant).is_some_and(|a| a.instantane.is_some());
        if !est_brouillon {
            return None;
        }
        self.annulable.take().and_then(|a| a.instantane)
    }

    /// Plusieurs réglages changés d'un seul geste (clé, avant, après) : une seule bulle, et
    /// « Annuler » les remet tous. Ceux qui ne changent rien sont oubliés.
    pub fn changer_groupe(&mut self, titre: &str, changements: Vec<Changement>, maintenant: Instant) {
        let mut avants = changements
            .into_iter()
            .filter(|(_, avant, apres)| avant != apres)
            .map(|(cle, avant, _)| (cle, avant));
        let Some((cle, avant)) = avants.next() else {
            return;
        };
        self.annulable = Some(Annulable {
            cle,
            titre: titre.to_string(),
            avant,
            autres: avants.collect(),
            instantane: None,
            quand: maintenant,
        });
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

    /// « Annuler » est cliqué : rend les clés et les valeurs d'avant, à remettre par le
    /// logiciel (une seule, sauf pour un groupe). Rien si la bulle a déjà disparu.
    pub fn annuler(&mut self, maintenant: Instant) -> Vec<(String, Valeur)> {
        match self.annulable.take() {
            Some(a) if a.instantane.is_some() => {
                // Un brouillon entier ne se défait que par `annuler_brouillon`.
                self.annulable = Some(a);
                Vec::new()
            }
            Some(a) if maintenant.saturating_duration_since(a.quand) < DUREE_ANNULER => {
                let mut tout = vec![(a.cle, a.avant)];
                tout.extend(a.autres);
                tout
            }
            _ => Vec::new(),
        }
    }

    /// La bulle est fermée sans annuler.
    pub fn oublier(&mut self) {
        self.annulable = None;
    }
}

/// « Conserver ces modifications ? » attend la réponse ce temps ; sans réponse, retour au
/// réglage d'avant (comme Windows après un changement de résolution d'écran).
pub const DUREE_CONSERVER: Duration = Duration::from_secs(15);

/// Ce qui attend la réponse « Conserver » ou « Revenir ».
#[derive(Debug, Clone, PartialEq)]
pub struct AConfirmer {
    /// Le nom du réglage changé en dernier, montré dans la carte (« Mode d'affichage »).
    pub titre: String,
    /// Les réglages changés (clé, valeur d'**avant le premier changement**), dans l'ordre où
    /// ils ont changé.
    pub valeurs: Vec<(String, Valeur)>,
    /// Un brouillon entier : son état d'avant le premier changement.
    pub brouillon: Option<Instantane>,
    /// Le dernier changement : le compte à rebours repart de là.
    quand: Instant,
}

/// Ce que « Revenir » (ou le délai écoulé, ou la fermeture) rend au logiciel : le brouillon
/// d'avant, à remettre d'abord par [`crate::Reglages::remettre`], puis les valeurs d'avant, à
/// remettre par [`crate::Reglages::changer`] (le même chemin que « Annuler »).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Retour {
    pub valeurs: Vec<(String, Valeur)>,
    pub brouillon: Option<Instantane>,
}

impl Retour {
    /// Rien à remettre.
    pub fn est_vide(&self) -> bool {
        self.valeurs.is_empty() && self.brouillon.is_none()
    }
}

/// « Conserver ces modifications ? » : un réglage qui peut couper l'écran, la souris, le
/// clavier ou la liaison est appliqué tout de suite (le logiciel s'en charge), puis la
/// carte demande de le garder ; sans réponse en [`DUREE_CONSERVER`], tout revient comme avant.
/// **Une seule confirmation à la fois** : un nouveau changement pendant le compte à rebours
/// garde l'état d'avant le **premier**, et le compte à rebours repart de zéro.
#[derive(Debug, Default)]
pub struct Confirmation {
    attente: Option<AConfirmer>,
}

impl Confirmation {
    /// Ouvre la confirmation, ou la continue (le titre du dernier changement ; compte à
    /// rebours remis à [`DUREE_CONSERVER`]).
    fn continuer(&mut self, titre: &str, maintenant: Instant) -> &mut AConfirmer {
        let a = self.attente.get_or_insert_with(|| AConfirmer {
            titre: String::new(),
            valeurs: Vec::new(),
            brouillon: None,
            quand: maintenant,
        });
        a.titre = titre.to_string();
        a.quand = maintenant;
        a
    }

    /// Garde la valeur d'avant d'une clé (celle d'avant le premier changement) ; revenue à
    /// cette valeur, la clé n'a plus rien à confirmer.
    fn noter(&mut self, cle: &str, avant: Valeur, apres: &Valeur) {
        let Some(a) = &mut self.attente else { return };
        if !a.valeurs.iter().any(|(k, _)| k == cle) {
            a.valeurs.push((cle.to_string(), avant));
        }
        if let Some(i) = a.valeurs.iter().position(|(k, v)| k == cle && v == apres) {
            a.valeurs.remove(i);
        }
    }

    /// Plus rien à confirmer : la carte disparaît.
    fn nettoyer(&mut self) {
        if self
            .attente
            .as_ref()
            .is_some_and(|a| a.valeurs.is_empty() && a.brouillon.is_none())
        {
            self.attente = None;
        }
    }

    /// Le réglage `cle` vient de passer de `avant` à `apres`, et doit être confirmé.
    /// Appeler [`Confirmation::echu`] avant (la bibliothèque le fait).
    pub fn changer(&mut self, cle: &str, titre: &str, avant: Valeur, apres: &Valeur, maintenant: Instant) {
        if avant == *apres {
            return;
        }
        self.continuer(titre, maintenant);
        self.noter(cle, avant, apres);
        self.nettoyer();
    }

    /// Plusieurs réglages changés d'un seul geste (clé, avant, après), à confirmer d'un seul
    /// coup. Ceux qui ne changent rien sont oubliés.
    pub fn changer_groupe(&mut self, titre: &str, changements: Vec<Changement>, maintenant: Instant) {
        let changements: Vec<Changement> = changements.into_iter().filter(|(_, a, b)| a != b).collect();
        if changements.is_empty() {
            return;
        }
        self.continuer(titre, maintenant);
        for (cle, avant, apres) in changements {
            self.noter(&cle, avant, &apres);
        }
        self.nettoyer();
    }

    /// Un brouillon entier vient d'être changé : `avant` est son état avant ce changement.
    /// Pendant une confirmation déjà ouverte, l'état d'avant le premier est gardé.
    pub fn changer_brouillon(&mut self, titre: &str, avant: Instantane, maintenant: Instant) {
        let a = self.continuer(titre, maintenant);
        if a.brouillon.is_none() {
            a.brouillon = Some(avant);
        }
    }

    /// La confirmation attendue, tant que le temps n'est pas écoulé.
    pub fn en_attente(&self, maintenant: Instant) -> Option<&AConfirmer> {
        self.attente
            .as_ref()
            .filter(|a| maintenant.saturating_duration_since(a.quand) < DUREE_CONSERVER)
    }

    /// Le temps qu'il reste avant le retour automatique.
    pub fn reste(&self, maintenant: Instant) -> Option<Duration> {
        self.en_attente(maintenant)
            .map(|a| DUREE_CONSERVER.saturating_sub(maintenant.saturating_duration_since(a.quand)))
    }

    /// Les secondes montrées (15, 14… 1) : le temps qui reste, arrondi au-dessus.
    pub fn secondes(&self, maintenant: Instant) -> Option<u64> {
        self.reste(maintenant)
            .map(|r| r.as_secs() + u64::from(r.subsec_nanos() > 0))
    }

    /// Dans combien de temps la carte change de seconde (pour redessiner juste à ce
    /// moment-là, pas plus souvent).
    pub fn prochain_changement(&self, maintenant: Instant) -> Option<Duration> {
        self.reste(maintenant).map(|r| {
            let dans_la_seconde = Duration::from_nanos(u64::from(r.subsec_nanos()));
            let pas = if dans_la_seconde.is_zero() {
                Duration::from_secs(1)
            } else {
                dans_la_seconde
            };
            // 5 ms de marge : on redessine juste après le changement de seconde.
            pas + Duration::from_millis(5)
        })
    }

    /// « Conserver » : les changements restent. `false` si rien n'attendait, ou si le délai
    /// était déjà écoulé (le retour automatique l'emporte : voir [`Confirmation::echu`]).
    pub fn conserver(&mut self, maintenant: Instant) -> bool {
        if self.en_attente(maintenant).is_none() {
            return false;
        }
        self.attente = None;
        true
    }

    /// « Revenir » (ou la fermeture des Paramètres) : tout ce qui attendait, avec les valeurs
    /// d'avant le premier changement, à rendre au logiciel. Vide si rien n'attendait.
    pub fn revenir(&mut self) -> Retour {
        match self.attente.take() {
            Some(a) => Retour {
                valeurs: a.valeurs,
                brouillon: a.brouillon,
            },
            None => Retour::default(),
        }
    }

    /// Le délai est écoulé : le retour automatique, à rendre au logiciel comme « Revenir ».
    /// `None` tant qu'il reste du temps (et quand rien n'attend).
    pub fn echu(&mut self, maintenant: Instant) -> Option<Retour> {
        let ecoule = self
            .attente
            .as_ref()
            .is_some_and(|a| maintenant.saturating_duration_since(a.quand) >= DUREE_CONSERVER);
        ecoule.then(|| self.revenir())
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
    fn annuler_un_brouillon_entier() {
        let t = Instant::now();
        let mut a = Annulation::default();
        let pas = |ms| t + Duration::from_millis(ms);
        a.changer_brouillon("Réglages", Instantane::new(vec![1, 2]), t);
        // Une frappe de plus : « Annuler » garde l'état d'avant la première.
        a.changer_brouillon("Réglages", Instantane::new(vec![9, 9]), pas(300));
        assert!(a.a_annuler(pas(300)).is_some());
        // Un brouillon ne se défait pas réglage par réglage.
        assert!(a.annuler(pas(400)).is_empty());
        let avant = a.annuler_brouillon(pas(400)).unwrap();
        assert_eq!(avant.get::<Vec<i32>>(), Some(&vec![1, 2]));
        assert_eq!(avant.get::<String>(), None);
        assert!(a.annuler_brouillon(pas(400)).is_none());
        // Trop tard : la bulle a disparu.
        a.changer_brouillon("Réglages", Instantane::new(1), t);
        assert!(a.annuler_brouillon(t + DUREE_ANNULER).is_none());
        // Un réglage seul n'est pas un brouillon.
        a.changer("x", "X", Valeur::Booleen(false), &Valeur::Booleen(true), t);
        assert!(a.annuler_brouillon(t).is_none());
        assert_eq!(a.annuler(t).len(), 1);
    }

    #[test]
    fn ordre_des_pages() {
        let n = navigation(&crate::textes::FRANCAIS, true, &propres());
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
        assert_eq!(navigation(&crate::textes::FRANCAIS, false, &[]).len(), 3);
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
        let n = navigation(&crate::textes::FRANCAIS, false, &propres());
        assert_eq!(rang_de_page(&n, Some("a_propos")), 4);
        assert_eq!(rang_de_page(&n, Some("inconnue")), 0);
        assert_eq!(rang_de_page(&n, None), 0);
    }

    #[test]
    fn recherche_sans_accents() {
        let n = navigation(&crate::textes::FRANCAIS, true, &propres());
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
            vec![("theme".into(), b(false))]
        );
        assert!(a.annuler(t + Duration::from_secs(1)).is_empty());
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
        assert!(a.annuler(t + DUREE_ANNULER).is_empty());
        // Un changement qui ne change rien n'ouvre pas de bulle.
        a.changer("x", "X", b(true), &b(true), t);
        assert!(a.a_annuler(t).is_none());
        // Un groupe : une seule bulle, « Annuler » remet tout ; ce qui ne change pas est oublié.
        a.changer_groupe(
            "Disposition",
            vec![
                ("gauche".into(), n(1.0), n(2.0)),
                ("pareil".into(), b(true), b(true)),
                ("droite".into(), n(3.0), n(4.0)),
            ],
            t,
        );
        assert_eq!(a.a_annuler(t).map(|x| x.titre.as_str()), Some("Disposition"));
        assert_eq!(
            a.annuler(t),
            vec![("gauche".into(), n(1.0)), ("droite".into(), n(3.0))]
        );
        a.changer_groupe("Rien", vec![("x".into(), b(true), b(true))], t);
        assert!(a.a_annuler(t).is_none());
    }

    fn s(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }

    /// Le compte à rebours : 15 s, puis 14… jusqu'à 1 ; il disparaît à 0 ; le prochain
    /// redessin tombe juste après le changement de seconde.
    #[test]
    fn conserver_compte_a_rebours() {
        let t = Instant::now();
        let mut c = Confirmation::default();
        assert_eq!(DUREE_CONSERVER, Duration::from_secs(15));
        assert!(c.en_attente(t).is_none());
        assert_eq!(c.secondes(t), None);
        c.changer(
            "mode",
            "Mode",
            Valeur::Texte("a".into()),
            &Valeur::Texte("b".into()),
            t,
        );
        assert_eq!(c.en_attente(t).map(|x| x.titre.as_str()), Some("Mode"));
        assert_eq!(c.secondes(t), Some(15));
        assert_eq!(c.secondes(t + s(1)), Some(15));
        assert_eq!(c.secondes(t + s(1000)), Some(14));
        assert_eq!(c.secondes(t + s(14_000)), Some(1));
        assert_eq!(c.secondes(t + s(14_999)), Some(1));
        assert_eq!(c.secondes(t + s(15_000)), None);
        assert_eq!(c.reste(t + s(5_000)), Some(s(10_000)));
        // Au milieu d'une seconde : redessiner à la fin de celle-ci ; pile sur une seconde :
        // dans une seconde. Toujours un peu après le changement.
        assert_eq!(c.prochain_changement(t + s(500)), Some(s(505)));
        assert_eq!(c.prochain_changement(t + s(1000)), Some(s(1005)));
        assert_eq!(c.prochain_changement(t + s(15_000)), None);
    }

    /// « Conserver » : les changements restent, plus rien n'attend, rien n'est rendu.
    #[test]
    fn conserver_garde_les_changements() {
        let t = Instant::now();
        let mut c = Confirmation::default();
        c.changer("x", "X", Valeur::Booleen(false), &Valeur::Booleen(true), t);
        assert!(c.conserver(t + s(3_000)));
        assert!(c.en_attente(t + s(3_000)).is_none());
        assert!(c.revenir().est_vide());
        assert!(c.echu(t + s(60_000)).is_none());
        // Rien n'attend : rien à conserver.
        assert!(!c.conserver(t));
        // Un changement qui ne change rien n'ouvre rien.
        c.changer("x", "X", Valeur::Booleen(true), &Valeur::Booleen(true), t);
        assert!(c.en_attente(t).is_none());
    }

    /// « Revenir » rend la valeur d'avant ; la fermeture des Paramètres fait de même (même
    /// appel) ; une seconde fois : rien.
    #[test]
    fn revenir_rend_la_valeur_d_avant() {
        let t = Instant::now();
        let mut c = Confirmation::default();
        c.changer("mode", "Mode", Valeur::Nombre(1.0), &Valeur::Nombre(2.0), t);
        let r = c.revenir();
        assert_eq!(r.valeurs, vec![("mode".to_string(), Valeur::Nombre(1.0))]);
        assert!(r.brouillon.is_none());
        assert!(c.en_attente(t).is_none());
        assert!(c.revenir().est_vide());
    }

    /// Délai écoulé : retour automatique, une seule fois ; trop tard pour « Conserver ».
    #[test]
    fn delai_ecoule_retour_automatique() {
        let t = Instant::now();
        let mut c = Confirmation::default();
        c.changer("mode", "Mode", Valeur::Nombre(1.0), &Valeur::Nombre(2.0), t);
        assert!(c.echu(t + s(14_999)).is_none());
        assert!(c.en_attente(t + s(14_999)).is_some());
        // Un clic arrivé à l'instant où le délai finit ne garde rien.
        assert!(!c.conserver(t + DUREE_CONSERVER));
        let r = c.echu(t + DUREE_CONSERVER).expect("retour automatique");
        assert_eq!(r.valeurs, vec![("mode".to_string(), Valeur::Nombre(1.0))]);
        assert!(c.echu(t + DUREE_CONSERVER).is_none());
        assert!(c.revenir().est_vide());
    }

    /// Deux changements de suite : une seule confirmation, l'état d'avant le PREMIER est
    /// rendu, et le compte à rebours repart au second. Revenir à la valeur de départ : plus
    /// rien à confirmer.
    #[test]
    fn deux_changements_gardent_l_avant_du_premier() {
        let t = Instant::now();
        let n = Valeur::Nombre;
        let mut c = Confirmation::default();
        c.changer("mode", "Mode", n(1.0), &n(2.0), t);
        c.changer("mode", "Mode", n(2.0), &n(3.0), t + s(10_000));
        // Le compte à rebours est reparti : 15 s après le second.
        assert_eq!(c.secondes(t + s(10_000)), Some(15));
        assert!(c.echu(t + s(20_000)).is_none());
        // Un autre réglage s'ajoute ; le titre est celui du dernier.
        c.changer("souris", "Souris", n(5.0), &n(6.0), t + s(12_000));
        assert_eq!(
            c.en_attente(t + s(12_000)).map(|a| a.titre.as_str()),
            Some("Souris")
        );
        let r = c.revenir();
        assert_eq!(
            r.valeurs,
            vec![("mode".to_string(), n(1.0)), ("souris".to_string(), n(5.0))]
        );
        // Revenu à la valeur de départ à la main : plus rien à confirmer.
        c.changer("mode", "Mode", n(1.0), &n(2.0), t);
        c.changer("mode", "Mode", n(2.0), &n(1.0), t + s(500));
        assert!(c.en_attente(t + s(500)).is_none());
        assert!(c.revenir().est_vide());
        // Un groupe, puis un autre changement : tout revient, une seule fois chacun.
        c.changer_groupe(
            "Écran",
            vec![
                ("gauche".into(), n(1.0), n(2.0)),
                ("pareil".into(), n(7.0), n(7.0)),
                ("droite".into(), n(3.0), n(4.0)),
            ],
            t,
        );
        c.changer("gauche", "Gauche", n(2.0), &n(9.0), t + s(100));
        let r = c.revenir();
        assert_eq!(
            r.valeurs,
            vec![("gauche".to_string(), n(1.0)), ("droite".to_string(), n(3.0))]
        );
        c.changer_groupe("Rien", vec![("x".into(), n(1.0), n(1.0))], t);
        assert!(c.en_attente(t).is_none());
    }

    /// Un brouillon entier : l'état d'avant le premier changement est rendu, avec les
    /// valeurs seules s'il y en a ; un second brouillon n'écrase pas le premier.
    #[test]
    fn brouillon_a_confirmer() {
        let t = Instant::now();
        let mut c = Confirmation::default();
        c.changer_brouillon("Réglages", Instantane::new(vec![1, 2]), t);
        c.changer_brouillon("Réglages", Instantane::new(vec![9]), t + s(2_000));
        c.changer(
            "mode",
            "Mode",
            Valeur::Booleen(false),
            &Valeur::Booleen(true),
            t + s(3_000),
        );
        assert_eq!(c.secondes(t + s(3_000)), Some(15));
        let r = c.revenir();
        assert!(!r.est_vide());
        assert_eq!(
            r.brouillon.as_ref().and_then(|b| b.get::<Vec<i32>>()),
            Some(&vec![1, 2])
        );
        assert_eq!(r.valeurs, vec![("mode".to_string(), Valeur::Booleen(false))]);
        assert!(c.revenir().est_vide());
    }

    /// La recherche ouvre une section : accents, plusieurs mots, rien de tapé.
    #[test]
    fn section_ouverte_par_la_recherche() {
        let mots = ["Luminosité", "Contraste"];
        assert!(section_trouvee("ecran", "Écrans", &mots));
        assert!(section_trouvee("LUMINOSITE", "Écrans", &mots));
        assert!(section_trouvee("contraste ecrans", "Écrans", &mots));
        assert!(section_trouvee("  contr ", "Écrans", &mots));
        assert!(!section_trouvee("contraste volume", "Écrans", &mots));
        assert!(!section_trouvee("", "Écrans", &mots));
        assert!(!section_trouvee("   ", "Écrans", &mots));
        assert!(!section_trouvee("son", "Écrans", &[]));
        assert!(section_trouvee("avancé", "Réglages avancés", &[]));
    }
}
