use crate::core::Applet;
use std::sync::Arc;

pub mod ash;
pub mod common;
pub mod double_lbracket;
pub mod hush;
#[allow(clippy::module_inception)]
pub mod shell;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "ash")]
    applets.push(Arc::new(ash::AshApplet));
    #[cfg(feature = "hush")]
    applets.push(Arc::new(hush::HushApplet));
    #[cfg(feature = "test_extended")]
    applets.push(Arc::new(double_lbracket::DoubleLBracketApplet));
    #[cfg(feature = "sh")]
    applets.push(Arc::new(shell::ShApplet));
}
