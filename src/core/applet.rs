use std::ffi::OsString;

pub type Result<T> = std::result::Result<T, crate::core::errors::BbError>;

/// Every BusyBox command is an `Applet`.
/// It takes raw `&[OsString]` arguments (preserving raw Unix bytes without forced UTF-8 conversion),
/// and returns an exit code `i32` or a structured `BbError`.
pub trait Applet: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, args: &[OsString]) -> Result<i32>;
}
