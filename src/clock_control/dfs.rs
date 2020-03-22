//! Dynamic Frequency Shifting control
//!

use super::Error;

const MAX_CALLBACKS: usize = 10;

#[derive(Copy, Clone)]
struct Locks {
    cpu: usize,
    apb: usize,
    awake: usize,
}

static DFS_MUTEX: spin::Mutex<Locks> = spin::Mutex::new(Locks {
    cpu: 0,
    apb: 0,
    awake: 0,
});

pub(super) struct DFS {
    callbacks: [&'static dyn Fn(); MAX_CALLBACKS],

    nr_callbacks: spin::Mutex<usize>,
}

pub struct ExecuteGuardCPU<'a> {
    clock_control: &'a mut super::ClockControl,
}
pub struct ExecuteGuardAPB<'a> {
    clock_control: &'a mut super::ClockControl,
}
pub struct ExecuteGuardAwake<'a> {
    clock_control: &'a mut super::ClockControl,
}

impl<'a> Drop for ExecuteGuardCPU<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_cpu_frequency();
    }
}

impl<'a> Drop for ExecuteGuardAPB<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_apb_frequency();
    }
}

impl<'a> Drop for ExecuteGuardAwake<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_awake();
    }
}

impl DFS {
    pub(crate) fn new() -> Self {
        DFS {
            callbacks: [&|| {}; MAX_CALLBACKS],

            nr_callbacks: spin::Mutex::new(0),
        }
    }
}

impl<'a> super::ClockControl {
    fn do_callbacks(&self) {
        // copy the callbacks to prevent needing to have interrupts disabled the entire time
        // as callback cannot be deleted this is ok.
        let (nr, callbacks) = xtensa_lx6_rt::interrupt::free(|_| {
            let nr = self.dfs.nr_callbacks.lock();
            (*nr, self.dfs.callbacks)
        });

        for i in 0..nr {
            callbacks[i]();
        }
    }

    pub(crate) fn lock_cpu_frequency(&'a mut self) -> ExecuteGuardCPU {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.cpu += 1;

            if data.cpu == 1 {
                self.set_cpu_frequency_max().unwrap();
                self.do_callbacks()
            }
        });
        ExecuteGuardCPU {
            clock_control: self,
        }
    }

    fn unlock_cpu_frequency(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.cpu -= 1;

            if data.cpu == 0 {
                if data.apb == 0 {
                    self.set_cpu_frequency_min().unwrap();
                } else {
                    self.set_cpu_frequency_apb().unwrap();
                }
                self.do_callbacks()
            }
        });
    }

    pub(crate) fn lock_apb_frequency(&'a mut self) -> ExecuteGuardAPB {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.apb += 1;

            if data.apb == 1 && data.cpu == 0 {
                self.set_cpu_frequency_apb().unwrap();
                self.do_callbacks()
            }
        });
        ExecuteGuardAPB {
            clock_control: self,
        }
    }

    fn unlock_apb_frequency(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.apb -= 1;

            if data.apb == 0 && data.cpu == 0 {
                self.set_cpu_frequency_min().unwrap();
                self.do_callbacks()
            }
        });
    }

    pub(crate) fn lock_awake(&'a mut self) -> ExecuteGuardAwake {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.awake += 1;
        });

        //TODO: unimplemented!();
        ExecuteGuardAwake {
            clock_control: self,
        }
    }

    fn unlock_awake(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.awake -= 1;

            //TODO: unimplemented!();
        });
    }

    /// Add callback which will be called when clock speeds are changed.
    ///
    /// NOTE: these callbacks are called in an interrupt free environment,
    /// so should be as short as possible
    ///
    /// TODO: at the moment only static lifetime callbacks are allow
    pub(crate) fn add_callback<F>(&mut self, f: &'static F) -> Result<(), Error>
    where
        F: Fn(),
    {
        // need to disable interrupts, because otherwise deadlock can arise
        // when interrupt is called after mutex is obtained and interrupt
        // routine also adds callback
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut nr = self.dfs.nr_callbacks.lock();

            if *nr >= MAX_CALLBACKS {
                return Err(Error::TooManyCallbacks);
            }

            self.dfs.callbacks[*nr] = f;
            *nr += 1;
            Ok(())
        })
    }
}
