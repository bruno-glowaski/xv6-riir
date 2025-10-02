use core::arch::asm;

pub fn setup(trapvec: unsafe extern "C" fn()) {
    // Define trap handler
    unsafe {
        asm!(
            "csrw mtvec, {}",
            in(reg) trapvec as u64 & 0xFFFFFFFC,
        );
    }
}

pub fn enable() {
    unsafe {
        asm!(
            "csrs mstatus, {}",
             in(reg) 1 << 3,
        )
    }
}

pub fn disable() {
    unsafe {
        asm!(
            "csrc mstatus, {}",
             in(reg) 1 << 3,
        )
    }
}
