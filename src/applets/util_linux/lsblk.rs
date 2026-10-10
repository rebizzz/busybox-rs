use crate::core::{Applet, Result};
use super::common::*;
use std::ffi::OsString;
use std::fs::{self};
use std::io::{self, BufRead, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

struct BlkDev {
    name: Vec<u8>,
    maj: u32,
    min: u32,
    size_bytes: u64,
    rm: u8,
    ro: u8,
    is_part: bool,
    mountpoint: Vec<u8>,
}

fn read_u64_file(p: &Path) -> u64 {
    fs::read(p)
        .ok()
        .and_then(|d| {
            let s = std::str::from_utf8(&d).ok()?.trim();
            s.parse::<u64>().ok()
        })
        .unwrap_or(0)
}

fn parse_majmin(b: &[u8]) -> Option<(u32, u32)> {
    let s = std::str::from_utf8(b).ok()?.trim();
    let (a, c) = s.split_once(':')?;
    Some((a.trim().parse().ok()?, c.trim().parse().ok()?))
}

pub struct LsblkApplet;
impl Applet for LsblkApplet {
    fn name(&self) -> &'static str {
        "lsblk"
    }
    fn description(&self) -> &'static str {
        "List block devices"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut show_all = false;
        let mut bytes = false;
        let mut nodeps = false;
        let mut exclude: Option<Vec<u32>> = None;
        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-a" || b == b"--all" {
                show_all = true;
            } else if b == b"-b" || b == b"--bytes" {
                bytes = true;
            } else if b == b"-d" || b == b"--nodeps" {
                nodeps = true;
            } else if b == b"-e" || b == b"--exclude" {
                i += 1;
                if i >= args.len() {
                    eprintln!("lsblk: option requires an argument");
                    return Ok(1);
                }
                exclude = Some(parse_maj_list(args[i].as_bytes()));
            } else if b.starts_with(b"-e") && b.len() > 2 {
                exclude = Some(parse_maj_list(&b[2..]));
            } else {
                eprintln!("lsblk: invalid option '{}'", String::from_utf8_lossy(b));
                return Ok(1);
            }
            i += 1;
        }

        let excluded: Vec<u32> = if show_all {
            Vec::new()
        } else {
            exclude.unwrap_or_else(|| vec![1])
        };

        let mut mounted: Vec<(Vec<u8>, u64)> = Vec::new();
        for line in fs::read("/proc/mounts")
            .unwrap_or_default()
            .split(|&b| b == b'\n')
        {
            let mut parts = line.split(|&b| b == b' ');
            parts.next();
            if let Some(mnt) = parts.next() {
                let mp = Path::new(std::ffi::OsStr::from_bytes(mnt));
                if let Ok(st) = fs::metadata(mp) {
                    use std::os::unix::fs::MetadataExt;
                    mounted.push((mnt.to_vec(), st.dev()));
                }
            }
        }
        let mut devs: Vec<BlkDev> = Vec::new();
        let mut names: Vec<Vec<u8>> = fs::read_dir("/sys/block")
            .map(|d| {
                d.flatten()
                    .map(|e| e.file_name().as_bytes().to_vec())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        for name in &names {
            let base = Path::new("/sys/block").join(std::ffi::OsStr::from_bytes(name));
            let dev_raw = fs::read(base.join("dev")).unwrap_or_default();
            let (maj, min) = match parse_majmin(&dev_raw) {
                Some(v) => v,
                None => continue,
            };
            if excluded.contains(&maj) {
                continue;
            }
            let want = libc::makedev(maj, min);
            let mp = mounted
                .iter()
                .find(|(_, d)| *d == want)
                .map(|(m, _)| m.clone())
                .unwrap_or_default();
            devs.push(BlkDev {
                name: name.clone(),
                maj,
                min,
                size_bytes: read_u64_file(&base.join("size")).saturating_mul(512),
                rm: read_u64_file(&base.join("removable")) as u8,
                ro: read_u64_file(&base.join("ro")) as u8,
                is_part: false,
                mountpoint: mp,
            });
            if nodeps {
                continue;
            }
            let mut kids: Vec<Vec<u8>> = fs::read_dir(&base)
                .map(|d| {
                    d.flatten()
                        .map(|e| e.file_name().as_bytes().to_vec())
                        .collect()
                })
                .unwrap_or_default();
            kids.sort();
            for kid in &kids {
                let kp = base.join(std::ffi::OsStr::from_bytes(kid));
                if !kp.join("partition").exists() {
                    continue;
                }
                let kdev = fs::read(kp.join("dev")).unwrap_or_default();
                let (kmaj, kmin) = match parse_majmin(&kdev) {
                    Some(v) => v,
                    None => continue,
                };

                let kwant = libc::makedev(kmaj, kmin);
                let kmp = mounted
                    .iter()
                    .find(|(_, d)| *d == kwant)
                    .map(|(m, _)| m.clone())
                    .unwrap_or_default();
                devs.push(BlkDev {
                    name: kid.clone(),
                    maj: kmaj,
                    min: kmin,
                    size_bytes: read_u64_file(&kp.join("size")).saturating_mul(512),
                    rm: read_u64_file(&kp.join("removable")) as u8,
                    ro: read_u64_file(&kp.join("ro")) as u8,
                    is_part: true,
                    mountpoint: kmp,
                });
            }
        }
        let stdout = io::stdout();
        let mut out = stdout.lock();
        out.write_all(b"NAME MAJ:MIN RM SIZE RO TYPE MOUNTPOINT\n")?;

        let mut line: Vec<u8> = Vec::new();
        for d in &devs {
            line.clear();
            if d.is_part {
                line.extend_from_slice(b"  ");
            }
            line.extend_from_slice(&d.name);
            line.push(b' ');
            put_num(&mut line, d.maj as u64);
            line.push(b':');
            put_num(&mut line, d.min as u64);
            line.push(b' ');
            line.push(b'0' + (d.rm & 1));
            line.push(b' ');
            if bytes {
                put_num(&mut line, d.size_bytes);
            } else {
                put_human(&mut line, d.size_bytes);
            }
            line.push(b' ');
            line.push(b'0' + (d.ro & 1));
            line.push(b' ');

            let typ: &[u8] = if d.is_part {
                b"part"
            } else if d.name.starts_with(b"loop") {
                b"loop"
            } else if d.name.starts_with(b"ram") {
                b"ram"
            } else {
                b"disk"
            };
            line.extend_from_slice(typ);
            line.push(b' ');
            line.extend_from_slice(&d.mountpoint);
            line.push(b'\n');
            out.write_all(&line)?;
        }
        Ok(0)
    }
}

fn parse_maj_list(b: &[u8]) -> Vec<u32> {
    b.split(|&c| c == b',')
        .filter_map(|s| std::str::from_utf8(s).ok()?.trim().parse::<u32>().ok())
        .collect()
}
