use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

pub const PIO_FONT: libc::c_ulong = 0x4B61;
pub const KDGKBENT: libc::c_ulong = 0x4B46;
pub const KDSKBENT: libc::c_ulong = 0x4B47;
pub const KDSETKEYCODE: libc::c_ulong = 0x4B4D;
pub const KDGKBMODE: libc::c_ulong = 0x4B44;
pub const KDSKBMODE: libc::c_ulong = 0x4B45;
pub const KIOCSOUND: libc::c_ulong = 0x4B2F;
pub const VT_ACTIVATE: libc::c_ulong = 0x5606;
pub const VT_WAITACTIVE: libc::c_ulong = 0x5607;
pub const VT_GETSTATE: libc::c_ulong = 0x5603;
pub const VT_DISALLOCATE: libc::c_ulong = 0x5608;

pub const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
pub const FBIOPUT_VSCREENINFO: libc::c_ulong = 0x4601;

pub const CDROMEJECT: libc::c_ulong = 0x5309;
pub const CDROMCLOSETRAY: libc::c_ulong = 0x5319;
pub const HDIO_GETGEO: libc::c_ulong = 0x0301;
pub const HDIO_DRIVE_CMD: libc::c_ulong = 0x031F;

#[repr(C)]
pub struct Kbentry {
    pub kb_table: u8,
    pub kb_index: u8,
    pub kb_value: u16,
}

#[repr(C)]
pub struct Kbkeycode {
    pub scancode: libc::c_uint,
    pub keycode: libc::c_uint,
}

#[repr(C)]
pub struct VtStat {
    pub v_active: libc::c_ushort,
    pub v_signal: libc::c_ushort,
    pub v_state: libc::c_ushort,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct FbBitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct FbVarScreeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: FbBitfield,
    pub green: FbBitfield,
    pub blue: FbBitfield,
    pub transp: FbBitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

#[repr(C)]
#[derive(Default)]
pub struct HdGeometry {
    pub heads: u8,
    pub sectors: u8,
    pub cylinders: u16,
    pub start: libc::c_ulong,
}

pub fn open_console() -> io::Result<File> {
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

pub fn parse_u32(s: &[u8]) -> Option<u32> {
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

