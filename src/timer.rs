use core::arch::asm;

pub const CLINT_BASE: u64 = 0x0200_0000;
pub const MTIMECMP: *mut u64 = (CLINT_BASE + 0x4000) as *mut u64;
pub const MTIME: *mut u64 = (CLINT_BASE + 0xBFF8) as *mut u64;

pub fn current_time() -> u64 {
    unsafe { MTIME.read_volatile() }
}

pub fn schedule(interval_in_cycles: u64) {
    // Set `mtimecmp`
    let time = current_time();
    let next_time = time + interval_in_cycles;
    unsafe {
        MTIMECMP.write_volatile(next_time);
    }

    // Enable timer interrupts
    unsafe {
        asm!("csrs mie, {}", in(reg) 1 << 7);
    }
}
