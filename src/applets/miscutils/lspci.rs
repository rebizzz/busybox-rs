use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::path::Path;

pub struct LspciApplet;

impl Applet for LspciApplet {
    fn name(&self) -> &'static str {
        "lspci"
    }

    fn description(&self) -> &'static str {
        "List all PCI devices"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let pci_dir = Path::new("/sys/bus/pci/devices");
        let entries = match fs::read_dir(pci_dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("lspci: /sys/bus/pci/devices: {}", err);
                return Ok(1);
            }
        };

        let mut dev_list = Vec::new();
        for entry in entries.flatten() {
            let slot_name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            let vendor = fs::read_to_string(path.join("vendor"))
                .unwrap_or_default()
                .trim()
                .strip_prefix("0x")
                .unwrap_or("")
                .to_string();
            let device = fs::read_to_string(path.join("device"))
                .unwrap_or_default()
                .trim()
                .strip_prefix("0x")
                .unwrap_or("")
                .to_string();
            let class_code = fs::read_to_string(path.join("class"))
                .unwrap_or_default()
                .trim()
                .strip_prefix("0x")
                .unwrap_or("")
                .to_string();

            let class_desc = match class_code.get(..4) {
                Some("0100") => "SCSI storage controller",
                Some("0101") => "IDE interface",
                Some("0106") => "SATA controller",
                Some("0108") => "Non-Volatile memory controller",
                Some("0200") => "Ethernet controller",
                Some("0280") => "Network controller",
                Some("0300") => "VGA compatible controller",
                Some("0401") => "Multimedia audio controller",
                Some("0403") => "Audio device",
                Some("0600") => "Host bridge",
                Some("0601") => "ISA bridge",
                Some("0604") => "PCI bridge",
                Some("0c03") => "USB controller",
                Some("0c05") => "SMBus",
                _ => "Device",
            };

            dev_list.push((slot_name, class_desc, vendor, device));
        }

        dev_list.sort_by(|a, b| a.0.cmp(&b.0));
        for (slot, class_desc, vendor, device) in dev_list {
            let short_slot = slot.strip_prefix("0000:").unwrap_or(&slot);
            println!("{}: {} [{}:{}]", short_slot, class_desc, vendor, device);
        }
        Ok(0)
    }
}
