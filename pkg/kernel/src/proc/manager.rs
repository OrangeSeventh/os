use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{collections::*, format, sync::*};
use spin::{Mutex, RwLock};

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, app_list: boot::AppListRef) {
    // FIXME: set init process as Running
    processor::set_pid(init.pid());
    // FIXME: set processor's current pid to init's pid

    // 在初始化时加载app_list
    let mut manager = ProcessManager::new(init);
    manager.set_app_list(app_list);

    PROCESS_MANAGER.call_once(|| manager);
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: Option<boot::AppListRef>,
}

impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list: None,
        }
    }

    // 设置app_list的方法
    pub fn set_app_list(&mut self, app_list: boot::AppListRef) {
        self.app_list = Some(app_list);
    }
    // 提供外部获取应用列表
    pub fn app_list(&self) -> Option<&boot::AppList> {
        self.app_list.as_ref().map(|app_list| &**app_list)
    }

    pub fn wait_pid(&self, pid:ProcessId) -> Option<isize>{
        if let Some(exit_code) = self.get_exit_code(pid){
            return Some(exit_code);
        }
        None
    }
    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) -> ProcessId {
        // FIXME: update current process's tick count
        let current = self.current();
        let current_pid = current.pid();
        if let Some(process) = self.get_proc(&current_pid) {
            let mut process_inner = process.write();
            // FIXME: update current process's context
            process_inner.tick();
            // 保存当前进程的上下文
            process_inner.save(context);
            // FIXME: push current process to ready queue if still alive
        }
        current_pid

        // lab4
        // let current = self.current();
        // let pid = current.pid();
        // let mut current = current.write();
        // if pid.0 == 2 {
        // // info!("Saving process {} {:?}", pid,context);
        // }
        // current.tick();
        // current.save(context);

        // self.push_ready(pid);

        // pid
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {
        let mut pid = processor::get_pid();
        // FIXME: fetch the next process from ready queue
        // let mut ready_queue = self.ready_queue.lock();
        // info!("queue{:?}",ready_queue);
        // FIXME: check if the next process is ready,

        //        continue to fetch if not ready

        // FIXME: restore next process's context

        // FIXME: update processor's current pid

        // FIXME: return next process's pid
        // while let Some(next_pid) = ready_queue.pop_front() {
        //     if next_pid == pid {
        //         continue;  // Skip if the next process is the current one
        //     }
        //     if let Some(next_process) = self.get_proc(&next_pid) {
        //         let mut next_process_inner = next_process.write();
        //         if next_process_inner.status() == ProgramStatus::Ready {
        //             next_process_inner.restore(context);
        //             processor::set_pid(next_pid);
        //             // info!("Switched to process {}\n", next_pid.0);
        //             return next_pid;
        //         }

        //     }

        // }
        //panic!("No ready process found to switch to");

        // self.print_process_list();

        while let Some(next) = self.ready_queue.lock().pop_front() {
            let map = self.processes.read();
            let proc = map.get(&next).expect("Process not found");

            if !proc.read().is_ready() {
                debug!("Process #{} is {:?}", next, proc.read().status());
                continue;
            }

            if pid != next {
                proc.write().restore(context);
                processor::set_pid(next);
                pid = next;
            }
            // if pid.0 == 2 {
            // info!("Current process: {:#?} Current context{:#?}", pid,context);
            // }
            break;
        }

        pid // Return the current PID if no suitable next process is found
    }

    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let mut proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);
    //     // alloc stack for the new process base on pid
    //     let stack_top = Arc::get_mut(&mut proc).unwrap().alloc_init_stack();
    //     // FIXME: set the stack frame
    //     proc.write().init_stack_frame(entry, stack_top);
    //     // 设置栈的范围
    //     proc.write().set_stack(stack_top-STACK_DEF_SIZE, STACK_DEF_SIZE); //
    //     let new_pid = proc.pid();
    //     // FIXME: add to process map
    //     self.add_proc(new_pid, proc);
    //     // FIXME: push to ready queue
    //     self.push_ready(new_pid);
    //     // FIXME: return new process pid
    //     new_pid
    // }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault

        let current_pid = processor::get_pid();
        let current_process = self.get_proc(&current_pid); //

        if current_process.is_none() {
            warn!("No current process for handling page fault");
            return false;
        }

        let process = current_process.unwrap();
        let mut process_inner = process.write();

        // 检查是否为越权访问错误
        if err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
            warn!(
                "Protection violation at address {:#x}, PID: {}",
                addr, current_pid
            );
            return false;
        }
        info!("before is on stack");
        // 检查地址是否位于当前进程的栈空间内
        // if !process_inner.is_on_stack(addr) {
        // let proc = self.current();
        // info!("fack");
        // let proc = proc.read();
        // info!("fbck");
        let flag = !process_inner.is_on_stack(addr);
        if flag {
            warn!(
                "Page fault at non-stack address {:#x}, PID: {}",
                addr, current_pid
            );
            return false;
        }
        info!("stack");
        let f = process_inner.handle_stack_page_fault(addr);
        info!("fcck");
        f
        // // 计算需要分配的页面数
        // let stack_back = process_inner.stack_base();
        // let pages_needed = if addr < stack_base {
        //     let page_start = Page::<Size4KiB>::containing_address(addr);
        //     let page_end = Page::<Size4KiB>::containing_address(stack_base);
        //     (page_end - page_start) as usize;
        // } else {
        //     1
        // };

        // // 分配所需的页面并更新页表
        // let mut mapper = process_inner.page_table_context_mut().mapper();
        // let frame_alloc = get_frame_alloc_for_sure();
        // let start_page = Page::<Size4KiB>::containing_address(addr);
        // let page_range = Page::range_inclusive(start_page, start_page + pages_needed - 1);
        // for page in page_range {
        //     let frame = frame.alloc.allocate_frame().expect("No frames available");
        //     let flags = PageTableFlags::PRESENT | PageTableFlags::WRTIABLE;
        //     unsafe {
        //         mapper.map_to(page, frame, flags, frame_alloc).expect("Failed to map frame").flush();
        //     }
        // }

        // // 更新进程的栈空间信息
        // process_inner.expand_stack(pages_needed * PAGE_SIZE);
    }

    // 这是lab3的kill
    // pub fn kill(&self, pid: ProcessId, ret: isize) {
    //     let proc = self.get_proc(&pid);

    //     if proc.is_none() {
    //         warn!("Process #{} not found.", pid);
    //         return;
    //     }

    //     let proc = proc.unwrap();

    //     if proc.read().status() == ProgramStatus::Dead {
    //         warn!("Process #{} is already dead.", pid);
    //         return;
    //     }

    //     trace!("Kill {:#?}", &proc);

    //     proc.kill(ret);
    // }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Memory | Status\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }
    
    pub fn get_exit_code(&self, pid: ProcessId) -> Option<isize> {
        let proc_opt = self.get_proc(&pid);
        if let Some(proc) = proc_opt {
            let proc_inner = proc.read();
            if proc_inner.status() == ProgramStatus::Dead {
                proc_inner.exit_code()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, parent, page_table, proc_data);
        let pid = proc.pid();
        // info!("1"); Y
        let mut inner = proc.write();
        // FIXME: load elf to process pagetable
        // inner.load_elf(elf);
        // // FIXME: alloc new stack for process
        // inner.init_stack_frame(
        //     VirtAddr::new_truncate(elf.header.pt2.entry_point()),
        //     VirtAddr::new_truncate(STACK_INIT_TOP),
        // );
        // // FIXME: mark process as ready
        // // drop(inner);

        inner.pause();
        // info!("2"); Y
        // trace!("New {:#?}", &proc);
        // info!("3");
        let stack_bot = inner.load_elf(elf, pid.0 as u64);
        let stack_top = stack_bot + STACK_DEF_SIZE - 8;
        // info!("4");
        inner.init_stack_frame(
            VirtAddr::new_truncate(elf.header.pt2.entry_point()), //获取 ELF 文件的入口地址
            VirtAddr::new_truncate(stack_top),
        );
        // info!("5");
        drop(inner);

        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        // info!("6");
        self.push_ready(pid);

        self.print_process_list();

        pid
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.current().read().read(fd, buf)
    }

    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.current().write().write(fd, buf)
    }

    pub fn kill_self(&self, ret: isize) {
        let current_pid = self.current().pid();
        info!("Process: {} is killed", current_pid);
        self.kill(current_pid, ret);
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        if let Some(process) = self.get_proc(&pid) {
            info!("Process: {} is killed", pid);
            process.kill(ret);
        }
    }
    pub fn still_alive(&self, pid: ProcessId) -> bool {
        self.get_proc(&pid)
            .map(|p| p.read().status() != ProgramStatus::Dead)
            .unwrap_or(false)
    }
}
