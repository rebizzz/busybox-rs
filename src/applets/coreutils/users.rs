use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct UsersApplet;

impl Applet for UsersApplet {
    fn name(&self) -> &'static str {
        "users"
    }
    fn description(&self) -> &'static str {
        "List the current users logged in"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        let who = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        println!("{}", who);
        Ok(0)
    }
}

