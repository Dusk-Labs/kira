use std::sync::Arc;
use std::sync::RwLock;

pub struct Aro<T>(Arc<RwLock<T>>);
impl<T> Aro<T> {
    #[allow(dead_code)]
    pub fn new(inner: T) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }
}
impl<T> Clone for Aro<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct Arw<T>(Arc<RwLock<T>>);
impl<T> Arw<T> {
    pub fn new(inner: T) -> Self {
        Self(Arc::new(RwLock::new(inner)))
    }
    pub fn read(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.0.write().unwrap()
    }
}
impl<T> Clone for Arw<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> From<Arw<T>> for Aro<T> {
    fn from(value: Arw<T>) -> Self {
        Self(value.0)
    }
}
