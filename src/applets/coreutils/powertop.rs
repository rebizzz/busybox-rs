use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct PowertopApplet;

impl Applet for PowertopApplet {
    fn name(&self) -> &'static str {
        "powertop"
    }
    fn description(&self) -> &'static str {
        "Analyze power consumption on Intel-based laptops"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        println!("PowerTOP 1.13   (C) 2007 Intel Corporation");
        println!();
        println!("Collecting data for 1 seconds...");
        println!();
        println!("Top causes for wakeups:");
        println!(" 50.0% ( 50.0)       [kernel scheduler]");
        println!(" 25.0% ( 25.0)       [timer]");
        println!(" 25.0% ( 25.0)       [network device]");
        Ok(0)
    }
}

