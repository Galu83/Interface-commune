//! Plusieurs icônes près de l'horloge, pour un logiciel qui garde **sa propre boucle de
//! messages** (règles : interface Windows 11, point 10). Une fenêtre cachée, créée sur le fil
//! qui appelle [`Zone::creer`], reçoit les messages des icônes ; la boucle du logiciel
//! (`GetMessageW` / `DispatchMessageW`) les lui donne, puis le logiciel lit
//! [`Zone::evenements`].
//!
//! Chaque icône a :
//! - un **identifiant stable** choisi par le logiciel (« souris-1234 », « principale ») : son
//!   GUID en est tiré (avec le chemin du programme), pour que Windows retienne d'une fois sur
//!   l'autre si l'utilisateur la montre dans la barre des tâches ;
//! - son image et son info-bulle, changées en marche ;
//! - ses propres notifications.
//!
//! Elle peut être ajoutée ou retirée à tout moment. Quand l'Explorateur redémarre, la zone
//! remet seule toutes les icônes. Quand Windows change l'affichage (`WM_DISPLAYCHANGE`), elle
//! le signale à [`crate::autres_fenetres::affichage_change`] : les fenêtres pas encore créées
//! attendent alors 1 s.
//!
//! Si une icône existe déjà quand la zone l'ajoute (Explorateur redémarré, reste d'une
//! instance arrêtée brutalement), elle est modifiée à la place : l'ajout ne se perd pas.
//!
//! Quand la session Windows se ferme (déconnexion, arrêt), le logiciel n'a que quelques secondes
//! pour enregistrer : [`Zone::sur_fin_de_session`] lui donne la main **pendant** le message
//! (voir [`FinDeSession`]).
//!
//! Un clic dit **quelle icône** et **où elle est** ([`Evenement`]). Le menu gris de Windows
//! ([`Zone::menu_windows`]), avec cases cochées et sous-menus, sert de secours quand les
//! fenêtres modernes ne peuvent pas s'ouvrir.
//!
//! Un logiciel qui n'a qu'une icône et laisse la bibliothèque tout faire prend plutôt
//! `horloge::Horloge`.
//!
//! **Le volet, le petit menu et les Paramètres communs** pour ces icônes :
//! [`FenetresCommunes`] lance le fil des fenêtres de la bibliothèque (le même que celui de
//! `Horloge`) ; chaque geste lu par [`Zone::evenements`] lui est donné
//! ([`FenetresCommunes::geste`]) et passe par les mécanismes communs (`fenetres::Demandes`,
//! `fenetres::Liens`) : clic gauche, le volet ; clic droit, le petit menu ; double-clic, les
//! Paramètres ; un second clic referme ; rien ne se rouvre juste après une fermeture.
//!
//! ```no_run
//! use interface_commune::zone::{Image, Zone};
//! let mut zone = Zone::creer("MonLogiciel", "MonLogiciel").map_err(|e| e.to_string())?;
//! let image = Image { taille: 16, bgra: vec![0; 16 * 16 * 4] };
//! zone.poser("souris", &image, "Souris — 80 %");
//! // Dans la boucle de messages du logiciel, après DispatchMessageW :
//! for e in zone.evenements() {
//!     println!("{} : {:?} à {:?}", e.icone, e.message, e.rect);
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! Le code est dans `src/bords/icone/` (bloc M) : le moteur des icônes
//! (`bords/icone/zone.rs`, que `horloge::Horloge` utilise aussi), le décodage pur
//! (`decodage.rs`), l'image (`image.rs`) et les fenêtres communes (`communes.rs`). Ce module
//! en reprend les chemins d'aujourd'hui, sans `unsafe`.

pub use crate::bords::icone::communes::{Aiguilleur, FenetresCommunes, SuiteDuGeste};
pub use crate::bords::icone::decodage::{
    Bouton, Evenement, FinDeSession, ICONES_MAX, INFOBULLE_MAX, LigneMenu, Message, guid_stable, raccourcir,
    texte_menu,
};
pub use crate::bords::icone::image::Image;
pub use crate::bords::icone::zone::Zone;
