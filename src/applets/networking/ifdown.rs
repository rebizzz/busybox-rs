use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IfdownApplet;
impl Applet for IfdownApplet {
    fn name(&self) -> &'static str {
        "ifdown"
    }
    fn description(&self) -> &'static str {
        "Take a network interface down"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ifdown <interface>");
            return Ok(1);
        }
        let ifname = &args[0];
        super::ifconfig::IfconfigApplet.run(&[ifname.clone(), OsString::from("down")])
    }
}
