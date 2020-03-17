//! ROM routine definitions

/*
type type_rom_i2c_readReg = unsafe extern "C" fn(block: u8, host_id: u8, reg_add: u8) -> u8;
type type_rom_i2c_readReg_Mask =
    unsafe extern "C" fn(block: u8, host_id: u8, reg_add: u8, msb: u8, lsb: u8) -> u8;
type type_rom_i2c_writeReg =
    *const unsafe extern "C" fn(block: u8, host_id: u8, reg_add: u8, data: u8);
type type_rom_i2c_writeReg_mask =
    unsafe extern "C" fn(block: u8, host_id: u8, reg_add: u8, msb: u8, lsb: u8, data: u8);

const ptr_rom_i2c_readReg: *const type_rom_i2c_readReg = 0x40004148 as *const type_rom_i2c_readReg;
const ptr_rom_i2c_readReg_Mask: *const type_rom_i2c_readReg_Mask =
    0x400041c0 as *const type_rom_i2c_readReg_Mask;
const ptr_rom_i2c_writeReg_Mask: *const type_rom_i2c_writeReg_mask =
    0x400041fc as *const type_rom_i2c_writeReg_mask;

pub(crate) fn rom_i2c_readReg(block: u8, host_id: u8, reg_add: u8) -> u8 {
    unsafe { (*ptr_rom_i2c_readReg)(block, host_id, reg_add) }
}
*/
/*
type type_rom_i2c_writeReg = unsafe extern "C" fn(block: u8, host_id: u8, reg_add: u8, data: u8);

const ptr_rom_i2c_writeReg: *const type_rom_i2c_writeReg =
    0x400041a4 as *const type_rom_i2c_writeReg;

pub(crate) fn rom_i2c_writeReg(block: u8, host_id: u8, reg_add: u8, data: u8) {
    unsafe { (*ptr_rom_i2c_writeReg)(block, host_id, reg_add, data) }
}
*/
/*
type type_uart_tx_one_char = *mut unsafe extern "C" fn(char: u8) -> u32;

pub struct ROM {
    pub testcall: type_uart_tx_one_char,
}

impl ROM {
    pub fn new() -> Self {
        ROM {
            testcall: 0x40009200 as type_uart_tx_one_char,
        }
    }
}

//  = 0x40009200 as type_uart_tx_one_char
*/
extern "C" {
    pub fn rom_i2c_writeReg(block: u8, host_id: u8, reg_add: u8, data: u8);

//   pub fn uart_tx_one_char(char: u8) -> u32;

}
