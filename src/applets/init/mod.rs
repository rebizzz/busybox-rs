use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod bootchartd;
pub mod crond;
pub mod crontab;
pub mod getty;
pub mod halt;
#[allow(clippy::module_inception)]
pub mod init;
pub mod klogd;
pub mod last;
pub mod linuxrc;
pub mod logread;
pub mod mesg;
pub mod poweroff;
pub mod reboot;
pub mod runlevel;
pub mod syslogd;
pub mod wall;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "init")]
    applets.push(Arc::new(init::InitApplet));
    #[cfg(feature = "linuxrc")]
    applets.push(Arc::new(linuxrc::LinuxrcApplet));
    #[cfg(feature = "runlevel")]
    applets.push(Arc::new(runlevel::RunlevelApplet));
    #[cfg(feature = "halt")]
    applets.push(Arc::new(halt::HaltApplet));
    #[cfg(feature = "poweroff")]
    applets.push(Arc::new(poweroff::PoweroffApplet));
    #[cfg(feature = "reboot")]
    applets.push(Arc::new(reboot::RebootApplet));
    #[cfg(feature = "bootchartd")]
    applets.push(Arc::new(bootchartd::BootchartdApplet));
    #[cfg(feature = "klogd")]
    applets.push(Arc::new(klogd::KlogdApplet));
    #[cfg(feature = "syslogd")]
    applets.push(Arc::new(syslogd::SyslogdApplet));
    #[cfg(feature = "logread")]
    applets.push(Arc::new(logread::LogreadApplet));
    #[cfg(feature = "crond")]
    applets.push(Arc::new(crond::CrondApplet));
    #[cfg(feature = "crontab")]
    applets.push(Arc::new(crontab::CrontabApplet));
    #[cfg(feature = "getty")]
    applets.push(Arc::new(getty::GettyApplet));
    #[cfg(feature = "last")]
    applets.push(Arc::new(last::LastApplet));
    #[cfg(feature = "mesg")]
    applets.push(Arc::new(mesg::MesgApplet));
    #[cfg(feature = "wall")]
    applets.push(Arc::new(wall::WallApplet));
}
