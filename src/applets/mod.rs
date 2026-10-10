use crate::core::Applet;
use std::sync::Arc;

pub mod archival;
pub mod console_tools;
pub mod coreutils;
pub mod editors;
pub mod findutils;
pub mod init;
pub mod loginutils;
pub mod miscutils;
pub mod modutils;
pub mod networking;
pub mod procps;
pub mod shell;
pub mod util_linux;

pub fn get_applets() -> Vec<Arc<dyn Applet>> {
    let mut applets: Vec<Arc<dyn Applet>> = Vec::new();
    archival::register(&mut applets);
    console_tools::register(&mut applets);
    coreutils::register(&mut applets);
    editors::register(&mut applets);
    findutils::register(&mut applets);
    init::register(&mut applets);
    loginutils::register(&mut applets);
    miscutils::register(&mut applets);
    modutils::register(&mut applets);
    networking::register(&mut applets);
    procps::register(&mut applets);
    shell::register(&mut applets);
    util_linux::register(&mut applets);
    applets
}
