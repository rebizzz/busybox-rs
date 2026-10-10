use super::common::*;
use crate::core::{Applet, Result};
use std::collections::HashMap;
use std::ffi::{CStr, CString, OsString};
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

pub struct LsscsiApplet;

impl Applet for LsscsiApplet {
    fn name(&self) -> &'static str {
        "lsscsi"
    }
    fn description(&self) -> &'static str {
        "List SCSI devices (or hosts) and their attributes"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let scsi_dir = Path::new("/sys/bus/scsi/devices");
        let mut out = Vec::new();

        if scsi_dir.exists() {
            if let Ok(entries) = fs::read_dir(scsi_dir) {
                let mut devs = Vec::new();
                for entry in entries.flatten() {
                    devs.push(entry.path());
                }
                devs.sort();

                for p in devs {
                    let dev_name = p.file_name().unwrap_or_default().to_string_lossy();

                    if !dev_name.contains(':') {
                        continue;
                    }

                    let vendor = fs::read_to_string(p.join("vendor")).unwrap_or_default();
                    let model = fs::read_to_string(p.join("model")).unwrap_or_default();
                    let dev_type = fs::read_to_string(p.join("type")).unwrap_or_default();
                    let type_str = match dev_type.trim() {
                        "0" => "disk   ",
                        "1" => "tape   ",
                        "4" => "worm   ",
                        "5" => "cd/dvd ",
                        _ => "process",
                    };

                    out.push(b'[');
                    out.extend_from_slice(dev_name.as_bytes());
                    out.extend_from_slice(b"]  ");
                    out.extend_from_slice(type_str.as_bytes());
                    out.push(b' ');
                    out.extend_from_slice(vendor.trim().as_bytes());
                    out.push(b' ');
                    out.extend_from_slice(model.trim().as_bytes());
                    out.push(b'\n');
                }
            }
        }

        if out.is_empty() {
            if let Ok(proc_scsi) = fs::read_to_string("/proc/scsi/scsi") {
                out.extend_from_slice(proc_scsi.as_bytes());
            }
        }

        print_bytes(&out);
        Ok(0)
    }
}
