mod context;
mod data;
mod manager;
mod paging;
mod pid;
mod process;
mod processor;

use manager::*;
use process::*;
use crate::memory::PAGE_SIZE;

use alloc::string::String;
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x0000_4000_0000_0000;

pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * PAGE_SIZE;
pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_PAGE: u64 = 512; // 此处可调用boot.conf
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * PAGE_SIZE;
pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init() {
    let mut kproc_data = ProcessData::new();

    // FIXME: set the kernel stack
    // 假设堆栈的大小为 KSTACK_DEF_SIZE，开始地址为 KSTACK_INIT_BOT
    kproc_data.set_stack(VirtAddr::new(KSTACK_INIT_BOT), KSTACK_DEF_SIZE);
    trace!("Init process data: {:#?}", kproc_data);

    // kernel process
    let kproc = {
        /* FIXME: create kernel process */ 
       // 获取当前页表
       let page_table = PageTableContext::new();
       // 创建内核进程
       Process::new(String::from("kernel"), None, page_table, Some(kproc_data))
       };
   kproc.write().resume();
   // 初始化进程管理器并将内核进程设置为当前运行的进程
   manager::init(kproc);
   // manager::init(kproc.clone());
   
   info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        let process_manager = get_process_manager();

        // 保存当前进程的上下文
        process_manager.save_current(context);

        // 切换到下一个进程，并更新上下文
        let next_pid = process_manager.switch_next(context);

        // 更新处理器状态，比如当前进程ID
        processor::set_pid(next_pid);
    });
}

pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let entry = VirtAddr::new(entry as usize as u64);
        get_process_manager().spawn_kernel_thread(entry, name, data)
    })
}

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}
