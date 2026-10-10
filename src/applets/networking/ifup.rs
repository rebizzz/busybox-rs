use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IfupApplet;
impl Applet for IfupApplet {
    fn name(&self) -> &'static str {
        "ifup"
    }
    fn description(&self) -> &'static str {
        "Bring a network interface up"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ifup <interface>");
            return Ok(1);
        }
        let ifname = &args[0];
        super::ifconfig::IfconfigApplet.run(&[ifname.clone(), OsString::from("up")])
    }
}
