use std::{
    fmt::Arguments,
    sync::{atomic::AtomicI8, OnceLock},
};

use datetime::DateTime;

/// Level of a log message.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Debug = 0,
    Info,
    Warn,
    Error,
}

/// A record that contains information of the log message.
#[derive(Debug, Clone)]
pub struct Record<'a> {
    module_path: &'static str,
    file: &'static str,
    line: u32,
    args: Arguments<'a>,
}

impl<'a> Record<'a> {
    /// Constructs a new log record.
    ///
    /// # Parameters
    /// - `module_path`: The module where the log occurred, obtained with `module_path!()`
    /// - `file`: The file where the log occurred, obtained with `file!()`
    /// - `line`: The line where the log occurred, obtained with `line!()`
    /// - `args`: Argument to that contains the message.
    pub fn new(
        module_path: &'static str,
        file: &'static str,
        line: u32,
        args: Arguments<'a>,
    ) -> Self {
        Record {
            module_path,
            file,
            line,
            args,
        }
    }

    /// Returns the module path where the log occurred.
    pub fn module_path(&self) -> &'static str {
        self.module_path
    }

    /// Returns the file where the log occurred.
    pub fn file(&self) -> &'static str {
        self.file
    }

    /// Returns the line where the log occurred.
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Returns the arguments of the log.
    pub fn args(&self) -> &Arguments<'a> {
        &self.args
    }
}

/// A logger.
pub trait Logger: Send + Sync {
    /// Send a message.
    fn log(&self, level: LogLevel, record: &Record<'_>);
}

/// A logger that does nothing.
pub struct NoopLogger;

/// A logger.
impl Logger for NoopLogger {
    fn log(&self, _level: LogLevel, _record: &Record<'_>) {}
}

static GLOBAL_LOGGER: OnceLock<Box<dyn Logger>> = OnceLock::new();
static MIN_LOG_LEVEL: AtomicI8 = AtomicI8::new(0);

/// Sets the current logger.
pub fn set_logger<T: Logger + 'static>(logger: T) {
    if GLOBAL_LOGGER.set(Box::new(logger)).is_err() {
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

        let dt = DateTime::now_utc();
        let time_str: String = colorize(&format!("{dt}"), "90"); // Light gray for time
        let module_str = colorize(record.module_path, "36"); // Light gray for module

        if level != LogLevel::Error {
            println!("{level_str} [{time_str}] [{module_str}]: {}", record.args);
        } else {
            eprintln!("{level_str} [{time_str}] [{module_str}]: {}", record.args);
        }
    }
}

/// Logs a message with the given level.
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::__log($level, $crate::Record::new(module_path!(), file!(), line!(), format_args!($($arg)*)))
    };
}

/// Logs a debug message.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Debug, $($arg)*)
    };
}

/// Logs a information message.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Info, $($arg)*)
    };
}

/// Logs a warning message.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Warn, $($arg)*)
    };
}

/// Logs a error message.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::log!($crate::LogLevel::Error, $($arg)*)
    };
}
