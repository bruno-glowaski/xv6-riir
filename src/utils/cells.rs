use core::cell::UnsafeCell;

pub struct Idc<T>(UnsafeCell<T>);

/// Behold: the "I Don't Care-cell", god of undefined behavior.
/// For all intents and purposes, treat this as an unsafe block.
impl<T> Idc<T> {
    pub const fn new(inner: T) -> Self {
        Self(UnsafeCell::new(inner))
    }

    #[inline]
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

unsafe impl<T> Sync for Idc<T> {}
