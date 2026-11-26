//! La zone des icônes (bloc M) : **le seul moteur** des icônes près de l'horloge. Une fenêtre
//! cachée, ses icônes (`Shell_NotifyIcon`), leurs rappels, leur remise quand l'Explorateur
//! redémarre, les notifications, le menu gris de Windows. `zone::Zone` (plusieurs icônes,
//! la boucle du logiciel) et `horloge::Horloge` (l'icône seule, la boucle de la bibliothèque)
//! s'en servent tous les deux.
//!
//! L'icône seule d'`Horloge` garde son numéro (`uID` 1, sans GUID : C-09) ; les icônes d'une
//! zone gardent leur GUID stable. D'où [`Ident`].

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
#[cfg(not(test))]
use windows::Win32::UI::Shell::Shell_NotifyIconW;
use windows::Win32::UI::Shell::{
    NIF_GUID, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_ERROR, NIIF_INFO,
    NIIF_RESPECT_QUIET_TIME, NIIF_WARNING, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFY_ICON_MESSAGE, NOTIFYICON_VERSION_4, NOTIFYICONDATAW, NOTIFYICONIDENTIFIER,
    Shell_NotifyIconGetRect,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, GWLP_USERDATA,
    GetCursorPos, GetWindowLongPtrW, HICON, HMENU, MENU_ITEM_FLAGS, MF_CHECKED, MF_GRAYED, MF_POPUP,
    MF_SEPARATOR, MF_STRING, PostMessageW, RegisterWindowMessageW, SetForegroundWindow, SetMenuDefaultItem,
    SetWindowLongPtrW, TPM_BOTTOMALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP,
    WM_DISPLAYCHANGE, WM_NULL,
};
use windows::core::{GUID, HSTRING, PCWSTR};

use super::decodage::{
    Evenement, FinDeSession, ICONES_MAX, INFOBULLE_MAX, LigneMenu, Message, ajouter_avec_repli, cles, copier,
    guid_stable, raccourcir, texte_menu,
};
use super::image::{Image, icone_windows};
use crate::bords::systeme::cachee::{ClasseDejaLa, fenetre_cachee};
use crate::place::Rect;
use ic_coeur::contrat::Niveau;

/// Le premier message de rappel des icônes d'une zone : chaque icône a le sien
/// (`PREMIER_MESSAGE + n`), ce qui dit sans erreur de quelle icône vient un geste.
pub(crate) const PREMIER_MESSAGE: u32 = WM_APP + 0x200;

/// Comment Windows reconnaît une icône.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ident {
    /// Les icônes d'une zone : un GUID stable (Windows retient l'icône montrée).
    Guid(u128),
    /// L'icône seule d'`Horloge` : son numéro (`uID`), comme avant.
    Numero(u32),
}

/// Une icône posée.
struct Posee {
    ident: Ident,
    message: u32,
    hicon: HICON,
    infobulle: String,
    /// Windows a accepté l'icône (sinon : réessayée au prochain changement).
    dans_la_barre: bool,
}

/// Ce que le logiciel fait de la fin de session ([`Zone::sur_fin_de_session`]).
type SurFinDeSession = Box<dyn FnMut(&FinDeSession)>;

/// Une procédure de plus, appelée **avant** celle de la zone pour chaque message de sa fenêtre
/// (sauf pendant son propre appel) : `Some(résultat)` si elle l'a traité. C'est par elle que
/// l'icône seule d'`Horloge` reçoit ses messages (commandes, menu de secours, arguments…).
pub(crate) type Complement = Box<dyn FnMut(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT>>;

/// Ce que partagent la zone et la procédure de sa fenêtre (même fil).
#[derive(Default)]
struct Partage {
    evenements: VecDeque<Evenement>,
    /// Le message de rappel de chaque icône d'une zone → son identifiant et son GUID.
    rappels: BTreeMap<u32, (String, Ident)>,
    taskbar_created: u32,
    /// L'Explorateur a redémarré : toutes les icônes sont à remettre.
    a_remettre: bool,
    /// La session se ferme (lu par [`Zone::fin_de_session`]).
    fin_de_session: Option<FinDeSession>,
    /// Ce que le logiciel fait tout de suite de la fin de session ([`Zone::sur_fin_de_session`]).
    sur_fin_de_session: Option<SurFinDeSession>,
    /// La procédure de plus ([`Complement`]).
    complement: Option<Complement>,
}

/// Plusieurs icônes près de l'horloge, sur le fil qui l'a créée (voir `zone`).
pub struct Zone {
    fenetre: HWND,
    graine: String,
    programme: String,
    taskbar_created: u32,
    icones: BTreeMap<String, Posee>,
    partage: Box<RefCell<Partage>>,
    /// La fenêtre est déjà détruite ([`Zone::detruire`]).
    detruite: bool,
}

impl Zone {
    /// Crée la fenêtre cachée des icônes sur ce fil. `nom` : le nom court du logiciel (sa
    /// classe de fenêtre en est tirée) ; `graine` : ce qui rend ses GUID uniques (gardée d'une
    /// version à l'autre, pour que Windows retienne les icônes montrées).
    pub fn creer(nom: &str, graine: &str) -> Result<Zone, String> {
        let classe = HSTRING::from(format!("{nom}-icones"));
        // Déjà enregistrée (une deuxième zone dans le même programme) : sans conséquence.
        Zone::avec_classe(&classe, &classe, ClasseDejaLa::Reprendre, graine)
    }

    /// Crée la fenêtre cachée sous cette classe et ce titre (l'icône seule d'`Horloge` : la
    /// classe de l'instance unique, qu'une deuxième instance retrouve).
    pub(crate) fn avec_classe(
        classe: &HSTRING,
        titre: &HSTRING,
        deja: ClasseDejaLa,
        graine: &str,
    ) -> Result<Zone, String> {
        let programme = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        // Pas `HWND_MESSAGE` : elle doit recevoir « TaskbarCreated », diffusé aux seules
        // fenêtres ordinaires.
        let fenetre = fenetre_cachee(classe, titre, procedure, deja)?;
        // SAFETY: nom constant ; « TaskbarCreated » est diffusé quand l'Explorateur redémarre.
        let taskbar_created = unsafe { RegisterWindowMessageW(windows::core::w!("TaskbarCreated")) };
        let partage = Box::new(RefCell::new(Partage {
            taskbar_created,
            ..Default::default()
        }));
        // SAFETY: fenêtre de ce fil ; l'adresse de `partage` (dans une boîte, donc fixe)
        // reste valable jusqu'à `detruire` (ou `Drop`), qui la remet à 0 avant de détruire la
        // fenêtre.
        unsafe {
            SetWindowLongPtrW(
                fenetre,
                GWLP_USERDATA,
                &*partage as *const RefCell<Partage> as isize,
            )
        };
        Ok(Zone {
            fenetre,
            graine: graine.into(),
            programme,
            taskbar_created,
            icones: BTreeMap::new(),
            partage,
            detruite: false,
        })
    }

    /// La fenêtre cachée (son adresse) : pour y poster des messages, par exemple.
    pub fn fenetre(&self) -> isize {
        self.fenetre.0 as isize
    }

    pub(crate) fn hwnd(&self) -> HWND {
        self.fenetre
    }

    /// Le message « TaskbarCreated » (l'Explorateur a redémarré), 0 si Windows l'a refusé.
    pub(crate) fn taskbar_created(&self) -> u32 {
        self.taskbar_created
    }

    /// Les identifiants des icônes posées.
    pub fn icones(&self) -> impl Iterator<Item = &str> {
        self.icones.keys().map(String::as_str)
    }

    /// Le GUID de l'icône `id` (le même d'un lancement à l'autre).
    pub fn guid(&self, id: &str) -> u128 {
        guid_stable(&self.graine, &self.programme, id)
    }

    fn donnees(&self, ident: Ident) -> NOTIFYICONDATAW {
        let mut d = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.fenetre,
            ..Default::default()
        };
        match ident {
            Ident::Guid(g) => {
                d.uFlags = NIF_GUID;
                d.guidItem = GUID::from_u128(g);
            }
            Ident::Numero(n) => d.uID = n,
        }
        d
    }

    /// Ajoute l'icône `id`, ou change son image et son info-bulle. Rend faux si Windows l'a
    /// refusée (l'Explorateur redémarre…) : la redemander plus tard.
    pub fn poser(&mut self, id: &str, image: &Image, infobulle: &str) -> bool {
        self.remettre_si_besoin();
        let Some(hicon) = icone_windows(image.taille as i32, &image.bgra) else {
            return false;
        };
        let infobulle = raccourcir(infobulle, INFOBULLE_MAX);
        if self.icones.contains_key(id) {
            return self.modifier(id, Some(hicon), Some(&infobulle));
        }
        let Some(message) = self.message_libre() else {
            // SAFETY: icône créée ci-dessus, jamais donnée à Windows.
            let _ = unsafe { DestroyIcon(hicon) };
            return false;
        };
        let ident = Ident::Guid(self.guid(id));
        // Une icône restée d'un programme arrêté brutalement a le même GUID : retirée avant.
        let reste = self.donnees(ident);
        // Échoue sans conséquence s'il n'y a rien à retirer.
        let _ = shell(NIM_DELETE, &reste);
        let ok = self.poser_hicon(id, ident, message, hicon, &infobulle);
        if let Ok(mut p) = self.partage.try_borrow_mut() {
            p.rappels.insert(message, (id.to_string(), ident));
        }
        ok
    }

    /// Pose une nouvelle icône `id` (absente jusque-là) : `hicon` est gardée par la zone (et
    /// détruite par elle si elle n'est pas nulle) ; `infobulle` est prise telle quelle. Ses
    /// gestes arrivent par le message `message`, sans passer par [`Zone::evenements`] (voir
    /// [`Zone::lire_rappel`]).
    pub(crate) fn poser_hicon(
        &mut self,
        id: &str,
        ident: Ident,
        message: u32,
        hicon: HICON,
        infobulle: &str,
    ) -> bool {
        let mut d = self.donnees(ident);
        d.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        d.hIcon = hicon;
        copier(&mut d.szTip, infobulle);
        let ok = self.ajouter(&mut d, message);
        self.icones.insert(
            id.to_string(),
            Posee {
                ident,
                message,
                hicon,
                infobulle: infobulle.to_string(),
                dans_la_barre: ok,
            },
        );
        ok
    }

    /// `NIM_ADD` puis la version 4 (gestes complets, point du geste dans `wparam`) ; si
    /// l'icône existe déjà, `NIM_MODIFY` à la place (voir [`ajouter_avec_repli`]).
    fn ajouter(&self, d: &mut NOTIFYICONDATAW, message: u32) -> bool {
        d.uFlags |= NIF_MESSAGE;
        d.uCallbackMessage = message;
        d.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let d: &NOTIFYICONDATAW = d;
        crate::sonde::etape("Shell_NotifyIcon (ajout de l'icône)");
        ajouter_avec_repli(
            || shell(NIM_ADD, d),
            || shell(NIM_MODIFY, d),
            || {
                let _ = shell(NIM_SETVERSION, d);
            },
        )
    }

    /// Change l'image (`Some`) ou l'info-bulle (`Some`, prise telle quelle) de l'icône `id`.
    /// Une icône que Windows n'avait pas acceptée est ajoutée à nouveau. L'ancienne image est
    /// détruite ; la nouvelle est gardée par la zone (détruite ici si l'icône n'existe pas).
    pub(crate) fn modifier(&mut self, id: &str, hicon: Option<HICON>, infobulle: Option<&str>) -> bool {
        let Some(p) = self.icones.get_mut(id) else {
            detruire_image(hicon);
            return false;
        };
        let ancienne = hicon.map(|h| std::mem::replace(&mut p.hicon, h));
        if let Some(t) = infobulle {
            p.infobulle = t.to_string();
        }
        let (ident, message, image, texte, dans_la_barre) =
            (p.ident, p.message, p.hicon, p.infobulle.clone(), p.dans_la_barre);
        let mut d = self.donnees(ident);
        d.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
        d.hIcon = image;
        copier(&mut d.szTip, &texte);
        let ok = if dans_la_barre {
            crate::sonde::etape("Shell_NotifyIcon (changement de l'icône)");
            shell(NIM_MODIFY, &d)
        } else {
            self.ajouter(&mut d, message)
        };
        if let Some(p) = self.icones.get_mut(id) {
            p.dans_la_barre = ok;
        }
        detruire_image(ancienne);
        ok
    }

    fn message_libre(&self) -> Option<u32> {
        let pris: Vec<u32> = self.icones.values().map(|p| p.message).collect();
        (0..ICONES_MAX as u32)
            .map(|n| PREMIER_MESSAGE + n)
            .find(|m| !pris.contains(m))
    }

    /// Retire l'icône `id` (rien si elle n'est pas posée).
    pub fn retirer(&mut self, id: &str) {
        let Some(p) = self.icones.remove(id) else {
            return;
        };
        let d = self.donnees(p.ident);
        let _ = shell(NIM_DELETE, &d);
        detruire_image(Some(p.hicon));
        if let Ok(mut s) = self.partage.try_borrow_mut()
            && s.rappels.get(&p.message).is_some_and(|(i, _)| i == id)
        {
            s.rappels.remove(&p.message);
        }
    }

    /// Une notification posée sur l'icône `id` (titre, texte, niveau). Windows la garde
    /// silencieuse quand « Ne pas déranger » est actif. Un clic dessus revient en
    /// [`Message::Notification`] de cette icône. Rend faux si l'icône n'est pas posée ou
    /// si Windows refuse.
    pub fn notifier(&self, id: &str, titre: &str, texte: &str, niveau: Niveau) -> bool {
        let Some(p) = self.icones.get(id) else {
            return false;
        };
        let mut d = self.donnees(p.ident);
        d.uFlags |= NIF_INFO;
        copier(&mut d.szInfoTitle, titre);
        copier(&mut d.szInfo, texte);
        d.dwInfoFlags = match niveau {
            Niveau::Information => NIIF_INFO,
            Niveau::Avertissement => NIIF_WARNING,
            Niveau::Erreur => NIIF_ERROR,
        } | NIIF_RESPECT_QUIET_TIME;
        shell(NIM_MODIFY, &d)
    }

    /// Le rectangle de l'icône `id` (pixels réels), si Windows le donne.
    pub fn rect(&self, id: &str) -> Option<Rect> {
        rect_icone(self.fenetre, self.icones.get(id)?.ident)
    }

    /// Les gestes et clics de notification reçus depuis le dernier appel, dans l'ordre. À
    /// appeler après chaque `DispatchMessageW` de la boucle du logiciel. Remet aussi les
    /// icônes si l'Explorateur a redémarré.
    pub fn evenements(&mut self) -> Vec<Evenement> {
        self.remettre_si_besoin();
        match self.partage.try_borrow_mut() {
            Ok(mut p) => p.evenements.drain(..).collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Le geste reçu par le message de rappel de l'icône `id` (posée par
    /// [`Zone::poser_hicon`]), décodé comme ceux de [`Zone::evenements`].
    pub(crate) fn lire_rappel(&self, id: &str, wp: WPARAM, lp: LPARAM) -> Option<Evenement> {
        decoder(self.fenetre, id, self.icones.get(id)?.ident, wp, lp)
    }

    /// Ce que le logiciel fait **tout de suite** quand la session Windows se ferme
    /// ([`FinDeSession`]) : `f` est appelée par la procédure de la fenêtre cachée, sur ce fil,
    /// pendant `WM_ENDSESSION`, et Windows attend son retour. C'est là qu'il faut enregistrer
    /// ce qui ne l'est pas encore (rapidement : l'arrêt n'attend que quelques secondes), sans
    /// rien ouvrir ni attendre l'utilisateur. Remplace la fonction précédente, s'il y en avait
    /// une.
    ///
    /// `f` s'exécute pendant `DispatchMessageW` : elle ne touche pas à la zone (qui est
    /// empruntée par la boucle du logiciel) ; elle appelle ce que le logiciel partage déjà
    /// avec sa boucle (un `Rc<RefCell<…>>`, un fichier à écrire…).
    pub fn sur_fin_de_session(&mut self, f: impl FnMut(&FinDeSession) + 'static) {
        if let Ok(mut p) = self.partage.try_borrow_mut() {
            p.sur_fin_de_session = Some(Box::new(f));
        }
    }

    /// La fin de session reçue depuis le dernier appel, s'il y en a eu une (`None` ensuite).
    /// À lire après `DispatchMessageW`, comme [`Zone::evenements`] : c'est un simple signal,
    /// **trop tardif pour enregistrer** (voir [`FinDeSession`] et [`Zone::sur_fin_de_session`]).
    pub fn fin_de_session(&mut self) -> Option<FinDeSession> {
        self.partage.try_borrow_mut().ok()?.fin_de_session.take()
    }

    /// La procédure de plus ([`Complement`]) ; remplace la précédente.
    pub(crate) fn completer(&self, f: Complement) {
        if let Ok(mut p) = self.partage.try_borrow_mut() {
            p.complement = Some(f);
        }
    }

    /// L'Explorateur a redémarré (« TaskbarCreated » reçu par la procédure) : chaque icône
    /// est remise.
    pub(crate) fn remettre_si_besoin(&mut self) {
        let remettre = self
            .partage
            .try_borrow_mut()
            .map(|mut p| std::mem::take(&mut p.a_remettre))
            .unwrap_or(false);
        if remettre {
            self.remettre_tout();
        }
    }

    /// Remet chaque icône, avec son image et son info-bulle (l'Explorateur a redémarré).
    pub(crate) fn remettre_tout(&mut self) {
        let ids: Vec<String> = self.icones.keys().cloned().collect();
        for id in ids {
            let Some(p) = self.icones.get(&id) else { continue };
            let mut d = self.donnees(p.ident);
            d.uFlags |= NIF_ICON | NIF_TIP | NIF_SHOWTIP;
            d.hIcon = p.hicon;
            copier(&mut d.szTip, &p.infobulle);
            let ok = self.ajouter(&mut d, p.message);
            if let Some(p) = self.icones.get_mut(&id) {
                p.dans_la_barre = ok;
            }
        }
    }

    /// Le menu gris de Windows au point `point` (pixels réels), jusqu'au choix : la clé de
    /// la ligne choisie, ou `None`. Les « & » des textes sont doublés ici. La boucle des
    /// messages tourne pendant que le menu est ouvert (les gestes arrivent ensuite par
    /// [`Zone::evenements`]).
    pub fn menu_windows(&self, point: [i32; 2], lignes: &[LigneMenu]) -> Option<String> {
        self.menu_windows_complet(point, lignes, None, "TrackPopupMenu (menu de la zone)")
    }

    /// [`Zone::menu_windows`], avec la ligne par défaut (en gras) de clé `defaut`, et le nom
    /// de la boucle imbriquée pour la sonde.
    pub(crate) fn menu_windows_complet(
        &self,
        point: [i32; 2],
        lignes: &[LigneMenu],
        defaut: Option<&str>,
        etiquette: &'static str,
    ) -> Option<String> {
        let mut toutes = Vec::new();
        cles(lignes, &mut toutes);
        // SAFETY: menus créés, remplis de chaînes vivantes pendant chaque appel, affichés
        // puis détruits ici (DestroyMenu détruit aussi les sous-menus) ; la fenêtre est celle
        // de ce fil.
        let choix = unsafe {
            let Ok(m) = CreatePopupMenu() else {
                return None;
            };
            let mut n = 0usize;
            remplir(m, lignes, defaut, &mut n);
            let _ = SetForegroundWindow(self.fenetre);
            let _imbriquee = crate::sonde::imbriquee(etiquette);
            let c = TrackPopupMenu(
                m,
                TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
                point[0],
                point[1],
                None,
                self.fenetre,
                None,
            )
            .0 as usize;
            // Recommandé par Windows : que le menu se ferme bien au clic suivant ailleurs.
            let _ = PostMessageW(Some(self.fenetre), WM_NULL, WPARAM(0), LPARAM(0));
            let _ = DestroyMenu(m);
            c
        };
        let cle = toutes.get(choix.checked_sub(1)?)?;
        (!cle.is_empty()).then(|| cle.clone())
    }

    /// Retire les icônes, puis détruit la fenêtre cachée (sa procédure ne lit plus rien de
    /// la zone). Sans effet la deuxième fois.
    pub(crate) fn detruire(&mut self) {
        if self.detruite {
            return;
        }
        let ids: Vec<String> = self.icones.keys().cloned().collect();
        for id in ids {
            self.retirer(&id);
        }
        self.detruite = true;
        // SAFETY: plus aucun message ne lira l'état partagé, puis la fenêtre de ce fil est
        // détruite.
        unsafe {
            SetWindowLongPtrW(self.fenetre, GWLP_USERDATA, 0);
            let _ = DestroyWindow(self.fenetre);
        }
    }
}

// Les appels à `Shell_NotifyIconW` (opération, numéro, GUID, message de rappel) : dans les
// essais, ils sont notés ici au lieu d'être faits, et réussissent ; aucune icône n'est
// montrée.
#[cfg(test)]
thread_local! {
    pub(crate) static APPELS: RefCell<Vec<(u32, u32, u128, u32)>> = const { RefCell::new(Vec::new()) };
}

/// `Shell_NotifyIconW` : vrai si Windows a accepté.
#[cfg(not(test))]
fn shell(op: NOTIFY_ICON_MESSAGE, d: &NOTIFYICONDATAW) -> bool {
    // SAFETY: `d` est complet et de la taille annoncée (`donnees`), et vit pendant l'appel.
    unsafe { Shell_NotifyIconW(op, d) }.as_bool()
}

/// Dans les essais : l'appel est noté ([`APPELS`]), rien n'est donné à Windows.
#[cfg(test)]
fn shell(op: NOTIFY_ICON_MESSAGE, d: &NOTIFYICONDATAW) -> bool {
    APPELS.with_borrow_mut(|a| a.push((op.0, d.uID, d.guidItem.to_u128(), d.uCallbackMessage)));
    true
}

/// Détruit une image d'icône donnée à la zone, si elle n'est pas nulle.
fn detruire_image(hicon: Option<HICON>) {
    if let Some(h) = hicon
        && !h.0.is_null()
    {
        // SAFETY: image créée pour la zone, qui n'est plus utilisée (remplacée ou retirée).
        let _ = unsafe { DestroyIcon(h) };
    }
}

/// Remplit un menu de Windows ; les lignes à choisir sont numérotées à partir de 1 dans
/// l'ordre de [`cles`]. La ligne de clé `defaut` est la ligne par défaut (en gras).
///
/// # Safety
/// `m` est un menu vivant de ce fil.
unsafe fn remplir(m: HMENU, lignes: &[LigneMenu], defaut: Option<&str>, n: &mut usize) {
    for l in lignes {
        match l {
            LigneMenu::Trait => {
                // SAFETY: voir la fonction.
                let _ = unsafe { AppendMenuW(m, MF_SEPARATOR, 0, PCWSTR::null()) };
            }
            LigneMenu::Choix {
                cle,
                texte,
                coche,
                actif,
            } => {
                *n += 1;
                let mut f: MENU_ITEM_FLAGS = MF_STRING;
                if *coche {
                    f |= MF_CHECKED;
                }
                if !*actif {
                    f |= MF_GRAYED;
                }
                // SAFETY: voir la fonction ; la chaîne vit pendant l'appel.
                let _ = unsafe { AppendMenuW(m, f, *n, &HSTRING::from(texte_menu(texte))) };
                if defaut.is_some_and(|d| d == cle) {
                    // SAFETY: voir la fonction ; `*n` est le numéro de la ligne ajoutée.
                    let _ = unsafe { SetMenuDefaultItem(m, *n as u32, 0) };
                }
            }
            LigneMenu::Sous { texte, actif, lignes } => {
                // SAFETY: sous-menu créé ici, rattaché à `m` (détruit avec lui).
                let Ok(s) = (unsafe { CreatePopupMenu() }) else {
                    continue;
                };
                // SAFETY: `s` est vivant.
                unsafe { remplir(s, lignes, defaut, n) };
                let mut f: MENU_ITEM_FLAGS = MF_STRING | MF_POPUP;
                if !*actif {
                    f |= MF_GRAYED;
                }
                // SAFETY: voir la fonction.
                let _ = unsafe { AppendMenuW(m, f, s.0 as usize, &HSTRING::from(texte_menu(texte))) };
            }
        }
    }
}

/// Le rectangle de l'icône `ident` de la fenêtre `h` (pixels réels).
fn rect_icone(h: HWND, ident: Ident) -> Option<Rect> {
    let mut id = NOTIFYICONIDENTIFIER {
        cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
        hWnd: h,
        ..Default::default()
    };
    match ident {
        Ident::Guid(g) => id.guidItem = GUID::from_u128(g),
        Ident::Numero(n) => id.uID = n,
    }
    // SAFETY: `id` local et complet.
    unsafe { Shell_NotifyIconGetRect(&id) }
        .ok()
        .map(|r| [r.left, r.top, r.right, r.bottom])
}

/// Le geste d'un message de rappel de l'icône `id` (`ident`, fenêtre `h`), s'il compte.
fn decoder(h: HWND, id: &str, ident: Ident, wp: WPARAM, lp: LPARAM) -> Option<Evenement> {
    let message = Message::de((lp.0 as u32) & 0xFFFF)?;
    // Avec la version 4, `wparam` porte le point du geste (x, y).
    let ancre = [
        i32::from(wp.0 as u16 as i16),
        i32::from((wp.0 >> 16) as u16 as i16),
    ];
    let rect = rect_icone(h, ident).unwrap_or_else(|| {
        let mut s = POINT::default();
        // SAFETY: POINT local.
        let _ = unsafe { GetCursorPos(&mut s) };
        [s.x - 8, s.y - 8, s.x + 8, s.y + 8]
    });
    Some(Evenement {
        icone: id.to_string(),
        message,
        ancre,
        rect,
    })
}

impl Drop for Zone {
    fn drop(&mut self) {
        self.detruire();
    }
}

/// Note la fin de session, puis appelle la fonction du logiciel ([`Zone::sur_fin_de_session`]).
/// La fonction est sortie de l'état partagé pendant l'appel : si elle appelle la zone, rien
/// n'est emprunté deux fois ; elle est remise après, sauf si elle en a posé une autre.
fn signaler_fin_de_session(partage: Option<&RefCell<Partage>>, fin: FinDeSession) {
    let Some(partage) = partage else { return };
    let mut f = {
        let Ok(mut p) = partage.try_borrow_mut() else {
            return;
        };
        p.fin_de_session = Some(fin);
        p.sur_fin_de_session.take()
    };
    if let Some(f) = f.as_mut() {
        f(&fin);
    }
    if let Ok(mut p) = partage.try_borrow_mut()
        && p.sur_fin_de_session.is_none()
    {
        p.sur_fin_de_session = f;
    }
}

/// Appelle la procédure de plus ([`Complement`]), sortie de l'état partagé pendant l'appel
/// (les messages qui arrivent pendant ce temps vont à la zone seule). Elle est remise après,
/// sauf si la zone a été détruite entre-temps ou si une autre a été posée.
fn appeler_le_complement(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
    // SAFETY: lecture d'une valeur de la fenêtre de ce fil.
    let adresse = unsafe { GetWindowLongPtrW(h, GWLP_USERDATA) };
    // SAFETY: la valeur rangée est 0 ou l'adresse du `Partage` de la zone, valable tant
    // qu'elle y est rangée (`detruire` la remet à 0 avant toute libération) ; les messages
    // arrivent sur ce même fil.
    let partage = unsafe { (adresse as *const RefCell<Partage>).as_ref() }?;
    let mut f = partage.try_borrow_mut().ok()?.complement.take()?;
    let r = f(h, msg, wp, lp);
    // La zone peut avoir été détruite pendant l'appel : `partage` n'est relu que si son
    // adresse est encore rangée dans la fenêtre.
    // SAFETY: comme ci-dessus.
    if unsafe { GetWindowLongPtrW(h, GWLP_USERDATA) } == adresse
        && let Ok(mut p) = partage.try_borrow_mut()
        && p.complement.is_none()
    {
        p.complement = Some(f);
    }
    r
}

extern "system" fn procedure(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if msg == WM_DISPLAYCHANGE {
        // Windows refait l'affichage (écran retiré, rendu, autre résolution) : les fenêtres pas
        // encore créées du logiciel attendent la fin de la reprise (1 s). Un logiciel qui n'a
        // pas sa propre procédure de fenêtre en profite aussi. Traitement par défaut ensuite.
        crate::autres_fenetres::affichage_change();
    }
    if let Some(r) = appeler_le_complement(h, msg, wp, lp) {
        return r;
    }
    // SAFETY: la valeur rangée est 0 ou l'adresse du `Partage` de la zone, valable jusqu'à
    // ce que `detruire` la remette à 0 ; les messages arrivent sur ce même fil. Relue après
    // le complément, qui a pu détruire la zone.
    let partage = unsafe { GetWindowLongPtrW(h, GWLP_USERDATA) } as *const RefCell<Partage>;
    if let Some(fin) = FinDeSession::de(msg, wp.0, lp.0) {
        // La session se ferme : le logiciel enregistre maintenant, avant que Windows ne
        // reprenne la main. Zéro : message traité.
        // SAFETY: voir ci-dessus.
        signaler_fin_de_session(unsafe { partage.as_ref() }, fin);
        return LRESULT(0);
    }
    // SAFETY: voir ci-dessus.
    if let Some(p) = unsafe { partage.as_ref() }
        && let Ok(mut p) = p.try_borrow_mut()
    {
        if msg == p.taskbar_created && msg != 0 {
            p.a_remettre = true;
            return LRESULT(0);
        }
        if let Some((id, ident)) = p.rappels.get(&msg).cloned() {
            // Bornée : un logiciel qui ne lirait jamais les événements ne remplit pas la
            // mémoire.
            if let Some(e) = decoder(h, &id, ident, wp, lp)
                && p.evenements.len() < 256
            {
                p.evenements.push_back(e);
            }
            return LRESULT(0);
        }
    }
    // SAFETY: traitement par défaut de Windows, avec les paramètres reçus.
    unsafe { DefWindowProcW(h, msg, wp, lp) }
}

#[cfg(test)]
mod essais;
