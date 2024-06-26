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
    ysos::wait(spawn_init());
    ysos::shutdown(boot_info);
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print_serial!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    info!("before res");
    proc::spawn("hello").unwrap()
}
