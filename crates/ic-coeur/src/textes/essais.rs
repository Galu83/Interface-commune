//! Les essais du bloc B (`docs/ORGANIGRAMME.md` §3.1 et §13) : les deux catalogues complets,
//! `remplir`, les raccourcis en toutes lettres, l'espace insécable (C-98). Données inventées : le
//! logiciel « Fictif ».

use super::*;

#[test]
fn remplir_dans_l_ordre() {
    assert_eq!(remplir("Version {}", &["1.2.3"]), "Version 1.2.3");
    assert_eq!(
        remplir(ANGLAIS.sous_licence, &["Démo", "MIT"]),
        "Démo is under the MIT licence."
    );
    assert_eq!(remplir("{} et {}", &["a"]), "a et ");
    assert_eq!(remplir("rien", &["a"]), "rien");
}

/// Chaque modèle a le même nombre de `{}` dans les deux langues, et aucun texte n'est
/// vide.
#[test]
fn deux_langues_completes() {
    let champs = |t: &Textes| -> Vec<&'static str> {
        vec![
            t.parametres,
            t.parametres_de,
            t.rechercher_mise_a_jour,
            t.quitter,
            t.ouvrir_le_journal,
            t.ouvrir_le_logiciel,
            t.version,
            t.accueil,
            t.general,
            t.mise_a_jour,
            t.a_propos,
            t.rechercher_un_parametre,
            t.active,
            t.desactive,
            t.modifie,
            t.annuler,
            t.fermer,
            t.confirmer,
            t.redemarrer_maintenant,
            t.retour,
            t.conserver_question,
            t.conserver_retour,
            t.conserver,
            t.revenir,
            t.demarrer_avec_windows,
            t.demarrer_explication,
            t.langue,
            t.langue_explication,
            t.notifications,
            t.notifications_explication,
            t.essayer,
            t.journal,
            t.journal_explication,
            t.journal_detaille,
            t.journal_detaille_explication,
            t.effacer_le_journal,
            t.effacer_journal_question,
            t.effacer_journal_explication,
            t.oui_effacer,
            t.remise_a_zero,
            t.retablir,
            t.retablir_explication,
            t.retablir_bouton,
            t.retablir_question,
            t.retablir_question_explication,
            t.non,
            t.oui_retablir,
            t.maj_pas_cherche,
            t.maj_a_jour,
            t.maj_recherche,
            t.maj_prete,
            t.maj_erreur,
            t.derniere_recherche,
            t.version_actuelle,
            t.installer_maintenant,
            t.nouveautes,
            t.reglages,
            t.dossier_installateurs,
            t.choisir,
            t.changer_raccourci,
            t.appuyer_touches,
            t.licence,
            t.sous_licence,
            t.credits,
            t.voir_licences,
            t.chercher_mises_a_jour,
            t.recherches_coupees,
            t.recherches_automatique,
            t.recherches_dossier,
            t.automatique,
            t.maj_coupee,
            t.prete_titre,
            t.prete_explication,
            t.plus_tard,
        ]
    };
    let (fr, en) = (champs(&FRANCAIS), champs(&ANGLAIS));
    assert_eq!(fr.len(), en.len());
    for (f, e) in fr.iter().zip(&en) {
        assert!(!f.is_empty() && !e.is_empty());
        assert_eq!(f.matches("{}").count(), e.matches("{}").count(), "{f} / {e}");
    }
    assert_eq!(Textes::de(Langue::Anglais).quitter, "Quit");
}

#[test]
fn les_modificateurs_en_toutes_lettres_dans_l_ordre_de_windows() {
    assert_eq!(lisible("ctrl+alt+shift+k"), "Ctrl+Alt+Maj+K");
    assert_eq!(lisible("Shift+Ctrl+K"), "Ctrl+Maj+K");
    assert_eq!(lisible("Ctrl+Maj+K"), "Ctrl+Maj+K");
    assert_eq!(lisible("super+d"), "Win+D");
    assert_eq!(lisible("Windows+Shift+S"), "Win+Maj+S");
    assert_eq!(lisible("Win+Ctrl+Shift+B"), "Win+Ctrl+Maj+B");
    assert_eq!(lisible("option+x"), "Alt+X");
    assert_eq!(lisible("control+x"), "Ctrl+X");
    assert_eq!(lisible("Alt"), "Alt");
    assert_eq!(lisible("Win"), "Win");
    assert_eq!(lisible("Ctrl+Ctrl+K"), "Ctrl+K", "pas de doublon");
}

#[test]
fn les_symboles_deviennent_des_mots() {
    assert_eq!(lisible("⌃⇧K"), "Ctrl+Maj+K");
    assert_eq!(lisible("⌥⇧⌃K"), "Ctrl+Alt+Maj+K");
    assert_eq!(lisible("⇧+⌃+K"), "Ctrl+Maj+K");
    assert_eq!(lisible("⌘K"), "Ctrl+K", "commande vaut Ctrl sous Windows");
    assert_eq!(lisible("Ctrl+Alt+→"), "Ctrl+Alt+Flèche droite");
    assert_eq!(lisible("Ctrl+←"), "Ctrl+Flèche gauche");
    assert_eq!(lisible("Maj+↑"), "Maj+Flèche haut");
    assert_eq!(lisible("Alt+↓"), "Alt+Flèche bas");
    assert_eq!(lisible("Ctrl+⌫"), "Ctrl+Retour arrière");
    assert_eq!(lisible("⇧↵"), "Maj+Entrée");
    for symbole in ['⇧', '⌃', '⌥', '⌘', '→', '←', '↑', '↓', '⏎', '⌫', '⇥', '⎋'] {
        let r = lisible(&format!("⌃{symbole}"));
        assert!(!r.contains(symbole), "{r}");
    }
}

#[test]
fn les_touches_nommees_en_francais() {
    assert_eq!(lisible("Escape"), "Échap");
    assert_eq!(lisible("esc"), "Échap");
    assert_eq!(lisible("Win+Escape"), "Win+Échap");
    assert_eq!(lisible("Enter"), "Entrée");
    assert_eq!(lisible("Ctrl+Space"), "Ctrl+Espace");
    assert_eq!(lisible("Backspace"), "Retour arrière");
    assert_eq!(lisible("Delete"), "Suppr");
    assert_eq!(lisible("Ctrl+Alt+Delete"), "Ctrl+Alt+Suppr");
    assert_eq!(lisible("Home"), "Début");
    assert_eq!(lisible("End"), "Fin");
    assert_eq!(lisible("PageUp"), "Page précédente");
    assert_eq!(lisible("Ctrl+PageDown"), "Ctrl+Page suivante");
    assert_eq!(lisible("Alt+Right"), "Alt+Flèche droite");
    assert_eq!(lisible("ArrowLeft"), "Flèche gauche");
    assert_eq!(lisible("ctrl+down"), "Ctrl+Flèche bas");
    assert_eq!(lisible("Up"), "Flèche haut");
    assert_eq!(lisible("Ctrl+Plus"), "Ctrl+Plus");
    assert_eq!(lisible("Ctrl+Minus"), "Ctrl+Moins");
    assert_eq!(lisible("Tab"), "Tab");
    assert_eq!(lisible("shift+tab"), "Maj+Tab");
}

#[test]
fn les_lettres_les_chiffres_et_les_touches_de_fonction() {
    assert_eq!(lisible("ctrl+k"), "Ctrl+K");
    assert_eq!(lisible("Ctrl+Alt+1"), "Ctrl+Alt+1");
    assert_eq!(lisible("f5"), "F5");
    assert_eq!(lisible("Maj+F10"), "Maj+F10");
    assert_eq!(lisible("alt+f4"), "Alt+F4");
    assert_eq!(lisible("ctrl+é"), "Ctrl+É");
    assert_eq!(lisible("Ctrl+,"), "Ctrl+,");
}

#[test]
fn le_signe_plus_est_une_touche() {
    assert_eq!(lisible("Ctrl++"), "Ctrl++");
    assert_eq!(lisible("ctrl+shift++"), "Ctrl+Maj++");
    assert_eq!(lisible("+"), "+");
}

#[test]
fn vide_espaces_et_inconnu() {
    assert_eq!(lisible(""), "");
    assert_eq!(lisible("   "), "");
    assert_eq!(lisible(" Ctrl + Maj + K "), "Ctrl+Maj+K");
    assert_eq!(
        lisible("Ctrl+volumeup"),
        "Ctrl+Volumeup",
        "un mot inconnu garde ses lettres"
    );
}

#[test]
fn la_forme_lisible_est_stable() {
    for r in [
        "ctrl+alt+shift+k",
        "Ctrl+Alt+→",
        "⌃⇧K",
        "Win+Escape",
        "Alt+PageUp",
        "Ctrl+Alt+Delete",
        "shift+tab",
        "F5",
        "Ctrl++",
        "Ctrl+Retour arrière",
        "Ctrl+Flèche droite",
        "Impr. écran",
        "Alt Gr+E",
    ] {
        let une = lisible(r);
        assert_eq!(lisible(&une), une, "{r}");
    }
}

/// Les champs d'un catalogue, lus dans sa forme de débogage : `(nom, valeur)`. Ainsi, un champ
/// ajouté à `Textes` entre de lui-même dans les essais ci-dessous, sans liste à tenir.
fn champs(t: &Textes) -> Vec<(String, String)> {
    format!("{t:#?}")
        .lines()
        .filter_map(|l| {
            let (nom, valeur) = l.trim().split_once(": ")?;
            let valeur = valeur.strip_suffix(',')?;
            Some((nom.to_string(), valeur.trim_matches('"').to_string()))
        })
        .collect()
}

#[test]
fn les_deux_catalogues_ont_les_memes_champs_et_aucun_vide() {
    let (fr, en) = (champs(&FRANCAIS), champs(&ANGLAIS));
    assert_eq!(
        fr.len(),
        74,
        "un champ ajouté ou retiré : mettre à jour cet essai"
    );
    let noms = |c: &[(String, String)]| c.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>();
    assert_eq!(noms(&fr), noms(&en));
    for ((nom, f), (_, e)) in fr.iter().zip(&en) {
        assert!(!f.is_empty() && !e.is_empty(), "{nom} est vide");
        assert_eq!(f.matches("{}").count(), e.matches("{}").count(), "{nom}");
    }
    assert_eq!(Textes::default(), FRANCAIS);
    assert_eq!(Textes::de(Langue::Francais), &FRANCAIS);
    assert_eq!(Langue::default(), Langue::Francais);
}

#[test]
fn les_modeles_se_remplissent_avec_le_logiciel_fictif() {
    assert_eq!(
        remplir(FRANCAIS.parametres_de, &["Fictif"]),
        "Paramètres de Fictif"
    );
    assert_eq!(remplir(ANGLAIS.parametres_de, &["Fictif"]), "Fictif Settings");
    assert_eq!(
        remplir(FRANCAIS.prete_titre, &["Fictif", "2.0.0"]),
        "Fictif 2.0.0 est prête"
    );
    assert_eq!(
        remplir(FRANCAIS.sous_licence, &["Fictif", "MIT"]),
        "Fictif est sous licence MIT."
    );
    assert_eq!(remplir(FRANCAIS.modifie, &["Thème"]), "« Thème » modifié.");
}

/// Les unités qu'un texte de la bibliothèque pourrait écrire après un nombre.
const UNITES: &[&str] = &["s", "%", "ms", "px", "Mo", "Ko", "Go", "min", "h", "j"];

/// Vrai si le texte a un nombre (« {} » ou un chiffre) suivi d'une espace simple puis d'une
/// unité : l'unité pourrait partir seule à la ligne suivante (C-98).
fn espace_simple_avant_une_unite(texte: &str) -> bool {
    texte.match_indices(' ').any(|(i, _)| {
        let avant = &texte[..i];
        let nombre = avant.ends_with("{}") || avant.ends_with(|c: char| c.is_ascii_digit());
        let mot: String = texte[i + 1..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '%')
            .collect();
        nombre && UNITES.contains(&mot.as_str())
    })
}

#[test]
fn le_detecteur_c98_voit_les_fautes_et_seulement_elles() {
    assert!(espace_simple_avant_une_unite("dans {} s"));
    assert!(espace_simple_avant_une_unite("après 24 h."));
    assert!(espace_simple_avant_une_unite("{} %"));
    assert!(!espace_simple_avant_une_unite("dans {}\u{a0}s"));
    assert!(!espace_simple_avant_une_unite("après 24\u{a0}h."));
    assert!(!espace_simple_avant_une_unite("Version {} disponible"));
    assert!(!espace_simple_avant_une_unite("{} {} est prête"));
    assert!(!espace_simple_avant_une_unite("{} est sous licence {}."));
}

/// C-98, sans écart permis : aucun texte des deux catalogues n'a d'espace simple entre un nombre
/// et son unité. Les deux écarts d'avant la refonte (`conserver_retour`, en français et en
/// anglais) sont corrigés, ainsi que « 24 h » du journal détaillé.
#[test]
fn c98_aucune_espace_simple_avant_une_unite() {
    for t in [&FRANCAIS, &ANGLAIS] {
        let fautes: Vec<_> = champs(t)
            .into_iter()
            .filter(|(_, v)| espace_simple_avant_une_unite(v))
            .collect();
        assert!(fautes.is_empty(), "espace simple avant une unité : {fautes:#?}");
    }
    assert_eq!(
        remplir(FRANCAIS.conserver_retour, &["15"]),
        "Retour au réglage d'avant dans 15\u{a0}s"
    );
    assert_eq!(
        remplir(ANGLAIS.conserver_retour, &["15"]),
        "Going back to the previous setting in 15\u{a0}s"
    );
    assert!(FRANCAIS.journal_detaille_explication.ends_with("24\u{a0}h."));
    assert!(ANGLAIS.journal_detaille_explication.ends_with("24\u{a0}h."));
}

#[test]
fn avec_unite_met_l_espace_insecable() {
    assert_eq!(INSECABLE, '\u{a0}');
    assert_eq!(avec_unite(42, "%"), "42\u{a0}%");
    assert_eq!(avec_unite(15u8, "s"), "15\u{a0}s");
    assert_eq!(avec_unite("2,5", "Go"), "2,5\u{a0}Go");
    assert_eq!(avec_unite(30, ""), "30", "sans unité, le nombre seul");
    assert!(!espace_simple_avant_une_unite(&avec_unite(7, "min")));
}

#[test]
fn insecables_suit_la_typographie_francaise() {
    assert_eq!(insecables("« Fictif »"), "«\u{a0}Fictif\u{a0}»");
    assert_eq!(
        insecables("Coupé : le logiciel ne cherche pas."),
        "Coupé\u{a0}: le logiciel ne cherche pas."
    );
    assert_eq!(insecables("Vraiment ? Oui !"), "Vraiment\u{a0}? Oui\u{a0}!");
    assert_eq!(insecables("un ; deux"), "un\u{a0}; deux");
    assert_eq!(insecables("rien à changer"), "rien à changer");
    assert_eq!(insecables(""), "");
    let une = insecables("« Fictif » : prêt ?");
    assert_eq!(insecables(&une), une, "la rappeler ne change rien");
}

/// Le cas trouvé au hasard (« ⌃ ⌃ », séparateur vide) : l'espace entre deux symboles ne fait
/// pas du second une touche ; `lisible` rend « Ctrl » du premier coup, plus « Ctrl+⌃ ».
#[test]
fn lisible_symboles_separes_d_une_espace() {
    assert_eq!(lisible("⌃ ⌃"), "Ctrl");
    assert_eq!(lisible("Ctrl"), "Ctrl");
    assert_eq!(lisible("⌃ ⇧ K"), "Ctrl+Maj+K");
    assert_eq!(lisible("⌃  ⇧K"), "Ctrl+Maj+K");
    assert_eq!(lisible("⌥ + ⌃ ⌘"), "Ctrl+Alt");
}

proptest::proptest! {
    /// `lisible` ne change pas deux fois : sa sortie, relue, reste la même (ORGANIGRAMME §3.1, B).
    #[test]
    fn lisible_ne_change_pas_deux_fois(
        morceaux in proptest::collection::vec(
            proptest::sample::select(vec![
                "ctrl", "Ctrl", "shift", "Maj", "alt", "option", "win", "super", "⌃", "⇧", "⌥",
                "⌘", "k", "K", "é", "1", ",", "f5", "F12", "escape", "Échap", "pageup", "Down",
                "→", "←", "↵", "⌫", "Delete", "space", "tab", "Impr. écran", "Alt Gr", " ",
                "volumeup", "+",
            ]),
            0..6,
        ),
        separateur in proptest::sample::select(vec!["+", " + ", ""]),
    ) {
        let raccourci = morceaux.join(separateur);
        let une = lisible(&raccourci);
        proptest::prop_assert_eq!(lisible(&une), une, "{}", raccourci);
    }
}
