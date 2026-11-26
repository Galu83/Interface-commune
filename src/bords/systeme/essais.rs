//! Les essais du système Windows : ils parlent à Windows pour de vrai, sur ce PC. Aucune
//! fenêtre n'est jamais montrée : toutes viennent de la fabrique de fenêtre cachée, et sont
//! détruites à la fin de chaque essai.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, DestroyWindow, IsWindowVisible, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos,
    WM_DPICHANGED,
};
use windows::core::HSTRING;

use super::cachee::{ClasseDejaLa, fenetre_cachee};
use super::dossier::{disque_local, dossier_de_depart, racine_de_lecteur};
use super::echelle::suivre_windows;
use super::fenetre::{CLASSE_WINIT, fenetre_du_fil, hwnd, rect_fenetre};

extern "system" fn par_defaut(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    // SAFETY: les paramètres viennent de Windows et sont rendus tels quels à la procédure par
    // défaut.
    unsafe { DefWindowProcW(h, m, w, l) }
}

/// Une fenêtre jamais montrée, de cette classe, avec ce titre, sur le fil qui appelle.
fn cachee(classe: &str, titre: &str) -> isize {
    fenetre_cachee(
        &HSTRING::from(classe),
        &HSTRING::from(titre),
        par_defaut,
        ClasseDejaLa::Reprendre,
    )
    .expect("fenêtre cachée")
    .0 as isize
}

fn detruire(fenetres: &[isize]) {
    for &h in fenetres {
        // SAFETY: fenêtres créées par l'essai, sur ce fil.
        let _ = unsafe { DestroyWindow(hwnd(h)) };
    }
}

/// `fenetre_du_fil` ne rend jamais une fenêtre d'une autre classe que celle de winit (ex. « NVOGLDC
/// invisible », créée cachée par le pilote OpenGL de NVIDIA dans chaque programme), même au
/// même titre ; deux fenêtres de winit du même titre ne se confondent pas (`prises`).
#[test]
fn fenetre_du_fil_ne_rend_que_les_fenetres_de_winit_pas_prises() {
    let titre = "Titre d'essai de la fenetre du fil";
    // L'étrangère est créée la première, puis deux fenêtres de winit au même titre.
    let etrangere = cachee("STATIC", titre);
    let premiere = cachee(CLASSE_WINIT, titre);
    let deuxieme = cachee(CLASSE_WINIT, titre);
    assert!(etrangere != 0 && premiere != 0 && deuxieme != 0);
    let a = fenetre_du_fil(titre, &[]).expect("une fenêtre de winit");
    assert_ne!(a, etrangere, "jamais la fenêtre étrangère");
    assert!(a == premiere || a == deuxieme);
    let b = fenetre_du_fil(titre, &[a]).expect("l'autre fenêtre de winit");
    assert_ne!(b, etrangere, "jamais la fenêtre étrangère");
    assert!(b != a && (b == premiere || b == deuxieme));
    assert_eq!(fenetre_du_fil(titre, &[a, b]), None, "les deux prises : rien");
    assert_eq!(fenetre_du_fil("Un autre titre", &[]), None);
    detruire(&[etrangere, premiere, deuxieme]);
}

/// La fabrique refuse une deuxième fenêtre de la même classe quand elle doit être la seule
/// (l'icône unique), et la reprend sinon (une deuxième zone d'icônes).
#[test]
fn la_fabrique_refuse_ou_reprend_une_classe_deja_la() {
    let classe = HSTRING::from(format!("essai-fabrique-{}", std::process::id()));
    let titre = HSTRING::from("Essai de la fabrique");
    let premiere = fenetre_cachee(&classe, &titre, par_defaut, ClasseDejaLa::Refuser).expect("la première");
    assert_eq!(
        fenetre_cachee(&classe, &titre, par_defaut, ClasseDejaLa::Refuser),
        Err("classe de fenêtre refusée".to_string())
    );
    let reprise = fenetre_cachee(&classe, &titre, par_defaut, ClasseDejaLa::Reprendre).expect("reprise");
    assert_ne!(premiere, reprise);
    // Une fenêtre cachée n'est jamais montrée.
    for h in [premiere, reprise] {
        // SAFETY: fenêtre créée par l'essai, sur ce fil.
        assert!(!unsafe { IsWindowVisible(h) }.as_bool());
    }
    detruire(&[premiere.0 as isize, reprise.0 as isize]);
}

/// Où la procédure « comme winit » pose la fenêtre quand son échelle change : ailleurs que là
/// où Windows le suggère.
const POSE_PAR_WINIT: [i32; 4] = [900, 700, 400, 250];

/// Une procédure qui répond à `WM_DPICHANGED` comme winit 0.30 : en posant la fenêtre à un
/// endroit qu'elle calcule elle-même.
extern "system" fn comme_winit(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if m == WM_DPICHANGED {
        let [x, y, cx, cy] = POSE_PAR_WINIT;
        // SAFETY: fenêtre de ce fil, jamais montrée ; rien n'est activé.
        let _ = unsafe { SetWindowPos(h, None, x, y, cx, cy, SWP_NOZORDER | SWP_NOACTIVATE) };
        return LRESULT(0);
    }
    par_defaut(h, m, w, l)
}

/// Le changement d'échelle (C-113) : une fenêtre suivie prend le rectangle que Windows
/// suggère, quoi que winit demande ; une fenêtre non suivie va où winit la pose ; hors d'un
/// changement d'échelle, rien n'est corrigé.
#[test]
fn le_changement_d_echelle_suit_windows() {
    // Deux fenêtres de la même classe, à la procédure « comme winit » : l'une suivie, l'autre
    // témoin.
    let classe = HSTRING::from(format!("essai-echelle-winit-{}", std::process::id()));
    let titre = HSTRING::from("Essai d'échelle");
    let nouvelle = || {
        fenetre_cachee(&classe, &titre, comme_winit, ClasseDejaLa::Reprendre)
            .expect("fenêtre cachée")
            .0 as isize
    };
    let (suivie, temoin) = (nouvelle(), nouvelle());
    assert!(suivre_windows(suivie));
    assert!(suivre_windows(suivie), "sans effet de plus la deuxième fois");
    let suggere = RECT {
        left: 100,
        top: 120,
        right: 400,
        bottom: 320,
    };
    let changer = |h: isize| {
        // SAFETY: fenêtre de ce fil ; le RECT suggéré vit pendant l'appel (envoi synchrone).
        unsafe {
            SendMessageW(
                hwnd(h),
                WM_DPICHANGED,
                Some(WPARAM(0x0090_0090)),
                Some(LPARAM((&raw const suggere) as isize)),
            )
        };
    };
    changer(suivie);
    changer(temoin);
    assert_eq!(
        rect_fenetre(suivie),
        Some([100, 120, 400, 320]),
        "le rectangle de Windows"
    );
    let [x, y, cx, cy] = POSE_PAR_WINIT;
    assert_eq!(
        rect_fenetre(temoin),
        Some([x, y, x + cx, y + cy]),
        "la place de winit"
    );
    // Hors d'un changement d'échelle, la fenêtre suivie va où on la pose.
    // SAFETY: fenêtre de ce fil, jamais montrée.
    let _ = unsafe {
        SetWindowPos(
            hwnd(suivie),
            None,
            10,
            20,
            300,
            200,
            SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
    assert_eq!(rect_fenetre(suivie), Some([10, 20, 310, 220]));
    detruire(&[suivie, temoin]);
}

/// Le sélecteur ne regarde jamais un chemin réseau depuis le fil des fenêtres : seulement
/// un disque fixe de ce PC.
#[test]
fn chemins_reseau_reconnus() {
    for reseau in [
        r"\\serveur\partage",
        r"\\?\C:\x",
        r"\\.\C:\x",
        "//serveur/partage",
        r"Installateurs\x",
        "C:",
        "",
    ] {
        assert_eq!(racine_de_lecteur(Path::new(reseau)), None, "{reseau}");
        assert!(!disque_local(Path::new(reseau)), "{reseau}");
    }
    assert_eq!(
        racine_de_lecteur(Path::new(r"Z:\Installateurs")).as_deref(),
        Some(r"Z:\")
    );
    assert_eq!(
        racine_de_lecteur(Path::new("d:/Installateurs")).as_deref(),
        Some(r"d:\")
    );
    // Le disque du système est un disque fixe.
    let systeme = std::env::var("SystemRoot").expect("SystemRoot");
    assert!(disque_local(Path::new(&systeme)), "{systeme}");
}

/// Un chemin qui passe par un lien ou une jonction n'est pas regardé comme local : il peut
/// mener à un partage réseau. Aucune fenêtre ne s'ouvre pendant l'essai.
#[test]
fn redirection_jamais_locale() {
    use std::os::windows::process::CommandExt;
    /// `CREATE_NO_WINDOW` : aucune fenêtre ne s'ouvre.
    const SANS_FENETRE: u32 = 0x0800_0000;
    let d = std::env::temp_dir().join(format!("interface-commune-jonction-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let cible = d.join("cible");
    std::fs::create_dir_all(cible.join("dedans")).expect("dossier");
    let jonction = d.join("jonction");
    let fait = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&jonction)
        .arg(&cible)
        .creation_flags(SANS_FENETRE)
        .output()
        .expect("mklink");
    assert!(fait.status.success(), "{fait:?}");
    assert!(disque_local(&cible.join("dedans")), "un vrai dossier");
    assert!(disque_local(&d.join("pas-encore-cree")), "pas encore créé");
    assert!(!disque_local(&jonction), "la jonction elle-même");
    assert!(
        !disque_local(&jonction.join("dedans")),
        "au travers de la jonction"
    );
    // Le sélecteur part alors du profil de l'utilisateur.
    let profil = PathBuf::from(std::env::var("USERPROFILE").expect("profil"));
    assert_eq!(dossier_de_depart(Some(&jonction)), Some(profil.clone()));
    assert_eq!(
        dossier_de_depart(Some(Path::new(r"\\serveur\partage"))),
        Some(profil.clone())
    );
    assert_eq!(dossier_de_depart(None), Some(profil));
    assert_eq!(dossier_de_depart(Some(&cible)), Some(cible.clone()));
    // Retirer la jonction ne touche pas à sa cible.
    std::fs::remove_dir(&jonction).expect("jonction retirée");
    assert!(cible.join("dedans").is_dir());
    let _ = std::fs::remove_dir_all(&d);
}

/// Savoir si un lecteur est un disque fixe n'attend pas le réseau : toutes les lettres,
/// lecteurs réseau compris (même injoignables), en bien moins d'une seconde.
#[test]
fn disque_local_repond_vite() {
    let debut = Instant::now();
    for lettre in 'A'..='Z' {
        let _ = disque_local(Path::new(&format!(r"{lettre}:\Installateurs")));
    }
    let duree = debut.elapsed();
    assert!(duree < Duration::from_secs(1), "{duree:?}");
}
