use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

#[derive(Debug)]
pub struct ArrayVec<T, const CAPACITY: usize> {
    buffer: [MaybeUninit<T>; CAPACITY],
    len: usize,
}

impl<T, const CAPACITY: usize> ArrayVec<T, CAPACITY> {
    pub const fn new() -> Self {
        Self {
            buffer: [const { MaybeUninit::uninit() }; CAPACITY],
            len: 0,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.buffer.as_ptr() as *const T
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.buffer.as_mut_ptr() as *mut T
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    pub fn push(&mut self, value: T) {
        self.buffer[self.len] = MaybeUninit::new(value);
        self.len += 1;
    }
}

impl<T, const CAPACITY: usize> Drop for ArrayVec<T, CAPACITY> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) };
    }
}

impl<T, const CAPACITY: usize> Deref for ArrayVec<T, CAPACITY> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const CAPACITY: usize> DerefMut for ArrayVec<T, CAPACITY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, I: SliceIndex<[T]>, const CAPACITY: usize> Index<I> for ArrayVec<T, CAPACITY> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>, const CAPACITY: usize> IndexMut<I> for ArrayVec<T, CAPACITY> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}
