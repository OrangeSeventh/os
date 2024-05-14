use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    page::{PageRange, PageRangeInclusive},
    Page,
};
use crate::{resource, ResourceSet};
use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,
    pub(super) resources: Arc<RwLock<ResourceSet>>,
    // process specific data
    pub(super) stack_segment: Option<PageRange>,
    pub(super) code_segments: Option<Vec<PageRangeInclusive>>,
    pub(super) stack_memory: usize,    
    pub(super) code_memory: usize,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            resources: Arc::new(RwLock::new(ResourceSet::default())),
            stack_segment: None,
            code_segments: None,
            stack_memory: 0,
            code_memory: 0,
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        self.stack_segment = Some(Page::range(start, start + size));
        self.stack_memory = size as usize;
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resources.read().read(fd, buf)
    }
    
    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resources.read().write(fd, buf)
    }
    
    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        info!("enter is on stack");
        if let Some(stack_range) = self.stack_segment {

            let addr = addr.as_u64();

            let start = stack_range.start.start_address().as_u64();

            let end = stack_range.end.start_address().as_u64();

            // Check if the address falls within the stack range
            // if addr >= start && addr < end {
            //     return true;
            // }
    
            // Alternatively, check if the address aligns with the expected stack region
            // using the STACK_START_MASK
            let cur_stack_bot = start;
            trace!("Current stack bot: {:#x}", cur_stack_bot);
            trace!("Address to access: {:#x}", addr);
            info!("{:#x} {:#x}", addr, cur_stack_bot);
            addr & STACK_START_MASK == cur_stack_bot & STACK_START_MASK
        } else {
            debug!("No stack segment found");
            false
        }
    }
    pub fn get_memory_usage(&self) -> usize {
        self.stack_memory + self.code_memory
    }
    
}
