pub struct LockExecute {
    mutex: spin::Mutex<usize>,
    start: fn(),
    stop: fn(),
}

pub struct LockExecuteGuard<'a> {
    lock_execute: &'a LockExecute,
}

impl LockExecute {
    pub const fn new(start: fn(), stop: fn()) -> Self {
        LockExecute {
            mutex: spin::Mutex::new(0),
            start,
            stop,
        }
    }

    pub fn lock(&self) -> LockExecuteGuard {
        if 1 == xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = self.mutex.lock();
            *data += 1;
            *data
        }) {
            (self.start)();
        }
        LockExecuteGuard {
            lock_execute: &self,
        }
    }
}

impl<'a> Drop for LockExecuteGuard<'a> {
    fn drop(&mut self) {
        if 0 == xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = self.lock_execute.mutex.lock();
            *data -= 1;
            *data
        }) {
            (self.lock_execute.stop)();
        }
    }
}
