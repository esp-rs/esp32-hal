/* Specify main memory areas */
MEMORY
{
  /* Use values from the ESP-IDF 'bootloader' component.
  /* TODO: Use human-readable lengths */
  /* TODO: Use the full memory map - this is just a test */
  /* vectors ( RX )       : ORIGIN = 0x40080000, len = 0x400 */
  iram_seg ( RX )       : ORIGIN = 0x40080400, len = 0xFC00
  dram_seg ( RW )       : ORIGIN = 0x3FFF0000, len = 0x10000

}

PROVIDE ( rom_i2c_writeReg = 0x400041a4 );
PROVIDE ( uart_tx_one_char = 0x40009200 );