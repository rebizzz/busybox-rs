use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod inflate;

pub mod tar;
pub mod cpio;
pub mod gzip;
pub mod gunzip;
pub mod bunzip2;
pub mod bzcat;
pub mod bzip2;
pub mod unlzma;
pub mod lzcat;
pub mod lzma;
pub mod unxz;
pub mod xz;
pub mod xzcat;
pub mod lzop;
pub mod cksum;
pub mod unzip;
pub mod zcat;
pub mod rpm;
pub mod rpm2cpio;
pub mod dpkg_deb;
pub mod dpkg;
pub mod nanddump;
pub mod nandwrite;
pub mod ubiattach;
pub mod ubidetach;
pub mod ubimkvol;
pub mod ubirename;
pub mod ubirmvol;
pub mod ubirsvol;
pub mod ubiupdatevol;
pub mod mt;
pub mod rx;
pub mod pipe_progress;
pub mod script;
pub mod scriptreplay;
pub mod getopt;
pub mod envdir;
pub mod envuidgid;
pub mod softlimit;
pub mod setuidgid;
pub mod chpst;
pub mod runsv;
pub mod runsvdir;
pub mod sv;
pub mod svc;
pub mod svlogd;
pub mod setpriv;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "tar")]
    applets.push(Arc::new(tar::TarApplet));
    #[cfg(feature = "cpio")]
    applets.push(Arc::new(cpio::CpioApplet));
    #[cfg(feature = "gzip")]
    applets.push(Arc::new(gzip::GzipApplet));
    #[cfg(feature = "gunzip")]
    applets.push(Arc::new(gunzip::GunzipApplet));
    #[cfg(feature = "bunzip2")]
    applets.push(Arc::new(bunzip2::Bunzip2Applet));
    #[cfg(feature = "bzcat")]
    applets.push(Arc::new(bzcat::BzcatApplet));
    #[cfg(feature = "bzip2")]
    applets.push(Arc::new(bzip2::Bzip2Applet));
    #[cfg(feature = "unlzma")]
    applets.push(Arc::new(unlzma::UnlzmaApplet));
    #[cfg(feature = "lzcat")]
    applets.push(Arc::new(lzcat::LzcatApplet));
    #[cfg(feature = "lzma")]
    applets.push(Arc::new(lzma::LzmaApplet));
    #[cfg(feature = "unxz")]
    applets.push(Arc::new(unxz::UnxzApplet));
    #[cfg(feature = "xz")]
    applets.push(Arc::new(xz::XzApplet));
    #[cfg(feature = "xzcat")]
    applets.push(Arc::new(xzcat::XzcatApplet));
    #[cfg(feature = "lzop")]
    applets.push(Arc::new(lzop::LzopApplet));
    #[cfg(feature = "cksum")]
    applets.push(Arc::new(cksum::CksumApplet));
    #[cfg(feature = "unzip")]
    applets.push(Arc::new(unzip::UnzipApplet));
    #[cfg(feature = "zcat")]
    applets.push(Arc::new(zcat::ZcatApplet));
    #[cfg(feature = "rpm")]
    applets.push(Arc::new(rpm::RpmApplet));
    #[cfg(feature = "rpm2cpio")]
    applets.push(Arc::new(rpm2cpio::Rpm2cpioApplet));
    #[cfg(feature = "dpkg_deb")]
    applets.push(Arc::new(dpkg_deb::DpkgDebApplet));
    #[cfg(feature = "dpkg")]
    applets.push(Arc::new(dpkg::DpkgApplet));
    #[cfg(feature = "nanddump")]
    applets.push(Arc::new(nanddump::NanddumpApplet));
    #[cfg(feature = "nandwrite")]
    applets.push(Arc::new(nandwrite::NandwriteApplet));
    #[cfg(feature = "ubiattach")]
    applets.push(Arc::new(ubiattach::UbiattachApplet));
    #[cfg(feature = "ubidetach")]
    applets.push(Arc::new(ubidetach::UbidetachApplet));
    #[cfg(feature = "ubimkvol")]
    applets.push(Arc::new(ubimkvol::UbimkvolApplet));
    #[cfg(feature = "ubirename")]
    applets.push(Arc::new(ubirename::UbirenameApplet));
    #[cfg(feature = "ubirmvol")]
    applets.push(Arc::new(ubirmvol::UbirmvolApplet));
    #[cfg(feature = "ubirsvol")]
    applets.push(Arc::new(ubirsvol::UbirsvolApplet));
    #[cfg(feature = "ubiupdatevol")]
    applets.push(Arc::new(ubiupdatevol::UbiupdatevolApplet));
    #[cfg(feature = "mt")]
    applets.push(Arc::new(mt::MtApplet));
    #[cfg(feature = "rx")]
    applets.push(Arc::new(rx::RxApplet));
    #[cfg(feature = "pipe_progress")]
    applets.push(Arc::new(pipe_progress::PipeProgressApplet));
    #[cfg(feature = "script")]
    applets.push(Arc::new(script::ScriptApplet));
    #[cfg(feature = "scriptreplay")]
    applets.push(Arc::new(scriptreplay::ScriptreplayApplet));
    #[cfg(feature = "getopt")]
    applets.push(Arc::new(getopt::GetoptApplet));
    #[cfg(feature = "envdir")]
    applets.push(Arc::new(envdir::EnvdirApplet));
    #[cfg(feature = "envuidgid")]
    applets.push(Arc::new(envuidgid::EnvuidgidApplet));
    #[cfg(feature = "softlimit")]
    applets.push(Arc::new(softlimit::SoftlimitApplet));
    #[cfg(feature = "setuidgid")]
    applets.push(Arc::new(setuidgid::SetuidgidApplet));
    #[cfg(feature = "chpst")]
    applets.push(Arc::new(chpst::ChpstApplet));
    #[cfg(feature = "runsv")]
    applets.push(Arc::new(runsv::RunsvApplet));
    #[cfg(feature = "runsvdir")]
    applets.push(Arc::new(runsvdir::RunsvdirApplet));
    #[cfg(feature = "sv")]
    applets.push(Arc::new(sv::SvApplet));
    #[cfg(feature = "svc")]
    applets.push(Arc::new(svc::SvcApplet));
    #[cfg(feature = "svlogd")]
    applets.push(Arc::new(svlogd::SvlogdApplet));
    #[cfg(feature = "setpriv")]
    applets.push(Arc::new(setpriv::SetprivApplet));
}
