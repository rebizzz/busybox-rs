use crate::core::Applet;
use std::sync::Arc;

pub mod interp;
#[allow(clippy::module_inception)]
pub mod shell;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "ash")]
    applets.push(Arc::new(interp::AshApplet));
    #[cfg(feature = "hush")]
    applets.push(Arc::new(interp::HushApplet));
    #[cfg(feature = "awk")]
    applets.push(Arc::new(interp::AwkApplet));
    #[cfg(feature = "ed")]
    applets.push(Arc::new(interp::EdApplet));
    #[cfg(feature = "vi")]
    applets.push(Arc::new(interp::ViApplet));
    #[cfg(feature = "less")]
    applets.push(Arc::new(interp::LessApplet));
    #[cfg(feature = "more")]
    applets.push(Arc::new(interp::MoreApplet));
    #[cfg(feature = "man")]
    applets.push(Arc::new(interp::ManApplet));
    #[cfg(feature = "dc")]
    applets.push(Arc::new(interp::DcApplet));
    #[cfg(feature = "bc")]
    applets.push(Arc::new(interp::BcApplet));
    #[cfg(feature = "test_extended")]
    applets.push(Arc::new(interp::DoubleLBracketApplet));
    #[cfg(feature = "xargs")]
    applets.push(Arc::new(shell::XargsApplet));
    #[cfg(feature = "sh")]
    applets.push(Arc::new(shell::ShApplet));
}
