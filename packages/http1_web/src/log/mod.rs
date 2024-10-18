use std::{
    cell::OnceCell,
    sync::{Arc, OnceLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, tag: &str, message: &str);
}

pub struct NoopLogger;
impl Logger for NoopLogger {
    fn log(&self, _level: LogLevel, _tag: &str, _message: &str) {}
}

pub static GLOBAL_LOGGER: OnceLock<Box<dyn Logger>> = OnceLock::new();

pub fn set_logger<T: Logger + 'static>(logger: T) {
    if let Err(_) = GLOBAL_LOGGER.set(Box::new(logger)) {
        panic!("logger was already set")
    }
}

pub fn get_logger() -> &'static dyn Logger {
    match GLOBAL_LOGGER.get() {
        Some(logger) => logger.as_ref(),
        None => &NoopLogger,
    }
}

pub struct ConsoleLogger;


#[macro_export]
macro_rules! info {
    ($level:expr, $msg:literal) => {
        $crate::log::get_logger().log($crate::log::LogLevel::Info, $msg);
    };
}

#[macro_export]
macro_rules! warn {
    ($level:expr, $msg:literal) => {
        get_logger().log($crate::log::LogLevel::Warn, $msg);
    };
}

#[macro_export]
macro_rules! error {
    ($level:expr, $msg:literal) => {
        get_logger().log($crate::log::LogLevel::Error, $msg);
    };
}
