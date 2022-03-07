#![no_std]
#![no_main]
//! This is an example for the ESP32 with a WS2812B LED strip.
use core::panic::PanicInfo;
use esp32::SPI2;
use esp32_hal::{dport::Split, dprintln, prelude::*, spi, target};
use smart_leds::{SmartLedsWrite, RGB, RGB8};

use esp32_hal::clock_control::{ClockControl, XTAL_FREQUENCY_AUTO};
use esp32_hal::gpio::{Gpio14, Gpio15, Gpio25, Output, PushPull, Unknown};
use esp32_hal::spi::SPI;
use ws2812_spi::Ws2812;
use xtensa_lx::timer::delay;

const NUM_LEDS: usize = 23;
const STEPS: u8 = 10;
const TOP_ROW: usize = 4;
const MID_ROW: usize = 10;
const DT: u32 = 5;
const BREATHING_MULTIPLIER: u32 = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LightData {
    leds: [RGB8; NUM_LEDS],
}
impl LightData {
    fn empty() -> Self {
        Self {
            leds: [RGB8::new(0, 0, 0); NUM_LEDS],
        }
    }
    fn from_gradient(from: RGB8, to: RGB8) -> Self {
        let mut result = [RGB8::default(); NUM_LEDS];
        let r_delta = to.r as i16 - from.r as i16;
        let g_delta = to.g as i16 - from.g as i16;
        let b_delta = to.b as i16 - from.b as i16;
        for i in 0..NUM_LEDS {
            let r = (from.r + (r_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            let g = (from.g + (g_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            let b = (from.b + (b_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            result[i] = RGB8 { r, g, b };
        }
        Self::from(result)
    }
    fn get_brightness(&self) -> u8 {
        self.leds
            .iter()
            .map(|led| led.r + led.g + led.b)
            .max()
            .unwrap()
    }
    fn write_to_strip(
        &self,
        strip: &mut Ws2812<SPI<SPI2, Gpio14<Unknown>, Gpio15<Output<PushPull>>, Gpio25<Unknown>>>,
    ) {
        strip.write(self.leds.iter().cloned()).unwrap();
    }
    fn get_led(&self, index: usize) -> RGB8 {
        self.leds[index]
    }
    fn set_color_all(&mut self, color: RGB8) {
        for i in 0..NUM_LEDS {
            self.set_color(i, color);
        }
    }
    fn set_red(&mut self, index: usize, red: u8) {
        self.leds[index].r = red;
    }
    fn set_green(&mut self, index: usize, green: u8) {
        self.leds[index].g = green;
    }
    fn set_blue(&mut self, index: usize, blue: u8) {
        self.leds[index].b = blue;
    }
    fn set_color(&mut self, led: usize, color: RGB8) {
        self.leds[led] = color;
    }
    fn set_lightness_percent_all(&mut self, lightness: f32) {
        for led in 0..self.leds.len() {
            self.set_lightness_percent(lightness, led);
        }
    }
    fn set_lightness_percent(&mut self, lightness: f32, led: usize) {
        self.leds[led].r = (self.leds[led].r as f32 * lightness) as u8;
        self.leds[led].g = (self.leds[led].g as f32 * lightness) as u8;
        self.leds[led].b = (self.leds[led].b as f32 * lightness) as u8;
    }
}
impl Default for LightData {
    fn default() -> Self {
        Self {
            leds: [RGB8::new(STEPS, STEPS, STEPS); NUM_LEDS],
        }
    }
}

impl From<[RGB8; NUM_LEDS]> for LightData {
    fn from(data: [RGB8; NUM_LEDS]) -> Self {
        Self { leds: data }
    }
}

struct Strip {
    ws: Ws2812<SPI<SPI2, Gpio14<Unknown>, Gpio15<Output<PushPull>>, Gpio25<Unknown>>>,
    data: LightData,
    brightness: u8,
}

impl Strip {
    fn fade_into(&mut self, data: LightData) {
        while self.data != data {
            for i in 0..NUM_LEDS {
                let r_delta = self.data.get_led(i).r as i32 - data.get_led(i).r as i32;
                let g_delta = self.data.get_led(i).g as i32 - data.get_led(i).g as i32;
                let b_delta = self.data.get_led(i).b as i32 - data.get_led(i).b as i32;
                let mut r_step = (r_delta as f32 * 0.05) as u8;
                let mut g_step = (g_delta as f32 * 0.05) as u8;
                let mut b_step = (b_delta as f32 * 0.05) as u8;
                if r_step == 0 {
                    r_step = 1;
                }
                if g_step == 0 {
                    g_step = 1;
                }
                if b_step == 0 {
                    b_step = 1;
                }
                if r_delta < 0 {
                    self.data.set_red(i, self.data.get_led(i).r + r_step);
                } else if r_delta > 0 {
                    self.data.set_red(i, self.data.get_led(i).r - r_step);
                }
                if g_delta < 0 {
                    self.data.set_green(i, self.data.get_led(i).g + g_step);
                } else if g_delta > 0 {
                    self.data.set_green(i, self.data.get_led(i).g - g_step);
                }
                if b_delta < 0 {
                    self.data.set_blue(i, self.data.get_led(i).b + b_step);
                } else if b_delta > 0 {
                    self.data.set_blue(i, self.data.get_led(i).b - b_step);
                }
            }
            self.write();
            delay(BREATHING_MULTIPLIER * 1_000_000);
        }
        self.get_brightness();
    }

    fn startup_animation(&mut self) {
        self.data = LightData::empty();
        self.write();
        for i in 0..TOP_ROW {
            self.set_color(RGB8::new(self.brightness, 0, 0), i);
            self.write();
            delay(5_000_000);
        }
        for i in TOP_ROW..MID_ROW + TOP_ROW {
            self.set_color(RGB8::new(0, self.brightness, 0), i);
            self.write();
            delay(5_000_000);
        }
        for i in MID_ROW + TOP_ROW..NUM_LEDS {
            self.set_color(RGB8::new(0, 0, self.brightness), i);
            self.write();
            delay(5_000_000);
        }
        delay(40_000_000 * 10);
    }

    fn shutdown_animation(&mut self) {
        let mut i = NUM_LEDS;
        while i > 0 {
            i -= 1;
            self.set_color(RGB8::new(0, 0, 0), i);
            self.write();
            delay(5_000_000);
        }
    }

    fn write(&mut self) {
        self.data.write_to_strip(&mut self.ws);
    }
    fn set_color(&mut self, color: RGB8, index: usize) {
        self.data.set_color(index, color);
        self.write();
    }
    fn set_solid(&mut self, color: RGB8) {
        self.data.set_color_all(color);
        self.write();
    }
    fn get_brightness(&mut self) {
        self.data.get_brightness();
    }
}

#[entry]
fn main() -> ! {
    let dp = target::Peripherals::take().unwrap();
    let (_, dport_clock_control) = dp.DPORT.split();

    let clkcntrl = ClockControl::new(
        dp.RTCCNTL,
        dp.APB_CTRL,
        dport_clock_control,
        XTAL_FREQUENCY_AUTO,
    )
    .unwrap();
    let (clkcntrl_config, _) = clkcntrl.freeze().unwrap();
    let pins = dp.GPIO.split();
    let data_out = pins.gpio15.into_push_pull_output();
    let spi: SPI<_, _, _, _> = SPI::<esp32::SPI2, _, _, _, _>::new(
        dp.SPI2,
        spi::Pins {
            sclk: pins.gpio14,
            sdo: data_out,
            sdi: Some(pins.gpio25),
            cs: None,
        },
        spi::config::Config {
            baudrate: 3.MHz().into(),
            bit_order: spi::config::BitOrder::MSBFirst,
            data_mode: spi::config::MODE_0,
        },
        clkcntrl_config,
    )
    .unwrap();
    let ws = Ws2812::new(spi);

    let mut strip = Strip {
        ws,
        data: LightData::from_gradient(RGB8::new(40, 0, 0), RGB::new(0, 0, 40)),
        brightness: 10,
    };
    loop {
        strip.startup_animation();
        delay(1_000_000);
        strip.fade_into(LightData::from_gradient(
            RGB8::new(40, 0, 0),
            RGB::new(0, 0, 40),
        ));
        delay(DT * 40_000_000);
        strip.shutdown_animation();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    dprintln!("\n\n*** {:?}", info);
    loop {}
}
