use crate::core::{Applet, Result};
use std::ffi::OsString;

pub struct WhoamiApplet;
impl Applet for WhoamiApplet {
    fn name(&self) -> &'static str {
        "whoami"
    }
    fn description(&self) -> &'static str {
        "Print effective user name"
    }
    fn run(&self, _args: &[OsString]) -> Result<i32> {
        if let Some(user) = crate::core::platform::get_current_username() {
            println!("{}", user);
            Ok(0)
        } else {
            eprintln!("whoami: cannot find name for user ID");
            Ok(1)
        }
    }
}
