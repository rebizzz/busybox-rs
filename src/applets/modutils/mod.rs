use crate::core::Applet;
use std::sync::Arc;

pub mod modules;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "lsmod")]
    applets.push(Arc::new(modules::LsmodApplet));
    #[cfg(feature = "insmod")]
    applets.push(Arc::new(modules::InsmodApplet));
    #[cfg(feature = "rmmod")]
    applets.push(Arc::new(modules::RmmodApplet));
    #[cfg(feature = "modprobe")]
    applets.push(Arc::new(modules::ModprobeApplet));
    #[cfg(feature = "depmod")]
    applets.push(Arc::new(modules::DepmodApplet));
    #[cfg(feature = "modinfo")]
    applets.push(Arc::new(modules::ModinfoApplet));
    #[cfg(feature = "acpid")]
    applets.push(Arc::new(modules::AcpidApplet));
    #[cfg(feature = "lsof")]
    applets.push(Arc::new(modules::LsofApplet));
    #[cfg(feature = "lsscsi")]
    applets.push(Arc::new(modules::LsscsiApplet));
    #[cfg(feature = "tree")]
    applets.push(Arc::new(modules::TreeApplet));
    #[cfg(feature = "mim")]
    applets.push(Arc::new(modules::MimApplet));
    #[cfg(feature = "logger")]
    applets.push(Arc::new(modules::LoggerApplet));
}
