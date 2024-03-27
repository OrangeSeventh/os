mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;

use apic::*;
use x86::cpuid::CpuId;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::{drivers, interrupt::consts::Irq, memory::physical_to_virtual};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt); // 注册异常处理程序到IDT中
            // TODO: clock::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            // TODO: serial::register_idt(&mut idt);
            serial::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
/// 初始化中断系统
pub fn init() {
    IDT.load(); // 加载IDT到处理器

    // FIXME: check and init APIC
    CpuId::new().get_feature_info().map(
        |f| f.has_apic()
    ).unwrap_or(false);

    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.cpu_init();
    drivers::serial::init();
    // FIXME: enable serial irq with IO APIC (use enable_irq)4
    enable_irq(Irq::Serial0 as u8, 0);
    debug!("Serial0(COM1) IRQ enabled.");    
    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
