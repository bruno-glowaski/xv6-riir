use crate::{
    device::{Device, DEVICES},
    spinlock::Mutex,
    uart,
};

static CONSOLE_LOCK: Mutex = Mutex::new("console");

struct Console;

impl Device for Console {
    fn read(&self, dst: &mut [u8]) -> i32 {
        0
    }

    fn write(&self, src: &[u8]) -> i32 {
        0
    }
}

pub fn init() {
    uart::init();
    unsafe { DEVICES.register_static_device_at(0, &Console) };
}
