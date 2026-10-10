use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::Write;

applet!(AsciiApplet, "ascii", "Print ASCII chart", run_ascii);
fn run_ascii(args: &[OsString]) -> Result<i32> {
    if !args.is_empty() {
        eprintln!("ascii: too many arguments");
        return Ok(1);
    }
    let names = [
        "NUL", "SOH", "STX", "ETX", "EOT", "ENQ", "ACK", "BEL", "BS", "HT", "LF", "VT", "FF", "CR",
        "SO", "SI", "DLE", "DC1", "DC2", "DC3", "DC4", "NAK", "SYN", "ETB", "CAN", "EM", "SUB",
        "ESC", "FS", "GS", "RS", "US", "SP",
    ];
    let mut out = wlock();
    let _ = writeln!(out, "Dec Hex Char Dec Hex Char Dec Hex Char Dec Hex Char");
    for r in 0u8..32 {
        for c in [r, r + 32, r + 64, r + 96] {
            let ch = if c < 33 {
                names[c as usize]
            } else if c == 127 {
                "DEL"
            } else {
                ""
            };
            if c < 33 || c == 127 {
                let _ = write!(out, "{:3} {:02X} {:<4}", c, c, ch);
            } else {
                let _ = write!(out, "{:3} {:02X} {}   ", c, c, c as char);
            }
            let _ = write!(out, "  ");
        }
        let _ = writeln!(out);
    }
    Ok(0)
}
