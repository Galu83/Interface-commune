//! La supervision d'une boucle de fenêtres tenue par le logiciel (façon 2 du bloc J) : il dit
//! comment lancer sa boucle et quoi faire de ses fenêtres perdues ([`Supervise`]) ; la
//! supervision attrape les paniques, compte les pertes et décide quand recréer, selon le cœur
//! (`ic_coeur::gestes::secours`, bloc D).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

use ic_coeur::gestes::secours::{Evenement, Fin, Perte, Reprise};

use super::Pertes;

/// Ce que la supervision fait tourner : la boucle des fenêtres du logiciel, et ce qu'il fait
/// quand elle est perdue. Pour un logiciel qui tient sa propre boucle eframe ; la boucle
/// toute faite de la bibliothèque ([`boucle_des_fenetres`](crate::fenetres::boucle_des_fenetres)) suit les mêmes règles.
pub trait Supervise {
    /// Un tour de la boucle des fenêtres (par exemple `eframe::run_native` avec
    /// `run_and_return`). `Ok` quand elle s'est fermée, `Err` quand elle n'a pas pu être
    /// créée. Une panique est attrapée par la supervision.
    fn boucle(&mut self) -> Result<(), String>;
    /// Les fenêtres sont perdues : le logiciel note `e.phrase()` au journal et oublie ce qui
    /// était ouvert. [`Reprise::Maintenant`] : il refait le clic qui s'ouvrait, pour les
    /// fenêtres recréées. [`Reprise::AuClicSuivant`] : il ouvre le menu de Windows pour ce
    /// clic ; le clic suivant revient aux fenêtres.
    fn perdues(&mut self, e: &Evenement);
    /// Après [`Reprise::AuClicSuivant`] : attend le clic suivant (ou toute autre demande aux
    /// fenêtres), puis rend `true` pour les recréer ; `false` : le logiciel se ferme.
    fn attendre_un_clic(&mut self) -> bool;
    /// Le logiciel se ferme : plus rien n'est recréé.
    fn quitter(&self) -> bool {
        false
    }
}

/// Fait tourner la boucle des fenêtres de `s` et la recrée quand elle est perdue : une
/// panique (attrapée ici, sa cause et son lieu gardés pour le journal), une fermeture par
/// Windows ou un échec passager. Jusqu'à [`RECREATIONS_EN_RAFALE`](ic_coeur::gestes::secours::RECREATIONS_EN_RAFALE) pertes en
/// [`RAFALE`](ic_coeur::gestes::secours::RAFALE),
/// elle est recréée tout de suite ; au-delà, le logiciel ouvre le menu de Windows pour le
/// clic en cours, et elle est recréée au clic suivant. Seul un échec au tout premier tour
/// (rien n'a jamais marché) arrête la supervision pour de bon. `horloge` : l'heure
/// (injectable pour les essais ; `Instant::now` sinon).
pub fn superviser_les_fenetres(s: &mut impl Supervise, horloge: impl Fn() -> Instant) -> Fin {
    crate::sonde::retenir_les_paniques();
    let mut pertes = Pertes::default();
    let mut premier_tour = true;
    loop {
        if s.quitter() {
            return Fin::Quitter;
        }
        let tour = catch_unwind(AssertUnwindSafe(|| s.boucle()));
        let premier = std::mem::replace(&mut premier_tour, false);
        if s.quitter() {
            return Fin::Quitter;
        }
        let perte = match tour {
            Ok(Ok(())) => Perte::Fermee,
            Ok(Err(e)) if premier => return Fin::Impossible(e),
            Ok(Err(e)) => Perte::Echec(e),
            Err(p) => Perte::Panique(cause_de_panique(&*p)),
        };
        // Un échec passager n'est pas une perte de plus : un nouvel essai au clic suivant.
        let (nombre, reprise) = ic_coeur::gestes::secours::decider_la_reprise(&perte, &mut pertes, &horloge);
        s.perdues(&Evenement {
            perte,
            nombre,
            reprise,
        });
        if reprise == Reprise::AuClicSuivant && !s.attendre_un_clic() {
            return Fin::Quitter;
        }
    }
}

/// Le texte d'une panique attrapée : son message, son lieu et la dernière étape du fil
/// (de quoi trouver la cause dans le journal).
pub fn cause_de_panique(p: &(dyn std::any::Any + Send)) -> String {
    let message = p
        .downcast_ref::<&str>()
        .map(|s| (*s).to_string())
        .or_else(|| p.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "sans message".into());
    let mut message: String = message.chars().take(300).collect();
    message = message.replace(['\r', '\n'], " ");
    let mut t = format!("panique : {message}");
    if let Some(lieu) = crate::sonde::lieu_de_la_derniere_panique() {
        t.push_str(&format!(", à {lieu}"));
    }
    if let Some(e) = crate::sonde::etape_du_fil() {
        t.push_str(&format!(", étape « {e} »"));
    }
    t
}
