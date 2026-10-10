use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct GunzipApplet;

impl Applet for GunzipApplet {
    fn name(&self) -> &'static str {
        "gunzip"
    }
    fn description(&self) -> &'static str {
        "gunzip"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::GunzipApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Bunzip2Applet;

impl Applet for Bunzip2Applet {
    fn name(&self) -> &'static str {
        "bunzip2"
    }
    fn description(&self) -> &'static str {
        "bunzip2"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::Bunzip2Applet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct BzcatApplet;

impl Applet for BzcatApplet {
    fn name(&self) -> &'static str {
        "bzcat"
    }
    fn description(&self) -> &'static str {
        "bzcat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::BzcatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UnlzmaApplet;

impl Applet for UnlzmaApplet {
    fn name(&self) -> &'static str {
        "unlzma"
    }
    fn description(&self) -> &'static str {
        "unlzma"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::UnlzmaApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct LzcatApplet;

impl Applet for LzcatApplet {
    fn name(&self) -> &'static str {
        "lzcat"
    }
    fn description(&self) -> &'static str {
        "lzcat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::LzcatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct LzmaApplet;

impl Applet for LzmaApplet {
    fn name(&self) -> &'static str {
        "lzma"
    }
    fn description(&self) -> &'static str {
        "lzma"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::LzmaApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct UnxzApplet;

impl Applet for UnxzApplet {
    fn name(&self) -> &'static str {
        "unxz"
    }
    fn description(&self) -> &'static str {
        "unxz"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::UnxzApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct XzApplet;

impl Applet for XzApplet {
    fn name(&self) -> &'static str {
        "xz"
    }
    fn description(&self) -> &'static str {
        "xz"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::XzApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct XzcatApplet;

impl Applet for XzcatApplet {
    fn name(&self) -> &'static str {
        "xzcat"
    }
    fn description(&self) -> &'static str {
        "xzcat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::archival::XzcatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct ZcatApplet;

impl Applet for ZcatApplet {
    fn name(&self) -> &'static str {
        "zcat"
    }
    fn description(&self) -> &'static str {
        "zcat"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::archival::package::ZcatApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

