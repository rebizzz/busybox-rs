use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::thread;

pub struct NprocApplet;
impl Applet for NprocApplet {
    fn name(&self) -> &'static str {
        "nproc"
    }
    fn description(&self) -> &'static str {
        "Print the number of processing units available"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let cpus = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        println!("{}", cpus);
        Ok(0)
    }
}
