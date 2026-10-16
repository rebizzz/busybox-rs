use crate::core::Applet;
use std::sync::Arc;

pub mod awk;
pub mod cmp;
pub mod ed;
pub mod date;
pub mod diff;
pub mod env;
pub mod expr;
pub mod hostid;
pub mod install;
pub mod join;
pub mod mktemp;
pub mod patch;
pub mod sed;
pub mod shuf;
pub mod split;
pub mod tac;
pub mod timeout;
pub mod truncate;
pub mod ts;
pub mod vi;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "awk")]
    applets.push(Arc::new(awk::AwkApplet));
    #[cfg(feature = "ed")]
    applets.push(Arc::new(ed::EdApplet));
    #[cfg(feature = "vi")]
    applets.push(Arc::new(vi::ViApplet));
    #[cfg(feature = "cmp")]
    applets.push(Arc::new(cmp::CmpApplet));
    #[cfg(feature = "sed")]
    applets.push(Arc::new(sed::SedApplet));
    #[cfg(feature = "join")]
    applets.push(Arc::new(join::JoinApplet));
    #[cfg(feature = "expr")]
    applets.push(Arc::new(expr::ExprApplet));
    #[cfg(feature = "diff")]
    applets.push(Arc::new(diff::DiffApplet));
    #[cfg(feature = "patch")]
    applets.push(Arc::new(patch::PatchApplet));
    #[cfg(feature = "split")]
    applets.push(Arc::new(split::SplitApplet));
    #[cfg(feature = "tac")]
    applets.push(Arc::new(tac::TacApplet));
    #[cfg(feature = "shuf")]
    applets.push(Arc::new(shuf::ShufApplet));
    #[cfg(feature = "truncate")]
    applets.push(Arc::new(truncate::TruncateApplet));
    #[cfg(feature = "ts")]
    applets.push(Arc::new(ts::TsApplet));
    #[cfg(feature = "timeout")]
    applets.push(Arc::new(timeout::TimeoutApplet));
    #[cfg(feature = "date")]
    applets.push(Arc::new(date::DateApplet));
    #[cfg(feature = "env")]
    applets.push(Arc::new(env::EnvApplet));
    #[cfg(feature = "mktemp")]
    applets.push(Arc::new(mktemp::MktempApplet));
    #[cfg(feature = "install")]
    applets.push(Arc::new(install::InstallApplet));
    #[cfg(feature = "hostid")]
    applets.push(Arc::new(hostid::HostidApplet));
}
