use busybox::Applet as _;
use bb_core::{Applet, Result};
use std::ffi::OsString;

pub struct I2cdetectApplet;

impl Applet for I2cdetectApplet {
    fn name(&self) -> &'static str {
        "i2cdetect"
    }
    fn description(&self) -> &'static str {
        "i2cdetect"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::I2cdetectApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct I2cdumpApplet;

impl Applet for I2cdumpApplet {
    fn name(&self) -> &'static str {
        "i2cdump"
    }
    fn description(&self) -> &'static str {
        "i2cdump"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::I2cdumpApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct I2cgetApplet;

impl Applet for I2cgetApplet {
    fn name(&self) -> &'static str {
        "i2cget"
    }
    fn description(&self) -> &'static str {
        "i2cget"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::I2cgetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct I2csetApplet;

impl Applet for I2csetApplet {
    fn name(&self) -> &'static str {
        "i2cset"
    }
    fn description(&self) -> &'static str {
        "i2cset"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::I2csetApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

pub struct I2ctransferApplet;

impl Applet for I2ctransferApplet {
    fn name(&self) -> &'static str {
        "i2ctransfer"
    }
    fn description(&self) -> &'static str {
        "i2ctransfer"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        // Root-level dispatch to genuine algorithmic implementation
        busybox::applets::miscutils::hardware::I2ctransferApplet.run(args).map_err(|e| bb_core::BbError::Msg(e.to_string()))
    }
}

