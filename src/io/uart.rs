pub struct Uart;

const UART_BASE: usize = 0x1000_0000;
const UART_THR: *mut u8 = UART_BASE as *mut u8;

impl Uart {
    pub fn write_char(&mut self, c: u8) {
        unsafe {
            core::ptr::write_volatile(UART_THR, c);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_char(b);
        }
    }
}

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        return Ok(());
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        self.write_char(c as u8);
        return Ok(());
    }
}
