use core::arch::asm;

pub const CLINT_BASE: u64 = 0x0200_0000;
pub const MTIMECMP: *mut u64 = (CLINT_BASE + 0x4000) as *mut u64;
pub const MTIME: *mut u64 = (CLINT_BASE + 0xBFF8) as *mut u64;

pub fn schedule(interval_in_cycles: u64) {
    // Set `mtimecmp`
    let time = unsafe { MTIME.read_volatile() };
    unsafe {
        MTIMECMP.write_volatile(time + interval_in_cycles);
    }

    // Enable timer interrupts
    unsafe {
        asm!("csrs mie, {}", in(reg) 1 << 7);
    }
}
