use core::fmt::Arguments;

pub mod uart;

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::io::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($args:tt)*) => ($crate::print!("{}\n", format_args!($($args)*)));
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    use core::fmt::Write;
    let mut uart = crate::io::uart::Uart;
    uart.write_fmt(args).unwrap();
}
