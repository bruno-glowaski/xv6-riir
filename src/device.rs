use crate::param::MAX_DEVICES;
use core::{cell::UnsafeCell, mem::MaybeUninit, ops::Index};

pub trait Device: Sync {
    fn read(&self, dst: &mut [u8]) -> i32 {
        0
    }
    fn write(&self, src: &[u8]) -> i32 {
        0
    }
}

pub struct DeviceList {
    inner_list: UnsafeCell<[MaybeUninit<&'static dyn Device>; MAX_DEVICES]>,
}

unsafe impl Sync for DeviceList {}

impl DeviceList {
    pub const fn new() -> Self {
        Self {
            inner_list: UnsafeCell::new([MaybeUninit::uninit(); MAX_DEVICES]),
        }
    }

    pub unsafe fn register_static_device_at<T: Device>(&self, index: usize, device: &'static T) {
        self.inner_list.get().as_mut().unwrap_unchecked()[index] = MaybeUninit::new(device);
    }
}

impl Index<usize> for DeviceList {
    type Output = &'static dyn Device;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.inner_list.get().as_ref().unwrap_unchecked()[index].assume_init_ref() }
    }
}

pub(crate) static DEVICES: DeviceList = DeviceList::new();
