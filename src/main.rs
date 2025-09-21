#![no_std]
#![no_main]
#![feature(int_format_into)]

mod io;

use core::{arch::naked_asm, fmt::Write, panic::PanicInfo};

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn _entry() -> ! {
    naked_asm!("la sp, __stack_end", "call start", "1: j 1b",)
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    let mut uart = io::uart::Uart;
    let location = info.location().unwrap();
    _ = uart.write_fmt(format_args!(
        "kernel panic @{}:{}:{}: {}",
        location.file(),
        location.line(),
        location.column(),
        info.message().as_str().unwrap()
    ));
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() -> ! {
    loop {}
}
