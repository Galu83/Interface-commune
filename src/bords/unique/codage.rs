//! La partie pure de l'instance unique (bloc N) : les noms du rendez-vous entre deux copies, le
//! codage et la lecture des arguments, ce que décide un lancement et sa traduction pour l'API.
//! Aucun appel à Windows ici : tout s'essaie sans fenêtre ni verrou
//! (`src/bords/unique/essais.rs`).

use std::fmt;

use ic_coeur::contrat::Reveil;

/// Le nombre d'arguments qu'un second lancement peut transmettre.
pub const ARGUMENTS_MAX: usize = 16;
/// La taille de ce qu'un second lancement peut transmettre (octets, tous arguments ensemble).
pub const ARGUMENTS_OCTETS_MAX: usize = 4096;
/// Le `dwData` d'un `WM_COPYDATA` d'arguments : tout autre est ignoré. Les quatre lettres
/// « ICAR » (Interface Commune, ARguments) ; c'est un format entre deux versions du même
/// logiciel : il ne change pas.
pub(crate) const MARQUE_ARGUMENTS: usize = 0x4943_4152;

/// Le nom du verrou d'un logiciel : dans l'espace de la session (`Local\`, C-18), jamais dans
/// `Global\`. Ce nom exact est une promesse : une nouvelle version doit voir le verrou de
/// l'ancienne pendant une mise à jour.
pub(crate) fn nom_du_verrou(nom: &str) -> String {
    format!("Local\\{nom}-une-seule")
}

/// Le nom de la classe de la fenêtre cachée du logiciel : le second lancement la cherche par
/// ce nom pour la prévenir (la fenêtre elle-même est créée par l'icône, bloc M).
pub(crate) fn nom_de_la_fenetre(nom: &str) -> String {
    format!("{nom}-principal")
}

/// Les arguments en octets : chacun en UTF-8, suivi d'un zéro. `None` s'il y en a trop, s'ils
/// sont trop gros, ou si l'un contient un zéro.
pub(crate) fn coder_arguments(args: &[String]) -> Option<Vec<u8>> {
    if args.len() > ARGUMENTS_MAX {
        return None;
    }
    let mut octets = Vec::new();
    for a in args {
        if a.contains('\0') {
            return None;
        }
        octets.extend_from_slice(a.as_bytes());
        octets.push(0);
    }
    (octets.len() <= ARGUMENTS_OCTETS_MAX).then_some(octets)
}

/// L'inverse de [`coder_arguments`] ; `None` pour des octets qui ne viennent pas de lui
/// (trop longs, trop d'arguments, UTF-8 invalide, zéro final absent).
pub(crate) fn decoder_arguments(octets: &[u8]) -> Option<Vec<String>> {
    if octets.len() > ARGUMENTS_OCTETS_MAX {
        return None;
    }
    if octets.is_empty() {
        return Some(Vec::new());
    }
    let (corps, fin) = octets.split_at(octets.len() - 1);
    if fin != [0] {
        return None;
    }
    let args: Vec<String> = corps
        .split(|o| *o == 0)
        .map(|a| std::str::from_utf8(a).map(str::to_string))
        .collect::<Result<_, _>>()
        .ok()?;
    (args.len() <= ARGUMENTS_MAX).then_some(args)
}

/// Un `WM_COPYDATA` reçu vaut la peine d'être lu : il porte la marque, son adresse n'est pas
/// nulle et sa taille est dans la borne. Vérifié **avant** de toucher à ses octets.
pub(crate) fn copie_a_lire(marque: usize, taille: usize, adresse_nulle: bool) -> bool {
    marque == MARQUE_ARGUMENTS && !adresse_nulle && taille <= ARGUMENTS_OCTETS_MAX
}

/// Pourquoi la copie déjà lancée n'a pas été prévenue (les erreurs nommées du bloc N,
/// ORGANIGRAMME §9). Écrites au journal, puis traduites en `Err(false)` pour l'API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErreurUnique {
    /// Sa fenêtre cachée est introuvable : elle démarre ou se ferme.
    FenetreIntrouvable,
    /// Windows n'a pas pris le message qui demande de s'ouvrir.
    MessageNonParti,
    /// Elle n'a pas répondu dans le délai (elle ne répond plus, ou se ferme).
    PasDeReponse,
    /// Elle a répondu, mais n'a pas pris les arguments (marque ou contenu refusés).
    ArgumentsNonPris,
    /// Les arguments dépassent les bornes (16, 4 Ko) ou contiennent un zéro : rien n'est
    /// envoyé.
    ArgumentsRefuses,
}

impl fmt::Display for ErreurUnique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ErreurUnique::FenetreIntrouvable => "la fenêtre de la copie déjà lancée est introuvable",
            ErreurUnique::MessageNonParti => "Windows n'a pas pris la demande d'ouverture",
            ErreurUnique::PasDeReponse => "la copie déjà lancée n'a pas répondu à temps",
            ErreurUnique::ArgumentsNonPris => "la copie déjà lancée n'a pas pris les arguments",
            ErreurUnique::ArgumentsRefuses => {
                "les arguments dépassent les bornes (16, 4 Ko) ou contiennent un zéro"
            }
        })
    }
}

/// Ce que fait un second lancement (le verrou est déjà pris par une autre copie).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Decision {
    /// Il n'a rien à dire à l'autre copie ([`Reveil::Rien`]) : il s'arrête sans chercher sa
    /// fenêtre.
    SArreter,
    /// Il demande à l'autre copie de s'ouvrir ([`Reveil::Parametres`]).
    Ouvrir,
    /// Il transmet ces octets (les arguments codés) à l'autre copie.
    Transmettre(Vec<u8>),
    /// Ses arguments ne passent pas : il s'arrête sans rien envoyer.
    Refuser(ErreurUnique),
}

/// La décision d'un second lancement, selon son [`Reveil`].
pub(crate) fn que_dire(reveil: &Reveil) -> Decision {
    match reveil {
        Reveil::Rien => Decision::SArreter,
        Reveil::Parametres => Decision::Ouvrir,
        Reveil::Arguments(args) => match coder_arguments(args) {
            Some(octets) => Decision::Transmettre(octets),
            None => Decision::Refuser(ErreurUnique::ArgumentsRefuses),
        },
    }
}

/// Comment finit un lancement (`U` : le verrou ; `()` dans les essais).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Issue<U> {
    /// Seul : il garde le verrou jusqu'à la fin du programme.
    Seul(U),
    /// L'autre copie est prévenue (elle s'ouvre, ou a pris les arguments).
    Prevenue,
    /// Il n'y avait rien à lui dire.
    RienADemander,
    /// La demande est perdue ; le lancement s'arrête quand même.
    Perdue(ErreurUnique),
}

/// La traduction pour l'API d'aujourd'hui (`Result<Unique, bool>`, ORGANIGRAMME §9) :
/// `Err(true)` quand l'autre copie est prévenue ou qu'il n'y avait rien à lui dire,
/// `Err(false)` quand la demande est perdue.
pub(crate) fn traduire<U>(issue: Issue<U>) -> Result<U, bool> {
    match issue {
        Issue::Seul(verrou) => Ok(verrou),
        Issue::Prevenue | Issue::RienADemander => Err(true),
        Issue::Perdue(_) => Err(false),
    }
}

/// La ligne du journal d'une demande perdue (`None` pour les autres issues).
pub(crate) fn ligne_du_journal<U>(issue: &Issue<U>) -> Option<String> {
    match issue {
        Issue::Perdue(e) => Some(format!("Second lancement : demande perdue ({e}).")),
        _ => None,
    }
}
