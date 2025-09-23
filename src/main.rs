#![no_std]
#![no_main]
#![feature(int_format_into)]

mod io;
mod setup;
mod timer;

use core::{arch::naked_asm, panic::PanicInfo};

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn _entry() -> ! {
    naked_asm!("la sp, __stack_end", "call start", "1: j 1b")
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _trapvec() -> ! {
    naked_asm!("call on_timer_int", "mret")
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

const TIMER_INTERVAL: u64 = 10_000_000;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() -> ! {
    setup::int();
    timer::schedule(TIMER_INTERVAL);
    loop {}
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn on_timer_int() {
    timer::schedule(TIMER_INTERVAL);
    let mut uart = io::uart::Uart;
    uart.write_str("Hello\n");
}
