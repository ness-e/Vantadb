// B2b: zero `unwrap`/`expect` remain in prod code below (poison recovery via
// `into_inner`); the file-level allow was removed so E1 stays enforced.
// Unit tests keep the crate-level `cfg_attr(test, allow(...))` from lib.rs.

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockExt<T> {
    fn lock_rwlock(&self) -> RwLockReadGuard<'_, T>;
    fn lock_rwlock_mut(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn lock_rwlock(&self) -> RwLockReadGuard<'_, T> {
        // INVARIANT (B2b): a poisoned lock means another thread panicked while
        // holding it. VantaDB recovers the guard via `into_inner` instead of
        // panicking: these locks guard plain data (`Vec`, index entries) whose
        // container stays memory-safe under unwind, and every caller here is
        // synchronous — propagating `Result` would force `?` through the hot
        // read path for no gain. Any logic-level inconsistency surfaces as a
        // validation error downstream, never as silent corruption. Revisit if
        // `vanta-chaos` stress ever observes poisoning in practice.
        self.read().unwrap_or_else(|e| e.into_inner())
    }
    fn lock_rwlock_mut(&self) -> RwLockWriteGuard<'_, T> {
        // INVARIANT (B2b): same as `lock_rwlock` — recover, don't panic.
        self.write().unwrap_or_else(|e| e.into_inner())
    }
}

pub trait MutexExt<T> {
    fn lock_mutex(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_mutex(&self) -> MutexGuard<'_, T> {
        // INVARIANT (B2b): same as `lock_rwlock` — recover, don't panic.
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_ext_lock() {
        let m = std::sync::Mutex::new(42u32);
        assert_eq!(*m.lock_mutex(), 42);
    }

    #[test]
    fn test_mutex_ext_mutate() {
        let m = std::sync::Mutex::new(0u32);
        *m.lock_mutex() = 7;
        assert_eq!(*m.lock_mutex(), 7);
    }

    #[test]
    fn test_rwlock_ext_read() {
        let rw = std::sync::RwLock::new(42u32);
        assert_eq!(*rw.lock_rwlock(), 42);
    }

    #[test]
    fn test_rwlock_ext_write() {
        let rw = std::sync::RwLock::new(0u32);
        *rw.lock_rwlock_mut() = 7;
        assert_eq!(*rw.lock_rwlock(), 7);
    }

    #[test]
    fn test_rwlock_ext_mutate_and_read() {
        let rw = std::sync::RwLock::new(String::from("hello"));
        {
            let mut guard = rw.lock_rwlock_mut();
            guard.push_str(" world");
        }
        assert_eq!(*rw.lock_rwlock(), "hello world");
    }

    // B2b RED: poisoned locks must NOT panic — helpers recover the guard.
    #[test]
    fn lock_mutex_recovers_from_poison() {
        let m = std::sync::Mutex::new(42u32);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = m.lock().unwrap();
            panic!("poison the mutex");
        }));
        assert!(m.is_poisoned());
        assert_eq!(*m.lock_mutex(), 42);
    }

    #[test]
    fn lock_rwlock_recovers_from_poison() {
        // NOTE: std poisons an RwLock only when a *writer* panics (readers
        // can't leave data inconsistent). Poison via a write guard, then
        // prove the read helper still recovers the guard.
        let rw = std::sync::RwLock::new(42u32);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut guard = rw.write().unwrap();
            *guard = 99;
            panic!("poison the rwlock");
        }));
        assert!(rw.is_poisoned());
        assert_eq!(*rw.lock_rwlock(), 99);
    }

    #[test]
    fn lock_rwlock_mut_recovers_from_poison() {
        let rw = std::sync::RwLock::new(0u32);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = rw.write().unwrap();
            panic!("poison the rwlock");
        }));
        assert!(rw.is_poisoned());
        *rw.lock_rwlock_mut() = 7;
        assert_eq!(*rw.lock_rwlock(), 7);
    }
}
