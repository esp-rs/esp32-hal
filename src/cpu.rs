use esp32::{DPORT, RTCCNTL};

// TODO: avoid using ROM functions
extern "C" {
    fn Cache_Flush_rom(cpu_no: u32);
    fn Cache_Read_Enable_rom(cpu_no: u32);
    fn ets_set_appcpu_boot_addr(f: extern "C" fn() -> !);
}

unsafe fn enable_cache(cpu_no: u32) {
    Cache_Flush_rom(cpu_no);
    Cache_Read_Enable_rom(cpu_no);
}

pub unsafe fn start_app_cpu(dport: &mut DPORT, rtccntl: &mut RTCCNTL, f: extern "C" fn() -> !) {
    // See IDF cpu_start.c.

    // The original stalls the other CPU while doing this, but here I assume the other CPU is
    // already stalled.
    enable_cache(1);

    rtccntl
        .sw_cpu_stall
        .modify(|_, w| w.sw_stall_appcpu_c1().bits(0));
    rtccntl
        .options0
        .modify(|_, w| w.sw_stall_appcpu_c0().bits(0));

    if !dport.appcpu_ctrl_b.read().appcpu_clkgate_en().bit() {
        dport
            .appcpu_ctrl_b
            .modify(|_, w| w.appcpu_clkgate_en().set_bit());
        dport
            .appcpu_ctrl_c
            .modify(|_, w| w.appcpu_runstall().clear_bit());
        dport
            .appcpu_ctrl_a
            .modify(|_, w| w.appcpu_resetting().set_bit());
        dport
            .appcpu_ctrl_a
            .modify(|_, w| w.appcpu_resetting().clear_bit());
    }

    ets_set_appcpu_boot_addr(f);
}
