use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process;
use std::sync::Arc;

use busybox::applets;
use busybox::core::Applet;

fn print_general_help(applets: &[Arc<dyn Applet>]) {
    eprintln!("BusyBox v1.39.0.git (rust) multi-call binary.");
    eprintln!("BusyBox is copyrighted by many authors between 1998-2026.");
    eprintln!("Licensed under MIT. See source distribution for detailed");
    eprintln!("copyright notices.\n");
    eprintln!("Usage: busybox [function [arguments]...]");
    eprintln!("   or: busybox --list[-full]");
    eprintln!("   or: busybox --show SCRIPT");
    eprintln!("   or: busybox --install [-s] [DIR]");
    eprintln!("   or: function [arguments]...\n");
    eprintln!("\tBusyBox is a multi-call binary that combines many common Unix");
    eprintln!("\tutilities into a single executable.  Most people will create a");
    eprintln!("\tlink to busybox for each function they wish to use and BusyBox");
    eprintln!("\twill act like whatever it was invoked as.\n");
    eprintln!("Currently defined functions:");

    let mut names: Vec<&str> = applets.iter().map(|a| a.name()).collect();
    names.sort();

    let mut line = String::from("\t");
    for (i, name) in names.iter().enumerate() {
        if i > 0 {
            line.push_str(", ");
        }
        if line.len() + name.len() > 76 {
            eprintln!("{}", line);
            line = String::from("\t");
        }
        line.push_str(name);
    }
    if !line.trim().is_empty() {
        eprintln!("{}", line);
    }
    eprintln!();
}

fn print_applet_help(applet: &dyn Applet) {
    println!("BusyBox v1.39.0.git (rust) multi-call binary.\n");
    println!("Usage: {} [arguments]...\n", applet.name());
    println!("{}\n", applet.description());
}

fn main() {
    let applets = applets::get_applets();
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
            print_general_help(&applets);
            process::exit(0);
        }

        let first_arg = raw_args[1].to_string_lossy();
        if first_arg == "--help" {
            if raw_args.len() == 2 || raw_args[2] == "busybox" {
                print_general_help(&applets);
                process::exit(0);
            }
            let sub = raw_args[2].to_string_lossy();
            if let Some(app) = applets.iter().find(|a| a.name() == sub) {
                print_applet_help(app.as_ref());
                process::exit(0);
            } else {
                eprintln!("{}: applet not found", sub);
                process::exit(127);
            }
        }

        if first_arg == "--list" || first_arg == "--list-full" {
            for app in &applets {
                println!("{}", app.name());
            }
            process::exit(0);
        }

        if first_arg.starts_with('-') {
            print_general_help(&applets);
            process::exit(0);
        }

        let applet_name = first_arg;
        let sub_args: Vec<OsString> = raw_args[2..].to_vec();

        if sub_args.first().map(|s| s.to_string_lossy()) == Some("--help".into()) {
            if let Some(app) = applets.iter().find(|a| a.name() == applet_name) {
                print_applet_help(app.as_ref());
                process::exit(0);
            }
        }

        if let Some(app) = applets.iter().find(|a| a.name() == applet_name) {
            match app.run(&sub_args) {
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
        // Multi-call binary dispatch via symlink or argv[0]
        let sub_args: Vec<OsString> = raw_args[1..].to_vec();
        if sub_args.first().map(|s| s.to_string_lossy()) == Some("--help".into()) {
            if let Some(app) = applets.iter().find(|a| a.name() == invocation_name) {
                print_applet_help(app.as_ref());
                process::exit(0);
            }
        }

        if let Some(app) = applets.iter().find(|a| a.name() == invocation_name) {
            match app.run(&sub_args) {
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
