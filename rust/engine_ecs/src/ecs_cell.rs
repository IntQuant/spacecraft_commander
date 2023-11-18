use std::cell::UnsafeCell;

use serde::{Deserialize, Serialize};

#[derive(Default)]
pub(crate) struct EcsCell<T: ?Sized> {
    inner: UnsafeCell<T>,
}

impl<T> EcsCell<T> {
    pub(crate) fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }

    pub(crate) fn get(&self) -> &T {
        // SAFETY: normally can only get &T with &EcsCell<T>.
        unsafe { &*self.inner.get() }
    }
    pub(crate) fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    /// SAFETY: ensure that Self::get() method doesn't get called while returned reference exists.
    pub(crate) unsafe fn get_mut_unsafe(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }
}

unsafe impl<T: ?Sized + Send> Send for EcsCell<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for EcsCell<T> {}

impl<T: Clone> Clone for EcsCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get().clone())
    }
}

impl<T: Serialize> Serialize for EcsCell<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.get().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for EcsCell<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = T::deserialize(deserializer)?;
        Ok(Self::new(inner))
    }
}
