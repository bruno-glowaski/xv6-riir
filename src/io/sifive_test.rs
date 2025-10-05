use core::arch::asm;

use crate::println;

const ADDR: *mut u8 = 0x0010_0000 as *mut u8;

const EXIT_FAILURE: u32 = 0x00003333;
const EXIT_SUCCESS: u32 = 0x00005555;
const EXIT_RESET: u32 = 0x00007777;

pub fn exit_success() -> ! {
    exit(EXIT_SUCCESS)
}

pub fn exit_error(code: u16) -> ! {
    let code = (code as u32) << 16;
    exit(code | EXIT_FAILURE)
}

pub fn reset() -> ! {
    exit(EXIT_RESET)
}

fn exit(code: u32) -> ! {
    unsafe {
        asm!(
            "sw {}, 0({})",
            in(reg) code,
            in(reg) ADDR
        );
    }
    println!("FAILED TO SHUTDOWN");
    loop {
        unsafe { asm!("wfi") };
    }
}
