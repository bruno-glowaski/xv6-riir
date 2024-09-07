#![no_std]
#![no_main]

core::arch::global_asm!(include_str!("entry.S"));

mod clint;
mod panic_handler;
mod param;
pub mod start;

#[no_mangle]
pub fn main() {
    loop {}
}
