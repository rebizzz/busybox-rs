use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct Md5SumApplet;

impl Applet for Md5SumApplet {
    fn name(&self) -> &'static str {
        "md5sum"
    }
    fn description(&self) -> &'static str {
        "md5sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::checksums::Md5SumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Sha1SumApplet;

impl Applet for Sha1SumApplet {
    fn name(&self) -> &'static str {
        "sha1sum"
    }
    fn description(&self) -> &'static str {
        "sha1sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::checksums::Sha1SumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Sha256SumApplet;

impl Applet for Sha256SumApplet {
    fn name(&self) -> &'static str {
        "sha256sum"
    }
    fn description(&self) -> &'static str {
        "sha256sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::checksums::Sha256SumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Sha512SumApplet;

impl Applet for Sha512SumApplet {
    fn name(&self) -> &'static str {
        "sha512sum"
    }
    fn description(&self) -> &'static str {
        "sha512sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::checksums::Sha512SumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Sha384sumApplet;

impl Applet for Sha384sumApplet {
    fn name(&self) -> &'static str {
        "sha384sum"
    }
    fn description(&self) -> &'static str {
        "sha384sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::Sha384sumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct Sha3sumApplet;

impl Applet for Sha3sumApplet {
    fn name(&self) -> &'static str {
        "sha3sum"
    }
    fn description(&self) -> &'static str {
        "sha3sum"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::coreutils::crypto_attr::Sha3sumApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

