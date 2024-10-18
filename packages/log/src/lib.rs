use std::{
    fmt::Arguments,
    sync::{atomic::AtomicI8, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Debug = 0,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct Record<'a> {
    module_path: &'static str,
    args: Arguments<'a>,
}

impl<'a> Record<'a> {
    pub fn new(module_path: &'static str, args: Arguments<'a>) -> Self {
        Record { module_path, args }
    }

    pub fn module_path(&self) -> &'static str {
        self.module_path
    }

    pub fn args(&self) -> &Arguments<'a> {
        &self.args
    }
}

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, record: &Record<'_>);
}

pub struct NoopLogger;

/// A logger.
impl Logger for NoopLogger {
    fn log(&self, _level: LogLevel, _record: &Record<'_>) {}
}

static GLOBAL_LOGGER: OnceLock<Box<dyn Logger>> = OnceLock::new();
static MIN_LOG_LEVEL: AtomicI8 = AtomicI8::new(0);

/// Sets the current logger.
pub fn set_logger<T: Logger + 'static>(logger: T) {
    if let Err(_) = GLOBAL_LOGGER.set(Box::new(logger)) {
        panic!("logger was already set")
    }
}

/// Gets the current logger.
pub fn get_logger() -> &'static dyn Logger {
    match GLOBAL_LOGGER.get() {
        Some(logger) => logger.as_ref(),
        None => &NoopLogger,
    }
}

/// Sets a global log filter.
pub fn set_log_filter(level: LogLevel) {
    MIN_LOG_LEVEL.store(level as i8, std::sync::atomic::Ordering::Release);
}

/// Check if the current level can be log.
pub fn can_log(level: LogLevel) -> bool {
    let min_level = MIN_LOG_LEVEL.load(std::sync::atomic::Ordering::Acquire);
    level as i8 >= min_level
}

#[doc(hidden)]
#[inline]
pub fn __log(level: LogLevel, record: Record<'_>) {
    if can_log(level) {
        get_logger().log(level, &record);
    }
}

/// A logger that logs to the current console.
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, record: &Record<'_>) {
        fn colorize(text: &str, color_code: &str) -> String {
            format!("\x1b[{}m{}\x1b[0m", color_code, text)
        }

        #[rustfmt::skip]
        let level_str = match level {
            LogLevel::Debug => colorize("DEBUG", "37"),  // White
            LogLevel::Info  => colorize("INFO ", "34"),   // Blue
            LogLevel::Warn  => colorize("WARN ", "33"),   // Yellow
            LogLevel::Error => colorize("ERROR", "31"),  // Red
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let ms = now.as_millis();

        let time_str: String = colorize(&format!("{:03}ms", ms), "90");      // Light gray for time
        let module_str = colorize(record.module_path, "36");         // Light gray for module

        if level != LogLevel::Error {
            println!(
                "{level_str} [{time_str}] [{module_str}]: {}",
                record.args
            );
        } else {
            eprintln!(
                "{level_str} [{time_str}] [{module_str}]: {}",
                record.args
            );
        }
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::__log($level, $crate::Record::new(module_path!(), format_args!($($arg)*)));
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Debug, $($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Info, $($arg)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Warn, $($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Error, $($arg)*)
    };
}
