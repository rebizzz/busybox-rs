use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub struct LprApplet;
impl Applet for LprApplet {
    fn name(&self) -> &'static str {
        "lpr"
    }
    fn description(&self) -> &'static str {
        "Send files to a print spooling daemon"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut printer = "lp";
        let mut files = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-P" && i + 1 < args.len() {
                printer = args[i + 1].to_str().unwrap_or("lp");
                i += 2;
                continue;
            } else if !b.starts_with(b"-") {
                files.push(args[i].clone());
            }
            i += 1;
        }

        let spool_dir = Path::new("/var/spool/lpd").join(printer);
        let _ = fs::create_dir_all(&spool_dir);

        let job_id = unsafe { libc::getpid() };
        let df_name = format!("dfA{:03}localhost", job_id % 1000);
        let cf_name = format!("cfA{:03}localhost", job_id % 1000);

        let df_path = spool_dir.join(df_name);
        let cf_path = spool_dir.join(cf_name);

        let mut data_out = match File::create(&df_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("lpr: {}: {}", df_path.display(), e);
                return Ok(1);
            }
        };

        if files.is_empty() {
            let mut stdin = io::stdin().lock();
            let _ = io::copy(&mut stdin, &mut data_out);
        } else {
            for f in files {
                if let Ok(mut src) = File::open(&f) {
                    let _ = io::copy(&mut src, &mut data_out);
                }
            }
        }

        let _ = fs::write(&cf_path, "Hlocalhost\nPuser\nJjob\n");

        Ok(0)
    }
}
