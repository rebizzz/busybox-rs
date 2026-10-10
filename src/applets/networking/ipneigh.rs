use super::common::*;
use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct IpneighApplet;
impl Applet for IpneighApplet {
    fn name(&self) -> &'static str {
        "ipneigh"
    }
    fn description(&self) -> &'static str {
        "Manage neighbour / ARP table"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        run_ip_neigh(args)
    }
}
