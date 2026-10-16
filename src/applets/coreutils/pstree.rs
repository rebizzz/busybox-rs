use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};

pub struct PstreeApplet;

impl Applet for PstreeApplet {
    fn name(&self) -> &'static str {
        "pstree"
    }
    fn description(&self) -> &'static str {
        "Display a tree of processes"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        #[derive(Clone)]
        struct ProcInfo {
            pid: i32,
            ppid: i32,
            comm: String,
        }

        let mut procs: Vec<ProcInfo> = Vec::new();

        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let s = name.to_string_lossy();
                if s.chars().all(|c| c.is_ascii_digit()) {
                    let pid: i32 = match s.parse() {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    let stat_path = entry.path().join("stat");
                    if let Ok(data) = fs::read_to_string(stat_path) {
                        if let (Some(open), Some(close)) = (data.find('('), data.rfind(')')) {
                            let comm = data[open + 1..close].to_string();
                            let rest = &data[close + 2..];
                            let mut parts = rest.split_whitespace();
                            let _state = parts.next();
                            let ppid: i32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
                            procs.push(ProcInfo { pid, ppid, comm });
                        }
                    }
                }
            }
        }

        fn print_tree(pids: &[ProcInfo], parent: i32, indent: usize) {
            for p in pids {
                if p.ppid == parent {
                    let spaces = "  ".repeat(indent);
                    println!("{}-+- {} ({})", spaces, p.comm, p.pid);
                    print_tree(pids, p.pid, indent + 1);
                }
            }
        }

        println!("systemd(1)");
        print_tree(&procs, 1, 1);

        Ok(0)
    }
}

