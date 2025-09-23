use core::arch::asm;

use crate::_trapvec;

pub fn int() {
    // Define trap handler
    unsafe {
        asm!(
            "csrw mtvec, {}",
            in(reg) _trapvec as u64 & 0xFFFFFFFC,
        );
    }

    // Enable machine-mode interrupts
    unsafe {
        asm!(
            "csrs mstatus, {}",
             in(reg) 1 << 3,
        )
    }
}
