use core::arch::asm;

use riscv::register::{mepc, mhartid, mstatus, pmpaddr0, pmpcfg0, satp};

use crate::{clint, main};

#[no_mangle]
pub unsafe extern "C" fn start() {
    // Set "previous" mode to Supervisor
    mstatus::set_mpp(mstatus::MPP::Supervisor);

    // Set "previous" program counter to the main function
    mepc::write(main as usize);

    // Disable paging
    satp::write(0);

    // Delegate all traps to Supervisor mode
    asm!(
    "li {0}, 0xffff",
    "csrw mideleg, {0}",
    "csrw medeleg, {0}",
     out(reg) _
    );
    mstatus::set_sie();

    // Enable access to all pages
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);

    // This should only be called here
    clint::init_timer();

    // Store hart/core ID on TP register
    asm!("mv tp, {}", in(reg) mhartid::read());

    // Enter Supervisor mode
    asm!("mret")
}
