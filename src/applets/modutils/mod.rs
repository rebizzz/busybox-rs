use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod lsmod;
pub mod insmod;
pub mod rmmod;
pub mod modprobe;
pub mod depmod;
pub mod modinfo;
pub mod acpid;
pub mod lsof;
pub mod lsscsi;
pub mod tree;
pub mod mim;
pub mod logger;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "lsmod")]
    applets.push(Arc::new(lsmod::LsmodApplet));
    #[cfg(feature = "insmod")]
    applets.push(Arc::new(insmod::InsmodApplet));
    #[cfg(feature = "rmmod")]
    applets.push(Arc::new(rmmod::RmmodApplet));
    #[cfg(feature = "modprobe")]
    applets.push(Arc::new(modprobe::ModprobeApplet));
    #[cfg(feature = "depmod")]
    applets.push(Arc::new(depmod::DepmodApplet));
    #[cfg(feature = "modinfo")]
    applets.push(Arc::new(modinfo::ModinfoApplet));
    #[cfg(feature = "acpid")]
    applets.push(Arc::new(acpid::AcpidApplet));
    #[cfg(feature = "lsof")]
    applets.push(Arc::new(lsof::LsofApplet));
    #[cfg(feature = "lsscsi")]
    applets.push(Arc::new(lsscsi::LsscsiApplet));
    #[cfg(feature = "tree")]
    applets.push(Arc::new(tree::TreeApplet));
    #[cfg(feature = "mim")]
    applets.push(Arc::new(mim::MimApplet));
    #[cfg(feature = "logger")]
    applets.push(Arc::new(logger::LoggerApplet));
}
