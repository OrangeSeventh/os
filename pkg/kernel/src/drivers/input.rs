use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use alloc::string::{String, ToString};
extern crate alloc;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use crate::drivers::uart16550::SerialPort;

// 定义你的输入数据类型，这里简单使用u8作为例子
type Key = u8;

// 初始化无锁输入缓冲区
lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}
lazy_static! {
    static ref SERIAL_PORT: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3F8));
}
// 标记是否应该停止读取输入
static STOP_READING: AtomicBool = AtomicBool::new(false);

// 将键值推入缓冲区
#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        // 如果有日志系统，可以在这里记录日志
        //warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

// 尝试从缓冲区中弹出键值
#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

// 从缓冲区中阻塞取出数据
pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
        // 检查是否应该停止读取
        if STOP_READING.load(Ordering::SeqCst) {
            break;
        }
        // 在这里可能需要让出CPU时间，例如调用`core::hint::spin_loop`或类似机制
    }
    0 // 返回一个默认值或特定的停止信号值
}

// 获取一行输入
pub fn get_line() -> String {
    let mut line = String::with_capacity(128); // 预先分配内存
    let mut serial_port = SERIAL_PORT.lock(); // 获取全局SerialPort实例的引用
    loop {
        let key = pop_key(); // 假设这个方法会阻塞等待并返回一个按键
        match key {
            0x08 | 0x7F => { // 退格键
                if !line.is_empty() {
                    line.pop(); // 移除字符串最后一个字符
                    backspace(&mut *serial_port); // 正确调用backspace方法
                }
            },
            b'\n' => break, // 换行符，结束循环
            _ => {
                line.push(key as char); // 将按键添加到字符串
                serial_port.send(key); // 正确发送按键
            }
        }
    }
    line
}

// 实现退格函数
fn backspace(serial_port: &mut SerialPort) {
    serial_port.send(0x08); // 发送退格
    serial_port.send(0x20); // 发送空格以覆盖原位置的字符
    serial_port.send(0x08); // 再次发送退格以移动光标位置
}

// 如果需要，实现停止读取的函数
pub fn stop_reading() {
    STOP_READING.store(true, Ordering::SeqCst);
}
