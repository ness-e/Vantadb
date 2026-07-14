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
