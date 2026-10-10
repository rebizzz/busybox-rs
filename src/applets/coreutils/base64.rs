use crate::core::Result;
use std::ffi::OsString;
use super::base32::*;

applet!(
    Base64Applet,
    "base64",
    "Base64 encode or decode",
    run_base64
);
fn run_base64(args: &[OsString]) -> Result<i32> {
    bxx_main("base64", args, false)
}

