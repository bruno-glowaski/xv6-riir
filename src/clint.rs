use riscv::register::{mhartid, mie, mscratch, mstatus, mtvec, utvec::TrapMode};

use crate::param::MAX_CPU;

pub const CLINT_BASE: usize = 0x200_0000;
pub const TIMER_INTERVAL: usize = 1000000;
pub const MTIMECMP_BASE_ADDR: usize = CLINT_BASE + 0x4000;
pub const MTIME_ADDR: usize = CLINT_BASE + 0xBFF8;

static mut TIMER_SCRATCH: [[u64; 8]; MAX_CPU] = [[0; 8]; MAX_CPU];

pub const fn get_mtimecmp_for_hart(hart_id: usize) -> usize {
    MTIMECMP_BASE_ADDR + 8 * hart_id
}

extern "C" {
    pub fn timervec();
}

pub(crate) unsafe fn init_timer() {
    let hart_id = mhartid::read();
    let interval = TIMER_INTERVAL as u64;
    let mtimecmp_ptr = get_mtimecmp_for_hart(hart_id) as *mut u64;
    let mtime_ptr = MTIME_ADDR as *mut u64;

    // Set mtimecmp to the current mtime plus the interrupt interval.
    mtimecmp_ptr.write_volatile(mtime_ptr.read_volatile() + interval);

    // Build machine mode scratch memory
    let scratch = &mut TIMER_SCRATCH[hart_id];
    scratch[3] = mtimecmp_ptr as u64;
    scratch[4] = interval;
    mscratch::write(scratch.as_ptr() as usize);

    // Define "timervec" as the machine trap handler
    mtvec::write(timervec as usize, TrapMode::Direct);

    // Enable machine-mode timer interrupts
    mstatus::set_mie();
    mie::set_mtimer();
}
