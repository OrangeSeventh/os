// #![no_std]
// #![no_main]

// #[macro_use]
// extern crate log;

// use core::arch::asm;
// use ysos_kernel as ysos;

// boot::entry_point!(kernel_main);

// pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
//     ysos::init(boot_info);

//     loop {
//         info!("Hello World from YatSenOS v2!");

//         for _ in 0..0x10000000 {
//             unsafe {
//                 asm!("nop");
//             }
//         }
//     }
// }
#![no_std]
#![no_main]

use core::arch::asm;

use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);
// fn write_to_address(address: usize, value: u8) {
//       unsafe {
//         asm!(
//           "mov [{0}], {1}",
//           in(reg) address,
//           in(reg_byte) value,
//           options(nostack, nomem, preserves_flags),
//         );
//       }
// }
pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    // write_to_address(0xffffff0000000000, 42);
    // unsafe {
    //     asm!("mov eax, 0; div eax", options(nomem, nostack));
    // }
    loop {
        print!("> ");
        let input = input::get_line();

        match input.trim() {
            "exit" => break,
            _ => {
                println!("You said: {}", input);
                println!("The counter value is {}", interrupt::clock::read_counter());
            }
        }
    }

    ysos::shutdown(boot_info);
}