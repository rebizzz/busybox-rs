use std::ffi::CStr;
use std::mem::MaybeUninit;

pub fn get_machine_arch() -> String {
    let mut uts = MaybeUninit::<libc::utsname>::uninit();
    let res = unsafe { libc::uname(uts.as_mut_ptr()) };
    if res == 0 {
        let uts = unsafe { uts.assume_init() };
        let cstr = unsafe { CStr::from_ptr(uts.machine.as_ptr()) };
        cstr.to_string_lossy().into_owned()
    } else {
        "unknown".to_string()
    }
}

pub fn get_current_username() -> Option<String> {
    unsafe {
        let uid = libc::geteuid();
        let pw = libc::getpwuid(uid);
        if !pw.is_null() && !(*pw).pw_name.is_null() {
            let cstr = CStr::from_ptr((*pw).pw_name);
            if let Ok(s) = cstr.to_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

pub fn get_umask() -> u32 {
    unsafe {
        let m = libc::umask(0);
        libc::umask(m);
        m as u32
    }
}

pub fn sync_disks() {
    unsafe { libc::sync() };
}
