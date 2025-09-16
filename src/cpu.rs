use core::{arch::asm, array, cell::UnsafeCell, num::NonZeroUsize};

use arr_macro::arr;
use array_macro::array;

use crate::param::MAX_CPU;

#[derive(Clone, Copy)]
pub struct PushOffState {
    nesting_level: NonZeroUsize,
    interrupts_enabled_before: bool,
}

/// Per CPU state
pub struct CPU {
    push_off_state: UnsafeCell<Option<PushOffState>>,
}

unsafe impl Sync for CPU {}

impl CPU {
    pub const fn default() -> Self {
        Self {
            push_off_state: UnsafeCell::new(None),
        }
    }

    pub fn my() -> &'static Self {
        &CPUS[cpuid()]
    }

    unsafe fn push_off(&self) {}
}

pub fn cpuid() -> usize {
    let id: usize;
    unsafe { asm!("mv {}, tp", out(reg) id) };
    id
}

pub static CPUS: [CPU; MAX_CPU] = array![_ => CPU::default(); MAX_CPU];
