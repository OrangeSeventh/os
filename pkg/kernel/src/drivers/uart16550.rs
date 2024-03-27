use core::fmt;
extern crate x86_64;
use x86_64::instructions::port::Port;
use bitflags::bitflags;

pub const PORT: u16 = 0x3f8;

bitflags!{
    pub struct LineStatus: u8 {
        const DATA_BITS_5 = 0b0000_0000;
        const DATA_BITS_6 = 0b0000_0001;
        const DATA_BITS_7 = 0b0000_0010;
        const DATA_BITS_8 = 0b0000_0011;
        const DLAB = 0b1000_0000;
    }
}


/// A port-mapped UART 16550 serial interface.
pub struct SerialPort {
    data_port: Port<u8>,
    interrupt_enable_port: Port<u8>,
    fifo_control_port: Port<u8>,
    line_control_port: Port<u8>,
    modem_control_port: Port<u8>,
    line_status_port: Port<u8>,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            data_port: Port::new(port),
            interrupt_enable_port: Port::new(port + 1),
            fifo_control_port: Port::new(port + 2),
            line_control_port: Port::new(port + 3),
            modem_control_port: Port::new(port + 4),
            line_status_port: Port::new(port + 5),
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) {
        unsafe {
            self.interrupt_enable_port.write(0x00); // Disable all interrupts
            self.line_control_port.write(LineStatus::DLAB.bits());     // Enable DLAB (set baud rate divisor)
            self.data_port.write(0x03);             // Set divisor to 3 (lo byte) 38400 baud
            self.interrupt_enable_port.write(0x00); //                 (hi byte)
            self.line_control_port.write(LineStatus::DATA_BITS_8.bits());     // 8 bits, no parity, one stop bit
            self.fifo_control_port.write(0xC7);     // Enable FIFO, clear them, with 14-byte threshold
            self.modem_control_port.write(0x0B);    // IRQs enabled, RTS/DSR set
            self.modem_control_port.write(0x1E);    // Set in loopback mode, test the serial chip
            self.data_port.write(0xAE);             // Test serial chip (send byte 0xAE and check if serial returns same byte)

            // Check if serial is faulty (i.e., not same byte as sent)
            if self.data_port.read() != 0xAE {
                panic!("Serial port is faulty"); // Use panic to handle the error simply
            }

            // If serial is not faulty, set it in normal operation mode
            self.modem_control_port.write(0x0F);
            // (lab2) 为串口开启中断
            self.interrupt_enable_port.write(0x01);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        unsafe {
            while self.line_status_port.read() & 0x20 == 0 {}
            self.data_port.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        unsafe {
            if self.line_status_port.read() & 0x01 != 0 {
                Some(self.data_port.read())
            } else {
                None
            }
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}

