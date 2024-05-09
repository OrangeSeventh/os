use log::{LevelFilter, Metadata, Record};

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG_LEVEL") {
        // Some("error") => LevelFilter::Error,
        // Some("warn") => LevelFilter::Warn,
        // Some("info") => LevelFilter::Info,
        // Some("debug") => LevelFilter::Debug,
        // Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Trace,  // 默认日志级别为 info
    });
    log::set_max_level(LevelFilter::Trace);
    // FIXME: Configure the logger
    info!("Current log level: {}", log::max_level());

    info!("Logger Initialized.");
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
