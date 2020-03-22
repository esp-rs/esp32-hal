//! Multi core safe lock which runs function at start and stop
//! when first lock acquired or last lock released
//!
pub struct LockExecute<T> {
    mutex: spin::Mutex<usize>,
    phantom: core::marker::PhantomData<T>,
}

pub struct LockExecuteGuard<'a, T> {
    lock_execute: &'a LockExecute<T>,
    stop: fn(&mut T),
    t: &'a mut T,
}

impl<T> LockExecute<T> {
    pub const fn new() -> Self {
        LockExecute {
            mutex: spin::Mutex::new(0),
            phantom: core::marker::PhantomData,
        }
    }

    pub fn lock<'a>(
        &'a self,
        t: &'a mut T,
        start: fn(&mut T),
        stop: fn(&mut T),
    ) -> LockExecuteGuard<'a, T> {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = self.mutex.lock();
            *data += 1;
            if *data == 1 {
                (start)(t)
            }
        });

        LockExecuteGuard {
            lock_execute: &self,
            stop: stop,
            t: t,
        }
    }

    pub fn count(&self) -> usize {
        *(self.mutex.lock())
    }
}

impl<'a, T> Drop for LockExecuteGuard<'a, T> {
    fn drop(&mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = self.lock_execute.mutex.lock();
            *data -= 1;
            if *data == 0 {
                (self.stop)(self.t)
            }
        });
    }
}
