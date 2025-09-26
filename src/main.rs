#![no_std]
#![no_main]
#![feature(int_format_into)]

mod io;
mod proc;
mod setup;
mod timer;

use core::{arch::naked_asm, panic::PanicInfo, ptr::addr_of_mut};

use crate::proc::{Context, switch};

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

static mut THREAD_1_STACK: [u8; 8 * 1024] = [0; 8 * 1024];

static mut START_CONTEXT: Context = unsafe { Context::zeroed() };
static mut THREAD_1_CONTEXT: Context = unsafe { Context::zeroed() };

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start() -> ! {
    println!("rxv6 start");
    println!("finished setup");
    println!("switching");
    unsafe {
        THREAD_1_CONTEXT = Context::new(addr_of_mut!(THREAD_1_STACK[8 * 1024 - 1]), process1);
    }
    unsafe {
        switch(&raw mut START_CONTEXT, &raw mut THREAD_1_CONTEXT);
    }
    println!("I'm start!");
    loop {}
}

pub fn process1() {
    println!("I'm process 1!");
    unsafe {
        switch(&raw mut THREAD_1_CONTEXT, &raw mut START_CONTEXT);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn on_timer_int() {
    timer::schedule(TIMER_INTERVAL);
    let mut uart = io::uart::Uart;
    uart.write_str("Hello\n");
}
