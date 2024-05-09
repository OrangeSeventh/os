#![no_std]
#![no_main]

use log::info;
use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    // proc::list_app();
    info!("init start");
    spawn_init();
    // ysos::wait(spawn_init());
    let mut count = 0;
    loop {
        // TODO: better way to show more than one process is running?
        count += 1;
        if count == 100000 {
            count = 0;
            break;
        }
        unsafe {
            x86_64::instructions::hlt();
        }
    }
    ysos::shutdown(boot_info);
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print_serial!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    info!("before res");
    proc::spawn("hello").unwrap()
}
