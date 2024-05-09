pub mod context;
mod data;
pub mod manager;
use crate::resource::Resource;
mod paging;
mod pid;
mod process;
mod processor;

use crate::memory::PAGE_SIZE;
use alloc::sync::Arc;
use alloc::vec::Vec;
use manager::*;
use process::*;

use alloc::string::{String, ToString};
pub use context::ProcessContext;
pub use data::ProcessData;
pub use paging::PageTableContext;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;
use xmas_elf::ElfFile;

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
pub fn init(boot_info: &'static boot::BootInfo) {
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
    // manager::init(kproc.clone());

    info!("Process Manager Initialized.");
    let app_list = boot_info.loaded_apps.as_ref().unwrap();
    manager::init(kproc, app_list);
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        let process_manager = get_process_manager();

        // 保存当前进程的上下文
        let pid: ProcessId = process_manager.save_current(context);

        // lab4 添加到队尾
        process_manager.push_ready(pid);

        // 切换到下一个进程，并更新上下文
        let next_pid = process_manager.switch_next(context);
       info!("current process id:{}\n",next_pid.0);
    //     // 更新处理器 状态，比如当前进程ID
    //     processor::set_pid(next_pid);
        process_manager.switch_next(context);
    });
}

// pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let entry = VirtAddr::new(entry as usize as u64);
//         get_process_manager().spawn_kernel_thread(entry, name, data)
//     })
// }

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // 确保执行代码时中断禁用
        // FIXME: get current process's environment variable
        get_process_manager().current().read().env(key)
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
pub fn info_cur_proc() {
    debug!("{:#?}", get_process_manager().current());
}

pub fn list_app() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list();
        if app_list.is_none() {
            println!("[!] No app found in list!");
            return;
        }

        // let apps = app_list
        //     .unwrap()
        //     .iter()
        //     .map(|app| app.name.as_str())
        //     .collect::<Vec<&str>>()
        //     .join(", ");
        let apps = app_list.unwrap();
        // TODO: print more information like size, entry point, etc.
        for app in apps {
            let name = &app.name;
            let size = app.elf.input.len(); // 假设 `input` 字段包含了程序的二进制数据
            let entry_point = app.elf.header.pt2.entry_point();

            println!(
                "[+] App list: {:<10} Size: {:<10} Entry Point: {:#x}",
                name, size, entry_point
            );
        }
    });
}

pub fn spawn(name: &str) -> Option<ProcessId> {
    // info!("enter spawn");
    let app = x86_64::instructions::interrupts::without_interrupts(|| {
        // info!("1");
        let app_list = get_process_manager().app_list()?;
        // info!("2");
        // info!("{}", name);
        app_list.iter().find(|&app| app.name.eq(name))
    })?;
    // info!("3");
    elf_spawn(name.to_string(), &app.elf)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // info!("1"); Y
        let process_name = name.to_lowercase();
        // info!("2"); Y       
        let parent = Arc::downgrade(&manager.current());
        // info!("3"); Y       
        let pid = manager.spawn(elf, name, Some(parent), None);
        info!("4");
        debug!("Spawned process: {}#{}", process_name, pid);
        info!("elf_spawn end!");
        pid
    });

    Some(pid)
}

pub fn read(fd: u8, buf: &mut [u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().read(fd, buf))
}

pub fn write(fd: u8, buf: &[u8]) -> isize {
    x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().write(fd, buf))
}

pub fn exit(ret: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: implement this for ProcessManager
        manager.kill_self(ret);
        manager.switch_next(context);
    })
}

#[inline]
pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // check if the process is still alive
        get_process_manager().still_alive(pid)
    })
}