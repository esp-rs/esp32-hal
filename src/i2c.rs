use crate::gpio::{InputPin, InputSignal, OutputPin, OutputSignal};
use crate::target::{i2c, DPORT, I2C0, I2C1};
use core::{ops::Deref, ptr};

const DPORT_BASE_ADDR: u32 = 0x3FF4_0000;
const AHB_BASE_ADDR: u32 = 0x6000_0000;
const FIFO_OFFSET: u32 = 0x1C;
const I2C0_OFFSET: u32 = 0x1_3000;
const I2C1_OFFSET: u32 = 0x2_7000;

const DPORT_I2C0_ADDR: u32 = DPORT_BASE_ADDR + I2C0_OFFSET;
const DPORT_I2C1_ADDR: u32 = DPORT_BASE_ADDR + I2C1_OFFSET;

const AHB_I2C0_ADDR: u32 = AHB_BASE_ADDR + I2C0_OFFSET;
const AHB_I2C1_ADDR: u32 = AHB_BASE_ADDR + I2C1_OFFSET;

// Corresponds to https://github.com/espressif/esp-idf/blob/1cb31e50943bb757966ca91ed7f4852692a5b0ed/components/hal/esp32/include/hal/i2c_ll.h#L90
const SOURCE_CLK_FREQ: u32 = 80_000_000;

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
        let mut i2c = I2C(i2c);

        // Configure SDA and SCL pins
        let (sda_out, sda_in, scl_out, scl_in) = if i2c.0.is_i2c0() {
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
                // Use Most Significant Bit first for sending and receiving data
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
        if self.0.is_i2c0() {
            dport.perip_rst_en.modify(|_, w| w.i2c0().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c0().clear_bit());
        } else {
            dport.perip_rst_en.modify(|_, w| w.i2c1().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c1().clear_bit());
        }
    }

    /// Enables the interface
    fn enable(&mut self, dport: &mut DPORT) {
        if self.0.is_i2c0() {
            dport.perip_clk_en.modify(|_, w| w.i2c0().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c0().clear_bit());
        } else {
            dport.perip_clk_en.modify(|_, w| w.i2c1().set_bit());
            dport.perip_rst_en.modify(|_, w| w.i2c1().clear_bit());
        }
    }

    /// Resets the transmit and receive FIFO buffers
    fn reset_fifo(&mut self) {
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.tx_fifo_rst().clear_bit());

        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().set_bit());
        self.0.fifo_conf.modify(|_, w| w.rx_fifo_rst().clear_bit());
    }

    /// Sets the filter with a supplied threshold in clock cycles for which a pulse must be present to pass the filter
    fn set_filter(&mut self, sda_threshold: Option<u8>, scl_threshold: Option<u8>) {
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

    /// Sets the frequency of the I2C interface by calculating and applying the associated timings
    fn set_frequency(&mut self, freq: u32) {
        // i2c_hal_set_bus_timing(&(i2c_context[i2c_num].hal), freq, 1);
        // i2c_ll_cal_bus_clk(80000000, freq, 0);
        let half_cycle = ((SOURCE_CLK_FREQ / freq) / 2) as u16;
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

    /// Gets the FIFO address given the operation type (R/W)
    fn fifo_addr(&self, operation_type: OperationType) -> u32 {
        // Errata 3.3: When written via DPORT, consecutive writes to the same address may be lost.
        // Errata 3.18: FIFO read operations are unpredictable via AHB.
        let base_addr = match (operation_type, self.0.is_i2c0()) {
            (OperationType::READ, true) => DPORT_I2C0_ADDR,
            (OperationType::READ, false) => DPORT_I2C1_ADDR,
            (OperationType::WRITE, true) => AHB_I2C0_ADDR,
            (OperationType::WRITE, false) => AHB_I2C1_ADDR,
        };

        base_addr + FIFO_OFFSET
    }

    // TODO: Enable ACK checks and return error if ACK check fails
    pub fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
        // Reset FIFO
        self.reset_fifo();

        // RSTART command
        self.0.comd0.write(|w| unsafe {
            w.command0()
                .bits(Command::Start.into())
        });

        // Load into FIFO
        unsafe {
            let fifo_addr = self.fifo_addr(OperationType::WRITE) as *mut u8;

            // Address
            ptr::write_volatile(fifo_addr, addr << 1 | OperationType::WRITE as u8);

            // Data
            for byte in bytes {
                ptr::write_volatile(fifo_addr, *byte);
            }
        }

        // WRITE command
        self.0.comd1.write(|w| unsafe {
            w.command1().bits(
                Command::Write {
                    ack_exp: Ack::ACK,
                    ack_check_en: false,
                    length: 1 + bytes.len() as u8
                }.into()
            )
        });

        // STOP command
        self.0.comd2.write(|w| unsafe {
            w.command2()
                .bits(Command::Stop.into())
        });

        // Start transmission
        self.0.ctr.modify(|_, w| w.trans_start().set_bit());

        // Busy wait for all three commands to be marked as done
        while self.0.comd0.read().command0_done().bit() != true {}
        while self.0.comd1.read().command1_done().bit() != true {}
        while self.0.comd2.read().command2_done().bit() != true {}

        Ok(())
    }

    // TODO: Enable ACK checks and return error if ACK check fails
    pub fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        // Reset FIFO
        self.reset_fifo();

        // RSTART command
        self.0.comd0.write(|w| unsafe {
            w.command0().bits(Command::Start.into())
        });

        // Load address into FIFO
        let fifo_addr = self.fifo_addr(OperationType::READ) as *mut u8;
        unsafe { ptr::write_volatile(fifo_addr, addr << 1 | OperationType::READ as u8) };

        // WRITE command
        self.0.comd1.write(|w| unsafe {
            w.command1().bits(Command::Write {
                ack_exp: Ack::ACK,
                ack_check_en: false,
                length: 1,
            }.into())
        });

        if buffer.len() > 1 {
            // READ command for first n - 1 bytes
            self.0.comd2.write(|w| unsafe {
                w.command2().bits(Command::Read {
                    ack_value: Ack::ACK,
                    length: buffer.len() as u8 - 1,
                }.into())
            });

            // READ command for final byte
            self.0.comd3.write(|w| unsafe {
                w.command3().bits(Command::Read {
                    ack_value: Ack::NACK,
                    length: 1,
                }.into())
            });

            // STOP command
            self.0.comd4.write(|w| unsafe {
                w.command4().bits(Command::Stop.into())
            });
        } else {
            // READ command for byte
            self.0.comd2.write(|w| unsafe {
                w.command2().bits(Command::Read {
                    ack_value: Ack::NACK,
                    length: 1,
                }.into())
            });

            // STOP command
            self.0.comd3.write(|w| unsafe {
                w.command3().bits(Command::Stop.into())
            });
        }

        // Start transmission
        self.0.ctr.modify(|_, w| w.trans_start().set_bit());

        // Busy wait for all commands to be marked as done
        while self.0.comd0.read().command0_done().bit() != true {}
        while self.0.comd1.read().command1_done().bit() != true {}

        if buffer.len() > 1 {
            while self.0.comd2.read().command2_done().bit() != true {}
            while self.0.comd3.read().command3_done().bit() != true {}
            while self.0.comd4.read().command4_done().bit() != true {}
        } else {
            while self.0.comd2.read().command2_done().bit() != true {}
            while self.0.comd3.read().command3_done().bit() != true {}
        }


        // Read bytes from FIFO
        for byte in buffer.iter_mut() {
            *byte = unsafe { ptr::read_volatile(fifo_addr) };
        }

        Ok(())
    }

    // TODO: Enable ACK checks and return error if ACK check fails
    pub fn write_read(
        &mut self,
        addr: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        // Reset FIFO
        self.reset_fifo();

        // START
        self.0.comd0.write(|w| unsafe {
            w.command0().bits(Command::Start.into())
        });

        // load into FIFO
        let fifo_addr = self.fifo_addr(OperationType::WRITE) as *mut u8;
        unsafe {
            // load address
            ptr::write_volatile(fifo_addr, addr << 1 | OperationType::WRITE as u8);

            // load data
            for byte in bytes {
                ptr::write_volatile(fifo_addr, *byte);
            }
        }

        // WRITE addr + data
        self.0.comd1.write(|w| unsafe {
            w.command1().bits(Command::Write {
                ack_exp: Ack::ACK,
                ack_check_en: false,
                length: 1 + bytes.len() as u8,
            }.into())
        });

        // repeat START
        self.0.comd2.write(|w| unsafe {
            w.command2().bits(Command::Start.into())
        });

        // load slave address into FIFO
        unsafe { ptr::write_volatile(fifo_addr, addr << 1 | OperationType::READ as u8) };

        // WRITE slave address
        self.0.comd3.write(|w| unsafe {
            w.command3().bits(Command::Write {
                ack_exp: Ack::ACK,
                ack_check_en: false,
                length: 1,
            }.into())
        });

        if buffer.len() > 1 {
            // READ first n - 1 bytes
            self.0.comd4.write(|w| unsafe {
                w.command4().bits(Command::Read {
                    ack_value: Ack::ACK,
                    length: buffer.len() as u8 - 1,
                }.into())
            });

            // READ last byte
            self.0.comd5.write(|w| unsafe {
                w.command5().bits(Command::Read {
                    ack_value: Ack::NACK,
                    length: 1,
                }.into())
            });

            // STOP
            self.0.comd6.write(|w| unsafe {
                w.command6().bits(Command::Stop.into())
            });
        } else {
            // READ byte
            self.0.comd4.write(|w| unsafe {
                w.command4().bits(Command::Read {
                    ack_value: Ack::NACK,
                    length: 1,
                }.into())
            });

            // STOP
            self.0.comd5.write(|w| unsafe {
                w.command5().bits(Command::Stop.into())
            });
        }

        // Start transmission
        self.0.ctr.modify(|_, w| w.trans_start().set_bit());

        // Busy wait for all commands to be marked as done
        while self.0.comd0.read().command0_done().bit() != true {}
        while self.0.comd1.read().command1_done().bit() != true {}
        while self.0.comd2.read().command2_done().bit() != true {}
        while self.0.comd3.read().command3_done().bit() != true {}
        while self.0.comd4.read().command4_done().bit() != true {}
        while self.0.comd5.read().command5_done().bit() != true {}
        if buffer.len() > 1 {
            while self.0.comd6.read().command6_done().bit() != true {}
        }

        // read bytes from FIFO
        let fifo_addr = self.fifo_addr(OperationType::READ) as *mut u8;
        for byte in buffer.iter_mut() {
            *byte = unsafe { ptr::read_volatile(fifo_addr) };
        }

        Ok(())
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
        self.write_read(addr, bytes, buffer)
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

/// I2C Command
enum Command {
    Start,
    Stop,
    Write {
        /// This bit is to set an expected ACK value for the transmitter.
        ack_exp: Ack,
        /// Enables checking the ACK value received against the ack_exp value.
        ack_check_en: bool,
        /// Length of data (in bytes) to be written. The maximum length is 255, while the minimum
        /// is 1.
        length: u8,
    },
    Read {
        /// Indicates whether the receiver will send an ACK after this byte has been received.
        ack_value: Ack,
        /// Length of data (in bytes) to be read. The maximum length is 255, while the minimum is 1.
        length: u8,
    },
}

impl From<Command> for u16 {
    fn from(c: Command) -> u16 {
        let opcode = match c {
            Command::Start => Opcode::RSTART,
            Command::Stop => Opcode::STOP,
            Command::Write { .. } => Opcode::WRITE,
            Command::Read { .. } => Opcode::READ,
        };

        let length = match c {
            Command::Start | Command::Stop => 0,
            Command::Write { length: l, .. } | Command::Read { length: l, .. } => l,
        };

        let ack_exp = match c {
            Command::Start | Command::Stop | Command::Read { .. } => Ack::NACK,
            Command::Write { ack_exp: exp, .. } => exp,
        };

        let ack_check_en = match c {
            Command::Start | Command::Stop | Command::Read { .. } => false,
            Command::Write { ack_check_en: en, .. } => en,
        };

        let ack_value = match c {
            Command::Start | Command::Stop | Command::Write { .. } => Ack::NACK,
            Command::Read { ack_value: ack, .. } => ack,
        };

        let mut cmd: u16 = length.into();

        if ack_check_en {
            cmd |= 1 << 8;
        } else {
            cmd &= !(1 << 8);
        }

        if ack_exp == Ack::NACK {
            cmd |= 1 << 9;
        } else {
            cmd &= !(1 << 9);
        }

        if ack_value == Ack::NACK {
            cmd |= 1 << 10;
        } else {
            cmd &= !(1 << 10);
        }

        cmd |= (opcode as u16) << 11;

        cmd
    }
}

enum OperationType {
    WRITE = 0,
    READ = 1,
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum Ack {
    ACK,
    NACK,
}

#[allow(dead_code)]
enum Opcode {
    RSTART = 0,
    WRITE = 1,
    READ = 2,
    STOP = 3,
    // TODO: figure out what params END needs
    END = 4,
}

pub trait Instance: Deref<Target = i2c::RegisterBlock> {
    /// Determines which interface corresponds to the current instance
    fn is_i2c0(&self) -> bool {
        self.deref() as *const i2c::RegisterBlock == I2C0::ptr()
    }
}

impl Instance for I2C0 {}

impl Instance for I2C1 {}
