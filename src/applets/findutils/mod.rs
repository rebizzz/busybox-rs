use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod find;
pub mod grep;
pub mod egrep;
pub mod fgrep;
pub mod xargs;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "find")]
    applets.push(Arc::new(find::FindApplet));
    #[cfg(feature = "grep")]
    applets.push(Arc::new(grep::GrepApplet));
    #[cfg(feature = "egrep")]
    applets.push(Arc::new(egrep::EgrepApplet));
    #[cfg(feature = "fgrep")]
    applets.push(Arc::new(fgrep::FgrepApplet));
    #[cfg(feature = "xargs")]
    applets.push(Arc::new(xargs::XargsApplet));
}
