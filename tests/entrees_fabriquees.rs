//! Le volet, le petit menu et les Paramètres, pilotés avec de **vraies entrées fabriquées par
//! SendInput** (clics, Tab, flèches, Entrée, Espace, Échap, Alt+F4) : chaque fenêtre doit
//! réagir. Règles : `interface-windows11.md`, point 11 (« pilotable par tous les outils qui
//! envoient des clics et des touches ») ; `securite.md`, points 11 et 18.
//!
//! **Ce test ouvre de vraies fenêtres et prend le clavier et la souris** : il ne tourne
//! jamais sur le PC du propriétaire. Sans la variable d'environnement `ESSAI_SENDINPUT=oui`
//! (posée seulement par le workflow GitHub, sous Windows), il ne fait rien. Il ne se lance
//! pas à la main.
//!
//! Un seul essai dans ce fichier : winit n'accepte qu'une boucle de fenêtres par programme.
//! Les étapes se suivent dans la même boucle :
//!
//! 1. petit menu : Échap le ferme ;
//! 2. petit menu : flèche bas, Entrée → l'action du logiciel part, le menu se ferme ;
//! 3. volet : Échap le ferme ;
//! 4. volet : Tab, Tab, Entrée → le réglage rapide part, le volet se ferme ;
//! 5. volet : clic sur la roue dentée → les Paramètres s'ouvrent ;
//! 6. Paramètres : Tab, Tab, Entrée change de page (navigation), puis Tab jusqu'à l'interrupteur
//!    « Démarrer avec Windows » et Espace → le réglage change ;
//! 7. Paramètres : Alt+F4 les ferme ;
//! 8. petit menu : clic sur « Paramètres » → ils se rouvrent.

#![cfg(windows)]
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
// Pourquoi c'est sûr : les appels Windows de ce fichier ne reçoivent que des structures
// locales bien remplies (SendInput, GetWindowRect…) et des poignées lues juste avant.
#![allow(unsafe_code)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use interface_commune::briques::Brique;
use interface_commune::fenetres::{Demande, Liens, Ouvert, canal};
use interface_commune::menu::Action;
use interface_commune::parametres::{CLE_FENETRE, Contexte};
use interface_commune::theme::Surface;
use interface_commune::volet::veiller;
use interface_commune::{
    Communs, Etat, EtatMiseAJour, GenreEtat, Identite, Logiciel, Page, Reglages, Valeur, cles, polices,
};

use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY,
    KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, MOUSE_EVENT_FLAGS, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT, MapVirtualKeyW, SendInput, VIRTUAL_KEY, VK_DOWN,
    VK_ESCAPE, VK_F4, VK_MENU, VK_RETURN, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetSystemMetrics, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, SM_CXSCREEN, SM_CYSCREEN, SetForegroundWindow,
};
use windows::core::BOOL;

fn icone(t: u32, _: bool) -> Vec<u8> {
    vec![255; (t * t * 4) as usize]
}

/// Un logiciel inventé qui note tout ce que les fenêtres lui demandent.
struct Essai {
    identite: Identite,
    /// Les actions reçues (réglage rapide du volet, ligne du petit menu), dans l'ordre.
    actions: Mutex<Vec<String>>,
    /// Les réglages changés par les Paramètres.
    changements: Mutex<Vec<String>>,
    demarrage: AtomicBool,
    /// La dernière image de la page « Exemple » des Paramètres (pour savoir si on l'a quittée).
    vue_exemple: Mutex<Option<Instant>>,
}

impl Logiciel for Essai {
    fn etat(&self) -> Etat {
        Etat {
            genre: GenreEtat::Pret,
            titre: "Essai".into(),
            ligne: "Prêt".into(),
            detail: None,
        }
    }
    fn briques(&self) -> Vec<Brique> {
        vec![Brique::Interrupteur {
            cle: "essai".into(),
            glyphe: Some('\u{E7F4}'),
            texte: "Mode essai".into(),
            actif: false,
        }]
    }
    fn actions_menu(&self) -> Vec<Action> {
        vec![Action::new("bonjour", '\u{E8BD}', "Dire bonjour")]
    }
    fn quitter(&self) {}
    fn noter(&self, _ligne: String) {}
}

impl Reglages for Essai {
    fn identite(&self) -> &Identite {
        &self.identite
    }
    fn action(&self, cle: &str) {
        self.actions.lock().unwrap().push(cle.to_string());
    }
    fn rechercher_mise_a_jour(&self) {}
    fn pages(&self) -> Vec<Page> {
        vec![Page::new("exemple", "Exemple", '\u{E8BD}')]
    }
    fn dessiner_page(&self, _cle: &str, ui: &mut egui::Ui, _page: &mut Contexte) {
        *self.vue_exemple.lock().unwrap() = Some(Instant::now());
        ui.label("Une page propre.");
    }
    fn communs(&self) -> Communs {
        Communs {
            demarrage: Some(self.demarrage.load(Ordering::Relaxed)),
            mise_a_jour: EtatMiseAJour::AJour,
            dossier_installateurs: "Installateurs".into(),
            ..Default::default()
        }
    }
    fn changer(&self, cle: &str, valeur: Valeur) {
        if let Valeur::Booleen(b) = valeur {
            if cle == cles::DEMARRAGE {
                self.demarrage.store(b, Ordering::Relaxed);
            }
            self.changements.lock().unwrap().push(format!("{cle}={b}"));
        }
    }
}

// ---- Les entrées fabriquées -------------------------------------------------------------

fn envoyer(entrees: &[INPUT]) {
    let n = unsafe { SendInput(entrees, std::mem::size_of::<INPUT>() as i32) };
    assert_eq!(
        n as usize,
        entrees.len(),
        "SendInput n'a pas accepté toutes les entrées"
    );
}

fn touche_entree(vk: VIRTUAL_KEY, relache: bool) -> INPUT {
    // Les flèches (et quelques autres) sont des touches « étendues ».
    let etendue = matches!(vk, VK_DOWN);
    let mut drapeaux = KEYBD_EVENT_FLAGS(0);
    if etendue {
        drapeaux |= KEYEVENTF_EXTENDEDKEY;
    }
    if relache {
        drapeaux |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: unsafe { MapVirtualKeyW(u32::from(vk.0), MAPVK_VK_TO_VSC) } as u16,
                dwFlags: drapeaux,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Appuie puis relâche une touche, comme un vrai clavier.
fn touche(vk: VIRTUAL_KEY) {
    envoyer(&[touche_entree(vk, false)]);
    std::thread::sleep(Duration::from_millis(40));
    envoyer(&[touche_entree(vk, true)]);
    std::thread::sleep(Duration::from_millis(150));
}

/// Alt+F4.
fn alt_f4() {
    envoyer(&[touche_entree(VK_MENU, false)]);
    std::thread::sleep(Duration::from_millis(40));
    envoyer(&[touche_entree(VK_F4, false)]);
    std::thread::sleep(Duration::from_millis(40));
    envoyer(&[touche_entree(VK_F4, true)]);
    envoyer(&[touche_entree(VK_MENU, true)]);
    std::thread::sleep(Duration::from_millis(150));
}

fn souris(dx: i32, dy: i32, drapeaux: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: drapeaux,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Déplace la souris à ce point de l'écran (pixels).
fn aller_a(x: i32, y: i32) {
    let (l, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let norme =
        |v: i32, max: i32| ((i64::from(v.clamp(0, max - 1)) * 65_535) / i64::from((max - 1).max(1))) as i32;
    envoyer(&[souris(
        norme(x, l),
        norme(y, h),
        MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
    )]);
    std::thread::sleep(Duration::from_millis(80));
}

/// Un clic gauche à ce point de l'écran (pixels).
fn cliquer(x: i32, y: i32) {
    aller_a(x, y);
    envoyer(&[souris(0, 0, MOUSEEVENTF_LEFTDOWN)]);
    std::thread::sleep(Duration::from_millis(60));
    envoyer(&[souris(0, 0, MOUSEEVENTF_LEFTUP)]);
    std::thread::sleep(Duration::from_millis(150));
}

// ---- Les fenêtres du programme ----------------------------------------------------------

struct Recherche {
    pid: u32,
    titre: String,
    trouvee: Option<HWND>,
}

unsafe extern "system" fn examiner(h: HWND, lp: LPARAM) -> BOOL {
    let r = unsafe { &mut *(lp.0 as *mut Recherche) };
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(h, Some(&mut pid)) };
    if pid != r.pid {
        return BOOL(1);
    }
    let mut tampon = [0u16; 256];
    let n = unsafe { GetWindowTextW(h, &mut tampon) };
    let titre = String::from_utf16_lossy(&tampon[..n.max(0) as usize]);
    if titre == r.titre {
        r.trouvee = Some(h);
        return BOOL(0);
    }
    BOOL(1)
}

/// La fenêtre de ce programme qui porte ce titre.
fn fenetre(titre: &str) -> Option<HWND> {
    let mut r = Recherche {
        pid: unsafe { GetCurrentProcessId() },
        titre: titre.to_string(),
        trouvee: None,
    };
    let _ = unsafe { EnumWindows(Some(examiner), LPARAM(&mut r as *mut Recherche as isize)) };
    r.trouvee
}

/// Le rectangle de la fenêtre (pixels) : gauche, haut, droite, bas.
fn rectangle(h: HWND) -> [i32; 4] {
    let mut r = RECT::default();
    unsafe { GetWindowRect(h, &mut r).expect("rectangle de la fenêtre") };
    [r.left, r.top, r.right, r.bottom]
}

/// L'échelle de l'écran de la fenêtre (1,0 à 96 ppp).
fn echelle(h: HWND) -> f32 {
    let dpi = unsafe { GetDpiForWindow(h) };
    if dpi == 0 { 1.0 } else { dpi as f32 / 96.0 }
}

/// Où est chargé ce module (diagnostic : quel OpenGL sert ?).
fn module_charge(nom: &str) -> String {
    use windows::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
    let n: Vec<u16> = nom.encode_utf16().chain(std::iter::once(0)).collect();
    let Ok(m) = (unsafe { GetModuleHandleW(windows::core::PCWSTR(n.as_ptr())) }) else {
        return "non chargé".into();
    };
    let mut tampon = [0u16; 512];
    let long = unsafe { GetModuleFileNameW(Some(m), &mut tampon) } as usize;
    String::from_utf16_lossy(&tampon[..long.min(512)])
}

/// Une photo de la fenêtre (pixels de l'écran) dans `captures/`, pour comprendre un échec :
/// le workflow la garde avec le rapport. Rien n'est capturé hors de la fenêtre de l'essai.
fn photographier(h: HWND, nom: &str) {
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
    };
    let [g, hh, d, b] = rectangle(h);
    let (l, ha) = (d - g, b - hh);
    if l <= 0 || ha <= 0 {
        return;
    }
    let mut px = vec![0u8; (l * ha * 4) as usize];
    unsafe {
        let ecran = GetDC(None);
        let mem = CreateCompatibleDC(Some(ecran));
        let bmp = CreateCompatibleBitmap(ecran, l, ha);
        let ancien = SelectObject(mem, bmp.into());
        let _ = BitBlt(mem, 0, 0, l, ha, Some(ecran), g, hh, SRCCOPY | CAPTUREBLT);
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: l,
                biHeight: -ha,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        GetDIBits(
            mem,
            bmp,
            0,
            ha as u32,
            Some(px.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        SelectObject(mem, ancien);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, ecran);
    }
    for p in px.chunks_mut(4) {
        p.swap(0, 2);
        p[3] = 255;
    }
    let dossier = std::path::Path::new("captures");
    let _ = std::fs::create_dir_all(dossier);
    if let Some(img) = image::RgbaImage::from_raw(l as u32, ha as u32, px) {
        let _ = img.save(dossier.join(format!("{nom}.png")));
    }
}

// ---- Le déroulement ---------------------------------------------------------------------

struct Essais {
    liens: Arc<Liens>,
    demandes: interface_commune::fenetres::Demandes,
    logiciel: Arc<Essai>,
    journal: Vec<String>,
    /// Le rectangle de l'icône inventée, tout en bas à droite de l'écran.
    icone: [i32; 4],
}

impl Essais {
    fn noter(&mut self, ligne: impl Into<String>) {
        let l = format!("[{:>6} ms] {}", self.debut().elapsed().as_millis(), ligne.into());
        eprintln!("{l}");
        self.journal.push(l);
    }

    fn debut(&self) -> &'static Instant {
        // ÉTAT GLOBAL (essais seulement) : l'heure de départ commune à toutes les lignes.
        static DEBUT: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
        DEBUT.get_or_init(Instant::now)
    }

    fn echec(&self, quoi: &str) -> ! {
        panic!(
            "{quoi}\nOuvert : {:?}\nActions : {:?}\nChangements : {:?}\nJournal :\n{}",
            self.liens.ouvert(),
            self.logiciel.actions.lock().unwrap(),
            self.logiciel.changements.lock().unwrap(),
            self.journal.join("\n")
        );
    }

    /// Attend que `condition` soit vraie (au plus `secondes`), sinon l'essai échoue.
    fn attendre(&mut self, quoi: &str, secondes: u64, mut condition: impl FnMut(&Essais) -> bool) {
        let debut = Instant::now();
        while !condition(self) {
            if debut.elapsed() > Duration::from_secs(secondes) {
                self.echec(&format!("Rien ne s'est passé : {quoi}"));
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        self.noter(format!("ok : {quoi}"));
    }

    fn actions(&self) -> Vec<String> {
        self.logiciel.actions.lock().unwrap().clone()
    }

    /// Ouvre le volet ou le petit menu, attend qu'il soit montré et prêt (la garde des
    /// premiers instants est passée), et lui donne le clavier.
    fn ouvrir(&mut self, surface: Surface) -> HWND {
        let titre = "Volet de Essai";
        let [g, h, d, b] = self.icone;
        let demande = match surface {
            Surface::Volet => Demande::Volet {
                icone: self.icone,
                clic: Instant::now(),
            },
            Surface::Menu => Demande::Menu {
                icone: self.icone,
                ancre: [(g + d) / 2, (h + b) / 2],
                clic: Instant::now(),
            },
        };
        // Une entrée fabriquée d'abord : Windows laisse alors le programme prendre le premier
        // plan (le clavier de l'essai est un vrai clavier).
        aller_a(10, 10);
        self.demandes.send(demande).expect("demande envoyée");
        self.attendre(&format!("{surface:?} montré"), 15, |e| {
            e.liens.ouvert() == Ouvert::Surface(surface)
        });
        let Some(fen) = fenetre(titre) else {
            self.echec("la fenêtre du volet n'existe pas");
        };
        // L'animation (83 ms) et la garde des premiers instants (250 ms).
        std::thread::sleep(Duration::from_millis(800));
        self.premier_plan(fen, surface);
        fen
    }

    /// La fenêtre a le clavier : sinon on la prend, puis on la clique en un endroit neutre.
    fn premier_plan(&mut self, fen: HWND, surface: Surface) {
        for essai in 0..3 {
            if unsafe { GetForegroundWindow() } == fen {
                self.noter("la fenêtre a le clavier");
                return;
            }
            self.noter(format!("pas le premier plan (essai {essai})"));
            let _ = unsafe { SetForegroundWindow(fen) };
            std::thread::sleep(Duration::from_millis(300));
            if unsafe { GetForegroundWindow() } == fen {
                continue;
            }
            // Le volet se clique sans danger sur son titre ; le petit menu, nulle part.
            if surface == Surface::Volet {
                let [g, h, ..] = rectangle(fen);
                let e = echelle(fen);
                cliquer(g + (30.0 * e) as i32, h + (28.0 * e) as i32);
            }
        }
        if unsafe { GetForegroundWindow() } != fen {
            self.echec("la fenêtre n'a pas pu prendre le clavier (Windows refuse le premier plan)");
        }
    }

    fn ferme(&mut self, quoi: &str) {
        self.attendre(quoi, 5, |e| e.liens.ouvert() == Ouvert::Rien);
        // Le temps de la fermeture complète avant l'ouverture suivante.
        std::thread::sleep(Duration::from_millis(700));
    }
}

#[test]
fn les_fenetres_repondent_aux_entrees_fabriquees() {
    if std::env::var("ESSAI_SENDINPUT").as_deref() != Ok("oui") {
        eprintln!(
            "essai ignoré : il ouvre de vraies fenêtres et pilote le clavier et la souris ; \
             il ne tourne que dans les vérifications GitHub (ESSAI_SENDINPUT=oui)."
        );
        return;
    }
    let liens = Arc::new(Liens::default());
    let (demandes, reception) = canal();
    let logiciel = Arc::new(Essai {
        identite: Identite {
            nom: "Essai".into(),
            version: "0.0.0".into(),
            licence: "MIT".into(),
            description: "Entrées fabriquées.".into(),
            icone,
            logo: None,
        },
        actions: Mutex::default(),
        changements: Mutex::default(),
        demarrage: AtomicBool::new(false),
        vue_exemple: Mutex::default(),
    });
    let pour_les_fenetres: Arc<dyn Logiciel> = logiciel.clone();
    let polices = polices::polices().expect("polices");
    let (fini, fin) = mpsc::channel();
    let (l2, li2) = (pour_les_fenetres, liens.clone());
    let fil = std::thread::spawn(move || {
        let r = veiller(l2, li2, polices, &reception, None);
        let _ = fini.send(format!("{:?}", r.as_ref().map(|_| ()).map_err(|e| e.0.clone())));
    });

    let (largeur, hauteur) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let mut e = Essais {
        liens: liens.clone(),
        demandes: demandes.clone(),
        logiciel: logiciel.clone(),
        journal: Vec::new(),
        icone: [largeur - 100, hauteur - 40, largeur - 84, hauteur - 24],
    };
    e.noter(format!("écran {largeur}×{hauteur}"));
    // Au repos : la fenêtre principale est prête.
    let debut = Instant::now();
    while liens.surface_prete.load(Ordering::Acquire) == 0 {
        if debut.elapsed() > Duration::from_secs(30) || fin.try_recv().is_ok() {
            e.noter(format!("opengl32 chargé : {}", module_charge("opengl32.dll")));
            e.noter(format!(
                "libgallium_wgl chargé : {}",
                module_charge("libgallium_wgl.dll")
            ));
            e.echec("la fenêtre principale n'a pas été préparée (OpenGL absent ?)");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(500));

    // 1. Petit menu : Échap le ferme.
    e.ouvrir(Surface::Menu);
    touche(VK_ESCAPE);
    e.ferme("Échap ferme le petit menu");

    // 2. Petit menu : flèche bas, Entrée → l'action du logiciel part, le menu se ferme.
    let fen = e.ouvrir(Surface::Menu);
    photographier(fen, "2a-menu-ouvert");
    touche(VK_DOWN);
    photographier(fen, "2b-menu-apres-fleche-bas");
    touche(VK_RETURN);
    photographier(fen, "2c-menu-apres-entree");
    e.attendre("Entrée dans le petit menu : l'action part", 5, |e| {
        e.actions().contains(&"bonjour".to_string())
    });
    e.ferme("le petit menu se ferme après son choix");

    // 3. Volet : Échap le ferme.
    e.ouvrir(Surface::Volet);
    touche(VK_ESCAPE);
    e.ferme("Échap ferme le volet");

    // 4. Volet : Tab (la roue), Tab (le réglage rapide), Entrée.
    let fen = e.ouvrir(Surface::Volet);
    photographier(fen, "4a-volet-ouvert");
    touche(VK_TAB);
    photographier(fen, "4b-volet-apres-tab-1");
    touche(VK_TAB);
    photographier(fen, "4c-volet-apres-tab-2");
    touche(VK_RETURN);
    photographier(fen, "4d-volet-apres-entree");
    e.attendre(
        "Tab, Tab, Entrée dans le volet : le réglage rapide change",
        5,
        |e| {
            e.logiciel
                .changements
                .lock()
                .unwrap()
                .contains(&"essai=true".to_string())
        },
    );
    // Le volet reste ouvert après un interrupteur : Échap le ferme.
    touche(VK_ESCAPE);
    e.ferme("Échap ferme le volet après le réglage");

    // 5. Volet : clic sur la roue dentée → les Paramètres s'ouvrent.
    let fen = e.ouvrir(Surface::Volet);
    let [_, haut, droite, _] = rectangle(fen);
    let ech = echelle(fen);
    e.noter(format!("rectangle du volet {:?}, échelle {ech}", rectangle(fen)));
    cliquer(droite - (30.0 * ech) as i32, haut + (28.0 * ech) as i32);
    e.attendre("clic sur la roue : les Paramètres s'ouvrent", 15, |e| {
        e.liens.autres.ouverte(CLE_FENETRE)
    });
    std::thread::sleep(Duration::from_millis(1_200));

    // 6. Paramètres : navigation et réglage au clavier.
    let titre = "Paramètres de Essai";
    let Some(p) = fenetre(titre) else {
        e.echec("la fenêtre des Paramètres est introuvable");
    };
    e.premier_plan(p, Surface::Menu);
    // Elle s'ouvre sur la page « Exemple » : on la voit dessinée.
    e.attendre("la page « Exemple » est dessinée", 5, |e| {
        e.logiciel.vue_exemple.lock().unwrap().is_some()
    });
    // Tab → « Exemple », Tab → « Général », Entrée : on quitte la page « Exemple ».
    touche(VK_TAB);
    touche(VK_TAB);
    touche(VK_RETURN);
    let [g, h, d, b] = rectangle(p);
    let avant = Instant::now();
    aller_a((g + d) / 2, (h + b) / 2);
    std::thread::sleep(Duration::from_millis(500));
    let derniere = *e.logiciel.vue_exemple.lock().unwrap();
    if derniere.is_some_and(|t| t > avant) {
        e.echec("Tab, Tab, Entrée n'a pas changé de page : la page « Exemple » est toujours dessinée");
    }
    e.noter("ok : la navigation au clavier a changé de page");
    // Tab, Tab (« Mise à jour », « À propos »), Tab (l'interrupteur « Démarrer avec Windows »),
    // Espace. Un ou deux Tab de plus au besoin, avant de conclure.
    touche(VK_TAB);
    touche(VK_TAB);
    for _ in 0..4 {
        touche(VK_TAB);
        touche(VK_SPACE);
        std::thread::sleep(Duration::from_millis(200));
        if e.logiciel
            .changements
            .lock()
            .unwrap()
            .contains(&format!("{}=true", cles::DEMARRAGE))
        {
            break;
        }
    }
    e.attendre("Espace sur l'interrupteur : le réglage change", 5, |e| {
        e.logiciel
            .changements
            .lock()
            .unwrap()
            .contains(&format!("{}=true", cles::DEMARRAGE))
    });

    // 7. Paramètres : Alt+F4 les ferme.
    e.premier_plan(p, Surface::Menu);
    alt_f4();
    e.attendre("Alt+F4 ferme les Paramètres", 10, |e| {
        !e.liens.autres.ouverte(CLE_FENETRE)
    });
    std::thread::sleep(Duration::from_millis(800));

    // 8. Petit menu : clic sur « Paramètres » → ils se rouvrent.
    let fen = e.ouvrir(Surface::Menu);
    let [g, h, ..] = rectangle(fen);
    let ech = echelle(fen);
    e.noter(format!("rectangle du petit menu {:?}", rectangle(fen)));
    // « Dire bonjour » (32), un trait (9) et des écarts, puis « Paramètres » : son centre.
    cliquer(g + (80.0 * ech) as i32, h + (66.0 * ech) as i32);
    e.attendre(
        "clic sur « Paramètres » du petit menu : ils s'ouvrent",
        15,
        |e| e.liens.autres.ouverte(CLE_FENETRE),
    );

    // La fin : tout se ferme, la boucle rend la main.
    liens.autres.tout_fermer();
    std::thread::sleep(Duration::from_millis(500));
    let _ = demandes.send(Demande::Quitter);
    let _ = fin.recv_timeout(Duration::from_secs(10));
    drop(fil);
}
