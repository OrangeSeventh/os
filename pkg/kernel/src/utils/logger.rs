use log::{LevelFilter, Metadata, Record};

pub fn init(log_level: &'static str) {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    let level = match log_level {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info,  // 默认日志级别为 info
    };
    // FIXME: Configure the logger
    log::set_max_level(level);  //设置日志级别过滤器
    info!("Logger Initialized with log level: {}", log_level);
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= metadata.level() // 根据日志级别决定是否记录
    }

    fn log(&self, record: &Record) {
        // FIXME: Implement the logger with serial output
        if self.enabled(record.metadata()){
            println!("File:{} Line:{} - {}: {}", record.file_static().unwrap(), record.line().unwrap(), record.target(),record.args());
        }
    }

    fn flush(&self) {}
}
