use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockExt<T> {
    fn lock_rwlock(&self) -> RwLockReadGuard<'_, T>;
    fn lock_rwlock_mut(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> RwLockExt<T> for RwLock<T> {
    fn lock_rwlock(&self) -> RwLockReadGuard<'_, T> {
        self.read().expect("RwLock poisoned")
    }
    fn lock_rwlock_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.write().expect("RwLock poisoned")
    }
}

pub trait MutexExt<T> {
    fn lock_mutex(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_mutex(&self) -> MutexGuard<'_, T> {
        self.lock().expect("Mutex poisoned")
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
}
