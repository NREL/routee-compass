use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

///
/// there are no read-only locks in the existing rust library
/// that prevent executors from having write access. this pair
/// of classes allows a driver application to instantiate a RwLock
/// on an object, and then issue read-only clones of that lock to
/// executors in a parallel setup.
///
/// see https://stackoverflow.com/a/68908523
///
pub struct DriverReadOnlyLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> DriverReadOnlyLock<T> {
    pub fn new(val: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(val)),
        }
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.inner.write()
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.inner.read()
    }

    pub fn read_only(&self) -> ExecutorReadOnlyLock<T> {
        ExecutorReadOnlyLock {
            inner: self.inner.clone(),
        }
    }
}

pub struct ExecutorReadOnlyLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> ExecutorReadOnlyLock<T> {
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.inner.read()
    }
}
