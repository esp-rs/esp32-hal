//! Dynamic Frequency Switching control
//!
//! #TODO
//! - Sleep functionality/Awake lock

use super::Error;
use xtensa_lx6::interrupt;

/// maximum number of callbacks
pub const MAX_CALLBACKS: usize = 10;

/// number of cpu, apb, awake and pll_d2 locks
#[derive(Copy, Clone, Debug)]
pub struct Locks {
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
pub struct LockCPU {}

/// A RAII implementation of a "scoped lock" for APB frequency.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_apb_frequency method on ClockControlConfig
pub struct LockAPB {}

/// A RAII implementation of a "scoped lock" for Awake state.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_awake method on ClockControlConfig
pub struct LockAwake {}

/// A RAII implementation of a "scoped lock" for PLL/2 frequency.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
/// This structure is created by the lock_plld2 method on ClockControlConfig
pub struct LockPllD2 {}

/// Drop of the RAII to unlock the CPU frequency
impl<'a> Drop for LockCPU {
    fn drop(&mut self) {
        unsafe {
            super::CLOCK_CONTROL
                .as_mut()
                .unwrap()
                .unlock_cpu_frequency();
        }
    }
}

/// Drop of the RAII to unlock the APB frequency
impl<'a> Drop for LockAPB {
    fn drop(&mut self) {
        unsafe {
            super::CLOCK_CONTROL
                .as_mut()
                .unwrap()
                .unlock_apb_frequency();
        }
    }
}

/// Drop of the RAII to unlock the Awake state
impl<'a> Drop for LockAwake {
    fn drop(&mut self) {
        unsafe {
            super::CLOCK_CONTROL.as_mut().unwrap().unlock_awake();
        }
    }
}

/// Drop of the RAII to unlock the PLL/2 frequency
impl<'a> Drop for LockPllD2 {
    fn drop(&mut self) {
        unsafe {
            super::CLOCK_CONTROL.as_mut().unwrap().unlock_plld2();
        }
    }
}

impl<'a> super::ClockControl {
    /// call all the callbacks
    fn do_callbacks(&self) {
        // copy the callbacks to prevent needing to have interrupts disabled the entire time
        // as callback cannot be deleted this is ok.
        let (nr, callbacks) = interrupt::free(|_| {
            let nr = self.dfs.nr_callbacks.lock();
            (*nr, self.dfs.callbacks)
        });

        for i in 0..nr {
            callbacks[i]();
        }
    }

    /// lock the CPU to maximum frequency
    pub(crate) fn lock_cpu_frequency(&'a mut self) -> LockCPU {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.cpu += 1;

            if data.cpu == 1 {
                if data.apb == 0 || self.cpu_frequency_locked > self.cpu_frequency_apb_locked {
                    self.set_cpu_frequency_locked(data.pll_d2 > 0).unwrap();
                    self.do_callbacks()
                }
            }
        });
        LockCPU {}
    }

    /// unlock the CPU frequency
    fn unlock_cpu_frequency(&'a mut self) {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.cpu -= 1;

            if data.cpu == 0 {
                if data.apb == 0 {
                    self.set_cpu_frequency_default(data.pll_d2 > 0).unwrap();
                } else {
                    self.set_cpu_frequency_apb_locked(data.pll_d2 > 0).unwrap();
                }
                self.do_callbacks()
            }
        });
    }

    // lock the CPU to APB frequency
    pub(crate) fn lock_apb_frequency(&'a mut self) -> LockAPB {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.apb += 1;

            if data.apb == 1 {
                if data.cpu == 0 || self.cpu_frequency_apb_locked > self.cpu_frequency_locked {
                    self.set_cpu_frequency_apb_locked(data.pll_d2 > 0).unwrap();
                    self.do_callbacks();
                }
            }
        });
        LockAPB {}
    }

    /// unlock the CPU from APB
    fn unlock_apb_frequency(&'a mut self) {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.apb -= 1;

            if data.apb == 0 {
                if data.cpu == 0 {
                    self.set_cpu_frequency_default(data.pll_d2 > 0).unwrap();
                } else {
                    self.set_cpu_frequency_locked(data.pll_d2 > 0).unwrap();
                }
                self.do_callbacks()
            }
        });
    }

    // lock in awake state
    pub(crate) fn lock_awake(&'a mut self) -> LockAwake {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.awake += 1;
        });

        // TODO: implement actual locking
        LockAwake {}
    }

    /// unlock from the awake state
    fn unlock_awake(&'a mut self) {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.awake -= 1;

            // TODO: implement actual unlocking
            unimplemented!();
        });
    }

    /// lock the PLL/2 frequency
    pub(crate) fn lock_plld2(&'a mut self) -> LockPllD2 {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.pll_d2 += 1;
            if data.pll_d2 == 1 && self.pll_frequency == super::FREQ_OFF {
                self.pll_enable(false).unwrap();
                self.do_callbacks();
            }
        });

        LockPllD2 {}
    }

    /// unlock the PLL/2 frequency
    fn unlock_plld2(&'a mut self) {
        interrupt::free(|_| {
            let mut data = DFS_MUTEX.lock();
            data.pll_d2 -= 1;

            if data.pll_d2 == 0 && self.cpu_source() != super::CPUSource::PLL {
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
        interrupt::free(|_| {
            let mut nr = self.dfs.nr_callbacks.lock();

            if *nr >= MAX_CALLBACKS {
                return Err(Error::TooManyCallbacks);
            }

            self.dfs.callbacks[*nr] = f;
            *nr += 1;
            Ok(())
        })
    }

    /// Get the current count of the PCU, APB, Awake and PLL/2 locks
    ///
    /// Note that this function cannot be used form within a callback
    /// as it tries to lock the mutex, leading to a dead-lock.
    pub fn get_lock_count(&self) -> Locks {
        let info = DFS_MUTEX.lock();
        *info
    }
}
