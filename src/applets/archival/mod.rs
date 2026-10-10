use crate::core::Applet;
use std::sync::Arc;

#[allow(clippy::module_inception)]
pub mod archival;
pub mod inflate;
pub mod package;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "tar")]
    applets.push(Arc::new(archival::TarApplet));
    #[cfg(feature = "cpio")]
    applets.push(Arc::new(archival::CpioApplet));
    #[cfg(feature = "gzip")]
    applets.push(Arc::new(archival::GzipApplet));
    #[cfg(feature = "gunzip")]
    applets.push(Arc::new(archival::GunzipApplet));
    #[cfg(feature = "bunzip2")]
    applets.push(Arc::new(archival::Bunzip2Applet));
    #[cfg(feature = "bzcat")]
    applets.push(Arc::new(archival::BzcatApplet));
    #[cfg(feature = "bzip2")]
    applets.push(Arc::new(archival::Bzip2Applet));
    #[cfg(feature = "unlzma")]
    applets.push(Arc::new(archival::UnlzmaApplet));
    #[cfg(feature = "lzcat")]
    applets.push(Arc::new(archival::LzcatApplet));
    #[cfg(feature = "lzma")]
    applets.push(Arc::new(archival::LzmaApplet));
    #[cfg(feature = "unxz")]
    applets.push(Arc::new(archival::UnxzApplet));
    #[cfg(feature = "xz")]
    applets.push(Arc::new(archival::XzApplet));
    #[cfg(feature = "xzcat")]
    applets.push(Arc::new(archival::XzcatApplet));
    #[cfg(feature = "lzop")]
    applets.push(Arc::new(archival::LzopApplet));
    #[cfg(feature = "cksum")]
    applets.push(Arc::new(archival::CksumApplet));
    #[cfg(feature = "unzip")]
    applets.push(Arc::new(package::UnzipApplet));
    #[cfg(feature = "zcat")]
    applets.push(Arc::new(package::ZcatApplet));
    #[cfg(feature = "rpm")]
    applets.push(Arc::new(package::RpmApplet));
    #[cfg(feature = "rpm2cpio")]
    applets.push(Arc::new(package::Rpm2cpioApplet));
    #[cfg(feature = "dpkg_deb")]
    applets.push(Arc::new(package::DpkgDebApplet));
    #[cfg(feature = "dpkg")]
    applets.push(Arc::new(package::DpkgApplet));
    #[cfg(feature = "nanddump")]
    applets.push(Arc::new(package::NanddumpApplet));
    #[cfg(feature = "nandwrite")]
    applets.push(Arc::new(package::NandwriteApplet));
    #[cfg(feature = "ubiattach")]
    applets.push(Arc::new(package::UbiattachApplet));
    #[cfg(feature = "ubidetach")]
    applets.push(Arc::new(package::UbidetachApplet));
    #[cfg(feature = "ubimkvol")]
    applets.push(Arc::new(package::UbimkvolApplet));
    #[cfg(feature = "ubirename")]
    applets.push(Arc::new(package::UbirenameApplet));
    #[cfg(feature = "ubirmvol")]
    applets.push(Arc::new(package::UbirmvolApplet));
    #[cfg(feature = "ubirsvol")]
    applets.push(Arc::new(package::UbirsvolApplet));
    #[cfg(feature = "ubiupdatevol")]
    applets.push(Arc::new(package::UbiupdatevolApplet));
    #[cfg(feature = "mt")]
    applets.push(Arc::new(package::MtApplet));
    #[cfg(feature = "rx")]
    applets.push(Arc::new(package::RxApplet));
    #[cfg(feature = "pipe_progress")]
    applets.push(Arc::new(package::PipeProgressApplet));
    #[cfg(feature = "script")]
    applets.push(Arc::new(package::ScriptApplet));
    #[cfg(feature = "scriptreplay")]
    applets.push(Arc::new(package::ScriptreplayApplet));
    #[cfg(feature = "getopt")]
    applets.push(Arc::new(package::GetoptApplet));
    #[cfg(feature = "envdir")]
    applets.push(Arc::new(package::EnvdirApplet));
    #[cfg(feature = "envuidgid")]
    applets.push(Arc::new(package::EnvuidgidApplet));
    #[cfg(feature = "softlimit")]
    applets.push(Arc::new(package::SoftlimitApplet));
    #[cfg(feature = "setuidgid")]
    applets.push(Arc::new(package::SetuidgidApplet));
    #[cfg(feature = "chpst")]
    applets.push(Arc::new(package::ChpstApplet));
    #[cfg(feature = "runsv")]
    applets.push(Arc::new(package::RunsvApplet));
    #[cfg(feature = "runsvdir")]
    applets.push(Arc::new(package::RunsvdirApplet));
    #[cfg(feature = "sv")]
    applets.push(Arc::new(package::SvApplet));
    #[cfg(feature = "svc")]
    applets.push(Arc::new(package::SvcApplet));
    #[cfg(feature = "svlogd")]
    applets.push(Arc::new(package::SvlogdApplet));
    #[cfg(feature = "setpriv")]
    applets.push(Arc::new(package::SetprivApplet));
}
