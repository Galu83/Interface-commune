//! Ouvrir et fermer le volet et le petit menu, en calcul pur : la décision d'un clic sur
//! l'icône d'après ce qui est ouvert, l'anti-rebond juste après une fermeture, la garde des
//! premiers instants, le clic en dehors qui ferme (C-28), et la mesure d'une ouverture pour le
//! journal. La bibliothèque (`interface_commune::fenetres`) reprend tout à ses chemins.

use std::time::Duration;

use super::temps::Horodatage;
use crate::placement::Rect;
use crate::theme::Surface;

/// Ce qui est ouvert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ouvert {
    #[default]
    Rien,
    /// Le volet (clic gauche) ou le petit menu (clic droit) est montré.
    Surface(Surface),
    Parametres,
}

/// Ce que fait un clic sur l'icône (volet ou petit menu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clic {
    /// Ouvrir la surface demandée (la précédente, s'il y en a une, laisse la place).
    Ouvrir,
    /// La surface demandée est déjà montrée : la fermer.
    Fermer,
    /// Ne rien faire (le clic vient de fermer la surface en lui prenant le focus).
    Ignorer,
}

/// La décision d'un clic sur l'icône pour `surface`, d'après ce qui est ouvert. Les Paramètres
/// ouverts ne bloquent pas le clic : il se comporte comme si rien n'était ouvert (le volet ou
/// le petit menu s'ouvre devant eux, qui restent ouverts derrière). `juste_fermee` : cette
/// surface vient de se fermer ([`juste_fermee`]).
pub fn decider_le_clic(ouvert: Ouvert, surface: Surface, juste_fermee: bool) -> Clic {
    match ouvert {
        Ouvert::Surface(s) if s == surface => Clic::Fermer,
        Ouvert::Surface(_) => Clic::Ouvrir,
        // Le clic sur l'icône vient de la fermer (perte du focus) : ne pas la rouvrir.
        Ouvert::Rien | Ouvert::Parametres if juste_fermee => Clic::Ignorer,
        Ouvert::Rien | Ouvert::Parametres => Clic::Ouvrir,
    }
}

/// Après la fermeture du volet ou du petit menu, un clic sur l'icône ne le rouvre pas
/// pendant ce temps (ce clic est celui qui l'a fermé).
pub const ANTI_REBOND: Duration = Duration::from_millis(400);

/// `surface` s'est-elle fermée il y a moins d'[`ANTI_REBOND`] ? `fermee` : la dernière
/// fermeture (son instant et sa surface). Le clic sur l'icône qui l'a fermée en lui prenant le
/// focus ne doit pas la rouvrir aussitôt.
pub fn juste_fermee<T: Horodatage>(fermee: Option<(T, Surface)>, surface: Surface, maintenant: T) -> bool {
    fermee.is_some_and(|(t, s)| s == surface && maintenant.depuis_ou_zero(t) < ANTI_REBOND)
}

/// Une ouverture plus longue que ce temps est notée au journal, avec le détail.
pub const LENTE: Duration = Duration::from_millis(150);

/// Pendant ce temps après l'ouverture, clics et touches sont ignorés : un programme qui
/// ouvre le volet ou le petit menu ne peut pas cliquer une ligne dans la foulée, ni une
/// frappe destinée à une autre fenêtre choisir quelque chose.
pub const GARDE: Duration = Duration::from_millis(250);
/// La garde se lève au plus tard après ce temps.
pub const GARDE_MAX: Duration = Duration::from_secs(1);

/// La garde des premiers instants : ce qui est cliqué ou tapé pendant [`GARDE`] après
/// l'ouverture ne choisit rien. Elle se lève ensuite dès qu'aucun bouton ni aucune touche
/// n'est tenu : un appui commencé pendant la garde ne compte pas, même relâché après. Au bout
/// de [`GARDE_MAX`], elle se lève quoi qu'il arrive.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Garde {
    levee: bool,
}

impl Garde {
    pub fn levee(&self) -> bool {
        self.levee
    }

    /// Ce qui est fait de `action` à cette image : rien tant que la garde n'est pas levée.
    /// `depuis` : le temps depuis l'ouverture ; `tenu` : un bouton ou une touche est enfoncé.
    pub fn filtrer<A>(&mut self, action: Option<A>, depuis: Duration, tenu: bool) -> Option<A> {
        let permise = if self.levee { action } else { None };
        if depuis >= GARDE_MAX || (depuis >= GARDE && !tenu) {
            self.levee = true;
        }
        permise
    }
}

/// Le point (pixels de l'écran) est-il en dehors de la fenêtre (gauche, haut, droite, bas) ?
/// Le bord gauche et le bord haut sont dedans, le bord droit et le bord bas dehors (comme
/// Windows).
pub fn est_dehors(point: [i32; 2], fenetre: Rect) -> bool {
    let [x, y] = point;
    !(fenetre[0] <= x && x < fenetre[2] && fenetre[1] <= y && y < fenetre[3])
}

/// Un appui de bouton de souris (gauche, droit ou milieu) à `point`, `depuis` l'ouverture de
/// la fenêtre `fenetre`, doit-il la fermer ? Seulement un appui en dehors d'elle, et pas
/// pendant la [`GARDE`] des premiers instants. Un appui dedans ne ferme jamais : c'est la
/// fenêtre qui en fait ce qu'elle veut (choisir une ligne).
pub fn fermer_sur_appui(point: [i32; 2], fenetre: Rect, depuis: Duration) -> bool {
    depuis >= GARDE && est_dehors(point, fenetre)
}

/// Repère les nouveaux appuis de bouton de souris dans une suite de relevés (« un bouton est
/// enfoncé »), sans dépendre du focus : un appui est le passage de « aucun bouton » à « un
/// bouton ». Un bouton déjà tenu au premier relevé ne compte pas ; il faut d'abord le
/// relâcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Appuis {
    tenu: bool,
}

impl Default for Appuis {
    fn default() -> Appuis {
        Appuis { tenu: true }
    }
}

impl Appuis {
    /// Ce relevé est-il un nouvel appui ?
    pub fn nouvel_appui(&mut self, tenu: bool) -> bool {
        let nouveau = tenu && !self.tenu;
        self.tenu = tenu;
        nouveau
    }
}

/// Le temps d'une ouverture, par étape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mesure {
    /// Du clic à la demande lue par le fil des fenêtres.
    pub attente: Duration,
    /// De la demande lue à la fenêtre montrée (taille, place, première image).
    pub mise_en_place: Duration,
    /// Les images dessinées pendant la mise en place.
    pub images: u8,
}

impl Mesure {
    pub fn total(&self) -> Duration {
        self.attente + self.mise_en_place
    }

    /// Faut-il la noter ? La première ouverture, et toute ouverture lente.
    pub fn a_noter(&self, premiere: bool) -> bool {
        premiere || self.total() > LENTE
    }

    /// La ligne du journal.
    pub fn phrase(&self, surface: Surface, premiere: bool) -> String {
        let quoi = match surface {
            Surface::Volet => "Volet",
            Surface::Menu => "Petit menu",
        };
        let pourquoi = if premiere {
            "première ouverture"
        } else {
            "ouverture lente"
        };
        format!(
            "{quoi} ouvert en {} ms ({pourquoi}) : {} ms pour arriver au fil des fenêtres, {} ms de mise en place ({} image(s)).",
            self.total().as_millis(),
            self.attente.as_millis(),
            self.mise_en_place.as_millis(),
            self.images
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le filet « clic en dehors » : un appui dedans ne ferme jamais, un appui dehors ferme
    /// (même sur les Paramètres, ou sur la barre où est l'icône), sauf pendant la garde des
    /// premiers instants.
    #[test]
    fn clic_dedans_ou_dehors() {
        let f = [100, 200, 400, 500];
        // Dedans : bords gauche et haut compris, coin droit-bas exclu.
        for p in [[100, 200], [250, 350], [399, 499], [100, 499], [399, 200]] {
            assert!(!est_dehors(p, f), "{p:?}");
            assert!(!fermer_sur_appui(p, f, Duration::from_secs(5)), "{p:?}");
        }
        // Dehors, de tous les côtés.
        for p in [
            [99, 300],
            [400, 300],
            [250, 199],
            [250, 500],
            [0, 0],
            [-50, 300],
            [1000, 900],
        ] {
            assert!(est_dehors(p, f), "{p:?}");
            assert!(fermer_sur_appui(p, f, GARDE), "{p:?}");
        }
        // Garde : un appui dehors dans les premiers instants ne ferme rien.
        assert!(!fermer_sur_appui([0, 0], f, Duration::ZERO));
        assert!(!fermer_sur_appui([0, 0], f, GARDE - Duration::from_millis(1)));
        // Un appui dedans ne ferme jamais, garde levée ou non.
        assert!(!fermer_sur_appui([250, 350], f, Duration::ZERO));
    }

    /// Les appuis se repèrent au passage « aucun bouton » → « un bouton » ; un bouton déjà
    /// tenu à l'ouverture ne ferme rien tant qu'il n'est pas relâché.
    #[test]
    fn appuis_de_souris() {
        let mut a = Appuis::default();
        assert!(!a.nouvel_appui(true), "déjà tenu à l'ouverture");
        assert!(!a.nouvel_appui(true));
        assert!(!a.nouvel_appui(false));
        assert!(a.nouvel_appui(true), "nouvel appui");
        assert!(!a.nouvel_appui(true), "toujours tenu : pas un deuxième appui");
        assert!(!a.nouvel_appui(false));
        assert!(a.nouvel_appui(true), "un autre appui");

        let mut b = Appuis::default();
        assert!(!b.nouvel_appui(false), "rien de tenu à l'ouverture");
        assert!(b.nouvel_appui(true));
    }

    /// Avec les Paramètres ouverts, un clic sur l'icône ouvre le volet (clic gauche) ou le
    /// petit menu (clic droit), comme quand rien n'est ouvert ; le cas « juste fermée » reste
    /// respecté ; un clic sur la surface déjà montrée la ferme ; l'autre surface la remplace.
    #[test]
    fn clic_avec_les_parametres_ouverts() {
        for surface in [Surface::Volet, Surface::Menu] {
            let autre = if surface == Surface::Volet {
                Surface::Menu
            } else {
                Surface::Volet
            };
            // Rien d'ouvert, ou seulement les Paramètres : même décision.
            for ouvert in [Ouvert::Rien, Ouvert::Parametres] {
                assert_eq!(decider_le_clic(ouvert, surface, false), Clic::Ouvrir);
                assert_eq!(decider_le_clic(ouvert, surface, true), Clic::Ignorer);
            }
            assert_eq!(
                decider_le_clic(Ouvert::Surface(surface), surface, false),
                Clic::Fermer
            );
            assert_eq!(
                decider_le_clic(Ouvert::Surface(autre), surface, false),
                Clic::Ouvrir
            );
        }
    }

    /// Le journal note la première ouverture et toute ouverture de plus de 150 ms, avec le
    /// détail ; pas les autres.
    #[test]
    fn temps_d_ouverture_au_journal() {
        let rapide = Mesure {
            attente: Duration::from_millis(3),
            mise_en_place: Duration::from_millis(30),
            images: 2,
        };
        assert!(rapide.a_noter(true));
        assert!(!rapide.a_noter(false));
        let lente = Mesure {
            mise_en_place: Duration::from_millis(200),
            ..rapide
        };
        assert!(lente.a_noter(false));
        let phrase = lente.phrase(Surface::Volet, false);
        assert!(phrase.contains("203 ms"), "{phrase}");
        assert!(phrase.contains("200 ms de mise en place"), "{phrase}");
        assert!(
            rapide
                .phrase(Surface::Menu, true)
                .starts_with("Petit menu ouvert")
        );
    }

    /// Rien n'est choisi pendant les premiers instants, ni par un appui commencé pendant la
    /// garde et relâché après ; ensuite, tout marche normalement.
    #[test]
    fn garde_des_premiers_instants() {
        let mut g = Garde::default();
        let ms = Duration::from_millis;
        assert_eq!(g.filtrer(Some(1), ms(10), false), None);
        assert_eq!(g.filtrer(None::<i32>, ms(300), true), None);
        assert!(!g.levee());
        assert_eq!(g.filtrer(Some(2), ms(320), false), None);
        assert!(g.levee());
        assert_eq!(g.filtrer(Some(3), ms(400), false), Some(3));
        let mut g = Garde::default();
        assert_eq!(g.filtrer(None::<i32>, ms(900), true), None);
        assert!(!g.levee());
        assert_eq!(g.filtrer(None::<i32>, GARDE_MAX, true), None);
        assert!(g.levee());
    }

    /// Un clic sur l'icône juste après la fermeture de cette même surface ne la rouvre pas ;
    /// passé l'anti-rebond, ou pour l'autre surface, si.
    #[test]
    fn juste_fermee_par_le_clic() {
        let t = Duration::from_secs(1_000);
        let ms = |n| t + Duration::from_millis(n);
        let fermee = Some((t, Surface::Volet));
        assert!(!juste_fermee(None, Surface::Volet, t));
        assert!(juste_fermee(fermee, Surface::Volet, t));
        assert!(juste_fermee(fermee, Surface::Volet, ms(399)));
        assert!(!juste_fermee(fermee, Surface::Volet, ms(400)));
        assert!(!juste_fermee(fermee, Surface::Menu, ms(10)));
        // Une horloge qui recule compte comme « tout de suite » (comme `Instant::elapsed`).
        assert!(juste_fermee(Some((ms(50), Surface::Menu)), Surface::Menu, t));
    }
}
