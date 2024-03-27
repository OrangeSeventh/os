use spin::Mutex;

use super::uart16550::SerialPort;

const SERIAL_IO_PORT: u16 = 0x3F8; // COM1 串行端口的I/O地址

once_mutex!(pub SERIAL: SerialPort); // 只初始化一次

pub fn clear_screen() {
    use core::fmt::Write;
    use spin::Once;
    
    static SERIAL: Once<Mutex<SerialPort>> = Once::new();
    // 发送ANSI转义序列来清屏

    SERIAL.call_once(|| Mutex::new(SerialPort::new(SERIAL_IO_PORT))).lock().write_str("\x1B[2J\x1B[H").unwrap();
}

pub fn init() {
    init_SERIAL(SerialPort::new(SERIAL_IO_PORT));

    // 直接在 Mut exGuard 上调用 init 方法
    //let mut serial_guard = get_serial_for_sure();
    //serial_guard.init(); // 直接在获取的 MutexGuard 上调用
    get_serial_for_sure().init();
    clear_screen();

    println!("{}", crate::get_ascii_header());
    println!("[+] Serial Initialized.");
}

// 提供对 SERIAL 的访问

guard_access_fn!(pub get_serial(SERIAL: SerialPort));
