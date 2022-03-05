//! Dynamic Frequency Switching control
//!
//! #TODO
//! - Sleep functionality/Awake lock

use super::Error;
use crate::prelude::*;

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

static DFS_MUTEX: CriticalSectionSpinLockMutex<Locks> = CriticalSectionSpinLockMutex::new(Locks {
    cpu: 0,
    apb: 0,
    awake: 0,
    pll_d2: 0,
});

/// DFS structure
pub(super) struct DFS {
    callbacks: [&'static dyn Fn(super::CPUSource, Hertz, Hertz, super::CPUSource, Hertz, Hertz);
        MAX_CALLBACKS],
    nr_callbacks: CriticalSectionSpinLockMutex<usize>,
}

impl DFS {
    pub(crate) fn new() -> Self {
        DFS {
            callbacks: [&|_, _, _, _, _, _| {}; MAX_CALLBACKS],

            nr_callbacks: CriticalSectionSpinLockMutex::new(0),
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
    fn do_callbacks(
        &self,
        cpu_source_before: super::CPUSource,
        cpu_frequency_before: Hertz,
        apb_frequency_before: Hertz,
        cpu_source_after: super::CPUSource,
        cpu_frequency_after: Hertz,
        apb_frequency_after: Hertz,
    ) {
        if cpu_source_after == cpu_source_before
            && cpu_frequency_after == cpu_frequency_before
            && apb_frequency_after == apb_frequency_before
        {
            return;
        }

        // copy the callbacks to prevent needing to have interrupts disabled the entire time
        // as callback cannot be deleted this is ok.
        let (nr, callbacks) = (&self.dfs.nr_callbacks).lock(|nr| (*nr, self.dfs.callbacks));

        for i in 0..nr {
            callbacks[i](
                cpu_source_before,
                cpu_frequency_before,
                apb_frequency_before,
                cpu_source_after,
                cpu_frequency_after,
                apb_frequency_after,
            );
        }
    }

    /// lock the CPU to maximum frequency
    pub(crate) fn lock_cpu_frequency(&'a mut self) -> LockCPU {
        (&DFS_MUTEX).lock(|data| {
            data.cpu += 1;

            if data.cpu == 1
                && (data.apb == 0 || self.cpu_frequency_locked > self.cpu_frequency_apb_locked)
            {
                let cpu_source_before = self.cpu_source;
                let cpu_frequency_before = self.cpu_frequency;
                let apb_frequency_before = self.apb_frequency;

                self.set_cpu_frequency_locked(data.pll_d2 > 0).unwrap();

                self.do_callbacks(
                    cpu_source_before,
                    cpu_frequency_before,
                    apb_frequency_before,
                    self.cpu_source,
                    self.cpu_frequency,
                    self.apb_frequency,
                );
            }
        });
        LockCPU {}
    }

    /// unlock the CPU frequency
    fn unlock_cpu_frequency(&'a mut self) {
        (&DFS_MUTEX).lock(|data| {
            data.cpu -= 1;

            if data.cpu == 0 {
                let cpu_source_before = self.cpu_source;
                let cpu_frequency_before = self.cpu_frequency;
                let apb_frequency_before = self.apb_frequency;

                if data.apb == 0 {
                    self.set_cpu_frequency_default(data.pll_d2 > 0).unwrap();
                } else {
                    self.set_cpu_frequency_apb_locked(data.pll_d2 > 0).unwrap();
                }

                self.do_callbacks(
                    cpu_source_before,
                    cpu_frequency_before,
                    apb_frequency_before,
                    self.cpu_source,
                    self.cpu_frequency,
                    self.apb_frequency,
                );
            }
        });
    }

    // lock the CPU to APB frequency
    pub(crate) fn lock_apb_frequency(&'a mut self) -> LockAPB {
        (&DFS_MUTEX).lock(|data| {
            data.apb += 1;

            if data.apb == 1 {
                if data.cpu == 0 || self.cpu_frequency_apb_locked > self.cpu_frequency_locked {
                    let cpu_source_before = self.cpu_source;
                    let cpu_frequency_before = self.cpu_frequency;
                    let apb_frequency_before = self.apb_frequency;

                    self.set_cpu_frequency_apb_locked(data.pll_d2 > 0).unwrap();

                    self.do_callbacks(
                        cpu_source_before,
                        cpu_frequency_before,
                        apb_frequency_before,
                        self.cpu_source,
                        self.cpu_frequency,
                        self.apb_frequency,
                    );
                }
            }
        });
        LockAPB {}
    }

    /// unlock the CPU from APB
    fn unlock_apb_frequency(&'a mut self) {
        (&DFS_MUTEX).lock(|data| {
            data.apb -= 1;

            if data.apb == 0 {
                let cpu_source_before = self.cpu_source;
                let cpu_frequency_before = self.cpu_frequency;
                let apb_frequency_before = self.apb_frequency;

                if data.cpu == 0 {
                    self.set_cpu_frequency_default(data.pll_d2 > 0).unwrap();
                } else {
                    self.set_cpu_frequency_locked(data.pll_d2 > 0).unwrap();
                }

                self.do_callbacks(
                    cpu_source_before,
                    cpu_frequency_before,
                    apb_frequency_before,
                    self.cpu_source,
                    self.cpu_frequency,
                    self.apb_frequency,
                );
            }
        });
    }

    // lock in awake state
    pub(crate) fn lock_awake(&'a mut self) -> LockAwake {
        (&DFS_MUTEX).lock(|data| {
            data.awake += 1;
        });

        // TODO: implement actual locking
        LockAwake {}
    }

    /// unlock from the awake state
    fn unlock_awake(&'a mut self) {
        (&DFS_MUTEX).lock(|data| {
            data.awake -= 1;

            // TODO: implement actual unlocking
            unimplemented!();
        });
    }

    /// lock the PLL/2 frequency
    pub(crate) fn lock_plld2(&'a mut self) -> LockPllD2 {
        (&DFS_MUTEX).lock(|data| {
            data.pll_d2 += 1;
            if data.pll_d2 == 1 && self.pll_frequency == super::FREQ_OFF {
                let cpu_source_before = self.cpu_source;
                let cpu_frequency_before = self.cpu_frequency;
                let apb_frequency_before = self.apb_frequency;

                self.pll_enable(false).unwrap();

                self.do_callbacks(
                    cpu_source_before,
                    cpu_frequency_before,
                    apb_frequency_before,
                    self.cpu_source,
                    self.cpu_frequency,
                    self.apb_frequency,
                );
            }
        });

        LockPllD2 {}
    }

    /// unlock the PLL/2 frequency
    fn unlock_plld2(&'a mut self) {
        (&DFS_MUTEX).lock(|data| {
            data.pll_d2 -= 1;

            if data.pll_d2 == 0 && self.cpu_source() != super::CPUSource::PLL {
                let cpu_source_before = self.cpu_source;
                let cpu_frequency_before = self.cpu_frequency;
                let apb_frequency_before = self.apb_frequency;

                self.pll_disable();

                self.do_callbacks(
                    cpu_source_before,
                    cpu_frequency_before,
                    apb_frequency_before,
                    self.cpu_source,
                    self.cpu_frequency,
                    self.apb_frequency,
                );
            }
        });
    }

    /// Add callback which will be called when clock speeds are changed.
    ///
    /// NOTE: these callbacks are called in an interrupt free environment,
    /// so should be as short as possible
    ///
    /// TODO: at the moment only static lifetime callbacks are allowed
    pub(crate) fn add_callback<F>(&mut self, f: &'static F) -> Result<(), Error>
    where
        F: Fn(super::CPUSource, Hertz, Hertz, super::CPUSource, Hertz, Hertz),
    {
        // need to disable interrupts, because otherwise deadlock can arise
        // when interrupt is called after mutex is obtained and interrupt
        // routine also adds callback

        let callbacks = &mut self.dfs.callbacks;
        (&self.dfs.nr_callbacks).lock(|nr| {
            if *nr >= MAX_CALLBACKS {
                return Err(Error::TooManyCallbacks);
            }

            callbacks[*nr] = f;
            *nr += 1;
            Ok(())
        })
    }

    /// Get the current count of the PCU, APB, Awake and PLL/2 locks
    ///
    /// Note that this function cannot be used form within a callback
    /// as it tries to lock the mutex, leading to a dead-lock.
    pub fn get_lock_count(&self) -> Locks {
        (&DFS_MUTEX).lock(|data| *data)
    }
}
