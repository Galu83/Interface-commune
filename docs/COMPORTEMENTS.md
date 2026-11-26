# Comportements de l'Interface commune

Étape 0 de l'étude « code propre » (24 nov. 2026). Ce document dresse la liste de **ce qu'un
logiciel qui utilise la bibliothèque peut voir** : ce qu'elle montre, ce qu'elle fait, ce
qu'elle promet. Une refonte, quelle qu'elle soit, doit garder chacune de ces lignes, ou dire
pourquoi elle la change.

Pour chaque comportement, on donne l'essai qui le vérifie :

- **auto** : un test automatique, sous la forme `fichier::nom_du_test` (les fichiers `src/…`
  ont leurs tests dans le module `tests` du fichier ; `tests/…` sont les essais à part) ;
- **auto, GitHub** : un test qui ne tourne que sur GitHub (il ouvre de vraies fenêtres, ou
  fabrique de faux clics avec `ESSAI_SENDINPUT=oui`) ;
- **manuel** : vérifié à la main par le propriétaire ou une session, sans test qui le garde ;
- **essai manquant** : rien ne le vérifie aujourd'hui. Une refonte pourrait le casser sans
  qu'on le voie.

Le relevé a été fait sur la branche de l'étude, au commit de départ de `main`. Les tests sans
fenêtre (`--lib` et les essais `sans_fenetre`, `conserver`, `couleurs`, `palette_pastilles`,
`carte_info`, `polices`, `licences`, `zone_affichage`) passent tous : **254 tests, 0 échec**.
Les essais en vraies fenêtres n'ont pas été lancés sur le PC (aucune fenêtre chez le
propriétaire pendant les essais) ; ils tournent sur GitHub.

## Mots employés

- **Le volet** : la fenêtre qui s'ouvre au clic gauche sur l'icône, près de l'horloge.
- **Le petit menu** : le menu du clic droit sur l'icône.
- **Une brique** : une ligne toute faite du volet (interrupteur, appareil, pastille…).
- **La boucle des fenêtres** : le fil qui dessine toutes les fenêtres egui du logiciel.
- **Le menu de secours** : le menu natif de Windows, montré quand egui ne peut pas dessiner.
- **Un instantané** : la photo des réglages, prise pour pouvoir « Annuler ».

## 1. L'icône près de l'horloge et ses gestes

| N° | Comportement | Essai |
|---|---|---|
| C-01 | Clic gauche sur l'icône : le volet s'ouvre ; clic droit : le petit menu ; double-clic : les Paramètres. | auto : `ic_coeur::gestes::clics::clic_gauche_droit_et_clavier`, `ic_coeur::gestes::clics::double_clic_les_parametres` |
| C-02 | Au clavier (Entrée, Espace, touche menu sur l'icône sélectionnée), mêmes suites qu'à la souris. | auto : `ic_coeur::gestes::clics::clic_gauche_droit_et_clavier` |
| C-03 | Si le logiciel a une fenêtre principale, le double-clic l'ouvre au lieu des Paramètres. | auto : `ic_coeur::gestes::clics::suite_selon_la_fenetre_principale`, `zone::double_clic_la_fenetre_principale` |
| C-04 | Les messages de Windows sur l'icône (clics, clavier, info-bulle) sont lus de la même façon par l'icône seule (`horloge`) et par la zone à plusieurs icônes (`zone`). | auto : `horloge::messages_de_windows_au_logiciel`, `zone::messages_de_l_icone`, `zone::gestes_pour_les_clics_communs`, `zone::gestes_par_les_mecanismes_communs` ; l'aiguillage unique des deux, dans le cœur (bloc D) : `ic_coeur::gestes::aiguilleur::*` (4 tests), `ic_coeur::gestes::clics::nom_des_gestes_pour_la_sonde` |
| C-05 | L'icône ne clignote pas quand son image change (même image : rien n'est renvoyé à Windows). | auto : `ic_coeur::gestes::icone::icone_contre_le_clignotement` (bloc D : dans le cœur, horloge inventée) |
| C-06 | L'icône revient toute seule si l'Explorateur de Windows redémarre. | manuel |
| C-07 | L'info-bulle trop longue est raccourcie à la limite de Windows, sans couper un caractère. | auto : `ic_coeur::gestes::icone::info_bulle_raccourcie`, `ic_coeur::gestes::icone::copie_dans_le_tampon_de_windows` (bloc D : un emoji n'est plus coupé en deux dans le tampon de Windows) |
| C-08 | Une icône de la zone garde le même identifiant d'un lancement à l'autre (GUID tiré du nom), et deux noms donnent deux identifiants. | auto : `zone::guid_stable_et_unique` |
| C-09 | Si Windows refuse d'ajouter l'icône (elle existe déjà), la bibliothèque la modifie à la place. | auto : `zone::ajout_avec_repli_sur_la_modification` |
| C-10 | L'image de l'icône se fait depuis des pixels RVBA du logiciel. | auto : `zone::image_depuis_rvba` |
| C-11 | La fin de session de Windows (déconnexion, arrêt) est reçue et rendue au logiciel (`FinDeSession`). | auto : `zone::fin_de_session_decodee`, `zone::fin_de_session_recue_par_la_fenetre_cachee` |
| C-12 | Une fenêtre bloquée ne retient pas la fermeture du logiciel. | auto : `horloge::fenetre_bloquee_ne_retient_pas_la_fermeture` |
| C-13 | Le changement d'affichage (écran branché, résolution) est signalé au logiciel. | auto : `tests/zone_affichage::la_zone_signale_le_changement_d_affichage` |
| C-14 | Les messages venus d'un autre fil (`Reveil`, `Poste`) arrivent sur le fil principal, dans l'ordre. | auto : `bords::poste::essais::code_pour_le_fil_principal`, `…::rafales_de_plusieurs_fils` (8 fils, ordre de chaque fil gardé), `…::le_fil_reveille_trouve_tout`, `…::chaque_demande_va_au_bon_fil` (bloc K : faux réveil, sans fenêtre) |

## 2. Une seule copie du logiciel

| N° | Comportement | Essai |
|---|---|---|
| C-15 | Une deuxième copie lancée ne s'ouvre pas : elle passe ses arguments à la première et s'arrête sans rien afficher. | auto : `bords::unique::essais::second_lancement_sans_rien_dire`, `bords::unique::essais::arguments_du_second_lancement`, `bords::unique::essais::bornes_exactes`, `bords::unique::essais::decisions_du_second_lancement`, `bords::unique::essais::traduction_pour_l_api` (bloc N) |
| C-16 | Les données reçues de la deuxième copie sont lues avec prudence (taille, texte valide). | auto : `bords::unique::essais::copie_de_donnees_lue_avec_prudence` (bloc N) |
| C-17 | La première copie montre ses Paramètres (ou la page demandée) quand la deuxième arrive. | auto (en partie) : `tests/comportements::c17_la_premiere_copie_montre_ses_parametres` (verrou pris, demande « Paramètres » et page voulue) ; le message reçu par la copie déjà lancée demande une vraie icône : GitHub |
| C-18 | La copie unique vaut par session Windows (verrou `Local\…`) : deux utilisateurs connectés en même temps ont chacun la leur. | auto : `tests/comportements::c18_une_seule_copie_par_session` (nom exact `Local\<nom>-une-seule`, absent de `Global\`) ; `bords::unique::essais::verrou_dans_la_session`, `bords::unique::essais::noms_du_rendez_vous` (bloc N) |

## 3. Les notifications

| N° | Comportement | Essai |
|---|---|---|
| C-19 | Trois niveaux de notification (information, attention, erreur), avec l'icône de Windows qui va avec. | auto : `bords::poste::essais::notification_a_trois_niveaux` |
| C-20 | Un clic sur une notification ouvre la page des Paramètres qu'elle nomme. | auto : `bords::poste::essais::clic_sur_une_notification` |
| C-21 | Le bouton « Essayer » des notifications n'en montre une que si on le demande. | auto : `tests/sans_fenetre::notifications_essayer_seulement_si_demande` |

## 4. Le volet

| N° | Comportement | Essai |
|---|---|---|
| C-22 | Le volet est préparé au repos ; deux ouvertures ne le reconstruisent pas (une seule fenêtre, cachée puis montrée). | auto : `fenetres::deux_ouvertures_ne_reconstruisent_pas_le_volet` |
| C-23 | En-tête du volet : nom du logiciel en gros, version petite et grise à droite, roue dentée des Paramètres à l'extrême droite ; puis l'état en couleur ; puis les briques. Ni « Quitter » ni ligne du bas. | auto : `tests/sans_fenetre::volet_et_petit_menu_sans_fenetre`, captures de `tests/sans_fenetre::captures_pour_la_documentation` (images non comparées) |
| C-24 | Un titre long ne chevauche jamais la roue dentée. | auto : `tests/sans_fenetre::titre_long_ne_chevauche_pas_la_roue` |
| C-25 | Rien n'est coupé : la largeur suit les textes, jusqu'à une largeur maximale ; au-delà, les phrases passent à la ligne. | auto : `tests/sans_fenetre::volet_borne_grandit_avec_son_contenu`, `tests/comportements::c25_un_long_texte_passe_a_la_ligne`, `ic_coeur::placement::surface::essais::largeur_entre_min_et_max` |
| C-26 | Volet de hauteur bornée : l'en-tête et l'état restent en place, les briques défilent, le pied reste fixe et la barre de défilement est visible. | auto : `tests/sans_fenetre::volet_borne_pied_fixe_et_barre_visible`, `tests/sans_fenetre::volet_borne_et_briques_avec_contexte` |
| C-27 | Le volet se place contre l'icône, du côté de la barre des tâches (bas, haut, gauche, droite, barre masquée), toujours en entier sur l'écran. | auto : `ic_coeur::placement::essais::barre_en_bas`, `ic_coeur::placement::essais::les_autres_cotes`, `ic_coeur::placement::essais::barre_masquee`, `ic_coeur::placement::essais::contre_l_icone`, `ic_coeur::placement::essais::contre_l_icone_toujours_dans_la_zone`, `ic_coeur::placement::essais::placement_du_volet` |
| C-28 | Un clic hors du volet le ferme ; un clic dedans ne le ferme pas. | auto : `ic_coeur::gestes::ouverture::clic_dedans_ou_dehors`, `…::appuis_de_souris`, `…::juste_fermee_par_le_clic` (bloc D : dans le cœur) |
| C-29 | Un deuxième clic sur l'icône tout de suite après l'ouverture n'est pas pris pour une fermeture (garde des premiers instants, anti-rebond de 400 ms). | auto : `ic_coeur::gestes::ouverture::garde_des_premiers_instants` (bloc D : dans le cœur) |
| C-30 | Animation d'ouverture : fondu de 83 ms parti de 50 %, à sa place ; fermeture en fondu de 83 ms ; aucune animation si Windows les a coupées. | auto : `ic_coeur::theme::essais::courbe_rapide_puis_ralentie`, `…::durees_de_windows`, `…::ouverture_vive`, `…::sans_effets_aucune_animation`, `…::un_retard_ne_fait_pas_sauter_l_animation` (bloc C : dans le cœur) |
| C-31 | La première image montrée est déjà dessinée (pas de fenêtre vide ou noire au premier clic) ; la fenêtre reste voilée tant qu'elle n'est pas vraiment montrée. | auto : `volet::premiere_image_montree_deja_visible`, `volet::voilee_sauf_montree_pour_de_vrai` |
| C-32 | Les fenêtres n'attendent pas la synchronisation verticale (le clic ne patiente pas derrière l'écran). | auto : `volet::fenetres_sans_synchronisation_verticale` |
| C-33 | Dans le volet, les flèches du clavier se comportent comme Tab (d'une brique à l'autre). | auto : `dessin::volet::clavier::essais::les_fleches_deviennent_tab`, `ic_coeur::gestes::clavier::essais::fleches_comme_tab` |
| C-34 | Le logiciel peut observer l'ouverture et la fermeture du volet sans fenêtre (veilleur public). | auto : `volet::veilleur_public_sans_fenetre` |
| C-35 | Le temps d'ouverture du volet est écrit au journal. | auto : `ic_coeur::gestes::ouverture::temps_d_ouverture_au_journal` (bloc D : dans le cœur) |

## 5. Les briques du volet

| N° | Comportement | Essai |
|---|---|---|
| C-36 | Toutes les briques ont les hauteurs communes du volet de Windows 11. | auto : `dessin::briques::hauteurs_communes`, `dessin::pastilles::hauteurs` (bloc F : dans `src/dessin/`) |
| C-37 | Une brique répond au clic, au clavier et rend son geste au logiciel. | auto : `dessin::briques::gestes`, `dessin::pastilles::gestes` |
| C-38 | La brique « appareil » montre une image et un bouton. | auto : `dessin::briques::appareil_avec_image_et_bouton` (la brique dans `dessin/briques/appareil.rs`) |
| C-39 | Les pastilles (plan des écrans) gardent les proportions des écrans, centrées dans leur zone, limitées par la largeur, quelle que soit l'origine des coordonnées ; refus d'un plan vide ; rangée quand il le faut ; petite case en couleur seule. | auto : `ic_coeur::placement::plan::plan_*` (6 tests, bloc D : la géométrie dans le cœur), `dessin::pastilles::{plan_ou_rangee, plan_petite_case_en_couleur_seule, plan_dans_le_volet, plan_ne_deplace_pas_le_texte}` |
| C-40 | Le texte d'une pastille reste au même endroit, et sa lettre est lisible sur son fond. | auto : `dessin::pastilles::{texte_au_meme_endroit, plan_ne_deplace_pas_le_texte}`, `ic_coeur::theme::essais::lettre_selon_le_fond`, `ic_coeur::theme::marque::{le_logo_si_la_police_l_a_sinon_la_lettre, le_texte_sur_une_zone_est_lisible}` (bloc F) |
| C-117 | Un écran débranché garde sa place dans le choix illustré (rangée ou plan), en pâle (opacité 0,4) avec un bord en tirets, lettre gardée ; débranché l'emporte sur la veille. | auto : `ic_coeur::theme::marque::debranche_l_emporte_sur_la_veille` (la décision et l'opacité, dans le cœur), `dessin::pastilles::debranche_l_emporte_sur_la_veille` (le dessin), `tests/references` (`cases-debranchees-*`) |
| C-41 | Les briques reçoivent le contexte du logiciel (`briques_avec_contexte`) et un pied (`briques_du_pied`). | auto : `tests/sans_fenetre::volet_borne_et_briques_avec_contexte` |

## 6. Le petit menu

| N° | Comportement | Essai |
|---|---|---|
| C-42 | Toujours le même ordre : actions du logiciel (la sienne « Ouvrir » en tête), Paramètres (E713), Rechercher une mise à jour (E895), lignes en plus (Ouvrir le journal…), Quitter. | auto : `ic_coeur::gestes::menu::ordre_du_petit_menu`, `ic_coeur::gestes::menu::ouvrir_le_logiciel_en_tete` |
| C-43 | Le petit menu prend les mesures des menus de Windows 11. | auto : `tests/sans_fenetre::petit_menu_aux_mesures_communes` |
| C-44 | Il s'ouvre au point du clic, toujours en entier sur l'écran. | auto : `ic_coeur::placement::essais::petit_menu_au_point`, `ic_coeur::placement::essais::petit_menu_toujours_dans_la_zone` |
| C-45 | Sans OpenGL (pilote absent ou cassé), le petit menu passe au menu de secours de Windows, avec les mêmes lignes numérotées. | auto : `fenetres::sans_opengl_le_menu_de_secours`, `zone::menu_windows_numerote_les_choix` |
| C-46 | Les raccourcis s'écrivent en toutes lettres (« Ctrl + Maj + K »). | auto : `tests/sans_fenetre::raccourcis_en_toutes_lettres`, `ic_coeur::textes::essais` (7 tests de `lisible`, plus `lisible_ne_change_pas_deux_fois` au hasard), `dessin::touches::ce_que_saisit_le_champ_de_raccourci_se_lit_bien` |

## 7. La boucle des fenêtres et ses secours

| N° | Comportement | Essai |
|---|---|---|
| C-47 | Le volet, le petit menu, les Paramètres et les fenêtres du logiciel sont dessinés par une seule boucle ; on peut les ouvrir depuis n'importe quel fil. | auto : `fenetres::suite_des_demandes`, `fenetres::parametres_ouverts_dans_la_boucle`, `autres_fenetres::plusieurs_fenetres_et_croix` ; dans la boucle du logiciel (façon 2, vue du dehors) : `compatibilite::powerpeek::les_parametres_s_ouvrent_dans_la_boucle_sans_fenetre` |
| C-48 | Un clic sur l'icône pendant que les Paramètres sont ouverts fait ce qu'il doit (volet devant, puis retour). | auto : `ic_coeur::gestes::ouverture::clic_avec_les_parametres_ouverts`, `fenetres::surface_devant_les_parametres_puis_retour` |
| C-49 | La fenêtre principale perdue (fermée par Windows, plantage d'egui) est refaite, et le clic qui l'a révélé ouvre quand même. | auto : `fenetres::fenetre_perdue_recreee_et_le_clic_ouvre_quand_meme` |
| C-50 | Après 3 pertes en 60 s, la bibliothèque arrête de refaire la fenêtre et passe au menu de secours ; elle réessaie au clic suivant. | auto : `fenetres::pertes_en_rafale_menu_de_secours_puis_recreee_au_clic_suivant`, `fenetres::supervision_rafale_puis_reprise_au_clic_suivant`, `fenetres::supervision_reprise_apres_rafale`, `fenetres::supervision_pertes_espacees_toujours_recreees`, `fenetres::supervision_echec_premier_tour_ou_passager`, `ic_coeur::gestes::secours::pertes_sur_la_derniere_rafale`, `…::reprise_selon_la_perte`, `…::phrases_des_pertes` (bloc D : la décision dans le cœur) ; vue du dehors, avec une horloge inventée : `compatibilite::powerpeek::la_supervision_vue_du_dehors` |
| C-51 | Si le fil des fenêtres s'arrête, le petit menu passe au menu de secours. | auto : `fenetres::fil_des_fenetres_arrete_menu_de_secours` |
| C-52 | Quitter arrête la boucle proprement ; rien n'est préparé après « Quitter ». | auto : `fenetres::supervision_quitter`, `fenetres::rien_n_est_prepare_apres_quitter`, `compatibilite::powerpeek::la_supervision_vue_du_dehors` |
| C-53 | Une fenêtre perdue par une panique est refaite sur le même fil. | auto, GitHub : `tests/recreation_apres_panique` |
| C-54 | Les Paramètres s'ouvrent au premier essai, juste après le démarrage. | auto, GitHub : `tests/premiere_creation` |
| C-55 | Des fenêtres ouvertes et fermées sans pause ne bloquent jamais la boucle. | auto, GitHub : `tests/fenetres_en_rafale` |
| C-56 | Les fenêtres répondent aux vrais clics et au vrai clavier de Windows. | auto, GitHub : `tests/entrees_fabriquees` (seulement avec `ESSAI_SENDINPUT=oui`) |
| C-57 | Seules les fenêtres de winit du fil sont reprises par la bibliothèque, jamais celles d'un autre. | auto : `systeme::fenetre_du_fil_ne_rend_que_les_fenetres_de_winit_pas_prises` |

## 8. Les autres fenêtres du logiciel (`autres_fenetres`)

| N° | Comportement | Essai |
|---|---|---|
| C-58 | Plusieurs fenêtres à la fois, chacune avec sa croix ; la croix ferme la bonne. | auto : `autres_fenetres::plusieurs_fenetres_et_croix` |
| C-59 | Le logiciel peut refuser une fermeture (crochet de fermeture). | auto : `autres_fenetres::crochet_de_fermeture_avec_refus` |
| C-60 | Une création ratée ne fait pas planter ; un nouvel essai suit. | auto : `autres_fenetres::creation_ratee_sans_panique_puis_nouvel_essai` |
| C-61 | Une fenêtre peut être créée cachée, puis montrée plus tard ; un contenu pas prêt garde sa fenêtre cachée. | auto : `autres_fenetres::creation_differee_jamais_montree_puis_reveillee`, `autres_fenetres::un_contenu_pas_pret_garde_sa_fenetre_cachee`, `autres_fenetres::attendre_ou_dessiner` |
| C-62 | Une panique dans le contenu d'une fenêtre ne tue pas les autres. | auto : `autres_fenetres::panique_du_contenu_suit_son_cours` |
| C-63 | Une fenêtre peut en ouvrir une autre ; une fenêtre peut être ouverte et fermée dans la même image ; recréée après destruction. | auto : `autres_fenetres::un_contenu_ouvre_une_autre_fenetre`, `autres_fenetres::deuxieme_fenetre_ouverte_puis_fermee_dans_la_meme_boucle`, `autres_fenetres::creation_apres_destruction` |
| C-64 | Une fenêtre glissée d'un écran à l'autre d'échelle différente (125 %, 150 %) suit la souris sans sauter ni revenir. | auto (calcul) : `ic_coeur::placement::essais::glisser_vers_l_ecran_du_haut`, `ic_coeur::placement::essais::glisser_vers_le_portable`, `ic_coeur::placement::essais::changement_d_echelle_*` ; manuel pour le vrai glisser (`echelle`) |

## 9. Les Paramètres

| N° | Comportement | Essai |
|---|---|---|
| C-65 | Fenêtre « Paramètres de <logiciel> » : en-tête avec le vrai logo, le nom et « Version x.y.z » ; navigation à gauche (lignes de 38 points) ; dessin de PowerPeek. | auto : `tests/sans_fenetre::entete_et_navigation_comme_powerpeek`, `tests/sans_fenetre::parametres_sans_fenetre` |
| C-66 | Ordre des pages : celles du logiciel, puis Général, Mise à jour, À propos. | auto : `ic_coeur::parametres::essais::ordre_des_pages` |
| C-67 | Une page arrivée après l'ouverture apparaît dans la navigation. | auto : `tests/sans_fenetre::pages_arrivees_apres_l_ouverture`, `ic_coeur::parametres::essais::page_gardee_quand_les_pages_changent` |
| C-68 | Recherche dès 7 pages, sans tenir compte des accents ; elle ouvre la section trouvée. | auto : `ic_coeur::parametres::essais::recherche_sans_accents`, `ic_coeur::parametres::essais::section_ouverte_par_la_recherche`, `ic_coeur::parametres::essais::recherche_montree_a_partir_de_sept_pages` |
| C-69 | Point d'alerte dans la navigation quand une page le demande. | auto : `tests/sans_fenetre::point_d_alerte_dans_la_navigation` |
| C-70 | Effet immédiat : un réglage change tout de suite ; la bulle « Annuler » reste 6 s, puis le changement est gardé ; deux changements de suite gardent l'« avant » du premier. | auto : `ic_coeur::parametres::essais_gestes::effet_immediat_et_annuler`, `ic_coeur::parametres::essais_gestes::la_bulle_disparait_apres_six_secondes`, `ic_coeur::parametres::essais_gestes::envois_d_un_changement_et_d_annuler`, `ic_coeur::parametres::essais_gestes::deux_changements_gardent_l_avant_du_premier`, `tests/sans_fenetre::effet_immediat_et_annuler_vers_le_logiciel` |
| C-71 | « Conserver ces changements ? » : compte à rebours ; Entrée, Échap, clic, délai écoulé et fermeture des Paramètres font revenir ou garder comme il faut. | auto : `tests/conserver::*` (8 tests), `ic_coeur::parametres::essais_gestes::conserver_*`, `ic_coeur::parametres::essais_gestes::revenir_rend_la_valeur_d_avant`, `ic_coeur::parametres::essais_gestes::delai_ecoule_retour_automatique`, `ic_coeur::parametres::essais_gestes::changer_a_confirmer_pendant_et_apres_le_delai`, `ic_coeur::parametres::essais_gestes::groupe_a_confirmer`, `ic_coeur::parametres::essais_gestes::echu_rend_le_brouillon_d_abord`, `ic_coeur::parametres::essais_gestes::retour_en_envois` |
| C-72 | Brouillon à confirmer : une page entière modifiée puis validée ou annulée d'un coup ; pied de page et fermeture qui demandent. | auto : `ic_coeur::parametres::essais_gestes::brouillon_a_confirmer`, `ic_coeur::parametres::essais_gestes::annuler_un_brouillon_entier`, `tests/sans_fenetre::brouillon_entier_pied_de_page_et_fermeture` |
| C-73 | `--parametres <page>` sur la ligne de commande ouvre la page nommée. | auto : `ic_coeur::parametres::essais::ligne_de_commande` |
| C-74 | Page Général : démarrage avec Windows (grisé si interdit), « Rétablir » du logiciel, carte propre au logiciel, dossier fixe. | auto : `tests/sans_fenetre::demarrage_grise_et_retablir_du_logiciel`, `tests/sans_fenetre::carte_propre_dans_general_et_dossier_fixe` |
| C-75 | Journal détaillé, et « Effacer » avec confirmation. | auto : `tests/sans_fenetre::journal_detaille_et_effacer_avec_confirmation` |
| C-76 | Page Mise à jour : recherches, dossier automatique ; une seule icône selon l'état ; titre et couleurs selon l'état, dans les deux langues ; ligne grise avec la version et la dernière recherche ; pas de redite du dossier. | auto : `tests/sans_fenetre::mise_a_jour_recherches_et_dossier_automatique`, `ic_coeur::parametres::essais_mise_a_jour::*` (8 tests : icône, titre, couleurs, ligne grise, pas de redite, bouton de la carte, carte du dossier) |
| C-77 | Page À propos : licences, crédits, groupe. | auto : `tests/sans_fenetre::a_propos_licences_credits_et_groupe` |
| C-78 | Bandeau « Redémarrer maintenant » quand un réglage le demande. | auto : `tests/comportements::c78_bandeau_redemarrer`, `ic_coeur::parametres::essais_gestes::bandeau_redemarrer` ; dessin : images de référence (`tests/references`) |
| C-79 | Taille et place de la fenêtre retenues d'une ouverture à l'autre, toujours en entier sur un écran ; retenues aussi 1 s après un déplacement ; jamais celles d'une fenêtre réduite ou agrandie. | auto : `ic_coeur::placement::essais::place_des_parametres`, `ic_coeur::placement::essais::place_retenue_*` (3 tests), `ic_coeur::placement::essais::trop_grand_rapetisse`, `ic_coeur::placement::essais::toujours_dans_la_zone`, `ic_coeur::placement::essais::valeurs_absurdes`, `tests/comportements::c79_place_des_parametres` (ouverture et fermeture) ; le suivi 1 s après un déplacement demande une vraie fenêtre : GitHub |
| C-80 | La colonne de navigation s'élargit pour un long nom de page. | auto : `tests/comportements::c80_la_navigation_s_elargit`, `ic_coeur::parametres::essais::largeur_de_la_colonne` (le calcul, bornes comprises) |
| C-81 | Les choix illustrés (images sous les choix) sont alignés. | auto : `tests/sans_fenetre::choix_illustres_alignes` |
| C-82 | Les Paramètres s'utilisent seuls, sans l'icône ni le volet (trait `Reglages`), y compris en anglais. | auto : `tests/sans_fenetre::parametres_seuls_en_anglais` ; le contrat seul, dans le cœur : `ic_coeur::contrat::essais::*` ; Chat PDC s'en sert |
| C-83 | La carte d'information reste dans la largeur donnée et n'est pas plus haute qu'il faut. | auto : `tests/carte_info::*` |
| C-84 | Coins arrondis et barre de titre sombre par Windows ; aucune animation à nous dans les Paramètres. | manuel |

## 10. Les fenêtres toutes faites (`toutes_faites`)

| N° | Comportement | Essai |
|---|---|---|
| C-85 | La fenêtre simple : texte, barre d'avancement, champ, choix, boutons, confirmation. | auto : `toutes_faites::boutons_champ_choix_et_confirmation` |
| C-86 | Sa hauteur suit le contenu. | auto : `toutes_faites::la_hauteur_suit_le_contenu` |
| C-87 | La fenêtre « version prête » (mise à jour) toute faite. | auto : `toutes_faites::version_prete_toute_faite` |
| C-88 | La fenêtre de lecture : le mot lu est surligné et suivi ; un clic entre deux mots ou sur un mot le rend au logiciel. | auto : `toutes_faites::mots_et_clic_entre_deux_mots`, `toutes_faites::mot_surligne_suivi_et_clique` |
| C-89 | Ces fenêtres tournent dans la boucle commune. | auto : `toutes_faites::fenetres_toutes_faites_dans_la_boucle` |

## 11. Le thème, les polices, les couleurs

| N° | Comportement | Essai |
|---|---|---|
| C-90 | Clair ou sombre comme Windows, et l'accent de Windows, relus toutes les 2 s et appliqués dès qu'ils changent. | auto : `ic_coeur::theme::essais::la_palette_de_windows_est_suivie`, `…::accent_par_defaut_et_accent_seul`, `…::relecture_toutes_les_deux_secondes`, `…::le_theme_de_windows_est_suivi`, `…::un_theme_impose_l_emporte` (horloge inventée), `tests/comportements::c90_le_theme_de_windows_est_relu` |
| C-91 | Les contrastes des couleurs du thème sont suffisants. | auto : `ic_coeur::theme::essais::contrastes_suffisants` |
| C-92 | Segoe UI Variable (repli Segoe UI / Semibold) et Segoe Fluent Icons (repli Segoe MDL2 Assets), lues sur le PC une seule fois, jamais livrées. | auto : `tests/polices::chaque_icone_existe`, `tests/polices::chaque_caractere_existe`, `bords::verification::*`, `tests/polices_windows10::c92_repli_des_polices_de_windows_10` |
| C-93 | Chaque icône commune existe dans les deux polices d'icônes ; un glyphe absent redonne la lettre. | auto : `tests/polices::chaque_icone_existe`, `bords::verification::glyphe_absent_redonne_la_lettre` |
| C-94 | La palette des pastilles : couleurs distinctes et lisibles (ΔE 2000), les six premiers rangs ne changent jamais. | auto : `ic_coeur::theme::essais_pastilles::*` (4 tests ; avant le bloc C : `tests/palette_pastilles.rs`) |
| C-95 | Le choix de couleur (carte) : même palette que les pastilles, clic, clavier, colonnes selon la place, « Annuler » remet l'ancienne. | auto : `tests/couleurs::*` (10 tests) |
| C-96 | Le nuancier : cases de 32 points, un clic donne la couleur, nom lu par les lecteurs d'écran. | auto : `dessin::nuancier::*` (4 tests) |
| C-97 | Tout ce qu'écrit la bibliothèque existe en français et en anglais, rempli dans l'ordre ; la langue change en direct. | auto : `ic_coeur::textes::essais::{deux_langues_completes, les_deux_catalogues_ont_les_memes_champs_et_aucun_vide, remplir_dans_l_ordre, les_modeles_se_remplissent_avec_le_logiciel_fictif}`, `tests/sans_fenetre::la_langue_change_en_direct` |
| C-98 | Les nombres ont une espace insécable avant leur unité. | auto : `ic_coeur::textes::essais::{c98_aucune_espace_simple_avant_une_unite, avec_unite_met_l_espace_insecable}`, `dessin::seuils::les_nombres_ont_une_espace_insecable`, `tests/comportements::c98_espace_insecable_avant_l_unite`, `dessin::briques::le_pourcentage_a_l_espace_insecable` ; les textes sont corrigés (bloc B de la refonte : `conserver_retour` et « 24 h » du journal détaillé) ; le pourcentage des appareils du volet aussi (bloc F, R2 : `briques::pourcent` passe par `avec_unite`) |

## 12. Les contrôles communs

| N° | Comportement | Essai |
|---|---|---|
| C-99 | La roue dentée en haut à droite ; Ctrl + virgule ouvre les Paramètres. | auto : `dessin::controles::interrupteur::roue_en_haut_a_droite_et_ctrl_virgule` (bloc F) |
| C-100 | L'interrupteur écrit à gauche (texte puis interrupteur). | auto : `dessin::controles::interrupteur::interrupteur_ecrit_a_gauche` (bloc F) |
| C-101 | Le champ de raccourci saisit une combinaison et l'écrit lisiblement. | auto : `dessin::controles::raccourci::saisie_d_un_raccourci`, `dessin::touches::ce_que_saisit_le_champ_de_raccourci_se_lit_bien` (bloc F) |
| C-102 | Le menu de choix : flèches qui font le tour, lignes grisées sautées, Début/Fin, Entrée, Échap, clic à côté qui ferme ; ouvert sous la case ou au-dessus selon la place, jamais hors de la fenêtre ; largeur bornée ; nom pour le lecteur d'écran. | auto : `dessin::menu_choix::*` (8 tests, le dessin) et, dans le cœur (bloc F), `ic_coeur::gestes::choix::*` (9 tests : les touches, la ligne initiale, le nom pour le lecteur d'écran) ; la place du menu, dans le cœur (bloc D) : `ic_coeur::placement::plan::le_menu_*` et `…::trop_haut_des_deux_cotes_le_menu_prend_le_cote_le_plus_grand` (4 tests), `…::rectangle_comme_egui` |
| C-103 | La barre des seuils : poignées glissées ou au clavier, limites entre leurs voisines, rien ne bouge sans focus. | auto : `dessin::seuils::*` (6 tests, souris, clavier et dessin) et, dans le cœur (bloc F), `ic_coeur::gestes::seuils::*` (4 tests : bornes, limites qui se touchent, pas du clavier) |
| C-104 | La mise au point (focus) visible au clavier sur chaque contrôle. | auto : `tests/comportements::c104_le_focus_se_voit` (bouton, bouton accent, bouton icône, interrupteur, ligne de menu ; clair et sombre) |

## 13. Captures, sonde et journal

| N° | Comportement | Essai |
|---|---|---|
| C-105 | Capture sans fenêtre : le rendu logiciel d'egui en PNG, découpe respectée, sans superposition. | auto : `capture::*` (3 tests) |
| C-106 | Les captures de la documentation se refont par un test. | auto : `tests/sans_fenetre::captures_pour_la_documentation` (images refaites, jamais comparées à une référence) |
| C-107 | La sonde écrit une ligne quand un fil est bloqué, puis une ligne de reprise ; un fil qui avance n'écrit rien ; la veille du PC n'est pas un blocage. | auto : la décision dans le cœur, à fausse horloge et fils simulés : `ic_coeur::gestes::sonde::*` (15 tests) ; les vrais fils, le journal et la vraie boucle à délais courts : `bords::sonde::*` (8 tests) |
| C-108 | Une création de fenêtre longue donne une ligne au journal. | auto : `ic_coeur::gestes::sonde::creation_longue_donne_une_ligne`, `ic_coeur::gestes::sonde::la_veille_n_allonge_pas_une_creation`, `bords::sonde::le_chien_ecrit_la_creation_qui_dure` |
| C-109 | Le lieu d'une panique est noté sans le chemin du PC. | auto : `ic_coeur::gestes::sonde::lieu_de_panique_sans_le_chemin_du_pc`, `bords::sonde::lieu_de_la_panique_garde_pour_le_fil` |
| C-110 | Le journal du logiciel est branché sur celui de la bibliothèque (`brancher_le_journal`). | auto : `tests/comportements::c110_le_journal_est_branche`, `bords::sonde::le_journal_branche_recoit_les_lignes` |

## 14. Le système

| N° | Comportement | Essai |
|---|---|---|
| C-111 | Un disque réseau ou redirigé n'est jamais pris pour un disque local ; la réponse est rapide. | auto : `systeme::chemins_reseau_reconnus`, `systeme::redirection_jamais_locale`, `systeme::disque_local_repond_vite` |
| C-112 | L'écran autour d'un point, la position de la souris, les effets d'animation de Windows. | auto : `ic_coeur::placement::essais::ecran_d_un_rectangle`, `ic_coeur::placement::essais::ecran_a_gauche_en_4k`, `tests/comportements::c112_ce_que_la_bibliotheque_lit_dans_windows` |
| C-113 | Le réglage d'échelle (DPI) du processus est posé au démarrage. | auto : `tests/comportements::c113_reglage_d_echelle_par_ecran` |

## 15. Promesses de l'API

| N° | Comportement | Essai |
|---|---|---|
| C-114 | Un logiciel compile en changeant seulement le `rev` de la bibliothèque dans son `Cargo.toml` (tant qu'aucun changement n'est annoncé dans `MIGRATION.md`). | auto : `tests/compatibilite` (un module par logiciel : ses imports, ses méthodes de traits, ses signatures) |
| C-115 | Le rendu ne change pas sans être annoncé : volet, petit menu, Paramètres gardent leurs mesures (`mesures`). | auto : `tests/references` (34 images comparées, à 6 près par couleur et 1 pixel sur 5 000 ; sautées si les polices du PC diffèrent de celles des références) |
| C-116 | Les licences des dépendances sont à jour dans l'avis MIT. | auto : `tests/licences::avis_mit_a_jour` |

## Bilan

- **117 comportements** relevés.
- **111 vérifiés par un test automatique** qui tourne sur le PC (C-64 en partie à la main ;
  C-17 et C-79 en partie, le reste sur GitHub).
- **4 vérifiés seulement sur GitHub** (vraies fenêtres ou faux clics).
- **2 vérifiés seulement à la main** (C-06, C-84).
- **0 sans essai.** Les 15 qui n'en avaient pas (C-17, C-18, C-25, C-78, C-79, C-80, C-90,
  C-92, C-98, C-104, C-110, C-112, C-113, C-114, C-115) ont reçu leurs essais le 25 nov. 2026
  (filets de la refonte, `tests/comportements.rs`, `tests/polices_windows10.rs`,
  `tests/compatibilite.rs`, `tests/references.rs`).
- **Plus aucun écart connu.** C-98 : les textes (`conserver_retour` en français et en anglais,
  « 24 h » du journal détaillé) sont corrigés par le bloc B de la refonte, le pourcentage des
  appareils du volet par le bloc F (`dessin::briques::le_pourcentage_a_l_espace_insecable`).

Les deux promesses faites à PowerPeek et KVMSoft sont gardées : **C-114** (l'API ne bouge pas,
`tests/compatibilite`) et **C-115** (le rendu ne bouge pas, `tests/references`). Une image de
référence ne se refait que sur décision écrite : `REFERENCES=refaire cargo test --test
references`, puis regarder chaque image changée avant de la committer.
