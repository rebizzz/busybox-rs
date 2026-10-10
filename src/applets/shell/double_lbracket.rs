use crate::core::{Applet, Result};
use std::ffi::OsString;
use std::path::Path;

pub struct DoubleLBracketApplet;
impl Applet for DoubleLBracketApplet {
    fn name(&self) -> &'static str {
        "[["
    }
    fn description(&self) -> &'static str {
        "Extended shell conditional expression evaluator"
    }
    fn run(&self, args: &[OsString]) -> Result<i32> {
        let mut items = Vec::new();
        for arg in args {
            let s = arg.to_string_lossy();
            if s != "]]" {
                items.push(s.to_string());
            }
        }

        if items.is_empty() {
            return Ok(1);
        }

        if let Some(idx) = items.iter().position(|x| x == "||") {
            let left_ok = eval_extended_test(&items[..idx]);
            if left_ok {
                return Ok(0);
            }
            let right_ok = eval_extended_test(&items[idx + 1..]);
            return Ok(if right_ok { 0 } else { 1 });
        }

        if let Some(idx) = items.iter().position(|x| x == "&&") {
            let left_ok = eval_extended_test(&items[..idx]);
            if !left_ok {
                return Ok(1);
            }
            let right_ok = eval_extended_test(&items[idx + 1..]);
            return Ok(if right_ok { 0 } else { 1 });
        }

        let res = eval_extended_test(&items);
        Ok(if res { 0 } else { 1 })
    }
}

fn eval_extended_test(items: &[String]) -> bool {
    if items.is_empty() {
        return false;
    }
    if items.len() == 1 {
        return !items[0].is_empty();
    }
    if items.len() == 2 {
        let op = &items[0];
        let arg = &items[1];
        return match op.as_str() {
            "-z" => arg.is_empty(),
            "-n" => !arg.is_empty(),
            "-f" => Path::new(arg).is_file(),
            "-d" => Path::new(arg).is_dir(),
            "-e" => Path::new(arg).exists(),
            "!" => !eval_extended_test(&items[1..]),
            _ => false,
        };
    }
    if items.len() == 3 {
        let lhs = &items[0];
        let op = &items[1];
        let rhs = &items[2];
        return match op.as_str() {
            "==" | "=" => lhs == rhs,
            "!=" => lhs != rhs,
            "-eq" => lhs.parse::<i64>().unwrap_or(0) == rhs.parse::<i64>().unwrap_or(0),
            "-ne" => lhs.parse::<i64>().unwrap_or(0) != rhs.parse::<i64>().unwrap_or(0),
            "-lt" => lhs.parse::<i64>().unwrap_or(0) < rhs.parse::<i64>().unwrap_or(0),
            "-le" => lhs.parse::<i64>().unwrap_or(0) <= rhs.parse::<i64>().unwrap_or(0),
            "-gt" => lhs.parse::<i64>().unwrap_or(0) > rhs.parse::<i64>().unwrap_or(0),
            "-ge" => lhs.parse::<i64>().unwrap_or(0) >= rhs.parse::<i64>().unwrap_or(0),
            _ => false,
        };
    }
    false
}
