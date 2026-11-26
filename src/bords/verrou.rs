//! Le seul mécanisme de verrou des bords (ORGANIGRAMME.md, §6) : un verrou empoisonné (un fil a
//! paniqué en le tenant) est repris tel quel, jamais par un `unwrap` qui propagerait la panique
//! aux autres fils. Les boucles des fenêtres, le guichet, la sonde, les polices et la reprise
//! après la veille passent tous par ici.
//!
//! Ce que le fil paniqué a laissé est repris tel quel : chaque donnée gardée sous un verrou des
//! bords reste utilisable après une panique (une liste, un drapeau, une file), et les fenêtres
//! sont refaites au besoin.

use std::sync::{LockResult, Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Le résultat d'une prise de verrou (ou d'une attente sur une `Condvar`), repris même
/// empoisonné.
pub(crate) fn sans_poison<G>(r: LockResult<G>) -> G {
    r.unwrap_or_else(PoisonError::into_inner)
}

/// Le verrou d'un partage entre les fils, repris même empoisonné.
pub(crate) fn verrou<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    sans_poison(m.lock())
}

/// La lecture d'un partage lu souvent et changé rarement, reprise même empoisonnée.
pub(crate) fn lecture<T>(r: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    sans_poison(r.read())
}

/// L'écriture d'un partage lu souvent et changé rarement, reprise même empoisonnée.
pub(crate) fn ecriture<T>(r: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    sans_poison(r.write())
}

#[cfg(test)]
mod essais {
    use super::*;
    use std::sync::{Arc, Condvar};
    use std::time::Duration;

    /// Empoisonne `m` : un fil panique en le tenant.
    fn empoisonner<T: Send + 'static>(m: &Arc<Mutex<T>>) {
        let m2 = m.clone();
        let _ = std::thread::spawn(move || {
            let _g = m2.lock();
            panic!("panique voulue de l'essai");
        })
        .join();
        assert!(m.is_poisoned());
    }

    #[test]
    fn un_verrou_empoisonne_est_repris_avec_ce_qu_il_garde() {
        let m = Arc::new(Mutex::new(vec![1, 2]));
        empoisonner(&m);
        verrou(&m).push(3);
        assert_eq!(*verrou(&m), [1, 2, 3]);
    }

    #[test]
    fn lecture_et_ecriture_reprises_apres_une_panique() {
        let r = Arc::new(RwLock::new(1));
        let r2 = r.clone();
        let _ = std::thread::spawn(move || {
            let _g = r2.write();
            panic!("panique voulue de l'essai");
        })
        .join();
        assert!(r.is_poisoned());
        *ecriture(&r) += 1;
        assert_eq!(*lecture(&r), 2);
    }

    #[test]
    fn une_attente_sur_un_verrou_empoisonne_rend_la_main() {
        let m = Arc::new(Mutex::new(false));
        empoisonner(&m);
        let cv = Condvar::new();
        let (g, fin) = sans_poison(cv.wait_timeout_while(verrou(&m), Duration::from_millis(5), |s| !*s));
        assert!(!*g);
        assert!(fin.timed_out());
    }
}
