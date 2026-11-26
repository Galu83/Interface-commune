//! La page commune « Mise à jour » : la carte de l'état (une seule icône, le titre et ses
//! couleurs, la ligne grise avec la version et la dernière recherche, le bouton qui va
//! avec), les nouveautés, puis les recherches automatiques et le dossier des installateurs.
//! Ce qu'on montre dans chaque état, le cœur le décide (`ic_coeur::parametres`, bloc E).

use egui::RichText;

use super::Parametres;
use crate::apparence::couleur;
use crate::dessin::{self, Genre, bouton, carte, carte_large, interrupteur_ecrit, titre_section};
use crate::parametres::modele::Valeur;
use crate::polices::semibold;
use crate::textes::Textes;
use crate::theme::{Palette, TAILLE_PETIT};
use crate::{Communs, Reglages, cles, icones};
use ic_coeur::parametres::{
    self as coeur, BoutonMiseAJour, CarteDossier, ligne_grise_mise_a_jour, texte_mise_a_jour,
};

impl Parametres {
    /// Toute la page : ce que la bibliothèque dessine, les changements rendus au logiciel, puis
    /// ce que le logiciel ajoute ([`Reglages::dessiner_mise_a_jour`]).
    pub(super) fn page_mise_a_jour(&mut self, ui: &mut egui::Ui, p: &Palette, l: &dyn Reglages, c: &Communs) {
        let t = self.textes.clone();
        carte_de_l_etat(ui, p, &t, l, c);
        nouveautes(ui, p, &t, c);
        titre_section(ui, t.reglages);
        if let Some((cle, titre, avant, apres)) = recherches(ui, p, &t, c) {
            self.changer(l, cle, titre, avant, apres);
        }
        dossier(ui, p, &t, l, c);
        self.avec_contexte(ui, p, l, |l, ui, c| l.dessiner_mise_a_jour(ui, c));
    }
}

/// La carte de l'état : l'icône, le titre et la ligne grise à gauche, le bouton à droite.
fn carte_de_l_etat(ui: &mut egui::Ui, p: &Palette, t: &Textes, l: &dyn Reglages, c: &Communs) {
    dessin::cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        // Le bouton d'abord (à droite) : le titre et la ligne grise prennent la place qui reste
        // et passent à la ligne, sans jamais passer sous le bouton.
        // Une hauteur fixe : une disposition de droite à gauche prendrait toute celle de la page.
        let taille = egui::vec2(ui.available_width(), 56.0);
        ui.allocate_ui_with_layout(taille, egui::Layout::right_to_left(egui::Align::Center), |ui| {
            bouton_de_l_etat(ui, p, t, l, c);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                titre_de_l_etat(ui, p, t, l, c);
            });
        });
    });
}

/// « Installer maintenant » ou « Rechercher une mise à jour », selon l'état (le cœur choisit).
fn bouton_de_l_etat(ui: &mut egui::Ui, p: &Palette, t: &Textes, l: &dyn Reglages, c: &Communs) {
    let Some(b) = coeur::bouton_mise_a_jour(c) else {
        return;
    };
    let (icone, texte) = match b {
        BoutonMiseAJour::Installer => (icones::INSTALLER, t.installer_maintenant),
        BoutonMiseAJour::Rechercher => (icones::MISE_A_JOUR, t.rechercher_mise_a_jour),
    };
    if bouton(ui, p, Some(icone), texte, Genre::Accent).clicked() {
        match b {
            BoutonMiseAJour::Installer => l.action(cles::INSTALLER),
            BoutonMiseAJour::Rechercher => l.rechercher_mise_a_jour(),
        }
    }
}

/// L'icône de l'état (une seule), son titre en couleur et la ligne grise dessous.
fn titre_de_l_etat(ui: &mut egui::Ui, p: &Palette, t: &Textes, l: &dyn Reglages, c: &Communs) {
    let (g, couleur_etat, titre) = texte_mise_a_jour(&c.mise_a_jour, p, t);
    let gris = ligne_grise_mise_a_jour(
        &c.mise_a_jour,
        &l.identite().version,
        c.derniere_recherche.as_deref(),
        t,
    );
    let (r, _) = ui.allocate_exact_size(egui::vec2(40.0, 44.0), egui::Sense::hover());
    dessin::icone(ui, r.center(), g, 30.0, couleur_etat);
    ui.vertical(|ui| {
        ui.add(
            egui::Label::new(
                RichText::new(dessin::insecables(&titre))
                    .font(semibold(18.0))
                    .color(couleur(p.texte)),
            )
            .wrap(),
        );
        ui.add(
            egui::Label::new(
                RichText::new(gris)
                    .size(TAILLE_PETIT)
                    .color(couleur(p.texte_secondaire)),
            )
            .wrap(),
        );
    });
}

/// Les nouveautés de la version prête, s'il y en a.
fn nouveautes(ui: &mut egui::Ui, p: &Palette, t: &Textes, c: &Communs) {
    if c.nouveautes.is_empty() {
        return;
    }
    titre_section(ui, t.nouveautes);
    dessin::cadre_carte(p).show(ui, |ui| {
        ui.set_width(ui.available_width());
        for n in &c.nouveautes {
            ui.add(egui::Label::new(format!("• {n}")).wrap());
        }
    });
}

/// « Chercher les mises à jour » : rend le changement (clé, titre, avant, après) s'il y en a un.
fn recherches(
    ui: &mut egui::Ui,
    p: &Palette,
    t: &Textes,
    c: &Communs,
) -> Option<(&'static str, &'static str, Valeur, Valeur)> {
    let actif = c.recherches?;
    let mut v = actif;
    let explication = coeur::explication_recherches(t, actif);
    carte(
        ui,
        p,
        icones::MISE_A_JOUR,
        t.chercher_mises_a_jour,
        explication,
        |ui| {
            interrupteur_ecrit(ui, p, t, &mut v);
        },
    );
    (v != actif).then_some((
        cles::RECHERCHES,
        t.chercher_mises_a_jour,
        Valeur::Booleen(actif),
        Valeur::Booleen(v),
    ))
}

/// Le dossier des installateurs : « Choisir » et « Automatique », quand le cœur les montre.
fn dossier(ui: &mut egui::Ui, p: &Palette, t: &Textes, l: &dyn Reglages, c: &Communs) {
    let Some(CarteDossier { choisir, automatique }) = coeur::carte_dossier(c) else {
        return;
    };
    carte_large(
        ui,
        p,
        icones::DOSSIER,
        t.dossier_installateurs,
        &c.dossier_installateurs,
        if automatique { 240.0 } else { 120.0 },
        |ui| {
            if choisir && bouton(ui, p, None, t.choisir, Genre::Normal).clicked() {
                l.action(cles::DOSSIER);
            }
            if automatique && bouton(ui, p, None, t.automatique, Genre::Normal).clicked() {
                l.action(cles::DOSSIER_AUTO);
            }
        },
    );
}
