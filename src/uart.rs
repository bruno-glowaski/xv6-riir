use bitflags::bitflags;

use crate::spinlock::Mutex;

const UART_BASE_ADDR: usize = 0x10000000;

// Taken from: http://byterunner.com/16550.html
#[repr(usize)]
enum UARTRegister {
    RxTxOrDivisorLatchLSBHolding = 0b000,
    InterruptEnableOrDivisorLatchMSB = 0b001,
    InterruptStatusOrFIFOControl = 0b010,
    LineControl = 0b011,
    LineStatus = 0b101,
}

bitflags! {
  struct InterruptEnableFlags: u8 {
    const Receive = 1 << 0;
    const Transmit = 1 << 1;
  }

  struct FIFOControlFlags: u8 {
    const FIFOEnable = 1 << 0;
    const ReceiverFIFOReset = 1 << 1;
    const TransmitFIFOReset = 1 << 2;
    const FIFOReset = 3 << 1;
  }

  struct LineControlFlags: u8 {
    const EightBits = 3 << 0;
    const BaudLatch = 3 << 7;
  }
}

struct UART {
    base_ptr: *mut u8,
}

impl UART {
    pub const unsafe fn from_base_addr(addr: usize) -> Self {
        Self {
            base_ptr: addr as *mut u8,
        }
    }

    pub fn init(&mut self) {
        self.write_ier(InterruptEnableFlags::empty());
        self.set_baud_rate(0x0003); // Set baud rate to 38.4K
        self.write_lcr(LineControlFlags::EightBits);
        self.write_fcr(FIFOControlFlags::FIFOEnable | FIFOControlFlags::FIFOReset);
        self.write_ier(InterruptEnableFlags::Receive | InterruptEnableFlags::Transmit);
    }

    pub fn put(&mut self, c: u8) {}

    #[inline]
    fn write_register(&mut self, register: UARTRegister, value: u8) {
        unsafe {
            self.base_ptr
                .offset(register as isize)
                .write_volatile(value)
        }
    }

    #[inline]
    fn read_register(&mut self, register: UARTRegister) -> u8 {
        unsafe { self.base_ptr.offset(register as isize).read_volatile() }
    }

    #[inline]
    fn write_ier(&mut self, flags: InterruptEnableFlags) {
        self.write_register(UARTRegister::InterruptEnableOrDivisorLatchMSB, flags.bits());
    }

    #[inline]
    fn set_baud_rate(&mut self, rate: u16) {
        self.write_lcr(LineControlFlags::BaudLatch);
        self.write_register(
            UARTRegister::RxTxOrDivisorLatchLSBHolding,
            (rate & 0xFF) as u8,
        );
        self.write_register(
            UARTRegister::InterruptEnableOrDivisorLatchMSB,
            (rate >> 8) as u8,
        );
    }

    #[inline]
    fn write_lcr(&mut self, flags: LineControlFlags) {
        self.write_register(UARTRegister::LineControl, flags.bits());
    }

    #[inline]
    fn write_fcr(&mut self, flags: FIFOControlFlags) {
        self.write_register(UARTRegister::InterruptStatusOrFIFOControl, flags.bits())
    }
}

pub(crate) static UART_LOCK: Mutex = Mutex::new("uart");

pub(crate) fn init() {
    let mut uart = unsafe { UART::from_base_addr(UART_BASE_ADDR) };
    uart.init();
}
