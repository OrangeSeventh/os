use boot::*;
use x86_64::instructions::interrupts;
use spin::Mutex;

lazy_static! {
    pub static ref UEFI_RUNTIME: Mutex<Option<UefiRuntime>> = Mutex::new(None);
}

pub fn init(boot_info: &'static BootInfo){
    unsafe{
        *UEFI_RUNTIME.lock() = Some(UefiRuntime::new(boot_info));
    }
}

pub struct UefiRuntime {
    runtime_service: &'static RuntimeServices,
}

impl UefiRuntime {
    pub unsafe fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            runtime_service: boot_info.system_table.runtime_services(),
        }
    }

    pub fn get_time(&self) -> Time {
        interrupts::without_interrupts(|| {
            self.runtime_service.get_time().unwrap()
        })
    }
}