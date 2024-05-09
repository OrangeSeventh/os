use super::*;
use crate::memory::*;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;

#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    page_table: Option<PageTableContext>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        page_table: PageTableContext,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: None,
            children: Vec::new(),
            page_table: Some(page_table),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
    }

    pub fn alloc_init_stack(&mut self) -> VirtAddr {
        // FIXME: alloc init stack base on self pid

        // let stack_size = STACK_DEF_SIZE;
        // let stack_start = VirtAddr::new(self.pid.0 as u64 *stack_size + STACK_INIT_BOT);
        // let stack_end = stack_start + stack_size;

        // let frame_allocator = &mut *get_frame_alloc_for_sure();
        // let mut page_table = unsafe { crate::memory::active_level_4_table() };

        // let page_range = {
        //     let stack_start_page = Page::containing_address(stack_start);
        //     let stack_end_page = Page::containing_address(stack_end - 1u64);
        //     Page::range_inclusive(stack_start_page, stack_end_page)
        // };
        
        // for page in page_range {
        //     let frame = frame_allocator
        //     .allocate_frame()
        //     .expect("no more frames");
        //     unsafe {
        //         page_table.map_to(page, frame, x86_64::structures::paging::PageTableFlags::WRITABLE, frame_allocator)
        //         .expect("map_to failed")
        //         .flush();
        //     }
        // }

        let mut inner = self.inner.write();
        let frame_allocator = &mut *get_frame_alloc_for_sure();
        let page_table = inner.page_table.as_ref().unwrap();
        let stack_bot = STACK_INIT_BOT - ( self.pid.0 as u64 -1 ) * STACK_DEF_SIZE;
        let stack_top = stack_bot + STACK_DEF_SIZE - 8;
        elf::map_range(
            stack_bot,
            STACK_DEF_PAGE,
            &mut page_table.mapper(),
            frame_allocator,
            true,
        )
        .unwrap();
        inner.proc_data.as_mut().unwrap().set_stack(VirtAddr::new(stack_bot), STACK_DEF_PAGE);
        VirtAddr::new(stack_top)
    }

}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.page_table.as_ref().unwrap().clone_l4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }
    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr){
        self.context.init_stack_frame(entry, stack_top)
    }
    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        context.restore(&mut self.context);
        self.pause();
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        if let Some(page_table) = self.page_table.as_ref() {
            page_table.load();
            self.resume();
        }
        // FIXME: restore the process's page table
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // FIXME: take and drop unused resources
        // lab3的时候写的
        // self.page_table = None;
        // self.proc_data = None;
        // 改为lab4的删除进程数据
        self.proc_data.take();
        self.page_table.take();
        info!("kill completed,status {:#?}",self.status);
        // for child in self.children.iter(){
        //     let mut child_inner = child.inner.write();
        //     child_inner.parent = None;
        // }
        // self.children.clear();
    }

    pub fn handle_stack_page_fault(&mut self, fault_addr:VirtAddr) -> bool {
        info!("handle_stack_page_fault,{:?}", fault_addr);
        let frame_alloc = &mut *get_frame_alloc_for_sure();

        let proc_data = self.proc_data.as_mut().unwrap();

        let mapper = &mut self.page_table.as_ref().unwrap().mapper();
        let start_page = Page::<Size4KiB>::containing_address(fault_addr);
        let count = proc_data.stack_segment.unwrap().start - start_page;   
        info!("{:?}", count);
        let res = elf::map_range(start_page.start_address().as_u64(), count, mapper, frame_alloc, true);
        if res.is_err() {
            info!("Failed to map stack page : {:?}", res);
            return false;
        }
        let now_pagerange = proc_data.stack_segment.unwrap();    
        let now_pagenum = now_pagerange.end - now_pagerange.start;
        proc_data.set_stack(start_page.start_address(), now_pagenum + count);
        true
    }

    pub fn load_elf(&mut self, elf: &ElfFile, pid: u64) -> u64 {
        let frame_alloc = &mut *get_frame_alloc_for_sure();
        let page_table = self.page_table.as_mut().unwrap();
        let mut mapper = page_table.mapper();
        let stack_bot = STACK_INIT_BOT - (pid - 1) * STACK_MAX_SIZE;
        let code_segments = elf::load_elf(
            elf,
            *PHYSICAL_OFFSET.get().unwrap(),
            &mut mapper,
            frame_alloc,
            true
        ).unwrap();
        let stack_segment = elf::map_range(stack_bot, STACK_DEF_PAGE, &mut mapper, frame_alloc, true).unwrap();

        let proc_data = self.proc_data.as_mut().unwrap();
        proc_data.code_segments = Some(code_segments);
        proc_data.stack_segment = Some(stack_segment);
        stack_bot
    }


}

impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:12} | {:7} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.status
        )?;
        Ok(())
    }
}
