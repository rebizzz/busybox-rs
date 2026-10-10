use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod login;
pub mod su;
pub mod sulogin;
pub mod passwd;
pub mod chpasswd;
pub mod cryptpw;
pub mod mkpasswd;
pub mod adduser;
pub mod deluser;
pub mod addgroup;
pub mod delgroup;
pub mod add_shell;
pub mod remove_shell;
pub mod vlock;
pub mod nologin;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "login")]
    applets.push(Arc::new(login::LoginApplet));
    #[cfg(feature = "su")]
    applets.push(Arc::new(su::SuApplet));
    #[cfg(feature = "sulogin")]
    applets.push(Arc::new(sulogin::SuloginApplet));
    #[cfg(feature = "passwd")]
    applets.push(Arc::new(passwd::PasswdApplet));
    #[cfg(feature = "chpasswd")]
    applets.push(Arc::new(chpasswd::ChpasswdApplet));
    #[cfg(feature = "cryptpw")]
    applets.push(Arc::new(cryptpw::CryptpwApplet));
    #[cfg(feature = "mkpasswd")]
    applets.push(Arc::new(mkpasswd::MkpasswdApplet));
    #[cfg(feature = "adduser")]
    applets.push(Arc::new(adduser::AdduserApplet));
    #[cfg(feature = "deluser")]
    applets.push(Arc::new(deluser::DeluserApplet));
    #[cfg(feature = "addgroup")]
    applets.push(Arc::new(addgroup::AddgroupApplet));
    #[cfg(feature = "delgroup")]
    applets.push(Arc::new(delgroup::DelgroupApplet));
    #[cfg(feature = "add-shell")]
    applets.push(Arc::new(add_shell::AddShellApplet));
    #[cfg(feature = "remove-shell")]
    applets.push(Arc::new(remove_shell::RemoveShellApplet));
    #[cfg(feature = "vlock")]
    applets.push(Arc::new(vlock::VlockApplet));
    #[cfg(feature = "nologin")]
    applets.push(Arc::new(nologin::NologinApplet));
}
