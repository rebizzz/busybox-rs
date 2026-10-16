use std::ffi::OsString;

pub type Result<T> = std::result::Result<T, crate::core::errors::BbError>;

pub type AppletFn = fn(&[OsString]) -> Result<i32>;

#[derive(Clone, Copy)]
pub struct AppletEntry {
    pub name: &'static str,
    pub run: AppletFn,
}

pub trait Applet: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, args: &[OsString]) -> Result<i32>;
}
