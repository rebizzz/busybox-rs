use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IpApplet;
impl Applet for IpApplet {
    fn name(&self) -> &'static str {
        "ip"
    }
    fn description(&self) -> &'static str {
        "Show / manipulate routing, network devices, interfaces and tunnels"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.is_empty() {
            eprintln!("Usage: ip [link|addr|route|neigh|rule|tunnel] ...");
            return Ok(1);
        }
        let sub = args[0].to_string_lossy();
        match sub.as_ref() {
            "link" | "l" => run_ip_link(&args[1..]),
            "addr" | "a" | "address" => run_ip_addr(&args[1..]),
            "route" | "r" | "ro" => run_ip_route(&args[1..]),
            "neigh" | "n" | "neighbor" => run_ip_neigh(&args[1..]),
            "rule" | "ru" => run_ip_rule(&args[1..]),
            "tunnel" | "tu" => run_ip_tunnel(&args[1..]),
            _ => {
                eprintln!("ip: unknown object '{}'", sub);
                Ok(1)
            }
        }
    }
}
