use crate::core::{Applet, Result};
use std::ffi::{OsStr, OsString};
use std::fs::{self};
use std::io::{self, BufRead, Read};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

pub struct PatchApplet;

impl Applet for PatchApplet {
    fn name(&self) -> &'static str {
        "patch"
    }
    fn description(&self) -> &'static str {
        "Apply a patch to files"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut strip_count = 0usize;
        let mut reverse = false;
        let mut patch_file: Option<PathBuf> = None;
        let mut target_file: Option<PathBuf> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b.starts_with(b"-p") {
                if b.len() > 2 {
                    strip_count = std::str::from_utf8(&b[2..])
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                } else if i + 1 < args.len() {
                    i += 1;
                    strip_count = std::str::from_utf8(args[i].as_bytes())
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                }
            } else if b == b"-R" || b == b"--reverse" {
                reverse = true;
            } else if b == b"-i" {
                if i + 1 < args.len() {
                    i += 1;
                    patch_file = Some(PathBuf::from(&args[i]));
                }
            } else if b.starts_with(b"-") {
            } else if target_file.is_none() {
                target_file = Some(PathBuf::from(&args[i]));
            }
            i += 1;
        }

        let patch_data = match &patch_file {
            Some(p) => match fs::read(p) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("patch: {}: {}", p.display(), e);
                    return Ok(1);
                }
            },
            None => {
                let mut buf = Vec::new();
                io::stdin().read_to_end(&mut buf)?;
                buf
            }
        };

        let mut deduced_file: Option<PathBuf> = None;
        let mut lines = Vec::new();
        for l in patch_data.split(|&b| b == b'\n') {
            lines.push(l);
        }

        let mut hunk_lines: Vec<&[u8]> = Vec::new();

        for l in &lines {
            if l.starts_with(b"--- ") {
                let name = l[4..].split(|&c| c == b'\t' || c == b' ').next().unwrap();
                if deduced_file.is_none() {
                    let mut parts: Vec<&[u8]> = name.split(|&c| c == b'/').collect();
                    if strip_count < parts.len() {
                        parts = parts[strip_count..].to_vec();
                    }
                    let stripped = parts.join(&b'/');
                    deduced_file = Some(PathBuf::from(OsStr::from_bytes(&stripped)));
                }
            } else if l.starts_with(b"+++ ") {
                let name = l[4..].split(|&c| c == b'\t' || c == b' ').next().unwrap();
                let mut parts: Vec<&[u8]> = name.split(|&c| c == b'/').collect();
                if strip_count < parts.len() {
                    parts = parts[strip_count..].to_vec();
                }
                let stripped = parts.join(&b'/');
                deduced_file = Some(PathBuf::from(OsStr::from_bytes(&stripped)));
            } else if l.starts_with(b"@@ ") {
            } else if l.starts_with(b"+")
                || l.starts_with(b"-")
                || l.starts_with(b" ")
                || l.is_empty()
            {
                hunk_lines.push(l);
            }
        }

        let file_to_patch = target_file.or(deduced_file);
        let path = match file_to_patch {
            Some(p) => p,
            None => {
                eprintln!("patch: cannot determine file to patch");
                return Ok(1);
            }
        };

        let orig_content = fs::read(&path).unwrap_or_default();
        let mut orig_lines: Vec<Vec<u8>> = orig_content
            .split(|&b| b == b'\n')
            .map(|s| s.to_vec())
            .collect();
        if orig_lines.last() == Some(&Vec::new()) {
            orig_lines.pop();
        }

        let mut new_lines = Vec::new();
        let mut orig_idx = 0;

        for hl in hunk_lines {
            if hl.is_empty() {
                continue;
            }
            let tag = hl[0];
            let content = &hl[1..];
            let effective_tag = if reverse {
                if tag == b'+' {
                    b'-'
                } else if tag == b'-' {
                    b'+'
                } else {
                    tag
                }
            } else {
                tag
            };

            match effective_tag {
                b' ' => {
                    new_lines.push(content.to_vec());
                    orig_idx += 1;
                }
                b'+' => {
                    new_lines.push(content.to_vec());
                }
                b'-' => {
                    orig_idx += 1;
                }
                _ => {}
            }
        }

        while orig_idx < orig_lines.len() {
            new_lines.push(orig_lines[orig_idx].clone());
            orig_idx += 1;
        }

        let mut out_data = Vec::new();
        for l in new_lines {
            out_data.extend_from_slice(&l);
            out_data.push(b'\n');
        }

        fs::write(&path, out_data)?;
        println!("patching file {}", path.display());

        Ok(0)
    }
}

