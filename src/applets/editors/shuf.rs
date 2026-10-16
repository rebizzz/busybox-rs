use crate::core::fs::open_or_stdin;
use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ShufApplet;

impl Applet for ShufApplet {
    fn name(&self) -> &'static str {
        "shuf"
    }
    fn description(&self) -> &'static str {
        "Generate random permutations"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut count: Option<usize> = None;
        let mut echo_mode = false;
        let mut input_range: Option<(i64, i64)> = None;
        let mut out_file: Option<PathBuf> = None;
        let mut positional = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-e" {
                echo_mode = true;
            } else if b == b"-n" && i + 1 < args.len() {
                i += 1;
                count = std::str::from_utf8(args[i].as_bytes())
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b.starts_with(b"-n") && b.len() > 2 {
                count = std::str::from_utf8(&b[2..])
                    .ok()
                    .and_then(|s| s.parse().ok());
            } else if b == b"-o" && i + 1 < args.len() {
                i += 1;
                out_file = Some(PathBuf::from(&args[i]));
            } else if b == b"-i" && i + 1 < args.len() {
                i += 1;
                if let Some((start, end)) = parse_range(args[i].as_bytes()) {
                    input_range = Some((start, end));
                }
            } else if b.starts_with(b"-i") && b.len() > 2 {
                if let Some((start, end)) = parse_range(&b[2..]) {
                    input_range = Some((start, end));
                }
            } else if b.starts_with(b"-") && b != b"-" {
            } else {
                positional.push(&args[i]);
            }
            i += 1;
        }

        fn parse_range(b: &[u8]) -> Option<(i64, i64)> {
            let s = std::str::from_utf8(b).ok()?;
            let mut parts = s.split('-');
            let start = parts.next()?.parse().ok()?;
            let end = parts.next()?.parse().ok()?;
            Some((start, end))
        }

        let mut items: Vec<Vec<u8>> = Vec::new();

        if echo_mode {
            for p in positional {
                items.push(p.as_bytes().to_vec());
            }
        } else if let Some((start, end)) = input_range {
            for num in start..=end {
                items.push(num.to_string().into_bytes());
            }
        } else {
            let p = if positional.is_empty() {
                Path::new("-")
            } else {
                Path::new(positional[0])
            };
            let r: Box<dyn BufRead> = if p == Path::new("-") {
                Box::new(BufReader::new(io::stdin()))
            } else {
                Box::new(BufReader::new(open_or_stdin(p)?))
            };
            for l in r.lines() {
                items.push(l?.into_bytes());
            }
        }

        let mut rng_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let n = items.len();
        for k in (1..n).rev() {
            rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (rng_seed as usize) % (k + 1);
            items.swap(k, j);
        }

        let out_count = count.unwrap_or(items.len()).min(items.len());
        let mut out_data = Vec::new();
        for item in &items[..out_count] {
            out_data.extend_from_slice(item);
            out_data.push(b'\n');
        }

        if let Some(out_p) = out_file {
            fs::write(out_p, out_data)?;
        } else {
            io::stdout().lock().write_all(&out_data)?;
        }

        Ok(0)
    }
}

