use std::os::unix::io::AsRawFd;
use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod loadfont;
pub mod setfont;
pub mod dumpkmap;
pub mod loadkmap;
pub mod setkeycodes;
pub mod showkey;
pub mod kbd_mode;
pub mod fgconsole;
pub mod chvt;
pub mod deallocvt;
pub mod fbset;
pub mod fbsplash;
pub mod beep;
pub mod eject;
pub mod hdparm;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "loadfont")]
    applets.push(Arc::new(loadfont::LoadfontApplet));
    #[cfg(feature = "setfont")]
    applets.push(Arc::new(setfont::SetfontApplet));
    #[cfg(feature = "dumpkmap")]
    applets.push(Arc::new(dumpkmap::DumpkmapApplet));
    #[cfg(feature = "loadkmap")]
    applets.push(Arc::new(loadkmap::LoadkmapApplet));
    #[cfg(feature = "setkeycodes")]
    applets.push(Arc::new(setkeycodes::SetkeycodesApplet));
    #[cfg(feature = "showkey")]
    applets.push(Arc::new(showkey::ShowkeyApplet));
    #[cfg(feature = "kbd_mode")]
    applets.push(Arc::new(kbd_mode::KbdModeApplet));
    #[cfg(feature = "fgconsole")]
    applets.push(Arc::new(fgconsole::FgconsoleApplet));
    #[cfg(feature = "chvt")]
    applets.push(Arc::new(chvt::ChvtApplet));
    #[cfg(feature = "deallocvt")]
    applets.push(Arc::new(deallocvt::DeallocvtApplet));
    #[cfg(feature = "fbset")]
    applets.push(Arc::new(fbset::FbsetApplet));
    #[cfg(feature = "fbsplash")]
    applets.push(Arc::new(fbsplash::FbsplashApplet));
    #[cfg(feature = "beep")]
    applets.push(Arc::new(beep::BeepApplet));
    #[cfg(feature = "eject")]
    applets.push(Arc::new(eject::EjectApplet));
    #[cfg(feature = "hdparm")]
    applets.push(Arc::new(hdparm::HdparmApplet));
}
