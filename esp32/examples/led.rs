#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;

const CORE_HZ: u32 = 40_000_000;

use xtensa_lx6_rt as _;

#[no_mangle]
fn main() -> ! {
    let gpio = unsafe { esp32::Peripherals::steal().GPIO };

    // Configure GPIO2 as Output.
    gpio.gpio_enable_w1ts_reg.write(|w| w.pin2().high());
    gpio.gpio_func2_out_sel_cfg.gpio_func_out_sel_cfg_reg.write(|w| unsafe {
        w.gpio_func_out_sel().bits(0x100)
    });

    loop {
        set_led(&gpio, true);
        delay(CORE_HZ);
        set_led(&gpio, false);
        delay(CORE_HZ);
    }
}

fn set_led(gpio: &esp32::GPIO, v: bool) {
    if v {
        // Set GPIO2 output.
        gpio.gpio_out_w1ts_reg.write(|w| w.pin2().high());
    } else {
        // Clear GPIO2 output.
        gpio.gpio_out_w1tc_reg.write(|w| w.pin2().high());
    }
}

/// cycle accurate delay using the cycle counter register
fn delay(clocks: u32) {
    // NOTE: does not account for rollover
    let target = get_ccount() + clocks;
    loop {
        if get_ccount() > target {
            break;
        }
    }
}

/// Performs a special register read to read the current cycle count.
/// In the future, this can be precompiled to a archive (.a) and linked to so we don't
/// have to require the asm nightly feature - see cortex-m-rt for more details
fn get_ccount() -> u32 {
    let x: u32;
    unsafe { asm!("rsr.ccount a2" : "={a2}"(x) ) };
    x
}

/// Basic panic handler - just loops
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
