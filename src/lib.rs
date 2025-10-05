#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "_test_main"]

pub mod io;
pub mod irq;
pub mod proc;
pub mod timer;
pub mod utils;

#[cfg(test)]
mod test;

#[inline]
#[cfg(test)]
pub fn test_main() {
    _test_main();
}
