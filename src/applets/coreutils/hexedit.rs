use super::common::*;
use crate::core::Result;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

applet!(
    HexeditApplet,
    "hexedit",
    "Show or patch bytes at offsets",
    run_hexedit
);
fn run_hexedit(args: &[OsString]) -> Result<i32> {
    if args.is_empty() {
        eprintln!("hexedit: needs a file argument");
        return Ok(1);
    }
    let path = Path::new(&args[0]).to_path_buf();
    if args.len() == 1 {
        let data = match read_all(path.as_os_str()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("hexedit: can't open '{}': {}", lossy(&args[0]), e);
                return Ok(1);
            }
        };
        let mut out = wlock();
        dump_lines(&mut out, &data, 0, 16, false);
        let _ = writeln!(out, "{:08x}", data.len());
        return Ok(0);
    }
    let off_s = lossy(&args[1]);
    let off: u64 = if let Some(h) = off_s
        .strip_prefix("0x")
        .or_else(|| off_s.strip_prefix("0X"))
    {
        match u64::from_str_radix(h, 16) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("hexedit: bad offset");
                return Ok(1);
            }
        }
    } else {
        match off_s.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("hexedit: bad offset");
                return Ok(1);
            }
        }
    };
    let mut bytes: Vec<u8> = Vec::new();
    for a in &args[2..] {
        let s = lossy(a);
        let t = s.trim();
        if !t.len().is_multiple_of(2) {
            eprintln!("hexedit: bad hex byte '{}'", t);
            return Ok(1);
        }
        for i in (0..t.len()).step_by(2) {
            match u8::from_str_radix(&t[i..i + 2], 16) {
                Ok(v) => bytes.push(v),
                Err(_) => {
                    eprintln!("hexedit: bad hex byte '{}'", t);
                    return Ok(1);
                }
            }
        }
    }
    use std::io::{Seek, SeekFrom};
    match File::options().read(true).write(true).open(&path) {
        Ok(mut f) => {
            if f.seek(SeekFrom::Start(off)).is_err() || f.write_all(&bytes).is_err() {
                eprintln!("hexedit: write failed");
                return Ok(1);
            }
            Ok(0)
        }
        Err(e) => {
            eprintln!("hexedit: can't open '{}': {}", lossy(&args[0]), e);
            Ok(1)
        }
    }
}

struct VolInfo {
    label: String,
    uuid: String,
    fstype: String,
}
fn probe_vol(path: &OsStr) -> Option<VolInfo> {
    use std::io::{Read, Seek, SeekFrom};
    let mut f = File::open(Path::new(path)).ok()?;

    let mut boot = [0u8; 512];
    if f.read_exact(&mut boot).is_ok() && boot[510] == 0x55 && boot[511] == 0xaa {
        let is_fat32 = boot[82..90] == *b"FAT32   ";
        let is_fat = is_fat32 || boot[54..62] == *b"FAT12   " || boot[54..62] == *b"FAT16   ";
        if is_fat {
            let (lab_off, ser_off) = if is_fat32 { (71, 67) } else { (43, 39) };
            let label = String::from_utf8_lossy(&boot[lab_off..lab_off + 11])
                .trim()
                .to_string();
            let ser = u32::from_le_bytes(boot[ser_off..ser_off + 4].try_into().unwrap_or([0; 4]));
            return Some(VolInfo {
                label,
                uuid: format!("{:04X}-{:04X}", ser >> 16, ser & 0xffff),
                fstype: "vfat".to_string(),
            });
        }
    }

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut sb = [0u8; 256];
        if f.read_exact(&mut sb).is_ok() && sb[56] == 0x53 && sb[57] == 0xef {
            let label = String::from_utf8_lossy(&sb[120..136])
                .trim_matches('\0')
                .trim()
                .to_string();
            let u = &sb[104..120];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );
            return Some(VolInfo {
                label,
                uuid,
                fstype: "ext2/3/4".to_string(),
            });
        }
    }

    if f.seek(SeekFrom::Start(1024)).is_ok() {
        let mut pg = vec![0u8; 4096];
        if f.read_exact(&mut pg).is_ok() && pg[4096 - 10..] == *b"SWAPSPACE2" {
            let label = String::from_utf8_lossy(&pg[1052..1068])
                .trim_matches('\0')
                .trim()
                .to_string();
            let u = &pg[1036..1052];
            let uuid = format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                u[0], u[1], u[2], u[3], u[4], u[5], u[6], u[7], u[8], u[9], u[10], u[11], u[12], u[13], u[14], u[15]
            );
            return Some(VolInfo {
                label,
                uuid,
                fstype: "swap".to_string(),
            });
        }
    }
    None
}

fn candidate_devs() -> Vec<OsString> {
    let mut v: Vec<OsString> = Vec::new();
    if let Ok(m) = std::fs::read("/proc/mounts") {
        for line in m.split(|&c| c == b'\n') {
            if let Some(sp) = line.iter().position(|&c| c == b' ') {
                let src = &line[..sp];
                if src.starts_with(b"/dev/") {
                    v.push(OsString::from(OsStr::from_bytes(src)));
                }
            }
        }
    }
    if let Ok(sys) = std::fs::read_dir("/sys/block") {
        for e in sys.flatten() {
            let nm = e.file_name();
            let devp = format!("/dev/{}", nm.to_string_lossy());
            let o = OsString::from(&devp);
            if !v.contains(&o) {
                v.push(o);
            }
        }
    }
    for guess in [
        "/dev/sda1",
        "/dev/vda1",
        "/dev/nvme0n1p1",
        "/dev/mmcblk0p1",
        "/dev/dm-0",
    ] {
        let o = OsString::from(guess);
        if !v.contains(&o) {
            v.push(o);
        }
    }
    v
}
