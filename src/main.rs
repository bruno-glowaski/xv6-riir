#![no_std]
#![no_main]
#![feature(int_format_into)]

mod io;
mod irq;
mod proc;
mod setup;
mod timer;
mod utils;

use core::{arch::naked_asm, panic::PanicInfo};

unsafe extern "C" {
    static mut __stack_size: u8;
    static mut __stack_start: u8;
    static mut __stack_end: u8;
}

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

#[cfg(miri)]
#[unsafe(no_mangle)]
fn miri_start(argc: isize, argv: *const *const u8) -> isize {
    unsafe {
        start();
    }
    return 0;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() -> ! {
    println!("rxv6 start");
    println!(
        "stack: [{:?}-{:?}]({})",
        &raw const __stack_start, &raw const __stack_end, &raw const __stack_size as usize
    );
    println!("Creating process 1...");
    proc::create_process(process1);
    println!("Creating process 2...");
    proc::create_process(process2);
    println!("Starting scheduler...");
    proc::run_scheduler();
}

pub fn process1() {
    loop {
        println!("I'm process 1!");
        proc::yield_self();
    }
}

pub fn process2() {
    loop {
        println!("I'm process 2!");
        proc::yield_self();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn on_timer_int() {
    timer::schedule(TIMER_INTERVAL);
    let mut uart = io::uart::Uart;
    uart.write_str("Hello\n");
}
