use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicIsize, AtomicU32, Ordering};

const FUTEX_WAIT: u32 = 0;
const FUTEX_WAKE: u32 = 1;

const UNLOCKED: u32 = 0;
const LOCKED: u32 = 1;
const CONTENDED: u32 = 2;

pub use core::sync::*;
pub use alloc::sync::Arc;
pub use core::cell::OnceCell;

pub type LockResult<T> = Result<T, PoisonError<T>>;
pub type TryLockResult<T> = Result<T, TryLockError<T>>;

#[derive(Debug)]
pub struct PoisonError<T>(T);

impl<T> PoisonError<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug)]
pub enum TryLockError<T> {
    WouldBlock,
    Poisoned(PoisonError<T>),
}

pub struct Mutex<T> {
    state: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(UNLOCKED),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
        self.lock_inner();
        Ok(MutexGuard { mutex: self })
    }

    fn lock_inner(&self) {
        if self.state.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            return;
        }
        loop {
            for _ in 0..100 {
                if self.state.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                    return;
                }
                core::hint::spin_loop();
            }
            if self.state.swap(CONTENDED, Ordering::Acquire) != CONTENDED
                && self.state.load(Ordering::Relaxed) == CONTENDED
            {
                let ptr = &self.state as *const AtomicU32 as *const u32;
                xv8_libc::futex(ptr, FUTEX_WAIT, CONTENDED);
            }
        }
    }

    pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
        match self.state.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => Ok(MutexGuard { mutex: self }),
            Err(_) => Err(TryLockError::WouldBlock),
        }
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        Ok(unsafe { &mut *self.value.get() })
    }

    pub fn into_inner(self) -> LockResult<T> {
        Ok(self.value.into_inner())
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

unsafe impl<T: Sync> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> core::fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MutexGuard").finish_non_exhaustive()
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.swap(UNLOCKED, Ordering::Release) == CONTENDED {
            let ptr = &self.mutex.state as *const AtomicU32 as *const u32;
            xv8_libc::futex(ptr, FUTEX_WAKE, 1);
        }
    }
}

pub struct RwLock<T> {
    state: AtomicIsize,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send + Sync> Send for RwLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self { state: AtomicIsize::new(0), value: UnsafeCell::new(value) }
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        loop {
            let state = self.state.load(Ordering::Acquire);
            if state < 0 {
                core::hint::spin_loop();
                continue;
            }
            if self
                .state
                .compare_exchange(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Ok(RwLockReadGuard { lock: self });
            }
        }
    }

    pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
        let state = self.state.load(Ordering::Acquire);
        if state < 0 {
            return Err(TryLockError::WouldBlock);
        }
        if self
            .state
            .compare_exchange(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Ok(RwLockReadGuard { lock: self })
        } else {
            Err(TryLockError::WouldBlock)
        }
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        while self.state.compare_exchange(0, -1, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        Ok(RwLockWriteGuard { lock: self })
    }

    pub fn try_write(&self) -> TryLockResult<RwLockWriteGuard<'_, T>> {
        if self.state.compare_exchange(0, -1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            Ok(RwLockWriteGuard { lock: self })
        } else {
            Err(TryLockError::WouldBlock)
        }
    }

    pub fn get_mut(&mut self) -> LockResult<&mut T> {
        Ok(unsafe { &mut *self.value.get() })
    }

    pub fn into_inner(self) -> LockResult<T> {
        Ok(self.value.into_inner())
    }
}

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> core::fmt::Debug for RwLockReadGuard<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RwLockReadGuard").finish_non_exhaustive()
    }
}

impl<T> Deref for RwLockReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}

impl<T> core::fmt::Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RwLockWriteGuard").finish_non_exhaustive()
    }
}

impl<T> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}

pub struct Condvar {
    counter: AtomicU32,
}

impl Condvar {
    pub const fn new() -> Self {
        Self { counter: AtomicU32::new(0) }
    }

    pub fn wait<'a, U>(&self, guard: MutexGuard<'a, U>) -> LockResult<MutexGuard<'a, U>> {
        let count = self.counter.load(Ordering::Relaxed);
        let mutex = guard.mutex;
        drop(guard);

        let ptr = &self.counter as *const AtomicU32 as *const u32;
        if xv8_libc::futex(ptr, FUTEX_WAIT, count) < 0 {
            // EAGAIN or spurious wakeup — proceed
        }

        mutex.lock()
    }

    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Ordering::Release);
        let ptr = &self.counter as *const AtomicU32 as *const u32;
        xv8_libc::futex(ptr, FUTEX_WAKE, 1);
    }

    pub fn notify_all(&self) {
        self.counter.fetch_add(1, Ordering::Release);
        let ptr = &self.counter as *const AtomicU32 as *const u32;
        xv8_libc::futex(ptr, FUTEX_WAKE, i32::MAX as u32);
    }
}

pub struct LazyLock<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: UnsafeCell<Option<F>>,
}

unsafe impl<T: Send + Sync, F: Send> Send for LazyLock<T, F> {}
unsafe impl<T: Send + Sync, F: Sync> Sync for LazyLock<T, F> {}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    pub const fn new(f: F) -> Self {
        Self { cell: OnceCell::new(), init: UnsafeCell::new(Some(f)) }
    }

    pub fn force(this: &Self) -> &T {
        this.cell.get_or_init(|| {
            let f = unsafe { (*this.init.get()).take().expect("LazyLock already initialized") };
            f()
        })
    }

    pub fn into_inner(this: Self) -> Result<T, F> {
        match this.cell.into_inner() {
            Some(v) => Ok(v),
            None => Err(this.init.into_inner().expect("LazyLock corrupted")),
        }
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;
    fn deref(&self) -> &T {
        Self::force(self)
    }
}

pub struct Barrier {
    count: AtomicU32,
    total: u32,
    condvar: Condvar,
    mutex: Mutex<()>,
}

impl Barrier {
    pub const fn new(n: usize) -> Self {
        Barrier {
            count: AtomicU32::new(0),
            total: n as u32,
            condvar: Condvar::new(),
            mutex: Mutex::new(()),
        }
    }

    pub fn wait(&self) -> bool {
        let guard = self.mutex.lock().unwrap();
        let count = self.count.fetch_add(1, Ordering::AcqRel) + 1;
        if count < self.total {
            let mut g = guard;
            while self.count.load(Ordering::Acquire) < self.total {
                g = self.condvar.wait(g).unwrap();
            }
            false
        } else {
            self.condvar.notify_all();
            true
        }
    }
}
