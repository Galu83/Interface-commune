//! Le sélecteur de dossier de Windows, et la parade qui le garde de tout chemin lent : il ne
//! part jamais d'un partage, d'un lecteur réseau, d'un lecteur amovible ni d'une jonction
//! (C-111). Chaque `unsafe` est commenté.

use std::path::{Path, PathBuf};

use windows::Win32::Storage::FileSystem::GetDriveTypeW;
use windows::core::HSTRING;

use super::fenetre::hwnd;

/// Le sélecteur de dossier de Windows (« Choisir… ») : le dossier choisi, ou `None` si
/// l'utilisateur annule. `proprietaire` : la fenêtre (HWND) au-dessus de laquelle il s'ouvre,
/// qu'il bloque tant qu'il est ouvert (pour les Paramètres :
/// [`crate::parametres::fenetre::choisir_dossier`]) ; `depart` : le dossier montré d'abord,
/// s'il est sur un disque de ce PC ([`dossier_de_depart`]). Seulement des dossiers du disque
/// (pas de bibliothèque ni d'emplacement virtuel). À appeler sur le fil des fenêtres : la
/// boîte tient ses propres messages pendant qu'elle est ouverte.
pub fn choisir_dossier(proprietaire: Option<isize>, depart: Option<&Path>, titre: &str) -> Option<PathBuf> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
        CoInitializeEx, CoTaskMemFree, CoUninitialize,
    };
    use windows::Win32::UI::Shell::{
        FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileOpenDialog, IShellItem,
        SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
    };
    // SAFETY: COM est initialisé pour ce fil (et libéré à la fin s'il l'a été ici) ; la
    // boîte de dialogue et les éléments sont des objets COM comptés par le crate `windows`,
    // libérés à la fin de leur portée ; les chaînes passées vivent pendant chaque appel ; le
    // texte rendu par `GetDisplayName` est copié, puis libéré une seule fois par
    // `CoTaskMemFree`. La fenêtre propriétaire est celle du logiciel, vivante pendant l'appel.
    unsafe {
        let com = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).is_ok();
        let choisi = (|| {
            let dialogue: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
            let options = dialogue.GetOptions().ok()?;
            dialogue
                .SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)
                .ok()?;
            let _ = dialogue.SetTitle(&HSTRING::from(titre));
            // Toujours un dossier de départ, sur un disque de ce PC : sans lui, Windows
            // rouvrirait de lui-même le dernier dossier choisi, peut-être un partage réseau
            // muet, qui ferait attendre ce fil de 20 à 60 s.
            if let Some(d) = dossier_de_depart(depart) {
                let element: Option<IShellItem> =
                    SHCreateItemFromParsingName(&HSTRING::from(d.as_os_str()), None).ok();
                if let Some(e) = element {
                    let _ = dialogue.SetFolder(&e);
                }
            }
            // Une erreur si l'utilisateur annule.
            let _imbriquee = crate::sonde::imbriquee("sélecteur de dossier");
            dialogue.Show(proprietaire.map(hwnd)).ok()?;
            let texte = dialogue
                .GetResult()
                .ok()?
                .GetDisplayName(SIGDN_FILESYSPATH)
                .ok()?;
            let chemin = texte.to_string().ok();
            CoTaskMemFree(Some(texte.0 as *const _));
            chemin.map(PathBuf::from)
        })();
        if com {
            CoUninitialize();
        }
        choisi
    }
}

/// Le dossier montré d'abord par le sélecteur : `voulu` s'il est sur un disque de ce PC,
/// sinon le profil de l'utilisateur, sinon la racine du disque du système. Toujours un
/// dossier local et sûr (jamais un partage réseau, qui peut faire attendre le fil).
pub fn dossier_de_depart(voulu: Option<&Path>) -> Option<PathBuf> {
    let profil = std::env::var_os("USERPROFILE").map(PathBuf::from);
    let systeme = std::env::var_os("SystemDrive").map(|d| {
        let mut r = d.to_string_lossy().into_owned();
        r.push('\\');
        PathBuf::from(r)
    });
    voulu
        .map(Path::to_path_buf)
        .into_iter()
        .chain(profil)
        .chain(systeme)
        .find(|d| disque_local(d) && d.is_dir())
}

/// Un chemin sur un disque fixe de ce PC (`C:\…`) : le seul genre de dossier regardé depuis
/// le fil des fenêtres. Jamais un chemin réseau ou de périphérique (`\\…`, `//…`), ni un
/// lecteur réseau (`Z:\…` relié à un partage, qui peut ne pas répondre), ni un lecteur
/// amovible ou de CD (qui peut se faire attendre), ni un chemin qui passe par un point de
/// redirection (lien symbolique ou jonction, qui peut mener à un partage réseau).
pub fn disque_local(d: &Path) -> bool {
    /// `DRIVE_FIXED` de Windows (un disque du PC).
    const DRIVE_FIXED: u32 = 3;
    let Some(racine) = racine_de_lecteur(d) else {
        return false;
    };
    // SAFETY: chaîne terminée par zéro, vivante pendant l'appel. Pour une racine de lecteur
    // (`Z:\`), Windows répond d'après sa table des lecteurs, sans attendre le réseau
    // (`essais::disque_local_repond_vite`).
    let fixe = unsafe { GetDriveTypeW(&HSTRING::from(racine)) == DRIVE_FIXED };
    fixe && sans_redirection(d)
}

/// Aucun dossier du chemin (de la racine vers le bout) n'est un point de redirection : lien
/// symbolique ou jonction. Chacun est regardé sans être suivi (`symlink_metadata`), et on
/// s'arrête au premier : on ne passe jamais au travers. Un dossier qui n'existe pas (encore)
/// arrête la recherche sans refus ; une autre erreur compte comme une redirection.
fn sans_redirection(d: &Path) -> bool {
    let mut parties: Vec<&Path> = d.ancestors().collect();
    parties.reverse();
    // La racine (`C:\`) elle-même n'est jamais une redirection.
    for p in parties.into_iter().skip(1) {
        match std::fs::symlink_metadata(p) {
            // Rust range parmi les liens les jonctions et les liens symboliques, pas les
            // fichiers de OneDrive à la demande.
            Ok(m) if m.file_type().is_symlink() => return false,
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return true,
            Err(_) => return false,
        }
    }
    true
}

/// `C:\` pour `C:\…` ou `C:/…` ; `None` pour tout autre chemin (réseau, relatif…).
pub(super) fn racine_de_lecteur(d: &Path) -> Option<String> {
    let b = d.to_str()?.as_bytes();
    (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'))
        .then(|| format!("{}:\\", char::from(b[0])))
}
