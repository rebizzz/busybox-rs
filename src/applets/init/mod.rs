use crate::core::Applet;
use std::sync::Arc;

#[allow(clippy::module_inception)]
pub mod init;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "init")]
    applets.push(Arc::new(init::InitApplet));
    #[cfg(feature = "linuxrc")]
    applets.push(Arc::new(init::LinuxrcApplet));
    #[cfg(feature = "runlevel")]
    applets.push(Arc::new(init::RunlevelApplet));
    #[cfg(feature = "halt")]
    applets.push(Arc::new(init::HaltApplet));
    #[cfg(feature = "poweroff")]
    applets.push(Arc::new(init::PoweroffApplet));
    #[cfg(feature = "reboot")]
    applets.push(Arc::new(init::RebootApplet));
    #[cfg(feature = "bootchartd")]
    applets.push(Arc::new(init::BootchartdApplet));
    #[cfg(feature = "klogd")]
    applets.push(Arc::new(init::KlogdApplet));
    #[cfg(feature = "syslogd")]
    applets.push(Arc::new(init::SyslogdApplet));
    #[cfg(feature = "logread")]
    applets.push(Arc::new(init::LogreadApplet));
    #[cfg(feature = "crond")]
    applets.push(Arc::new(init::CrondApplet));
    #[cfg(feature = "crontab")]
    applets.push(Arc::new(init::CrontabApplet));
    #[cfg(feature = "getty")]
    applets.push(Arc::new(init::GettyApplet));
    #[cfg(feature = "last")]
    applets.push(Arc::new(init::LastApplet));
    #[cfg(feature = "mesg")]
    applets.push(Arc::new(init::MesgApplet));
    #[cfg(feature = "wall")]
    applets.push(Arc::new(init::WallApplet));
}
