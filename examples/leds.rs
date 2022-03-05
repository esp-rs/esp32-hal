#![no_std]
#![no_main]

use esp32::SPI2;
use esp32_hal::{dport::Split, prelude::*, spi, target};
use panic_halt;
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
const BOT_ROW: usize = 9;
const DT: u32 = 5;
const BREATHING_MULTIPLIER: u32 = 10;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LightData {
    leds: [RGB8; NUM_LEDS],
}
impl LightData {
    fn rgb() -> Self{
        let mut result = Self::empty();
        // Blink the LED's in a blue-green-red pattern.
        for led in result.iter_mut().step_by(3) {
            led.b = 0x10;
        }

        if NUM_LEDS > 1 {
            for led in result.iter_mut().skip(1).step_by(3) {
                led.g = 0x10;
            }
        }

        if NUM_LEDS > 2 {
            for led in result.iter_mut().skip(2).step_by(3) {
                led.r = 0x10;
            }
        }
        result
    }
    fn from_gradient(from: RGB8, to: RGB8) -> Self {
        let mut result = [RGB8::default(); NUM_LEDS];
        let r_delta = to.r as i16 - from.r as i16;
        let g_delta = to.g as i16 - from.g as i16;
        let b_delta = to.b as i16 - from.b as i16;
        for i in 0..NUM_LEDS{
            let r = (from.r + (r_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            let g = (from.g + (g_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            let b = (from.b + (b_delta * i as i16 / (NUM_LEDS - 1) as i16) as u8) as u8;
            result[i] = RGB8 {r, g, b};
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
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut RGB8> {
        self.leds.iter_mut()
    }
    fn write_to_strip(
        &self,
        strip: &mut Ws2812<SPI<SPI2, Gpio14<Unknown>, Gpio15<Output<PushPull>>, Gpio25<Unknown>>>,
    ) {
        strip.write(self.leds.iter().cloned()).unwrap();
    }
    fn is_off(&self) -> bool {
        if self.leds.iter().any(|l| l.r > 0 && l.g > 0 && l.b > 0) {
            return false;
        }
        true
    }
    fn get_led(&self, index: usize) -> RGB8 {
        self.leds[index]
    }
    fn set_color_all(&mut self, color: RGB8) {
        for i in 0..NUM_LEDS {
            self.set_color(i, color);
        }
    }
    fn set_red_all(&mut self, red: u8) {
        for i in 0..NUM_LEDS {
            self.set_red(i, red);
        }
    }
    fn set_green_all(&mut self, green: u8) {
        for i in 0..NUM_LEDS {
            self.set_green(i, green);
        }
    }
    fn set_blue_all(&mut self, blue: u8) {
        for i in 0..NUM_LEDS {
            self.set_blue(i, blue);
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
    fn set_lightness_all(&mut self, lightness: u8) {
        for led in 0..self.leds.len() {
            self.set_lightness(lightness, led);
        }
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
    fn set_lightness(&mut self, lightness: u8, led: usize) {
        self.leds[led].r = lightness;
        self.leds[led].g = lightness;
        self.leds[led].b = lightness;
    }
    /// Sets an entire row of LEDs to a single color
    ///
    /// # Arguments
    /// * `row` - The row to set. 0 is the top row, 1 is the middle row, and everything else is the bottom row.
    fn edit_row(&mut self, row: usize, color: RGB8) {
        match row {
            0 => {
                for led in 0..TOP_ROW {
                    self.leds[led] = color;
                }
            }
            1 => {
                for led in TOP_ROW..MID_ROW + TOP_ROW {
                    self.leds[led] = color;
                }
            }
            _ => {
                for led in MID_ROW + TOP_ROW..NUM_LEDS {
                    self.leds[led] = color;
                }
            }
        }
    }
    fn get_iter(&mut self) -> impl Iterator<Item = &mut RGB8> {
        self.leds.iter_mut()
    }
    fn empty() -> Self {
        Self {
            leds: [RGB8::new(0, 0, 0); NUM_LEDS],
        }
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
    fn breathe(&mut self) {
        let data_clone = self.data.clone();
        let empty: LightData = LightData::empty();
        self.fade_into(empty);
        self.fade_into(data_clone);
    }
    fn fill_with_data_animated(&mut self, data: LightData) {
        for i in 0..NUM_LEDS {
            self.data.set_color(i, data.get_led(i));
            delay(5_000_000);
            self.write();
        }
        self.get_brightness();
    }

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
    fn off(&mut self) {
        self.data.set_color_all(RGB8::new(0, 0, 0));
        self.write();
    }
    fn set_color(&mut self, color: RGB8, index: usize) {
        self.data.set_color(index, color);
        self.write();
    }
    fn set_solid(&mut self, color: RGB8) {
        self.data.set_color_all(color);
        self.write();
    }
    fn set_lightness(&mut self, percentage: f32) {
        let value = (percentage * 255.0) as u8;
        self.brightness = value;
        for led in self.data.iter_mut() {
            if led.r > 0 {
                led.r = value;
            }
            if led.g > 0 {
                led.g = value;
            }
            if led.b > 0 {
                led.b = value;
            }
        }
        if self.is_off() {
            self.data.get_iter().for_each(|led| {
                led.r = value;
                led.g = value;
                led.b = value;
            })
        }
        self.write();
    }
    fn red_and_white(&mut self) {
        for i in 0..TOP_ROW + MID_ROW - 5 {
            self.set_color(RGB8::new(self.brightness, 0, 0), i);
        }
        for i in TOP_ROW + MID_ROW - 5..NUM_LEDS {
            self.set_color(
                RGB8::new(self.brightness, self.brightness, self.brightness),
                i,
            );
        }
        self.write();
    }
    fn white(&mut self) {
        self.set_solid(RGB8::new(self.brightness, self.brightness, self.brightness));
        self.write();
    }

    fn is_off(&self) -> bool {
        self.data.is_off()
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
        ws: ws,
        data: LightData::from_gradient(RGB8::new(40, 0, 0), RGB::new(0, 0, 40)),
        brightness: 10,
    };
    loop {
        strip.write();
        delay(DT * 40_000_000);
    }
}
