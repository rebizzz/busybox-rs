use crate::core::{Applet, Result};
use std::ffi::{CString, OsString};
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;

pub struct DateApplet;

fn print_date_usage() {
    eprintln!(
        "BusyBox-RS v{} multi-call binary.\n\nUsage: date [OPTIONS] [+FMT] [[-s] TIME]\n\nDisplay time (using +FMT), or set time",
        env!("CARGO_PKG_VERSION")
    );
}

impl Applet for DateApplet {
    fn name(&self) -> &'static str {
        "date"
    }
    fn description(&self) -> &'static str {
        "Print or set system date and time"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut utc = false;
        let mut format_arg: Option<Vec<u8>> = None;
        let mut date_str: Option<Vec<u8>> = None;

        let mut i = 0;
        while i < args.len() {
            let b = args[i].as_bytes();
            if b == b"-u" || b == b"--utc" || b == b"--universal" {
                utc = true;
            } else if b == b"-R" || b == b"--rfc-2822" || b == b"--rfc-822" {
                format_arg = Some(b"%a, %d %b %Y %H:%M:%S %z".to_vec());
            } else if b == b"-d" && i + 1 < args.len() {
                i += 1;
                date_str = Some(args[i].as_bytes().to_vec());
            } else if b.starts_with(b"-d") && b.len() > 2 {
                date_str = Some(b[2..].to_vec());
            } else if b.starts_with(b"+") {
                format_arg = Some(b[1..].to_vec());
            } else {
                print_date_usage();
                return Ok(1);
            }
            i += 1;
        }

        let mut tm: libc::tm = unsafe { std::mem::zeroed() };

        if let Some(ref d) = date_str {
            if d.starts_with(b"@") {
                let time_sec: libc::time_t = std::str::from_utf8(&d[1..])
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                unsafe {
                    if utc {
                        libc::gmtime_r(&time_sec, &mut tm);
                    } else {
                        libc::localtime_r(&time_sec, &mut tm);
                    }
                }
            } else if let Ok(s) = std::str::from_utf8(d) {
                let now: libc::time_t = unsafe { libc::time(std::ptr::null_mut()) };
                let mut base_tm: libc::tm = unsafe { std::mem::zeroed() };
                unsafe {
                    if utc {
                        libc::gmtime_r(&now, &mut base_tm);
                    } else {
                        libc::localtime_r(&now, &mut base_tm);
                    }
                }
                base_tm.tm_hour = 0;
                base_tm.tm_min = 0;
                base_tm.tm_sec = 0;

                let fmts = [
                    "%R",
                    "%T",
                    "%m.%d-%R",
                    "%m.%d-%T",
                    "%Y.%m.%d-%R",
                    "%Y.%m.%d-%T",
                    "%b %d %T %Y",
                    "%Y-%m-%d %R %z",
                    "%Y-%m-%d %T %z",
                    "%Y-%m-%d %R%z",
                    "%Y-%m-%d %T%z",
                    "%Y-%m-%d %R",
                    "%Y-%m-%d %T",
                    "%Y-%m-%d %H",
                    "%Y-%m-%d",
                ];

                let mut parsed = false;
                let mut has_tz = false;
                if let Ok(c_str) = CString::new(s) {
                    for f in &fmts {
                        if let Ok(c_fmt) = CString::new(*f) {
                            let mut test_tm = base_tm;
                            let res = unsafe {
                                libc::strptime(c_str.as_ptr(), c_fmt.as_ptr(), &mut test_tm)
                            };
                            if !res.is_null() && unsafe { *res == 0 } {
                                parsed = true;
                                has_tz = f.contains(&"%z");
                                tm = test_tm;
                                break;
                            }
                        }
                    }
                }

                if !parsed {
                    let dot_pos = s.find('.');
                    let main_part = match dot_pos {
                        Some(p) => &s[..p],
                        None => s,
                    };
                    let sec: i32 = match dot_pos {
                        Some(p) => match s[p + 1..].parse() {
                            Ok(sec_val) => sec_val,
                            Err(_) => {
                                print_date_usage();
                                return Ok(1);
                            }
                        },
                        None => 0,
                    };
                    if main_part.chars().all(|c| c.is_ascii_digit()) && main_part.len() == 12 {
                        let y: i32 = main_part[0..4].parse().unwrap_or(0);
                        let m: i32 = main_part[4..6].parse().unwrap_or(0);
                        let day: i32 = main_part[6..8].parse().unwrap_or(0);
                        let h: i32 = main_part[8..10].parse().unwrap_or(0);
                        let min: i32 = main_part[10..12].parse().unwrap_or(0);
                        tm = base_tm;
                        tm.tm_year = y - 1900;
                        tm.tm_mon = m - 1;
                        tm.tm_mday = day;
                        tm.tm_hour = h;
                        tm.tm_min = min;
                        tm.tm_sec = sec;
                        parsed = true;
                    }
                }

                if !parsed {
                    print_date_usage();
                    return Ok(1);
                }

                unsafe {
                    let t: libc::time_t = if has_tz {
                        tm.tm_sec -= tm.tm_gmtoff as libc::c_int;
                        tm.tm_isdst = 0;
                        libc::timegm(&mut tm)
                    } else if utc {
                        tm.tm_isdst = 0;
                        libc::timegm(&mut tm)
                    } else {
                        tm.tm_isdst = -1;
                        libc::mktime(&mut tm)
                    };
                    if utc {
                        libc::gmtime_r(&t, &mut tm);
                    } else {
                        libc::localtime_r(&t, &mut tm);
                    }
                }
            } else {
                print_date_usage();
                return Ok(1);
            }
        } else {
            let now: libc::time_t = unsafe { libc::time(std::ptr::null_mut()) };
            unsafe {
                if utc {
                    libc::gmtime_r(&now, &mut tm);
                } else {
                    libc::localtime_r(&now, &mut tm);
                }
            }
        }

        unsafe {
            let fmt_str = if let Some(f) = format_arg {
                f
            } else if utc {
                b"%a %b %e %H:%M:%S UTC %Y".to_vec()
            } else {
                b"%a %b %e %H:%M:%S %Z %Y".to_vec()
            };
            let cfmt = CString::new(fmt_str).unwrap_or_default();
            let mut buf = [0u8; 256];
            let len = libc::strftime(
                buf.as_mut_ptr() as *mut libc::c_char,
                buf.len(),
                cfmt.as_ptr(),
                &tm,
            );
            if len > 0 {
                let out = io::stdout();
                let mut lock = out.lock();
                lock.write_all(&buf[..len])?;
                lock.write_all(b"\n")?;
            }
        }

        Ok(0)
    }
}

