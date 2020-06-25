use {
    crate::{
        gpio::{InputPin, InputSignal, OutputPin, OutputSignal},
        target::{i2c, DPORT, I2C0, I2C1},
    },
    core::ops::Deref,
};

pub struct I2C<T>(T);

impl<T> I2C<T>
where
    T: Instance,
{
    pub fn new<SDA: OutputPin + InputPin, SCL: OutputPin + InputPin>(
        i2c: T,
        mut pins: Pins<SDA, SCL>,
        frequency: u32,
        dport: &mut DPORT,
    ) -> Self {
        let mut i2c = Self(i2c);

        // Configure SDA and SCL pins
        let (sda_out, sda_in, scl_out, scl_in) = if is_i2c0(&i2c.0) {
            (
                OutputSignal::I2CEXT0_SDA,
                InputSignal::I2CEXT0_SDA,
                OutputSignal::I2CEXT0_SCL,
                InputSignal::I2CEXT0_SCL,
            )
        } else {
            (
                OutputSignal::I2CEXT1_SDA,
                InputSignal::I2CEXT1_SDA,
                OutputSignal::I2CEXT1_SCL,
                InputSignal::I2CEXT1_SCL,
            )
        };

        pins.sda
            .set_to_open_drain_output()
            .enable_input(true)
            .internal_pull_up(true)
            .connect_peripheral_to_output(sda_out)
            .connect_input_to_peripheral(sda_in);

        pins.sda.set_output_high(true);

        pins.scl
            .set_to_open_drain_output()
            .enable_input(true)
            .internal_pull_up(true)
            .connect_peripheral_to_output(scl_out)
            .connect_input_to_peripheral(scl_in);

        // Reset and enable the I2C peripheral
        i2c.reset(dport);
        i2c.enable(dport);

        // Disable all I2C interrupts
        i2c.0.int_ena.write(|w| unsafe { w.bits(0) });
        // Clear all I2C interrupts
        i2c.0.int_clr.write(|w| unsafe { w.bits(0x3FFF) });

        i2c.0.ctr.modify(|_, w| unsafe {
            // Clear register
            w.bits(0)
                // Set I2C controller to master mode
                .ms_mode()
                .set_bit()
                // Use open drain output for SDA and SCL
                .sda_force_out()
                .set_bit()
                .scl_force_out()
                .set_bit()
                // Use Most Siginificant Bit first for sending and receiving data
                .tx_lsb_first()
                .clear_bit()
                .rx_lsb_first()
                .clear_bit()
        });

        // Set to FIFO mode
        i2c.0.fifo_conf.modify(|_, w| w.nonfifo_en().clear_bit());

        // Reset FIFO
        i2c.reset_fifo();

        // Configure filter
        i2c.set_filter(Some(7), Some(7));

        // Configure frequency
        i2c.set_frequency(frequency);

        // Enable clocks
        i2c.0.ctr.modify(|_, w| w.clk_en().set_bit());

        i2c
    }

    /// Resets the interface
    fn reset(&mut self, dport: &mut DPORT) {
        if is_i2c0(&self.0) {
            dport.perip_rst_en.modify(|_, w| w.i2c0().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c0().clear_bit());
        } else {
            dport.perip_rst_en.modify(|_, w| w.i2c1().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c1().clear_bit());
        }
    }

    /// Enables the interface
    fn enable(&mut self, dport: &mut DPORT) {
        if is_i2c0(&self.0) {
            dport.perip_clk_en.modify(|_, w| w.i2c0().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c0().clear_bit());
        } else {
            dport.perip_clk_en.modify(|_, w| w.i2c1().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c1().clear_bit());
        }
    }

    /// Resets the transmit and receive FIFO buffers
    fn reset_fifo(&mut self) {
        //i2c_ll_txfifo_rst(hal->dev);
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().clear_bit());
        //i2c_ll_rxfifo_rst(hal->dev);
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().clear_bit());
    }

    /// Sets the filter with a supplied threshold in clock cycles for which a pulse must be present to pass the filter
    fn set_filter(&mut self, sda_threshold: Option<u8>, scl_threshold: Option<u8>) {
        // i2c_hal_set_filter(&(i2c_context[i2c_num].hal), 7);

        match sda_threshold {
            Some(threshold) => {
                self.0
                    .sda_filter_cfg
                    .modify(|_, w| unsafe { w.sda_filter_thres().bits(threshold) });
                self.0
                    .sda_filter_cfg
                    .modify(|_, w| w.sda_filter_en().set_bit());
            }
            None => self
                .0
                .sda_filter_cfg
                .modify(|_, w| w.sda_filter_en().clear_bit()),
        }

        match scl_threshold {
            Some(threshold) => {
                self.0
                    .scl_filter_cfg
                    .modify(|_, w| unsafe { w.scl_filter_thres().bits(threshold) });
                self.0
                    .scl_filter_cfg
                    .modify(|_, w| w.scl_filter_en().set_bit());
            }
            None => self
                .0
                .scl_filter_cfg
                .modify(|_, w| w.scl_filter_en().clear_bit()),
        }
    }

    /// Sets the freqency of the I2C interface by calculating and applying the associated timings
    fn set_frequency(&mut self, freq: u32) {
        // i2c_hal_set_bus_timing(&(i2c_context[i2c_num].hal), freq, 1);
        // i2c_ll_cal_bus_clk(80000000, freq, 0);
        let half_cycle = ((80_000_000 / freq) / 2) as u16;
        let scl_low = half_cycle;
        let scl_high = half_cycle;
        let sda_hold = half_cycle / 2;
        let sda_sample = scl_high / 2;
        let setup = half_cycle;
        let hold = half_cycle;
        // By default we set the timeout value to 10 bus cycles
        let tout = half_cycle * 20;

        unsafe {
            // scl period
            self.0.scl_low_period.write(|w| w.period().bits(scl_low));
            self.0.scl_high_period.write(|w| w.period().bits(scl_high));

            // sda sample
            self.0.sda_hold.write(|w| w.time().bits(sda_hold));
            self.0.sda_sample.write(|w| w.time().bits(sda_sample));

            // setup
            self.0.scl_rstart_setup.write(|w| w.time().bits(setup));
            self.0.scl_stop_setup.write(|w| w.time().bits(setup));

            // hold
            self.0.scl_start_hold.write(|w| w.time().bits(hold));
            self.0.scl_stop_hold.write(|w| w.time().bits(hold));

            // timeout
            self.0.to.write(|w| w.time_out_reg().bits(tout.into()));
        }
    }

    pub fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        // TODO: Use bytes.chunk(255) to remove this limitation
        assert!(bytes.len() < 255);

        // Use the correct FIFO address for the current I2C peripheral
        let fifo_addr = if is_i2c0(&self.0) {
            0x6001301c as *mut u8
        } else {
            0x6002701c as *mut u8
        };

        // Reset FIFO
        self.reset_fifo();

        // RSTART command
        self.0.comd0.write(|w| unsafe { w.command0().bits(0) });

        // Load bytes into FIFO
        unsafe {
            // Address
            core::ptr::write_volatile(fifo_addr, addr << 1 | 0);

            // Data
            for byte in bytes {
                core::ptr::write_volatile(fifo_addr, *byte);
            }
        }

        // WRITE command
        self.0.comd1.write(|w| unsafe {
            w.command1()
                .bits(0b00_1100_0000_0000 | (1 + bytes.len() as u8) as u16)
        });

        // STOP command
        self.0
            .comd2
            .write(|w| unsafe { w.command2().bits(0b01_1000_0000_0000) });

        // Start transmission
        self.0.ctr.modify(|_, w| w.trans_start().set_bit());

        // Busy wait for all three commands to be marked as done
        while self.0.comd0.read().command0_done().bit() != true {}
        while self.0.comd1.read().command1_done().bit() != true {}
        while self.0.comd2.read().command2_done().bit() != true {}

        Ok(())
    }

    pub fn read(&mut self, _addr: u8, _bytes: &mut [u8]) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn write_then_read(
        &mut self,
        _addr: u8,
        _bytes: &[u8],
        _buffer: &mut [u8],
    ) -> Result<(), Error> {
        unimplemented!()
    }

    /// Return the raw interface to the underlying I2C peripheral
    pub fn free(self) -> T {
        self.0
    }
}

/// Implementation of embedded_hal::blocking::i2c Traits

impl<T> embedded_hal::blocking::i2c::Write for I2C<T>
where
    T: Instance,
{
    type Error = Error;

    fn write<'w>(&mut self, addr: u8, bytes: &'w [u8]) -> Result<(), Error> {
        self.write(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::Read for I2C<T>
where
    T: Instance,
{
    type Error = Error;

    fn read<'w>(&mut self, addr: u8, bytes: &'w mut [u8]) -> Result<(), Error> {
        self.read(addr, bytes)
    }
}

impl<T> embedded_hal::blocking::i2c::WriteRead for I2C<T>
where
    T: Instance,
{
    type Error = Error;

    fn write_read<'w>(
        &mut self,
        addr: u8,
        bytes: &'w [u8],
        buffer: &'w mut [u8],
    ) -> Result<(), Error> {
        self.write_then_read(addr, bytes, buffer)
    }
}

/// Pins used by the I2C interface
///
/// Note that any two pins may be used
/// TODO: enforce this in the type system
pub struct Pins<SDA: OutputPin + InputPin, SCL: OutputPin + InputPin> {
    pub sda: SDA,
    pub scl: SCL,
}

#[derive(Debug)]
pub enum Error {
    Transmit,
    Receive,
}

/// Helper function for determining which interface corresponds to the current instance
fn is_i2c0<T: Instance>(t: &T) -> bool {
    (t.deref() as *const i2c::RegisterBlock) as u32 == 0x3ff53000
}

pub trait Instance: Deref<Target = i2c::RegisterBlock> {}

impl Instance for I2C0 {}

impl Instance for I2C1 {}
