use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod pgrep;
pub mod pkill;
pub mod pidof;
pub mod top;
pub mod pmap;
pub mod pwdx;
pub mod vmstat;
pub mod killall5;
pub mod renice;
pub mod ionice;
pub mod chrt;
pub mod taskset;
pub mod setsid;
pub mod start_stop_daemon;
pub mod chroot;
pub mod cttyhack;
pub mod ps;
pub mod kill;
pub mod killall;
pub mod free;
pub mod uptime;
pub mod uname;
pub mod hostname;
pub mod id;
pub mod groups;
pub mod logname;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "pgrep")]
    applets.push(Arc::new(pgrep::PgrepApplet));
    #[cfg(feature = "pkill")]
    applets.push(Arc::new(pkill::PkillApplet));
    #[cfg(feature = "pidof")]
    applets.push(Arc::new(pidof::PidofApplet));
    #[cfg(feature = "top")]
    applets.push(Arc::new(top::TopApplet));
    #[cfg(feature = "pmap")]
    applets.push(Arc::new(pmap::PmapApplet));
    #[cfg(feature = "pwdx")]
    applets.push(Arc::new(pwdx::PwdxApplet));
    #[cfg(feature = "vmstat")]
    applets.push(Arc::new(vmstat::VmstatApplet));
    #[cfg(feature = "killall5")]
    applets.push(Arc::new(killall5::Killall5Applet));
    #[cfg(feature = "renice")]
    applets.push(Arc::new(renice::ReniceApplet));
    #[cfg(feature = "ionice")]
    applets.push(Arc::new(ionice::IoniceApplet));
    #[cfg(feature = "chrt")]
    applets.push(Arc::new(chrt::ChrtApplet));
    #[cfg(feature = "taskset")]
    applets.push(Arc::new(taskset::TasksetApplet));
    #[cfg(feature = "setsid")]
    applets.push(Arc::new(setsid::SetsidApplet));
    #[cfg(feature = "start_stop_daemon")]
    applets.push(Arc::new(start_stop_daemon::StartStopDaemonApplet));
    #[cfg(feature = "chroot")]
    applets.push(Arc::new(chroot::ChrootApplet));
    #[cfg(feature = "cttyhack")]
    applets.push(Arc::new(cttyhack::CttyhackApplet));
    #[cfg(feature = "ps")]
    applets.push(Arc::new(ps::PsApplet));
    #[cfg(feature = "kill")]
    applets.push(Arc::new(kill::KillApplet));
    #[cfg(feature = "killall")]
    applets.push(Arc::new(killall::KillallApplet));
    #[cfg(feature = "free")]
    applets.push(Arc::new(free::FreeApplet));
    #[cfg(feature = "uptime")]
    applets.push(Arc::new(uptime::UptimeApplet));
    #[cfg(feature = "uname")]
    applets.push(Arc::new(uname::UnameApplet));
    #[cfg(feature = "hostname")]
    applets.push(Arc::new(hostname::HostnameApplet));
    #[cfg(feature = "id")]
    applets.push(Arc::new(id::IdApplet));
    #[cfg(feature = "groups")]
    applets.push(Arc::new(groups::GroupsApplet));
    #[cfg(feature = "logname")]
    applets.push(Arc::new(logname::LognameApplet));
}
