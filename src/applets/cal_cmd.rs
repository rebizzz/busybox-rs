use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;

pub struct CalApplet;

// Days in each month for non-leap year (index 0 is unused, 1-12)
const DAYS_IN_MONTH: [u32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const SEP1752: [u32; 19] = [
    1, 2, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
];

const FIRST_MISSING_DAY: i64 = 639787; // 3 Sep 1752
const NUMBER_MISSING_DAYS: i64 = 11;
const SATURDAY: i64 = 6;
const MAXDAYS: usize = 42;
const SPACE: i32 = -1;

fn leap_year(yr: u32) -> bool {
    if yr <= 1752 {
        yr.is_multiple_of(4)
    } else {
        (yr.is_multiple_of(4) && !yr.is_multiple_of(100)) || yr.is_multiple_of(400)
    }
}

fn centuries_since_1700(yr: u32) -> i64 {
    if yr > 1700 {
        (yr as i64 / 100) - 17
    } else {
        0
    }
}

fn quad_centuries_since_1700(yr: u32) -> i64 {
    if yr > 1600 {
        ((yr as i64) - 1600) / 400
    } else {
        0
    }
}

fn leap_years_since_year_1(yr: u32) -> i64 {
    (yr as i64 / 4) - centuries_since_1700(yr) + quad_centuries_since_1700(yr)
}

fn day_array(month: u32, year: u32, weekstart: usize, julian: bool, days: &mut [i32; MAXDAYS]) {
    days.fill(SPACE);

    if month == 9 && year == 1752 {
        let j_offset = if julian { 244 } else { 0 };
        for (oday, &val) in SEP1752.iter().enumerate() {
            let idx = oday + 2 - weekstart;
            days[idx] = (val + j_offset) as i32;
        }
        return;
    }

    let mut day: i64 = 1;
    if month > 2 && leap_year(year) {
        day += 1;
    }

    let mut i = month as usize;
    while i > 1 {
        i -= 1;
        day += DAYS_IN_MONTH[i] as i64;
    }

    let temp: i64 = ((year - 1) as i64) * 365 + leap_years_since_year_1(year - 1) + day;
    let dw_raw = if temp < FIRST_MISSING_DAY {
        (temp - 1 + SATURDAY) % 7
    } else {
        ((temp - 1 + SATURDAY) - NUMBER_MISSING_DAYS) % 7
    };
    let dw = ((dw_raw - (weekstart as i64) + 7) % 7) as usize;

    let start_day: u32 = if julian { day as u32 } else { 1 };
    let mut dm = DAYS_IN_MONTH[month as usize];
    if month == 2 && leap_year(year) {
        dm += 1;
    }

    for (slot, d) in days[dw..].iter_mut().zip(start_day..start_day + dm) {
        *slot = d as i32;
    }
}

fn trim_trailing_spaces_and_print(s: &str) {
    let trimmed = s.trim_end_matches(' ');
    println!("{}", trimmed);
}

fn build_row(julian: bool, dp: &[i32]) -> String {
    let mut res = String::new();
    for &day in dp.iter().take(7) {
        if day != SPACE {
            if julian {
                res.push_str(&format!("{:4}", day));
            } else {
                res.push_str(&format!("{:3}", day));
            }
        } else {
            let width = if julian { 4 } else { 3 };
            for _ in 0..width {
                res.push(' ');
            }
        }
    }
    res
}

fn center_string(s: &str, width: usize) -> String {
    let len = s.len();
    if len >= width {
        s.to_string()
    } else {
        let left_pad = (width - len) / 2;
        let right_pad = width - len - left_pad;
        format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
    }
}

fn get_current_date() -> (u32, u32) {
    unsafe {
        let mut t: libc::time_t = 0;
        libc::time(&mut t);
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        ((tm.tm_mon + 1) as u32, (tm.tm_year + 1900) as u32)
    }
}

fn show_usage() {
    eprintln!("Usage: cal [-jmy] [[MONTH] YEAR]\n\nDisplay a calendar\n\n\t-j\tUse julian dates\n\t-m\tWeek starts on Monday\n\t-y\tDisplay the entire year");
}

impl Applet for CalApplet {
    fn name(&self) -> &'static str {
        "cal"
    }

    fn description(&self) -> &'static str {
        "Display a calendar"
    }

    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut opt_julian = false;
        let mut opt_monday = false;
        let mut opt_year = false;
        let mut positional = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            let bytes = arg.as_bytes();
            if bytes == b"--" {
                for next_arg in &args[i + 1..] {
                    positional.push(next_arg);
                }
                break;
            } else if bytes.starts_with(b"-") && bytes.len() > 1 {
                for &b in &bytes[1..] {
                    match b {
                        b'j' => opt_julian = true,
                        b'm' => opt_monday = true,
                        b'y' => opt_year = true,
                        _ => {
                            eprintln!("cal: invalid option -- '{}'", b as char);
                            show_usage();
                            return Ok(1);
                        }
                    }
                }
            } else {
                positional.push(arg);
            }
            i += 1;
        }

        let weekstart = if opt_monday { 1 } else { 0 };
        let (cur_month, cur_year) = get_current_date();

        let (month, year) = match positional.len() {
            0 => {
                if opt_year {
                    (0, cur_year)
                } else {
                    (cur_month, cur_year)
                }
            }
            1 => {
                let y_str = positional[0].to_string_lossy();
                let y: u32 = match y_str.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("cal: invalid number '{}'", y_str);
                        return Ok(1);
                    }
                };
                if !(1..=9999).contains(&y) {
                    eprintln!("cal: number {} is not in 1..9999 range", y);
                    return Ok(1);
                }
                (0, y)
            }
            2 => {
                let m_str = positional[0].to_string_lossy();
                let m: u32 = match m_str.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("cal: invalid number '{}'", m_str);
                        return Ok(1);
                    }
                };
                if !(1..=12).contains(&m) {
                    eprintln!("cal: number {} is not in 1..12 range", m);
                    return Ok(1);
                }

                let y_str = positional[1].to_string_lossy();
                let y: u32 = match y_str.parse() {
                    Ok(val) => val,
                    Err(_) => {
                        eprintln!("cal: invalid number '{}'", y_str);
                        return Ok(1);
                    }
                };
                if !(1..=9999).contains(&y) {
                    eprintln!("cal: number {} is not in 1..9999 range", y);
                    return Ok(1);
                }

                if opt_year {
                    (0, y)
                } else {
                    (m, y)
                }
            }
            _ => {
                show_usage();
                return Ok(1);
            }
        };

        // Day headings
        let day_names = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let day_headings = if opt_julian {
            let mut s = String::new();
            for d in 0..7 {
                let idx = (d + weekstart) % 7;
                s.push_str(&format!(" {:>2} ", day_names[idx]));
            }
            s.trim_end().to_string()
        } else {
            let mut parts = Vec::new();
            for d in 0..7 {
                let idx = (d + weekstart) % 7;
                parts.push(day_names[idx]);
            }
            parts.join(" ")
        };

        if month != 0 {
            // Single month display
            let mut days = [SPACE; MAXDAYS];
            day_array(month, year, weekstart, opt_julian, &mut days);

            let header = format!("{} {}", MONTH_NAMES[(month - 1) as usize], year);
            let week_len = if opt_julian { 27 } else { 20 };
            let pad = if week_len > header.len() {
                (week_len - header.len()) / 2
            } else {
                0
            };
            println!("{}{}", " ".repeat(pad), header);
            println!("{}", day_headings);

            for row in 0..6 {
                let dp = &days[row * 7..(row + 1) * 7];
                let row_str = build_row(opt_julian, dp);
                trim_trailing_spaces_and_print(&row_str);
            }
        } else {
            // Full year display
            let mut all_days = [[SPACE; MAXDAYS]; 12];
            for (m, slot) in all_days.iter_mut().enumerate() {
                day_array(m as u32 + 1, year, weekstart, opt_julian, slot);
            }

            let full_width = if opt_julian {
                28 * 2 + 2 - 1 // 57
            } else {
                20 * 3 + 2 * 2 // 64
            };
            let year_str = format!("{}", year);
            let year_pad = if full_width > year_str.len() {
                (full_width - year_str.len()) / 2
            } else {
                0
            };
            println!("{}{}\n", " ".repeat(year_pad), year_str);

            let step = if opt_julian { 2 } else { 3 };
            let col_width = if opt_julian { 28 } else { 20 };

            for m_start in (0..12).step_by(step) {
                // Print month names
                let mut title_line = String::new();
                for col in 0..step {
                    let m_idx = m_start + col;
                    let m_title = center_string(MONTH_NAMES[m_idx], col_width);
                    title_line.push_str(&m_title);
                    if col + 1 < step {
                        title_line.push_str("  ");
                    }
                }
                trim_trailing_spaces_and_print(&title_line);

                // Print day headings
                let mut heading_line = String::new();
                for col in 0..step {
                    heading_line.push_str(&day_headings);
                    if col + 1 < step {
                        heading_line.push_str("  ");
                    }
                }
                println!("{}", heading_line);

                // Print 6 rows
                for row in 0..6 {
                    let mut row_line = String::new();
                    for col in 0..step {
                        let m_idx = m_start + col;
                        let dp = &all_days[m_idx][row * 7..(row + 1) * 7];
                        let r = build_row(opt_julian, dp);
                        row_line.push_str(&r);
                        if col + 1 < step {
                            // Ensure column width matches
                            if r.len() < col_width {
                                row_line.push_str(&" ".repeat(col_width - r.len()));
                            }
                            row_line.push_str("  ");
                        }
                    }
                    trim_trailing_spaces_and_print(&row_line);
                }
            }
        }

        Ok(0)
    }
}
