use crate::core::Applet;
use std::sync::Arc;

#[allow(clippy::module_inception)]
pub mod procps;
pub mod sysinfo;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "pgrep")]
    applets.push(Arc::new(procps::PgrepApplet));
    #[cfg(feature = "pkill")]
    applets.push(Arc::new(procps::PkillApplet));
    #[cfg(feature = "pidof")]
    applets.push(Arc::new(procps::PidofApplet));
    #[cfg(feature = "top")]
    applets.push(Arc::new(procps::TopApplet));
    #[cfg(feature = "pmap")]
    applets.push(Arc::new(procps::PmapApplet));
    #[cfg(feature = "pwdx")]
    applets.push(Arc::new(procps::PwdxApplet));
    #[cfg(feature = "vmstat")]
    applets.push(Arc::new(procps::VmstatApplet));
    #[cfg(feature = "killall5")]
    applets.push(Arc::new(procps::Killall5Applet));
    #[cfg(feature = "renice")]
    applets.push(Arc::new(procps::ReniceApplet));
    #[cfg(feature = "ionice")]
    applets.push(Arc::new(procps::IoniceApplet));
    #[cfg(feature = "chrt")]
    applets.push(Arc::new(procps::ChrtApplet));
    #[cfg(feature = "taskset")]
    applets.push(Arc::new(procps::TasksetApplet));
    #[cfg(feature = "setsid")]
    applets.push(Arc::new(procps::SetsidApplet));
    #[cfg(feature = "start_stop_daemon")]
    applets.push(Arc::new(procps::StartStopDaemonApplet));
    #[cfg(feature = "chroot")]
    applets.push(Arc::new(procps::ChrootApplet));
    #[cfg(feature = "cttyhack")]
    applets.push(Arc::new(procps::CttyhackApplet));
    #[cfg(feature = "ps")]
    applets.push(Arc::new(sysinfo::PsApplet));
    #[cfg(feature = "kill")]
    applets.push(Arc::new(sysinfo::KillApplet));
    #[cfg(feature = "killall")]
    applets.push(Arc::new(sysinfo::KillallApplet));
    #[cfg(feature = "free")]
    applets.push(Arc::new(sysinfo::FreeApplet));
    #[cfg(feature = "uptime")]
    applets.push(Arc::new(sysinfo::UptimeApplet));
    #[cfg(feature = "uname")]
    applets.push(Arc::new(sysinfo::UnameApplet));
    #[cfg(feature = "hostname")]
    applets.push(Arc::new(sysinfo::HostnameApplet));
    #[cfg(feature = "id")]
    applets.push(Arc::new(sysinfo::IdApplet));
    #[cfg(feature = "groups")]
    applets.push(Arc::new(sysinfo::GroupsApplet));
    #[cfg(feature = "logname")]
    applets.push(Arc::new(sysinfo::LognameApplet));
}
