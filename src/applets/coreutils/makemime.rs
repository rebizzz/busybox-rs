use super::common::*;
use crate::core::Result;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;

applet!(
    MakemimeApplet,
    "makemime",
    "Create MIME-encoded message",
    run_makemime
);
fn run_makemime(args: &[OsString]) -> Result<i32> {
    let (mut ctype, mut cdisp, mut eol) = (
        String::from("application/octet-stream"),
        None::<String>,
        "\n",
    );
    let mut files: Vec<OsString> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let b = ab(&args[i]);
        if b == b"-c" {
            i += 1;
            if i >= args.len() {
                eprintln!("makemime: -c needs an argument");
                return Ok(1);
            }
            ctype = lossy(&args[i]);
        } else if b.starts_with(b"-c") && b.len() > 2 {
            ctype = String::from_utf8_lossy(&b[2..]).into_owned();
        } else if b == b"-C" {
            i += 1;
            if i >= args.len() {
                eprintln!("makemime: -C needs an argument");
                return Ok(1);
            }
            cdisp = Some(lossy(&args[i]));
        } else if b == b"-e" {
            eol = "\r\n";
        } else if b.len() > 1 && b[0] == b'-' && b != b"-" {
            eprintln!("makemime: invalid option '{}'", lossy(&args[i]));
            return Ok(1);
        } else {
            files.push(args[i].clone());
        }
        i += 1;
    }
    let inp: Vec<u8> = if files.is_empty() {
        let mut v = Vec::new();
        if std::io::stdin().lock().read_to_end(&mut v).is_err() {
            return Ok(1);
        }
        v
    } else {
        match read_all(files[0].as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("makemime: can't open '{}': {}", lossy(&files[0]), e);
                return Ok(1);
            }
        }
    };
    let mut out = wlock();
    let disp = cdisp.as_deref().unwrap_or("attachment");
    let _ = write!(out, "Content-Type: {}{}Content-Transfer-Encoding: base64{}Content-Disposition: {}; filename=\"{}\"{}", ctype, eol, eol, disp, files.first().map(lossy).unwrap_or_else(|| "stdin".to_string()), eol);
    let _ = write!(out, "{}", eol);
    for c in b64_enc(&inp).chunks(76) {
        let _ = out.write_all(c);
        let _ = out.write_all(eol.as_bytes());
    }
    Ok(0)
}
