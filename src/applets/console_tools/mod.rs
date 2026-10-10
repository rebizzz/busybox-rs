use crate::core::Applet;
use std::sync::Arc;

pub mod console;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "loadfont")]
    applets.push(Arc::new(console::LoadfontApplet));
    #[cfg(feature = "setfont")]
    applets.push(Arc::new(console::SetfontApplet));
    #[cfg(feature = "dumpkmap")]
    applets.push(Arc::new(console::DumpkmapApplet));
    #[cfg(feature = "loadkmap")]
    applets.push(Arc::new(console::LoadkmapApplet));
    #[cfg(feature = "setkeycodes")]
    applets.push(Arc::new(console::SetkeycodesApplet));
    #[cfg(feature = "showkey")]
    applets.push(Arc::new(console::ShowkeyApplet));
    #[cfg(feature = "kbd_mode")]
    applets.push(Arc::new(console::KbdModeApplet));
    #[cfg(feature = "fgconsole")]
    applets.push(Arc::new(console::FgconsoleApplet));
    #[cfg(feature = "chvt")]
    applets.push(Arc::new(console::ChvtApplet));
    #[cfg(feature = "deallocvt")]
    applets.push(Arc::new(console::DeallocvtApplet));
    #[cfg(feature = "fbset")]
    applets.push(Arc::new(console::FbsetApplet));
    #[cfg(feature = "fbsplash")]
    applets.push(Arc::new(console::FbsplashApplet));
    #[cfg(feature = "beep")]
    applets.push(Arc::new(console::BeepApplet));
    #[cfg(feature = "eject")]
    applets.push(Arc::new(console::EjectApplet));
    #[cfg(feature = "hdparm")]
    applets.push(Arc::new(console::HdparmApplet));
}
