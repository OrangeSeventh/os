use lazy_static::lazy_static;
use x86::bits64::rflags::stac;
use x86_64::registers::segmentation::Segment;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const CLOCK_IST_INDEX: u16 = 2;
pub const IST_SIZES: [usize; 3] = [0x1000, 0x1000, 0x1000];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // initialize the TSS with the static buffers
        // will be allocated on the bss section when the kernel is load
        // 使用静态缓冲区初始化TSS
        // 内核加载时将在bss段分配缓冲区
        // DO NOT MODIFY THE FOLLOWING CODE
        tss.privilege_stack_table[0] = { // 初始化特权级堆栈表
            const STACK_SIZE: usize = IST_SIZES[0]; // 定义STACK_SIZE为IST_SIZES数组的第一个元素
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Privilege Stack  : 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            ); // 打印特权级堆栈的起始地址和结束地址
            stack_end
        };
        // FIXME: fill tss.interrupt_stack_table with the static stack buffers like above
        // You can use `tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize]`        
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[DOUBLE_FAULT_IST_INDEX as usize]; // 使用IST_SIZES定义堆栈大小
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // 分配静态堆栈
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() }); // 获取堆栈的起始虚拟地址
            let stack_end = stack_start + STACK_SIZE as u64; // 计算堆栈的结束虚拟地址
            info!(
                "Double Fault Stack: 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] ={
            const STACK_SIZE: usize = IST_SIZES[PAGE_FAULT_IST_INDEX as usize]; // 使用IST_SIZES定义堆栈大小
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // 分配静态堆栈
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() }); // 获取堆栈的起始虚拟地址
            let stack_end = stack_start + STACK_SIZE as u64; // 计算堆栈的结束虚拟地址
            info!(
                "Page Fault Stack: 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            ); // 打印Page Fault堆栈的起始地址和结束地址
            stack_end
        };
        // 设置时钟中断堆栈
        tss.interrupt_stack_table[CLOCK_IST_INDEX as usize] = {
            const STACK_SIZE: usize = IST_SIZES[CLOCK_IST_INDEX as usize];
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { STACK.as_ptr() });
            let stack_end = stack_start + STACK_SIZE as u64;
            info!(
                "Clock Interrupt Stack: 0x{:016x}-0x{:016x}",
                stack_start.as_u64(),
                stack_end.as_u64()
            );
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, KernelSelectors) = {
        let mut gdt = GlobalDescriptorTable::new(); // 创建一个新的GlobalDescriptorTable实例
        let code_selector = gdt.append(Descriptor::kernel_code_segment());  // 向GDT中追加内核代码段描述符
        let data_selector = gdt.append(Descriptor::kernel_data_segment());  // 最佳内核数据段描述符
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            KernelSelectors { // 返回初始化后的KernelSelectors
                code_selector,
                data_selector,
                tss_selector,
            },
        )
    };
}

#[derive(Debug)]
pub struct KernelSelectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{CS, DS, ES, FS, GS, SS};
    use x86_64::instructions::tables::load_tss;
    use x86_64::PrivilegeLevel;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.data_selector);
        SS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        ES::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        FS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        GS::set_reg(SegmentSelector::new(0, PrivilegeLevel::Ring0));
        load_tss(GDT.1.tss_selector);
    }

    let mut size = 0;

    for &s in IST_SIZES.iter() {
        size += s;
    }

    let (size, unit) = crate::humanized_size(size as u64);
    info!("Kernel IST Size  : {:>7.*} {}", 3, size, unit);

    info!("GDT Initialized.");
}

pub fn get_selector() -> &'static KernelSelectors {
    &GDT.1
}
