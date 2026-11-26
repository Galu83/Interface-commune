//! La sonde des fils, en calcul pur (blocs D et O de l'organigramme) : décider qu'un fil est
//! bloqué, qu'il repart, que le PC a dormi, qu'une création de fenêtre dure ; et les lignes du
//! journal qui le disent. Rien n'est lu ici : ni l'heure, ni les compteurs, ni les fils.
//!
//! La bibliothèque (`interface_commune::sonde`, écrite dans `src/bords/sonde/`) garde les
//! compteurs atomiques, le fil du chien de garde et le journal ; à chaque tour, elle relève
//! chaque fil ([`Releve`]), passe l'instant ([`Horodatage`]) et écrit ce que rend
//! [`Surveillance::tour`]. Les essais passent une `Duration` comptée depuis une origine
//! inventée : une fausse horloge, aucune attente réelle.
//!
//! Les lignes ne contiennent jamais rien de l'utilisateur : seulement des noms écrits dans le
//! code (`&'static str`) et le nom qu'un logiciel a donné à sa fenêtre.

use std::time::Duration;

use super::temps::Horodatage;

/// Les délais de la surveillance : une seule copie, que le logiciel change par
/// `interface_commune::sonde::regler` (ses tests les raccourcissent pour ne jamais attendre).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delais {
    /// Le temps entre deux lectures des compteurs.
    pub pas: Duration,
    /// Le temps sans battement après lequel un fil est dit bloqué.
    pub blocage: Duration,
}

impl Default for Delais {
    /// 5 s entre deux lectures, bloqué après 15 s sans battement.
    fn default() -> Delais {
        Delais {
            pas: Duration::from_secs(5),
            blocage: Duration::from_secs(15),
        }
    }
}

/// Un écart entre deux tours d'une boucle de surveillance qui dépasse largement son `pas`
/// (plus de trois pas, et plus de 3 s) : la boucle elle-même n'a pas tourné, le PC dormait. Les
/// horloges de Windows continuent de compter pendant la veille : sans cela, au réveil, chaque
/// fil paraîtrait bloqué depuis toute la durée du sommeil.
pub fn est_une_veille(ecart: Duration, pas: Duration) -> bool {
    ecart > (pas * 3).max(Duration::from_secs(3))
}

/// Ce que la surveillance lit d'un fil à un tour. `K` : de quoi reconnaître le fil d'un tour à
/// l'autre (l'adresse de son compteur dans la bibliothèque, un numéro dans les essais) ; `T` :
/// l'instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Releve<K, T> {
    pub cle: K,
    /// Le nom du fil, écrit dans le code.
    pub nom: &'static str,
    /// Son compteur de battements (il avance à chaque tour de sa boucle).
    pub battements: u64,
    /// Sa dernière étape posée ; vide s'il n'en a posé aucune.
    pub etape: &'static str,
    /// La boucle imbriquée où il est : son nom et son début.
    pub imbriquee: Option<(&'static str, T)>,
}

/// Ce que la surveillance constate à un tour, une ligne de journal chacun ([`Constat::ligne`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constat {
    /// La boucle de surveillance elle-même n'a pas tourné pendant `ecart` : le PC dormait. Les
    /// mesures repartent de zéro.
    Veille { ecart: Duration },
    /// Le fil ne bat plus depuis `depuis` (une seule fois par blocage).
    Bloque {
        fil: &'static str,
        depuis: Duration,
        /// Sa dernière étape ; vide s'il n'en a posé aucune.
        etape: &'static str,
        /// La boucle imbriquée où il est, et depuis combien de temps.
        imbriquee: Option<(&'static str, Duration)>,
    },
    /// Le fil, dit bloqué, bat de nouveau ; `apres` : la durée totale du blocage.
    Repart { fil: &'static str, apres: Duration },
}

impl Constat {
    /// La ligne du journal, mot pour mot celle d'avant la refonte.
    pub fn ligne(&self) -> String {
        match self {
            Constat::Veille { ecart } => format!(
                "Reprise après une veille d'environ {} s : les mesures de blocage repartent de zéro.",
                ecart.as_secs()
            ),
            Constat::Bloque {
                fil,
                depuis,
                etape,
                imbriquee,
            } => {
                let dans = imbriquee.map_or(String::new(), |(n, d)| {
                    format!(" dans « {n} » depuis {} s,", d.as_secs())
                });
                let etape = if etape.is_empty() { "aucune" } else { etape };
                format!(
                    "Le fil « {fil} » ne répond plus depuis {} s :{dans} dernière étape : {etape}.",
                    depuis.as_secs()
                )
            }
            Constat::Repart { fil, apres } => {
                format!("Le fil « {fil} » repart après {} s de blocage.", apres.as_secs())
            }
        }
    }
}

/// Le suivi d'un fil d'un tour à l'autre.
#[derive(Debug, Clone, Copy)]
struct Suivi<T> {
    dernier: u64,
    /// Le moment où le compteur a changé pour la dernière fois (ou où on l'a vu la première
    /// fois).
    depuis: T,
    signale: bool,
}

/// La décision « bloqué » : un compteur immobile depuis le délai de blocage donne un
/// [`Constat::Bloque`] (un seul par blocage), puis un [`Constat::Repart`] quand il bouge.
#[derive(Debug, Clone)]
pub struct Surveillance<K, T> {
    blocage: Duration,
    suivis: Vec<(K, Suivi<T>)>,
    /// Le pas entre deux tours, et l'heure du dernier tour : un écart bien plus long que le pas
    /// veut dire que la boucle de surveillance elle-même n'a pas tourné (le PC dormait). `None`
    /// sans pas : aucune veille n'est cherchée.
    pas: Option<Duration>,
    dernier_tour: Option<T>,
}

impl<K: Copy + Eq, T: Horodatage> Surveillance<K, T> {
    /// Une surveillance sans pas connu : un grand écart entre deux tours reste un blocage.
    pub fn new(blocage: Duration) -> Self {
        Surveillance {
            blocage,
            suivis: Vec::new(),
            pas: None,
            dernier_tour: None,
        }
    }

    /// Comme [`Surveillance::new`], pour une boucle qui tourne tous les `pas` : un écart bien
    /// plus long entre deux tours est une veille du PC ([`est_une_veille`]) ; les repères de
    /// tous les fils repartent de zéro, sans alerte, avec un seul [`Constat::Veille`].
    pub fn avec_pas(blocage: Duration, pas: Duration) -> Self {
        Surveillance {
            pas: Some(pas),
            ..Surveillance::new(blocage)
        }
    }

    /// Celle des [`Delais`] : bloqué après `blocage`, un tour tous les `pas`.
    pub fn des_delais(d: Delais) -> Self {
        Surveillance::avec_pas(d.blocage, d.pas)
    }

    /// Un tour à `maintenant`, avec le relevé de chaque fil inscrit ; un fil absent du relevé
    /// est oublié (il est sorti de la surveillance).
    pub fn tour(&mut self, maintenant: T, fils: &[Releve<K, T>]) -> Vec<Constat> {
        let mut constats = Vec::new();
        let precedent = self.dernier_tour.replace(maintenant);
        if let (Some(pas), Some(precedent)) = (self.pas, precedent) {
            let ecart = maintenant.depuis_ou_zero(precedent);
            if est_une_veille(ecart, pas) {
                // La boucle a dormi : aucun fil n'est bloqué pour autant. Les repères repartent
                // de zéro (chaque fil est revu comme la première fois), sans alerte.
                self.suivis.clear();
                constats.push(Constat::Veille { ecart });
            }
        }
        self.suivis.retain(|(k, _)| fils.iter().any(|f| f.cle == *k));
        for f in fils {
            if let Some(c) = self.constat(maintenant, f) {
                constats.push(c);
            }
        }
        constats
    }

    /// Le constat d'un fil à ce tour, s'il y en a un.
    fn constat(&mut self, maintenant: T, f: &Releve<K, T>) -> Option<Constat> {
        let neuf = Suivi {
            dernier: f.battements,
            depuis: maintenant,
            signale: false,
        };
        let Some(i) = self.suivis.iter().position(|(k, _)| *k == f.cle) else {
            self.suivis.push((f.cle, neuf));
            return None;
        };
        let s = &mut self.suivis[i].1;
        if f.battements != s.dernier {
            let repart = s.signale.then(|| Constat::Repart {
                fil: f.nom,
                apres: maintenant.depuis_ou_zero(s.depuis),
            });
            *s = neuf;
            return repart;
        }
        let immobile = maintenant.depuis_ou_zero(s.depuis);
        if immobile < self.blocage || s.signale {
            return None;
        }
        s.signale = true;
        Some(Constat::Bloque {
            fil: f.nom,
            depuis: immobile,
            etape: f.etape,
            imbriquee: f
                .imbriquee
                .map(|(n, debut)| (n, maintenant.depuis_ou_zero(debut))),
        })
    }
}

// ---------------------------------------------------------------------------------------
// Le chien de garde des créations de fenêtre

/// Une création de fenêtre qui dure plus longtemps que cela est écrite au journal.
pub const SEUIL_DE_CREATION: Duration = Duration::from_secs(3);

/// Le pas du chien des créations : il regarde souvent, un tiers du seuil, entre 20 ms et 1 s.
pub fn pas_des_creations(seuil: Duration) -> Duration {
    (seuil / 3).clamp(Duration::from_millis(20), Duration::from_secs(1))
}

/// Une création de fenêtre en cours, vue par le chien de garde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnCreation<T> {
    pub id: u64,
    /// Le nom que le logiciel a donné à sa fenêtre.
    pub nom: String,
    pub debut: T,
    /// L'heure du début, en secondes UTC depuis le 1er janvier 1970 (pour le journal).
    pub debut_utc: u64,
    /// Déjà écrite au journal ?
    pub signalee: bool,
}

/// Les lignes à écrire à `maintenant` : une, une seule fois, pour chaque création qui dure
/// depuis plus de `seuil`.
pub fn lignes_des_creations<T: Horodatage>(
    l: &mut [EnCreation<T>],
    maintenant: T,
    seuil: Duration,
) -> Vec<String> {
    let mut lignes = Vec::new();
    for c in l.iter_mut() {
        let duree = maintenant.depuis_ou_zero(c.debut);
        if duree >= seuil && !c.signalee {
            c.signalee = true;
            lignes.push(format!(
                "Fenêtre « {} » : sa création dure depuis {} s (commencée à {} UTC) ; le fil des fenêtres ne rend plus la main.",
                c.nom,
                duree.as_secs(),
                heure_du_jour(c.debut_utc)
            ));
        }
    }
    lignes
}

/// Un tour du chien de garde des créations à `maintenant` : si l'écart avec le tour précédent
/// (`dernier`) est une veille du PC ([`est_une_veille`]), les créations en cours repartent de
/// zéro, sans alerte (la surveillance écrit la ligne de reprise) ; sinon, les lignes de
/// [`lignes_des_creations`].
pub fn tour_des_creations<T: Horodatage>(
    l: &mut [EnCreation<T>],
    maintenant: T,
    dernier: &mut T,
    pas: Duration,
    seuil: Duration,
) -> Vec<String> {
    let ecart = maintenant.depuis_ou_zero(*dernier);
    *dernier = maintenant;
    if est_une_veille(ecart, pas) {
        for c in l.iter_mut() {
            c.debut = maintenant;
            c.signalee = false;
        }
        return Vec::new();
    }
    lignes_des_creations(l, maintenant, seuil)
}

/// La ligne de la fin d'une création, à `maintenant` : seulement si sa longueur a déjà été
/// écrite au journal.
pub fn fin_de_creation<T: Horodatage>(c: &EnCreation<T>, maintenant: T) -> Option<String> {
    c.signalee.then(|| {
        format!(
            "Fenêtre « {} » : la création, commencée à {} UTC, s'est enfin terminée après {} s.",
            c.nom,
            heure_du_jour(c.debut_utc),
            maintenant.depuis_ou_zero(c.debut).as_secs()
        )
    })
}

// ---------------------------------------------------------------------------------------
// Les autres lignes du journal

/// L'heure UTC du jour, `14:03:22`, de secondes UTC depuis le 1er janvier 1970.
pub fn heure_du_jour(secondes_utc: u64) -> String {
    let s = secondes_utc % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

/// La ligne d'un clic reçu par l'icône (`"gauche"`, `"droit"`, `"double"`…).
pub fn ligne_du_clic(quel: &str, secondes_utc: u64) -> String {
    format!("Icône : clic {quel} reçu à {} UTC.", heure_du_jour(secondes_utc))
}

/// La ligne du début d'une boucle imbriquée.
pub fn debut_d_imbriquee(nom: &str) -> String {
    format!("Boucle imbriquée « {nom} » : début.")
}

/// La ligne de la fin d'une boucle imbriquée, après `duree`.
pub fn fin_d_imbriquee(nom: &str, duree: Duration) -> String {
    format!("Boucle imbriquée « {nom} » : fin après {} ms.", duree.as_millis())
}

/// Le lieu d'une panique pour le journal : les deux derniers éléments du chemin du fichier
/// source et la ligne, jamais le chemin du PC, quel que soit le séparateur.
pub fn lieu_court(fichier: &str, ligne: u32) -> String {
    let morceaux: Vec<&str> = fichier.split(['/', '\\']).filter(|m| !m.is_empty()).collect();
    let debut = morceaux.len().saturating_sub(2);
    format!("{}:{ligne}", morceaux.get(debut..).unwrap_or_default().join("/"))
}

#[cfg(test)]
mod essais;
