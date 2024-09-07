#![no_std]
#![no_main]

core::arch::global_asm!(include_str!("entry.S"));

mod panic_handler;
mod param;

#[no_mangle]
fn main() {
    loop {}
}
