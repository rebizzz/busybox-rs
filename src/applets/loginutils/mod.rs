use crate::core::Applet;
use std::sync::Arc;

pub mod login;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "login")]
    applets.push(Arc::new(login::LoginApplet));
    #[cfg(feature = "su")]
    applets.push(Arc::new(login::SuApplet));
    #[cfg(feature = "sulogin")]
    applets.push(Arc::new(login::SuloginApplet));
    #[cfg(feature = "passwd")]
    applets.push(Arc::new(login::PasswdApplet));
    #[cfg(feature = "chpasswd")]
    applets.push(Arc::new(login::ChpasswdApplet));
    #[cfg(feature = "cryptpw")]
    applets.push(Arc::new(login::CryptpwApplet));
    #[cfg(feature = "mkpasswd")]
    applets.push(Arc::new(login::MkpasswdApplet));
    #[cfg(feature = "adduser")]
    applets.push(Arc::new(login::AdduserApplet));
    #[cfg(feature = "deluser")]
    applets.push(Arc::new(login::DeluserApplet));
    #[cfg(feature = "addgroup")]
    applets.push(Arc::new(login::AddgroupApplet));
    #[cfg(feature = "delgroup")]
    applets.push(Arc::new(login::DelgroupApplet));
    #[cfg(feature = "add_shell")]
    applets.push(Arc::new(login::AddShellApplet));
    #[cfg(feature = "remove_shell")]
    applets.push(Arc::new(login::RemoveShellApplet));
    #[cfg(feature = "vlock")]
    applets.push(Arc::new(login::VlockApplet));
    #[cfg(feature = "nologin")]
    applets.push(Arc::new(login::NologinApplet));
}
