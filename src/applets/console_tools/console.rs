use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;

const PIO_FONT: libc::c_ulong = 0x4B61;
const KDGKBENT: libc::c_ulong = 0x4B46;
const KDSKBENT: libc::c_ulong = 0x4B47;
const KDSETKEYCODE: libc::c_ulong = 0x4B4D;
const KDGKBMODE: libc::c_ulong = 0x4B44;
const KDSKBMODE: libc::c_ulong = 0x4B45;
const KIOCSOUND: libc::c_ulong = 0x4B2F;
const VT_ACTIVATE: libc::c_ulong = 0x5606;
const VT_WAITACTIVE: libc::c_ulong = 0x5607;
const VT_GETSTATE: libc::c_ulong = 0x5603;
const VT_DISALLOCATE: libc::c_ulong = 0x5608;

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;

const CDROMEJECT: libc::c_ulong = 0x5309;
const CDROMCLOSETRAY: libc::c_ulong = 0x5319;
const HDIO_GETGEO: libc::c_ulong = 0x0301;
const HDIO_DRIVE_CMD: libc::c_ulong = 0x031F;

#[repr(C)]
struct Kbentry {
    kb_table: u8,
    kb_index: u8,
    kb_value: u16,
}

#[repr(C)]
struct Kbkeycode {
    scancode: libc::c_uint,
    keycode: libc::c_uint,
}

#[repr(C)]
struct VtStat {
    v_active: libc::c_ushort,
    v_signal: libc::c_ushort,
    v_state: libc::c_ushort,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct FbVarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
#[derive(Default)]
struct HdGeometry {
    heads: u8,
    sectors: u8,
    cylinders: u16,
    start: libc::c_ulong,
}

fn open_console() -> io::Result<File> {
    for dev in &["/dev/tty0", "/dev/tty", "/dev/console", "/dev/tty1"] {
        if let Ok(f) = OpenOptions::new().read(true).write(true).open(dev) {
            return Ok(f);
        }
        if let Ok(f) = OpenOptions::new().read(true).open(dev) {
            return Ok(f);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "cannot open any console",
    ))
}

fn parse_u32(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut val = 0u32;
    for &b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        val = val.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(val)
}

pub struct LoadfontApplet;

impl Applet for LoadfontApplet {
    fn name(&self) -> &'static str {
        "loadfont"
    }

    fn description(&self) -> &'static str {
        "Load console font"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut font_data = Vec::new();
        if args.len() <= 1 {
            let mut stdin = io::stdin().lock();
            let _ = stdin.read_to_end(&mut font_data);
        } else {
            let path = Path::new(&args[1]);
            match File::open(path) {
                Ok(mut f) => {
                    let _ = f.read_to_end(&mut font_data);
                }
                Err(e) => {
                    eprintln!("loadfont: {}: {}", path.display(), e);
                    return Ok(1);
                }
            }
        }

        if font_data.is_empty() {
            eprintln!("loadfont: empty font data");
            return Ok(1);
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("loadfont: {}", e);
                return Ok(1);
            }
        };

        let ret = unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                PIO_FONT,
                font_data.as_ptr() as *const libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "loadfont: PIO_FONT ioctl failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct SetfontApplet;

impl Applet for SetfontApplet {
    fn name(&self) -> &'static str {
        "setfont"
    }

    fn description(&self) -> &'static str {
        "Load EGA/VGA console screen font"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: setfont FONT_FILE");
            return Ok(1);
        }
        let font_path = &args[1];
        let mut font_data = Vec::new();
        let path = Path::new(font_path);
        match File::open(path) {
            Ok(mut f) => {
                let _ = f.read_to_end(&mut font_data);
            }
            Err(e) => {
                eprintln!("setfont: {}: {}", path.display(), e);
                return Ok(1);
            }
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("setfont: {}", e);
                return Ok(1);
            }
        };

        let raw_font = if font_data.len() > 4 && font_data[0] == 0x36 && font_data[1] == 0x04 {
            let hdr_size = 4;
            &font_data[hdr_size..]
        } else {
            &font_data[..]
        };

        let ret = unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                PIO_FONT,
                raw_font.as_ptr() as *const libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!("setfont: PIO_FONT failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct DumpkmapApplet;

impl Applet for DumpkmapApplet {
    fn name(&self) -> &'static str {
        "dumpkmap"
    }

    fn description(&self) -> &'static str {
        "Dump keyboard translation table to standard output"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("dumpkmap: {}", e);
                return Ok(1);
            }
        };

        let mut stdout = io::stdout().lock();

        let _ = stdout.write_all(b"bkeymap\0");

        let max_tables: u8 = 7;
        let mut flags = [0u8; 256];
        for (i, item) in flags.iter_mut().enumerate().take(max_tables as usize) {
            *item = 1 << i;
        }
        let _ = stdout.write_all(&flags);

        for table in 0..max_tables {
            for keycode in 0..128u8 {
                let mut ent = Kbentry {
                    kb_table: table,
                    kb_index: keycode,
                    kb_value: 0,
                };
                let ret = unsafe {
                    libc::ioctl(
                        console.as_raw_fd(),
                        KDGKBENT,
                        &mut ent as *mut Kbentry as *mut libc::c_void,
                    )
                };
                let val: u16 = if ret == 0 { ent.kb_value } else { 0 };
                let _ = stdout.write_all(&val.to_le_bytes());
            }
        }
        let _ = stdout.flush();
        Ok(0)
    }
}

pub struct LoadkmapApplet;

impl Applet for LoadkmapApplet {
    fn name(&self) -> &'static str {
        "loadkmap"
    }

    fn description(&self) -> &'static str {
        "Load keyboard translation table from standard input"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("loadkmap: {}", e);
                return Ok(1);
            }
        };

        let mut stdin = io::stdin().lock();
        let mut magic = [0u8; 8];
        if stdin.read_exact(&mut magic).is_err() || &magic != b"bkeymap\0" {
            eprintln!("loadkmap: bad magic header");
            return Ok(1);
        }

        let mut flags = [0u8; 256];
        if stdin.read_exact(&mut flags).is_err() {
            eprintln!("loadkmap: truncated keymap flags");
            return Ok(1);
        }

        for (table, &flag) in flags.iter().enumerate() {
            if flag == 0 {
                continue;
            }
            for keycode in 0..128u8 {
                let mut buf = [0u8; 2];
                if stdin.read_exact(&mut buf).is_err() {
                    break;
                }
                let val = u16::from_le_bytes(buf);
                let mut ent = Kbentry {
                    kb_table: table as u8,
                    kb_index: keycode,
                    kb_value: val,
                };
                unsafe {
                    libc::ioctl(
                        console.as_raw_fd(),
                        KDSKBENT,
                        &mut ent as *mut Kbentry as *mut libc::c_void,
                    );
                }
            }
        }
        Ok(0)
    }
}

pub struct SetkeycodesApplet;

impl Applet for SetkeycodesApplet {
    fn name(&self) -> &'static str {
        "setkeycodes"
    }

    fn description(&self) -> &'static str {
        "Set kernel scancode-to-keycode mapping"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        #[allow(clippy::manual_is_multiple_of)]
        if args.len() < 3 || (args.len() - 1) % 2 != 0 {
            eprintln!("Usage: setkeycodes SCANCODE KEYCODE ...");
            return Ok(1);
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("setkeycodes: {}", e);
                return Ok(1);
            }
        };

        let mut i = 1;
        while i < args.len() {
            let scan_str = args[i].as_bytes();
            let key_str = args[i + 1].as_bytes();
            let scan = if scan_str.starts_with(b"0x") || scan_str.starts_with(b"0X") {
                u32::from_str_radix(std::str::from_utf8(&scan_str[2..]).unwrap_or(""), 16).ok()
            } else {
                parse_u32(scan_str)
            };
            let key = parse_u32(key_str);

            match (scan, key) {
                (Some(s), Some(k)) => {
                    let mut a = Kbkeycode {
                        scancode: s,
                        keycode: k,
                    };
                    let ret = unsafe {
                        libc::ioctl(
                            console.as_raw_fd(),
                            KDSETKEYCODE,
                            &mut a as *mut Kbkeycode as *mut libc::c_void,
                        )
                    };
                    if ret < 0 {
                        eprintln!(
                            "setkeycodes: failed setting scancode {} to {}: {}",
                            s,
                            k,
                            io::Error::last_os_error()
                        );
                        return Ok(1);
                    }
                }
                _ => {
                    eprintln!("setkeycodes: invalid scancode or keycode");
                    return Ok(1);
                }
            }
            i += 2;
        }
        Ok(0)
    }
}

pub struct ShowkeyApplet;

impl Applet for ShowkeyApplet {
    fn name(&self) -> &'static str {
        "showkey"
    }

    fn description(&self) -> &'static str {
        "Display scancodes or keycodes sent by the keyboard"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut mode_k = true;
        let mut timeout_sec = 10;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-s" {
                mode_k = false;
            } else if b == b"-k" {
                mode_k = true;
            } else if b == b"-a" {
                mode_k = false;
            } else if b.starts_with(b"-t") {
                let rest = if b.len() > 2 {
                    &b[2..]
                } else if i + 1 < args.len() {
                    i += 1;
                    args[i].as_bytes()
                } else {
                    b""
                };
                if let Some(t) = parse_u32(rest) {
                    timeout_sec = t as i32;
                }
            }
            i += 1;
        }

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("showkey: {}", e);
                return Ok(1);
            }
        };

        let mut old_mode: libc::c_int = 0;
        unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                KDGKBMODE,
                &mut old_mode as *mut libc::c_int as *mut libc::c_void,
            );
        }

        let new_mode = if mode_k { 2 } else { 1 };
        unsafe {
            libc::ioctl(console.as_raw_fd(), KDSKBMODE, new_mode as libc::c_ulong);
        }

        println!(
            "Press any keys (program exits {}s after last keypress)...",
            timeout_sec
        );
        let mut buf = [0u8; 64];
        let fd = console.as_raw_fd();
        loop {
            let mut pollfd = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_sec * 1000) };
            if ret <= 0 {
                break;
            }
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if n <= 0 {
                break;
            }
            for &byte in &buf[..n as usize] {
                let keycode = (byte & 0x7F) as u32;
                let release = (byte & 0x80) != 0;
                if mode_k {
                    println!(
                        "keycode {} {}",
                        keycode,
                        if release { "release" } else { "press" }
                    );
                } else {
                    println!("0x{:02x}", byte);
                }
            }
        }

        unsafe {
            libc::ioctl(console.as_raw_fd(), KDSKBMODE, old_mode as libc::c_ulong);
        }
        Ok(0)
    }
}

pub struct KbdModeApplet;

impl Applet for KbdModeApplet {
    fn name(&self) -> &'static str {
        "kbd_mode"
    }

    fn description(&self) -> &'static str {
        "Report or set keyboard mode"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("kbd_mode: {}", e);
                return Ok(1);
            }
        };

        if args.len() <= 1 {
            let mut mode: libc::c_int = 0;
            let ret = unsafe {
                libc::ioctl(
                    console.as_raw_fd(),
                    KDGKBMODE,
                    &mut mode as *mut libc::c_int as *mut libc::c_void,
                )
            };
            if ret < 0 {
                eprintln!("kbd_mode: KDGKBMODE failed: {}", io::Error::last_os_error());
                return Ok(1);
            }
            let mode_str = match mode {
                0 => "raw (SCANCODE)",
                1 => "mediumraw (KEYCODE)",
                2 => "default (ASCII)",
                3 => "Unicode (UTF-8)",
                4 => "off",
                _ => "unknown",
            };
            println!("The keyboard is in {} mode", mode_str);
            return Ok(0);
        }

        let arg = args[1].as_bytes();
        let new_mode = match arg {
            b"-s" => 0,
            b"-k" => 1,
            b"-a" => 2,
            b"-u" => 3,
            _ => {
                eprintln!("Usage: kbd_mode [-a|-k|-s|-u]");
                return Ok(1);
            }
        };

        let ret = unsafe { libc::ioctl(console.as_raw_fd(), KDSKBMODE, new_mode as libc::c_ulong) };
        if ret < 0 {
            eprintln!("kbd_mode: KDSKBMODE failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct FgconsoleApplet;

impl Applet for FgconsoleApplet {
    fn name(&self) -> &'static str {
        "fgconsole"
    }

    fn description(&self) -> &'static str {
        "Print the number of the active virtual console"
    }

    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("fgconsole: {}", e);
                return Ok(1);
            }
        };

        let mut vt = VtStat {
            v_active: 0,
            v_signal: 0,
            v_state: 0,
        };
        let ret = unsafe {
            libc::ioctl(
                console.as_raw_fd(),
                VT_GETSTATE,
                &mut vt as *mut VtStat as *mut libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "fgconsole: VT_GETSTATE failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        println!("{}", vt.v_active);
        Ok(0)
    }
}

pub struct ChvtApplet;

impl Applet for ChvtApplet {
    fn name(&self) -> &'static str {
        "chvt"
    }

    fn description(&self) -> &'static str {
        "Change foreground virtual terminal"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: chvt N");
            return Ok(1);
        }

        let vt_num = match parse_u32(args[1].as_bytes()) {
            Some(n) => n,
            None => {
                eprintln!("chvt: invalid number '{}'", args[1].to_string_lossy());
                return Ok(1);
            }
        };

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("chvt: {}", e);
                return Ok(1);
            }
        };

        let fd = console.as_raw_fd();
        let r1 = unsafe { libc::ioctl(fd, VT_ACTIVATE, vt_num as libc::c_ulong) };
        if r1 < 0 {
            eprintln!("chvt: VT_ACTIVATE failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        let r2 = unsafe { libc::ioctl(fd, VT_WAITACTIVE, vt_num as libc::c_ulong) };
        if r2 < 0 {
            eprintln!("chvt: VT_WAITACTIVE failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct DeallocvtApplet;

impl Applet for DeallocvtApplet {
    fn name(&self) -> &'static str {
        "deallocvt"
    }

    fn description(&self) -> &'static str {
        "Deallocate unused virtual consoles"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let vt_num = if args.len() > 1 {
            match parse_u32(args[1].as_bytes()) {
                Some(n) => n,
                None => {
                    eprintln!("deallocvt: invalid number");
                    return Ok(1);
                }
            }
        } else {
            0
        };

        let console = match open_console() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("deallocvt: {}", e);
                return Ok(1);
            }
        };

        let ret =
            unsafe { libc::ioctl(console.as_raw_fd(), VT_DISALLOCATE, vt_num as libc::c_ulong) };
        if ret < 0 {
            eprintln!("deallocvt: failed: {}", io::Error::last_os_error());
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct FbsetApplet;

impl Applet for FbsetApplet {
    fn name(&self) -> &'static str {
        "fbset"
    }

    fn description(&self) -> &'static str {
        "Show or set framebuffer video modes"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fb_dev = "/dev/fb0";
        let mut show_all = false;
        let mut set_geo = None;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-fb" && i + 1 < args.len() {
                i += 1;
                if let Ok(s) = std::str::from_utf8(args[i].as_bytes()) {
                    fb_dev = s;
                }
            } else if b == b"-a" || b == b"--all" {
                show_all = true;
            } else if (b == b"-g" || b == b"--geometry") && i + 5 < args.len() {
                let xr = parse_u32(args[i + 1].as_bytes()).unwrap_or(0);
                let yr = parse_u32(args[i + 2].as_bytes()).unwrap_or(0);
                let vxr = parse_u32(args[i + 3].as_bytes()).unwrap_or(xr);
                let vyr = parse_u32(args[i + 4].as_bytes()).unwrap_or(yr);
                let d = parse_u32(args[i + 5].as_bytes()).unwrap_or(32);
                set_geo = Some((xr, yr, vxr, vyr, d));
                i += 5;
            }
            let _ = show_all;
            i += 1;
        }

        let f = match OpenOptions::new().read(true).write(true).open(fb_dev) {
            Ok(f) => f,
            Err(_) => match OpenOptions::new().read(true).open(fb_dev) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("fbset: cannot open {}: {}", fb_dev, e);
                    return Ok(1);
                }
            },
        };

        let mut vinfo = FbVarScreeninfo::default();
        let ret = unsafe {
            libc::ioctl(
                f.as_raw_fd(),
                FBIOGET_VSCREENINFO,
                &mut vinfo as *mut FbVarScreeninfo as *mut libc::c_void,
            )
        };
        if ret < 0 {
            eprintln!(
                "fbset: FBIOGET_VSCREENINFO failed: {}",
                io::Error::last_os_error()
            );
            return Ok(1);
        }

        if let Some((xr, yr, vxr, vyr, d)) = set_geo {
            vinfo.xres = xr;
            vinfo.yres = yr;
            vinfo.xres_virtual = vxr;
            vinfo.yres_virtual = vyr;
            vinfo.bits_per_pixel = d;
            let ret2 = unsafe {
                libc::ioctl(
                    f.as_raw_fd(),
                    FBIOPUT_VSCREENINFO,
                    &mut vinfo as *mut FbVarScreeninfo as *mut libc::c_void,
                )
            };
            if ret2 < 0 {
                eprintln!(
                    "fbset: FBIOPUT_VSCREENINFO failed: {}",
                    io::Error::last_os_error()
                );
                return Ok(1);
            }
        } else {
            println!("mode \"{}x{}\"", vinfo.xres, vinfo.yres);
            println!(
                "    geometry {} {} {} {} {}",
                vinfo.xres,
                vinfo.yres,
                vinfo.xres_virtual,
                vinfo.yres_virtual,
                vinfo.bits_per_pixel
            );
            println!(
                "    timings {} {} {} {} {} {} {}",
                vinfo.pixclock,
                vinfo.left_margin,
                vinfo.right_margin,
                vinfo.upper_margin,
                vinfo.lower_margin,
                vinfo.hsync_len,
                vinfo.vsync_len
            );
            println!(
                "    rgba {}/{}, {}/{}, {}/{}, {}/{}",
                vinfo.red.length,
                vinfo.red.offset,
                vinfo.green.length,
                vinfo.green.offset,
                vinfo.blue.length,
                vinfo.blue.offset,
                vinfo.transp.length,
                vinfo.transp.offset
            );
            println!("endmode");
        }
        Ok(0)
    }
}

pub struct FbsplashApplet;

impl Applet for FbsplashApplet {
    fn name(&self) -> &'static str {
        "fbsplash"
    }

    fn description(&self) -> &'static str {
        "Display image centered on framebuffer"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut fb_dev = "/dev/fb0";
        let mut image_file = None;
        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-d" && i + 1 < args.len() {
                i += 1;
                if let Ok(s) = std::str::from_utf8(args[i].as_bytes()) {
                    fb_dev = s;
                }
            } else if b == b"-s" && i + 1 < args.len() {
                i += 1;
                image_file = Some(&args[i]);
            } else if b == b"-i" && i + 1 < args.len() {
                i += 1;
            }
            i += 1;
        }

        let img_path = match image_file {
            Some(p) => p,
            None => {
                eprintln!("Usage: fbsplash -s IMAGE_FILE [-d FB_DEV]");
                return Ok(1);
            }
        };

        let mut img_data = Vec::new();
        let path = Path::new(img_path);
        match File::open(path) {
            Ok(mut f) => {
                let _ = f.read_to_end(&mut img_data);
            }
            Err(e) => {
                eprintln!("fbsplash: {}: {}", path.display(), e);
                return Ok(1);
            }
        }

        let fb_file = match OpenOptions::new().write(true).open(fb_dev) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("fbsplash: {}: {}", fb_dev, e);
                return Ok(1);
            }
        };

        let mut writer = io::BufWriter::new(fb_file);
        let _ = writer.write_all(&img_data);
        let _ = writer.flush();
        Ok(0)
    }
}

pub struct BeepApplet;

impl Applet for BeepApplet {
    fn name(&self) -> &'static str {
        "beep"
    }

    fn description(&self) -> &'static str {
        "Beep the console speaker"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut freq: u32 = 440;
        let mut length_ms: u32 = 200;
        let mut reps: u32 = 1;
        let mut delay_ms: u32 = 100;

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-f" && i + 1 < args.len() {
                i += 1;
                freq = parse_u32(args[i].as_bytes()).unwrap_or(440);
            } else if b == b"-l" && i + 1 < args.len() {
                i += 1;
                length_ms = parse_u32(args[i].as_bytes()).unwrap_or(200);
            } else if b == b"-r" && i + 1 < args.len() {
                i += 1;
                reps = parse_u32(args[i].as_bytes()).unwrap_or(1);
            } else if b == b"-d" && i + 1 < args.len() {
                i += 1;
                delay_ms = parse_u32(args[i].as_bytes()).unwrap_or(100);
            }
            i += 1;
        }

        if let Ok(console) = open_console() {
            let period = freq
                .checked_div(1)
                .and_then(|_| 1193180u32.checked_div(freq))
                .unwrap_or(0);
            for r in 0..reps {
                unsafe {
                    libc::ioctl(console.as_raw_fd(), KIOCSOUND, period as libc::c_ulong);
                    libc::usleep((length_ms * 1000) as libc::useconds_t);
                    libc::ioctl(console.as_raw_fd(), KIOCSOUND, 0 as libc::c_ulong);
                }
                if r + 1 < reps {
                    unsafe {
                        libc::usleep((delay_ms * 1000) as libc::useconds_t);
                    }
                }
            }
        } else {
            let mut out = io::stdout().lock();
            for _ in 0..reps {
                let _ = out.write_all(b"\x07");
                let _ = out.flush();
            }
        }
        Ok(0)
    }
}

pub struct EjectApplet;

impl Applet for EjectApplet {
    fn name(&self) -> &'static str {
        "eject"
    }

    fn description(&self) -> &'static str {
        "Eject removable media"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut close_tray = false;
        let mut dev_path = "/dev/cdrom";

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-t" || b == b"-c" {
                close_tray = true;
            } else if !b.starts_with(b"-") {
                if let Ok(s) = std::str::from_utf8(b) {
                    dev_path = s;
                }
            }
            i += 1;
        }

        let f = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(dev_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("eject: {}: {}", dev_path, e);
                return Ok(1);
            }
        };

        let cmd = if close_tray {
            CDROMCLOSETRAY
        } else {
            CDROMEJECT
        };
        let ret = unsafe { libc::ioctl(f.as_raw_fd(), cmd, 0 as libc::c_ulong) };
        if ret < 0 {
            eprintln!(
                "eject: failed on {}: {}",
                dev_path,
                io::Error::last_os_error()
            );
            return Ok(1);
        }
        Ok(0)
    }
}

pub struct HdparmApplet;

impl Applet for HdparmApplet {
    fn name(&self) -> &'static str {
        "hdparm"
    }

    fn description(&self) -> &'static str {
        "Get/set ATA/SATA drive parameters"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        if args.len() < 2 {
            eprintln!("Usage: hdparm [options] [device ...]");
            return Ok(1);
        }

        let mut get_geo = false;
        let mut standby = false;
        let mut dev_names = Vec::new();

        let mut i = 1;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-g" {
                get_geo = true;
            } else if b == b"-y" {
                standby = true;
            } else if !b.starts_with(b"-") {
                dev_names.push(&args[i]);
            }
            i += 1;
        }

        if dev_names.is_empty() {
            eprintln!("hdparm: no device specified");
            return Ok(1);
        }

        let mut ret_code = 0;
        for dev in dev_names {
            let path = Path::new(dev);
            let f = match OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("hdparm: {}: {}", path.display(), e);
                    ret_code = 1;
                    continue;
                }
            };

            println!("\n{}:", path.display());

            if standby {
                let mut cmd = [0xE0u8, 0, 0, 0];
                let ret = unsafe {
                    libc::ioctl(
                        f.as_raw_fd(),
                        HDIO_DRIVE_CMD,
                        cmd.as_mut_ptr() as *mut libc::c_void,
                    )
                };
                if ret < 0 {
                    eprintln!("  standby failed: {}", io::Error::last_os_error());
                    ret_code = 1;
                } else {
                    println!("  issuing standby command");
                }
            }

            if get_geo || !standby {
                let mut geo = HdGeometry::default();
                let ret = unsafe {
                    libc::ioctl(
                        f.as_raw_fd(),
                        HDIO_GETGEO,
                        &mut geo as *mut HdGeometry as *mut libc::c_void,
                    )
                };
                if ret == 0 {
                    println!(
                        " geometry      = {}/{}/{}, sectors = {}, start = {}",
                        geo.cylinders,
                        geo.heads,
                        geo.sectors,
                        geo.cylinders as u64 * geo.heads as u64 * geo.sectors as u64,
                        geo.start
                    );
                } else if get_geo {
                    eprintln!("  HDIO_GETGEO failed: {}", io::Error::last_os_error());
                    ret_code = 1;
                }
            }
        }
        Ok(ret_code)
    }
}
