//! Dynamic Frequency Switching control
//!

use super::Error;

/// maximum number of callbacks
pub const MAX_CALLBACKS: usize = 10;

/// number of cpu, apb and awake locks
#[derive(Copy, Clone)]
struct Locks {
    cpu: usize,
    apb: usize,
    awake: usize,
    pll_d2: usize,
}

static DFS_MUTEX: spin::Mutex<Locks> = spin::Mutex::new(Locks {
    cpu: 0,
    apb: 0,
    awake: 0,
    pll_d2: 0,
});

/// DFS structure
pub(super) struct DFS {
    callbacks: [&'static dyn Fn(); MAX_CALLBACKS],
    nr_callbacks: spin::Mutex<usize>,
}

impl DFS {
    pub(crate) fn new() -> Self {
        DFS {
            callbacks: [&|| {}; MAX_CALLBACKS],

            nr_callbacks: spin::Mutex::new(0),
        }
    }
}

/// A RAII implementation of a "scoped lock" for CPU frequency.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_cpu_frequency method on ClockControlConfig
pub struct LockCPU<'a> {
    clock_control: &'a mut super::ClockControl,
}

/// A RAII implementation of a "scoped lock" for APB frequency.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_apb_frequency method on ClockControlConfig
pub struct LockAPB<'a> {
    clock_control: &'a mut super::ClockControl,
}

/// A RAII implementation of a "scoped lock" for Awake state.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_awake method on ClockControlConfig
pub struct LockAwake<'a> {
    clock_control: &'a mut super::ClockControl,
}

/// A RAII implementation of a "scoped lock" for PLL/2 frequency.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_plld2 method on ClockControlConfig
pub struct LockPllD2<'a> {
    clock_control: &'a mut super::ClockControl,
}

impl<'a> Drop for LockCPU<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_cpu_frequency();
    }
}

impl<'a> Drop for LockAPB<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_apb_frequency();
    }
}

impl<'a> Drop for LockAwake<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_awake();
    }
}

impl<'a> Drop for LockPllD2<'a> {
    fn drop(&mut self) {
        self.clock_control.unlock_plld2();
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

    // lock the CPU to maximum frequency
    pub(crate) fn lock_cpu_frequency(&'a mut self) -> LockCPU {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.cpu += 1;

            if data.cpu == 1 {
                self.set_cpu_frequency_locked().unwrap();
                self.do_callbacks()
            }
        });
        LockCPU {
            clock_control: self,
        }
    }

    fn unlock_cpu_frequency(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.cpu -= 1;

            if data.cpu == 0 {
                if data.apb == 0 {
                    self.set_cpu_frequency_default().unwrap();
                } else {
                    self.set_cpu_frequency_apb_locked().unwrap();
                }
                self.do_callbacks()
            }
        });
    }

    // lock the CPU to APB frequency
    pub(crate) fn lock_apb_frequency(&'a mut self) -> LockAPB {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.apb += 1;

            if data.apb == 1 && data.cpu == 0 {
                self.set_cpu_frequency_apb_locked().unwrap();
                self.do_callbacks();
            }
        });
        LockAPB {
            clock_control: self,
        }
    }

    fn unlock_apb_frequency(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.apb -= 1;

            if data.apb == 0 && data.cpu == 0 {
                self.set_cpu_frequency_default().unwrap();
                self.do_callbacks()
            }
        });
    }

    // lock in awake state
    pub(crate) fn lock_awake(&'a mut self) -> LockAwake {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.awake += 1;
        });

        unimplemented!();
        LockAwake {
            clock_control: self,
        }
    }

    fn unlock_awake(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.awake -= 1;

            unimplemented!();
        });
    }

    // lock in awake state
    pub(crate) fn lock_plld2(&'a mut self) -> LockPllD2 {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.pll_d2 += 1;
            if data.pll_d2 == 1 && self.pll_frequency == super::FREQ_OFF {
                self.pll_enable(false);
                self.do_callbacks();
            }
        });

        LockPllD2 {
            clock_control: self,
        }
    }

    fn unlock_plld2(&'a mut self) {
        xtensa_lx6_rt::interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.pll_d2 -= 1;

            if data.pll_d2 == 0 && super::CPUSource::PLL != self.cpu_source() {
                self.pll_disable();
                self.do_callbacks();
            }
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
