use crossbeam::atomic::ArcCell;
use std::sync::Arc;

/// Wrapper for static settings/config around ArcCell
pub struct Atom<T>
where
    T: 'static,
{
    data: ArcCell<T>,
}

impl<T> Atom<T> {
    pub fn new(d: T) -> Atom<T> {
        Atom {
            data: ArcCell::new(Arc::new(d)),
        }
    }

    pub fn get(&self) -> Arc<T> {
        self.data.get()
    }

    pub fn set(&self, d: T) -> Arc<T> {
        self.data.set(Arc::new(d))
    }
}
