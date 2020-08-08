//! DPort peripheral configuration
//!
//! This peripheral contains many registers, which are used for various different functions.
//! Registers needed in other blocks can be split off.
//!
use crate::target::{dport, DPORT};
use xtensa_lx6::mutex::mutex_trait::Mutex;
use xtensa_lx6::mutex::CriticalSectionSpinLockMutex;

/// Cpu Period Configuration Register
pub struct ClockControl {}

/// DPort registers related to clock control
impl ClockControl {
    pub(crate) fn cpu_per_conf(&self) -> &dport::CPU_PER_CONF {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).cpu_per_conf }
    }
    pub(crate) fn appcpu_ctrl_a(&self) -> &dport::APPCPU_CTRL_A {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).appcpu_ctrl_a }
    }
    pub(crate) fn appcpu_ctrl_b(&self) -> &dport::APPCPU_CTRL_B {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).appcpu_ctrl_b }
    }
    pub(crate) fn appcpu_ctrl_c(&self) -> &dport::APPCPU_CTRL_C {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).appcpu_ctrl_c }
    }
    pub(crate) fn appcpu_ctrl_d(&self) -> &dport::APPCPU_CTRL_D {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).appcpu_ctrl_d }
    }
    pub(crate) fn app_cache_ctrl(&self) -> &dport::APP_CACHE_CTRL {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).app_cache_ctrl }
    }
    pub(crate) fn pro_cache_ctrl(&self) -> &dport::PRO_CACHE_CTRL {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*DPORT::ptr()).pro_cache_ctrl }
    }
}

/// Trait to split the DPORT peripheral into subsets
pub trait Split {
    fn split(self) -> (DPORT, ClockControl);
}

impl Split for DPORT {
    /// function to split the DPORT peripheral into subsets
    fn split(self) -> (DPORT, ClockControl) {
        (self, ClockControl {})
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Peripheral {
    TIMERS = 0,
    SPI0_SPI1 = 1,
    UART0 = 2,
    WDG = 3,
    I2S0 = 4,
    UART1 = 5,
    SPI2 = 6,
    I2C0 = 7,
    UHCI0 = 8,
    RMT = 9,
    PCNT = 10,
    LEDC = 11,
    UHCI1 = 12,
    TIMERGROUP0 = 13,
    EFUSE = 14,
    TIMERGROUP1 = 15,
    SPI3 = 16,
    PWM0 = 17,
    I2C1 = 18,
    CAN = 19,
    PWM1 = 20,
    I2S1 = 21,
    SPI_DMA = 22,
    UART2 = 23,
    UART_MEM = 24,
    PWM2 = 25,
    PWM3 = 26,
    AES = 32 + 0,
    SHA = 32 + 1,
    RSA = 32 + 2,
    SECUREBOOT = 32 + 3,
    SIGNATURE = 32 + 4,

    SDIO_SLAVE = 64 + 4,
    SDIO_HOST = 64 + 13,
    EMAC = 64 + 14,
    RNG = 64 + 15,

    WIFI = 96,
    BT = 97,
    WIFI_BT_COMMON = 98,
    BT_LC = 99,
    BT_BASEBAND = 100,
}

static PERIPHERAL_MUTEX: CriticalSectionSpinLockMutex<()> = CriticalSectionSpinLockMutex::new(());

const WIFI_CLK_MASK: u32 = 0x406; // bits 1, 2, 10
const BT_CLK_MASK: u32 = 0x61; // bits 11, 16, 17
const WIFI_BT_CLK_MASK: u32 = 0x3c9; // bits 0, 3, 6, 7, 8, 9
const BT_LC_CLK_MASK: u32 = 0x3000; // bit 16, 17
const BT_BASEBAND_CLK_MASK: u32 = 0x0800; // bit 11

const EMAC_RST_MASK: u32 = 0x80; // bit 7
const SDIO_HOST_RST_MASK: u32 = 0x40; // bit 6
const SDIO_SLAVE_RST_MASK: u32 = 0x20; // bit 5

const UART_MASK: u32 = 1 << (Peripheral::UART0 as u32)
    | 1 << (Peripheral::UART1 as u32)
    | 1 << (Peripheral::UART2 as u32);

pub fn enable_peripheral(peripheral: Peripheral) {
    let bitnr = peripheral as u32;
    let dport = unsafe { &(*DPORT::ptr()) };
    (&PERIPHERAL_MUTEX).lock(|_| unsafe {
        if bitnr < 32 {
            let mut mask = 1 << bitnr;
            match peripheral {
                Peripheral::UART0 | Peripheral::UART1 | Peripheral::UART2 => {
                    mask |= 1 << (Peripheral::UART_MEM as u32)
                }
                _ => {}
            }

            dport.perip_clk_en.modify(|r, w| w.bits(r.bits() | mask));
            dport.perip_rst_en.modify(|r, w| w.bits(r.bits() & !mask));
        } else if bitnr < 64 {
            let mut mask = 1 << (bitnr - 32);
            match peripheral {
                Peripheral::AES => {
                    mask |= 1 << (Peripheral::SIGNATURE as u32 - 32)
                        | 1 << (Peripheral::SECUREBOOT as u32 - 32);
                }
                Peripheral::SHA => {
                    mask |= 1 << (Peripheral::SECUREBOOT as u32 - 32);
                }
                Peripheral::RSA => {
                    mask |= 1 << (Peripheral::SIGNATURE as u32 - 32);
                }
                _ => {}
            }

            dport.peri_clk_en.modify(|r, w| w.bits(r.bits() | mask));
            dport.peri_rst_en.modify(|r, w| w.bits(r.bits() & !mask));
        } else if bitnr < 96 {
            let mask = 1 << (bitnr - 64);
            dport.wifi_clk_en.modify(|r, w| w.bits(r.bits() | mask));

            let rst_mask = match peripheral {
                Peripheral::EMAC => EMAC_RST_MASK,
                Peripheral::SDIO_HOST => SDIO_HOST_RST_MASK,
                Peripheral::SDIO_SLAVE => SDIO_SLAVE_RST_MASK,
                _ => 0,
            };
            dport
                .core_rst_en
                .modify(|r, w| w.bits(r.bits() & !rst_mask));
        } else {
            let mask = match peripheral {
                Peripheral::WIFI => WIFI_CLK_MASK,
                Peripheral::BT => BT_CLK_MASK,
                Peripheral::WIFI_BT_COMMON => WIFI_BT_CLK_MASK,
                Peripheral::BT_BASEBAND => BT_BASEBAND_CLK_MASK,
                Peripheral::BT_LC => BT_LC_CLK_MASK,
                _ => 0,
            };
            dport.wifi_clk_en.modify(|r, w| w.bits(r.bits() | mask));
            // no reset done
        }
    });
}

pub fn disable_peripheral(peripheral: Peripheral) {
    let bitnr = peripheral as u32;
    let dport = unsafe { &(*DPORT::ptr()) };
    (&PERIPHERAL_MUTEX).lock(|_| unsafe {
        if bitnr < 32 {
            let mut mask = 1 << bitnr;
            dport.perip_clk_en.modify(|r, w| {
                if r.bits() & !mask & UART_MASK == 0 {
                    mask &= !(1 << (Peripheral::UART_MEM as u32));
                }
                w.bits(r.bits() & !mask)
            });
            dport.perip_rst_en.modify(|r, w| w.bits(r.bits() | mask));
        } else if bitnr < 64 {
            let mask = 1 << (bitnr - 32);

            dport.peri_clk_en.modify(|r, w| w.bits(r.bits() & !mask));
            dport.peri_rst_en.modify(|r, w| w.bits(r.bits() | mask));
        } else if bitnr < 96 {
            let mask = 1 << (bitnr - 64);
            dport.wifi_clk_en.modify(|r, w| w.bits(r.bits() & !mask));

            let rst_mask = match peripheral {
                Peripheral::EMAC => EMAC_RST_MASK,
                Peripheral::SDIO_HOST => SDIO_HOST_RST_MASK,
                Peripheral::SDIO_SLAVE => SDIO_SLAVE_RST_MASK,
                _ => 0,
            };
            dport.core_rst_en.modify(|r, w| w.bits(r.bits() | rst_mask));
        } else {
            let mask = match peripheral {
                Peripheral::WIFI => WIFI_CLK_MASK,
                Peripheral::BT => BT_CLK_MASK,
                Peripheral::WIFI_BT_COMMON => WIFI_BT_CLK_MASK,
                Peripheral::BT_BASEBAND => BT_BASEBAND_CLK_MASK,
                Peripheral::BT_LC => BT_LC_CLK_MASK,
                _ => 0,
            };
            dport.wifi_clk_en.modify(|r, w| w.bits(r.bits() & !mask));
            // no reset done
        }
    });
}

pub fn reset_peripheral(peripheral: Peripheral) {
    let bitnr = peripheral as u32;
    let dport = unsafe { &(*DPORT::ptr()) };
    (&PERIPHERAL_MUTEX).lock(|_| unsafe {
        if bitnr < 32 {
            let mask = 1 << bitnr;
            dport.perip_rst_en.modify(|r, w| w.bits(r.bits() | mask));
            dport.perip_rst_en.modify(|r, w| w.bits(r.bits() & !mask));
        } else if bitnr < 64 {
            let mask = 1 << (bitnr - 32);
            dport.peri_rst_en.modify(|r, w| w.bits(r.bits() | mask));
            dport.peri_rst_en.modify(|r, w| w.bits(r.bits() & !mask));
        } else if bitnr < 96 {
            let rst_mask = match peripheral {
                Peripheral::EMAC => EMAC_RST_MASK,
                Peripheral::SDIO_HOST => SDIO_HOST_RST_MASK,
                Peripheral::SDIO_SLAVE => SDIO_SLAVE_RST_MASK,
                _ => 0,
            };
            dport.core_rst_en.modify(|r, w| w.bits(r.bits() | rst_mask));
            dport
                .core_rst_en
                .modify(|r, w| w.bits(r.bits() & !rst_mask));
        } else {
            // do nothing for WiFi / Bluetooth
        }
    });
}
