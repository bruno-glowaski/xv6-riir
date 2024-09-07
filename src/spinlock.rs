use core::sync::atomic::AtomicBool;

use crate::cpu::CPU;

pub struct Mutex {
    locked: AtomicBool,

    name: &'static str,
    cpu: Option<&'static CPU>,
}

impl Mutex {
    pub(crate) const fn new(name: &'static str) -> Self {
        Self {
            locked: AtomicBool::new(false),
            cpu: None,
            name,
        }
    }
}
