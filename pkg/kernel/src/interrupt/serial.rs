// use alloc::vec::Vec;
// use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
// use super::consts::*;
// use crate::drivers::input::{push_key, try_pop_key}; 
// //use crate::drivers::uart16550::SerialPort;
// use crate::drivers::serial::get_serial_for_sure;
// // use crate::drivers::input::push_key;
// use pc_keyboard::{DecodedKey, KeyCode};
// use core::str;

// pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
//     idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
//         .set_handler_fn(serial_handler);
// }

// pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
//     receive();
//     super::ack();
// }

// /// Receive character from uart 16550
// /// Should be called on every interrupt
// pub fn receive() {
//     let mut buf = Vec::with_capacity(4); // UTF-8字符最多4字节
//     while let Some(byte) = get_serial_for_sure().receive() {
//         match byte {
//             127 => push_key(b'\x08'), // 退格
//             13 => push_key(b'\n'),    // 换行
//             _ => {
//                 if let Some(key) = get_serial_for_sure().receive() {
//                     let decoded_key = DecodedKey::Unicode(key as char); // 假设转换逻辑
//                     match decoded_key {
//                         DecodedKey::Unicode(c) => {
//                             if let Some(byte) = c.encode_utf8(&mut [0; 4]).as_bytes().get(0) {
//                                 push_key(*byte);
//                             }
//                         },
//                         _ => {} // 处理其他DecodedKey情况
//                     }
//                 }
//             }
//         }
//     }
// }

use alloc::vec::{self, Vec};
use pc_keyboard::DecodedKey;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::drivers::input::push_key; 
use crate::drivers::serial::get_serial_for_sure;

use super::consts::{Interrupts, Irq};

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    receive();
    super::ack();
}

/// Receive character from uart 16550
/// Should be called on every interrupt
pub fn receive() {
    let mut buf = vec::Vec::with_capacity(4);
    while let Some(scancode) = get_serial_for_sure().receive() {
        match scancode {
            127 => push_key(DecodedKey::Unicode('\x08')),
            13 => push_key(DecodedKey::Unicode('\n')),
            c => {
                buf.push(c);

                if let Ok(s) = core::str::from_utf8(&buf) {
                    let chr = s.chars().next().unwrap();
                    push_key(DecodedKey::Unicode(chr));
                    buf.clear();
                }
            }
        }
    }
}