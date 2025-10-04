use core::{
    cell::UnsafeCell,
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

/// Primitive for tracking read-write locks. Doesn't
/// actually protect anything.
pub struct RWLock {
    tag: &'static str,
    state: AtomicUsize,
}

#[derive(Debug)]
pub enum TryLockError {
    HasWriter,
    HasReaders(usize),
}

pub type TryLockResult<T> = Result<T, TryLockError>;

const RWLOCK_FREE: usize = 0;
const RWLOCK_MAX_READERS: usize = RWLOCK_WRITER_ONLY - 1;
const RWLOCK_WRITER_ONLY: usize = usize::MAX;

impl RWLock {
    pub const fn new(tag: &'static str) -> Self {
        Self {
            tag,
            state: AtomicUsize::new(0),
        }
    }

    pub fn try_read(&self) -> TryLockResult<(&Self, usize)> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state == RWLOCK_WRITER_ONLY {
                return Err(TryLockError::HasWriter);
            }
            if state == RWLOCK_MAX_READERS {
                return Err(TryLockError::HasReaders(state));
            }
            if self
                .state
                .compare_exchange(state, state + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok((self, state + 1));
            }
        }
    }

    pub fn release_read(&self) {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state == RWLOCK_WRITER_ONLY {
                panic!("Tried to release a reader while a writer exists");
            }
            if state == 0 {
                panic!("Tried to release a reader while none exists");
            }
            if self
                .state
                .compare_exchange(state, state - 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn try_write(&self) -> TryLockResult<&Self> {
        match self.state.compare_exchange(
            RWLOCK_FREE,
            RWLOCK_WRITER_ONLY,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(self),
            Err(state) if state == RWLOCK_WRITER_ONLY => Err(TryLockError::HasWriter),
            Err(readers) => Err(TryLockError::HasReaders(readers)),
        }
    }

    pub fn release_write(&self) {
        match self.state.compare_exchange(
            RWLOCK_WRITER_ONLY,
            RWLOCK_FREE,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => {}
            Err(state) if state == 0 => panic!("Tried to release a writer while none exists"),
            Err(readers) => panic!("Tried to release a writer while {} readers exists", readers),
        }
    }

    pub fn tag(&self) -> &'static str {
        self.tag
    }
}

impl Debug for RWLock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RWLock(#")?;
        f.write_str(self.tag())?;
        f.write_str(":")?;
        match self.state.load(Ordering::Acquire) {
            0 => f.write_str("*"),
            RWLOCK_WRITER_ONLY => f.write_str("W"),
            readers => f.write_fmt(format_args!("{}R", readers)),
        }?;
        f.write_str(")")
    }
}

/// Multithreaded version of RefCell
pub struct RWCell<T> {
    data: UnsafeCell<T>,
    lock: RWLock,
}

impl<T> RWCell<T> {
    pub const fn new(tag: &'static str, data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            lock: RWLock::new(tag),
        }
    }

    pub fn tag(&self) -> &'static str {
        self.lock.tag()
    }

    pub fn try_read_with_readers(&self) -> TryLockResult<(RWCellReader<'_, T>, usize)> {
        self.lock.try_read().map(|(lock, readers)| {
            (
                RWCellReader {
                    data: self.data.get(),
                    lock,
                },
                readers,
            )
        })
    }

    pub fn try_read(&self) -> TryLockResult<RWCellReader<'_, T>> {
        self.try_read_with_readers().map(|(guard, _)| guard)
    }

    pub fn read(&self) -> RWCellReader<'_, T> {
        self.try_read().expect("failed to acquire read lock")
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        *self.read()
    }

    pub fn get_ptr(&self) -> *const T {
        self.read().as_ptr()
    }

    pub fn try_write(&self) -> TryLockResult<RWCellWriter<'_, T>> {
        self.lock.try_write().map(|lock| RWCellWriter {
            data: self.data.get(),
            lock,
        })
    }

    pub fn write(&self) -> RWCellWriter<'_, T> {
        self.try_write().expect("failed to acquire write lock")
    }

    pub fn set(&self, new_data: T) {
        *self.write() = new_data;
    }

    pub fn get_mut_ptr(&self) -> *mut T {
        self.write().as_ptr()
    }
}

impl<T: Debug> Debug for RWCell<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RWCell(#")?;
        f.write_str(self.tag())?;
        f.write_str(", ")?;
        match self.try_read_with_readers() {
            Ok((guard, 1)) => f.write_fmt(format_args!("*, {:?}", *guard)),
            Ok((guard, readers)) => f.write_fmt(format_args!("{}R, {:?}", readers, *guard)),
            Err(TryLockError::HasWriter) => f.write_str("W, .."),
            Err(TryLockError::HasReaders(_)) => f.write_str("<MAX>R, .."),
        }?;
        f.write_str(")")
    }
}

unsafe impl<T> Sync for RWCell<T> {}

pub struct RWCellReader<'rwcell, T> {
    data: *const T,
    lock: &'rwcell RWLock,
}

impl<T> RWCellReader<'_, T> {
    pub fn as_ptr(&self) -> *const T {
        self.data
    }
}

impl<T> AsRef<T> for RWCellReader<'_, T> {
    fn as_ref(&self) -> &T {
        // SAFETY: no writer should exist while a reader does
        unsafe { &*self.as_ptr() }
    }
}

impl<T> Deref for RWCellReader<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> Drop for RWCellReader<'_, T> {
    fn drop(&mut self) {
        self.lock.release_read();
    }
}

pub struct RWCellWriter<'rwcell, T> {
    data: *mut T,
    lock: &'rwcell RWLock,
}

impl<T> RWCellWriter<'_, T> {
    pub fn as_ptr(&self) -> *mut T {
        self.data
    }
}

impl<T> AsRef<T> for RWCellWriter<'_, T> {
    fn as_ref(&self) -> &T {
        // SAFETY: no other lock can exist while a writer does
        unsafe { &*self.as_ptr() }
    }
}

impl<T> AsMut<T> for RWCellWriter<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        // SAFETY: no other lock can exist while a writer does
        unsafe { &mut *self.as_ptr() }
    }
}

impl<T> Deref for RWCellWriter<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for RWCellWriter<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Drop for RWCellWriter<'_, T> {
    fn drop(&mut self) {
        self.lock.release_write();
    }
}
