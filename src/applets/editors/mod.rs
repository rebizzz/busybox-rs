use crate::core::Applet;
use std::sync::Arc;

pub mod cmp;
pub mod editor;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "cmp")]
    applets.push(Arc::new(cmp::CmpApplet));
    #[cfg(feature = "sed")]
    applets.push(Arc::new(editor::SedApplet));
    #[cfg(feature = "join")]
    applets.push(Arc::new(editor::JoinApplet));
    #[cfg(feature = "expr")]
    applets.push(Arc::new(editor::ExprApplet));
    #[cfg(feature = "diff")]
    applets.push(Arc::new(editor::DiffApplet));
    #[cfg(feature = "patch")]
    applets.push(Arc::new(editor::PatchApplet));
    #[cfg(feature = "split")]
    applets.push(Arc::new(editor::SplitApplet));
    #[cfg(feature = "tac")]
    applets.push(Arc::new(editor::TacApplet));
    #[cfg(feature = "shuf")]
    applets.push(Arc::new(editor::ShufApplet));
    #[cfg(feature = "truncate")]
    applets.push(Arc::new(editor::TruncateApplet));
    #[cfg(feature = "ts")]
    applets.push(Arc::new(editor::TsApplet));
    #[cfg(feature = "timeout")]
    applets.push(Arc::new(editor::TimeoutApplet));
    #[cfg(feature = "date")]
    applets.push(Arc::new(editor::DateApplet));
    #[cfg(feature = "env")]
    applets.push(Arc::new(editor::EnvApplet));
    #[cfg(feature = "mktemp")]
    applets.push(Arc::new(editor::MktempApplet));
    #[cfg(feature = "install")]
    applets.push(Arc::new(editor::InstallApplet));
    #[cfg(feature = "hostid")]
    applets.push(Arc::new(editor::HostidApplet));
}
