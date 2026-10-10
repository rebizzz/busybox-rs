use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, Write};
use std::process::Command;

pub struct IfplugdApplet;
impl Applet for IfplugdApplet {
    fn name(&self) -> &'static str {
        "ifplugd"
    }
    fn description(&self) -> &'static str {
        "Link detection daemon for network interfaces"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut iface = "eth0".to_string();
        let mut run_cmd: Option<String> = None;

        let mut i = 0;
        while i < args.len() {
            let s = args[i].to_string_lossy();
            if s == "-i" && i + 1 < args.len() {
                iface = args[i + 1].to_string_lossy().to_string();
                i += 1;
            } else if s == "-r" && i + 1 < args.len() {
                run_cmd = Some(args[i + 1].to_string_lossy().to_string());
                i += 1;
            }
            i += 1;
        }

        let carrier_path = format!("/sys/class/net/{}/carrier", iface);
        let operstate_path = format!("/sys/class/net/{}/operstate", iface);

        let carrier = fs::read_to_string(&carrier_path)
            .map(|s| s.trim() == "1")
            .unwrap_or_else(|_| {
                fs::read_to_string(&operstate_path)
                    .map(|s| s.trim() == "up")
                    .unwrap_or(false)
            });

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = writeln!(
            out,
            "ifplugd: interface {} link is {}",
            iface,
            if carrier { "up" } else { "down" }
        );

        if let Some(cmd) = run_cmd {
            let arg = if carrier { "up" } else { "down" };
            let _ = Command::new(&cmd).arg(&iface).arg(arg).status();
        }

        Ok(0)
    }
}
