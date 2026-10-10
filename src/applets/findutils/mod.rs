use crate::core::Applet;
use std::sync::Arc;

pub mod find;
pub mod grep;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "find")]
    applets.push(Arc::new(find::FindApplet));
    #[cfg(feature = "grep")]
    applets.push(Arc::new(grep::GrepApplet));
    #[cfg(feature = "egrep")]
    applets.push(Arc::new(grep::EgrepApplet));
    #[cfg(feature = "fgrep")]
    applets.push(Arc::new(grep::FgrepApplet));
}
pub mod xargs;
