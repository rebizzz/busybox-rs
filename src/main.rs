use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process;

use busybox::core::AppletEntry;
use busybox::{APPLETS, find_applet};

fn print_general_help() {
    print!("BusyBox-RS v{} multi-call binary.\n", env!("CARGO_PKG_VERSION"));
    print!("Copyright (C) 2026 Busybox-RS Contributors and upstream authors.\n");
    print!("Licensed under GPLv2. See source distribution for detailed\n");
    print!("copyright notices.\n\n");
    print!("Usage: busybox [function [arguments]...]\n");
    print!("   or: busybox --list[-full]\n");
    print!("   or: busybox --show SCRIPT\n");
    print!("   or: busybox --install [-s] [DIR]\n");
    print!("   or: function [arguments]...\n\n");
    print!("\tBusyBox is a multi-call binary that combines many common Unix\n");
    print!("\tutilities into a single executable.  Most people will create a\n");
    print!("\tlink to busybox for each function they wish to use and BusyBox\n");
    print!("\twill act like whatever it was invoked as.\n\n");
    print!("Currently defined functions:\n");

    let mut line = String::from("\t");
    for (i, entry) in APPLETS.iter().enumerate() {
        if i > 0 {
            line.push_str(", ");
        }
        if line.len() + entry.name.len() > 76 {
            print!("{},\n", line);
            line = String::from("\t");
        }
        line.push_str(entry.name);
    }
    if !line.trim().is_empty() {
        print!("{}\n", line);
    }
}

fn print_applet_help(entry: &AppletEntry) {
    print!("BusyBox-RS v{} multi-call binary.\n\n", env!("CARGO_PKG_VERSION"));
    print!("Usage: {} [arguments]...\n\n", entry.name);
}

fn main() {
    let raw_args: Vec<OsString> = env::args_os().collect();

    let invocation_name = raw_args
        .first()
        .map(|s| {
            let p = Path::new(s);
            p.file_name().and_then(|n| n.to_str()).unwrap_or("busybox")
        })
        .unwrap_or("busybox");

    let is_direct_busybox =
        invocation_name == "busybox" || invocation_name.ends_with("busybox-suffix");

    if is_direct_busybox {
        if raw_args.len() <= 1 {
            print_general_help();
            process::exit(0);
        }

        let first_arg = raw_args[1].to_string_lossy();
        if first_arg == "--help" {
            if raw_args.len() == 2 || raw_args[2] == "busybox" {
                print_general_help();
                process::exit(0);
            }
            let sub = raw_args[2].to_string_lossy();
            if let Some(entry) = find_applet(&sub) {
                print_applet_help(entry);
                process::exit(0);
            } else {
                eprintln!("{}: applet not found", sub);
                process::exit(127);
            }
        }

        if first_arg == "--list" || first_arg == "--list-full" {
            for entry in APPLETS {
                println!("{}", entry.name);
            }
            process::exit(0);
        }

        if first_arg.starts_with('-') {
            print_general_help();
            process::exit(0);
        }

        let applet_name = first_arg;
        let sub_args: Vec<OsString> = raw_args[2..].to_vec();

        if sub_args.first().map(|s| s.to_string_lossy()) == Some("--help".into()) && applet_name != "test" && applet_name != "[" && applet_name != "echo" {
            if let Some(entry) = find_applet(&applet_name) {
                print_applet_help(entry);
                process::exit(0);
            }
        }

        if let Some(entry) = find_applet(&applet_name) {
            match (entry.run)(&sub_args) {
                Ok(code) => process::exit(code),
                Err(e) => {
                    eprintln!("{}: {}", applet_name, e);
                    process::exit(1);
                }
            }
        } else {
            eprintln!("{}: applet not found", applet_name);
            process::exit(127);
        }
    } else {
        let sub_args: Vec<OsString> = raw_args[1..].to_vec();
        if sub_args.first().map(|s| s.to_string_lossy()) == Some("--help".into()) {
            if let Some(entry) = find_applet(invocation_name) {
                print_applet_help(entry);
                process::exit(0);
            }
        }

        if let Some(entry) = find_applet(invocation_name) {
            match (entry.run)(&sub_args) {
                Ok(code) => process::exit(code),
                Err(e) => {
                    eprintln!("{}: {}", invocation_name, e);
                    process::exit(1);
                }
            }
        } else {
            eprintln!("{}: applet not found", invocation_name);
            process::exit(127);
        }
    }
}
