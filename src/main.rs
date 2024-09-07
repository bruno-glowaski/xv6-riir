#![no_std]
#![no_main]

core::arch::global_asm!(include_str!("asm/entry.S"));
core::arch::global_asm!(include_str!("asm/timervec.S"));

mod clint;
mod panic_handler;
mod param;
pub mod start;

#[no_mangle]
pub fn main() {
    loop {}
}
