use super::consts::*;
use crate::{memory::gdt, proc::ProcessContext};
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    // 为 IRQ0（时钟中断）设置中断处理程序
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler)
        .set_stack_index(gdt::CLOCK_IST_INDEX);
}

// 时钟中断的处理程序
// pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         // // 增加计数器的值，每增加到一定值（这里用 0x10000 作为示例）时打印一次 "Tick!"
//         // if inc_counter() % 0x10000 == 0 {
//         //     println!("Tick! @{}", read_counter());
//         // }
//         // 发送中断结束信号给 APIC，表明已经处理完毕当前中断
//         super::ack();
//     });
// }
pub extern "C" fn clock(mut context: ProcessContext) {
    crate::proc::switch(&mut context);
    super::ack();
}

as_handler!(clock);

// // 定义一个原子类型的静态变量 COUNTER，用于计数时钟中断发生的次数
// static COUNTER: AtomicU64 = AtomicU64::new(0);

// // 读取当前的计数器值
// #[inline]
// pub fn read_counter() -> u64 {
//     // 使用 Relaxed 顺序加载 COUNTER 的值
//     COUNTER.load(Ordering::Relaxed)
// }

// // 增加计数器的值并返回新值
// #[inline]
// pub fn inc_counter() -> u64 {
//     // 使用 Relaxed 顺序增加 COUNTER 的值，返回增加后的值
//     COUNTER.fetch_add(1, Ordering::Relaxed) + 1
// }
