use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod fdisk;
pub mod fsck;
pub mod fsck_minix;
pub mod mkfs_minix;
pub mod mkdosfs;
pub mod mkfs_vfat;
pub mod mke2fs;
pub mod mkfs_ext2;
pub mod fatlabel;
pub mod fstrim;
pub mod raidautorun;
pub mod resume;
pub mod linux32;
pub mod linux64;
pub mod setarch;
pub mod pivot_root;
pub mod switch_root;
pub mod run_init;
pub mod adjtimex;
pub mod hwclock;
pub mod rtcwake;
pub mod seedrng;
pub mod smemcap;
pub mod readprofile;
pub mod rdev;
pub mod fallocate;
pub mod run_parts;
pub mod sysctl;
pub mod stty;
pub mod unshare;
pub mod nsenter;
pub mod lpd;
pub mod lpq;
pub mod lpr;
pub mod svok;
pub mod dumpleases;
pub mod mkswap;
pub mod swapon;
pub mod swapoff;
pub mod fsfreeze;
pub mod ttysize;
pub mod openvt;
pub mod watch;
pub mod setlogcons;
pub mod setserial;
pub mod partprobe;
pub mod nbd_client;
pub mod mount;
pub mod umount;
pub mod dmesg;
pub mod lsblk;
pub mod flock;
pub mod test;
pub mod lbracket;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "fdisk")]
    applets.push(Arc::new(fdisk::FdiskApplet));
    #[cfg(feature = "fsck")]
    applets.push(Arc::new(fsck::FsckApplet));
    #[cfg(feature = "fsck_minix")]
    applets.push(Arc::new(fsck_minix::FsckMinixApplet));
    #[cfg(feature = "mkfs_minix")]
    applets.push(Arc::new(mkfs_minix::MkfsMinixApplet));
    #[cfg(feature = "mkdosfs")]
    applets.push(Arc::new(mkdosfs::MkdosfsApplet));
    #[cfg(feature = "mkfs_vfat")]
    applets.push(Arc::new(mkfs_vfat::MkfsVfatApplet));
    #[cfg(feature = "mke2fs")]
    applets.push(Arc::new(mke2fs::Mke2fsApplet));
    #[cfg(feature = "mkfs_ext2")]
    applets.push(Arc::new(mkfs_ext2::MkfsExt2Applet));
    #[cfg(feature = "fatlabel")]
    applets.push(Arc::new(fatlabel::FatlabelApplet));
    #[cfg(feature = "fstrim")]
    applets.push(Arc::new(fstrim::FstrimApplet));
    #[cfg(feature = "raidautorun")]
    applets.push(Arc::new(raidautorun::RaidautorunApplet));
    #[cfg(feature = "resume")]
    applets.push(Arc::new(resume::ResumeApplet));
    #[cfg(feature = "linux32")]
    applets.push(Arc::new(linux32::Linux32Applet));
    #[cfg(feature = "linux64")]
    applets.push(Arc::new(linux64::Linux64Applet));
    #[cfg(feature = "setarch")]
    applets.push(Arc::new(setarch::SetarchApplet));
    #[cfg(feature = "pivot_root")]
    applets.push(Arc::new(pivot_root::PivotRootApplet));
    #[cfg(feature = "switch_root")]
    applets.push(Arc::new(switch_root::SwitchRootApplet));
    #[cfg(feature = "run_init")]
    applets.push(Arc::new(run_init::RunInitApplet));
    #[cfg(feature = "adjtimex")]
    applets.push(Arc::new(adjtimex::AdjtimexApplet));
    #[cfg(feature = "hwclock")]
    applets.push(Arc::new(hwclock::HwclockApplet));
    #[cfg(feature = "rtcwake")]
    applets.push(Arc::new(rtcwake::RtcwakeApplet));
    #[cfg(feature = "seedrng")]
    applets.push(Arc::new(seedrng::SeedrngApplet));
    #[cfg(feature = "smemcap")]
    applets.push(Arc::new(smemcap::SmemcapApplet));
    #[cfg(feature = "readprofile")]
    applets.push(Arc::new(readprofile::ReadprofileApplet));
    #[cfg(feature = "rdev")]
    applets.push(Arc::new(rdev::RdevApplet));
    #[cfg(feature = "fallocate")]
    applets.push(Arc::new(fallocate::FallocateApplet));
    #[cfg(feature = "run_parts")]
    applets.push(Arc::new(run_parts::RunPartsApplet));
    #[cfg(feature = "sysctl")]
    applets.push(Arc::new(sysctl::SysctlApplet));
    #[cfg(feature = "stty")]
    applets.push(Arc::new(stty::SttyApplet));
    #[cfg(feature = "unshare")]
    applets.push(Arc::new(unshare::UnshareApplet));
    #[cfg(feature = "nsenter")]
    applets.push(Arc::new(nsenter::NsenterApplet));
    #[cfg(feature = "lpd")]
    applets.push(Arc::new(lpd::LpdApplet));
    #[cfg(feature = "lpq")]
    applets.push(Arc::new(lpq::LpqApplet));
    #[cfg(feature = "lpr")]
    applets.push(Arc::new(lpr::LprApplet));
    #[cfg(feature = "svok")]
    applets.push(Arc::new(svok::SvokApplet));
    #[cfg(feature = "dumpleases")]
    applets.push(Arc::new(dumpleases::DumpleasesApplet));
    #[cfg(feature = "mkswap")]
    applets.push(Arc::new(mkswap::MkswapApplet));
    #[cfg(feature = "swapon")]
    applets.push(Arc::new(swapon::SwaponApplet));
    #[cfg(feature = "swapoff")]
    applets.push(Arc::new(swapoff::SwapoffApplet));
    #[cfg(feature = "fsfreeze")]
    applets.push(Arc::new(fsfreeze::FsfreezeApplet));
    #[cfg(feature = "ttysize")]
    applets.push(Arc::new(ttysize::TtysizeApplet));
    #[cfg(feature = "openvt")]
    applets.push(Arc::new(openvt::OpenvtApplet));
    #[cfg(feature = "watch")]
    applets.push(Arc::new(watch::WatchApplet));
    #[cfg(feature = "setlogcons")]
    applets.push(Arc::new(setlogcons::SetlogconsApplet));
    #[cfg(feature = "setserial")]
    applets.push(Arc::new(setserial::SetserialApplet));
    #[cfg(feature = "partprobe")]
    applets.push(Arc::new(partprobe::PartprobeApplet));
    #[cfg(feature = "nbd_client")]
    applets.push(Arc::new(nbd_client::NbdClientApplet));
    #[cfg(feature = "mount")]
    applets.push(Arc::new(mount::MountApplet));
    #[cfg(feature = "umount")]
    applets.push(Arc::new(umount::UmountApplet));
    #[cfg(feature = "dmesg")]
    applets.push(Arc::new(dmesg::DmesgApplet));
    #[cfg(feature = "lsblk")]
    applets.push(Arc::new(lsblk::LsblkApplet));
    #[cfg(feature = "flock")]
    applets.push(Arc::new(flock::FlockApplet));
    #[cfg(feature = "test")]
    applets.push(Arc::new(test::TestApplet));
    #[cfg(feature = "test")]
    applets.push(Arc::new(lbracket::LBracketApplet));
}
