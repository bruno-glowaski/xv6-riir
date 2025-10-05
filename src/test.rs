use core::{any::type_name, arch::naked_asm, panic::PanicInfo};

use crate::{io, print, println, test_main};

unsafe extern "C" {
    static mut __stack_size: u8;
    static mut __stack_start: u8;
    static mut __stack_end: u8;
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _entry() -> ! {
    naked_asm!("la sp, __stack_end", "call start", "1: j 1b")
}

#[unsafe(no_mangle)]
pub extern "C" fn start() -> ! {
    test_main();
    io::sifive_test::exit_success();
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!(" [failed]\n{}", info);
    io::sifive_test::exit_error(1);
}

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests:", tests.len());
    for (i, test) in tests.iter().enumerate() {
        print!("\t- {}/{}: {}...", i + 1, tests.len(), test.test_name());
        test.run_test();
        println!(" [ok]");
    }
    println!("all tests passed.")
}

pub trait Testable {
    fn test_name(&self) -> &'static str;
    fn run_test(&self);
}

impl<T: Fn()> Testable for T {
    fn test_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn run_test(&self) {
        self()
    }
}

#[test_case]
pub fn ok_test() {}
