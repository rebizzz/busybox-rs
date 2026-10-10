use crate::core::Applet;
use std::sync::Arc;

pub mod disk_fs;
pub mod sys_arch;
pub mod sys_control;
pub mod util;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "fdisk")]
    applets.push(Arc::new(disk_fs::FdiskApplet));
    #[cfg(feature = "fsck")]
    applets.push(Arc::new(disk_fs::FsckApplet));
    #[cfg(feature = "fsck_minix")]
    applets.push(Arc::new(disk_fs::FsckMinixApplet));
    #[cfg(feature = "mkfs_minix")]
    applets.push(Arc::new(disk_fs::MkfsMinixApplet));
    #[cfg(feature = "mkdosfs")]
    applets.push(Arc::new(disk_fs::MkdosfsApplet));
    #[cfg(feature = "mkfs_vfat")]
    applets.push(Arc::new(disk_fs::MkfsVfatApplet));
    #[cfg(feature = "mke2fs")]
    applets.push(Arc::new(disk_fs::Mke2fsApplet));
    #[cfg(feature = "mkfs_ext2")]
    applets.push(Arc::new(disk_fs::MkfsExt2Applet));
    #[cfg(feature = "fatlabel")]
    applets.push(Arc::new(disk_fs::FatlabelApplet));
    #[cfg(feature = "fstrim")]
    applets.push(Arc::new(disk_fs::FstrimApplet));
    #[cfg(feature = "raidautorun")]
    applets.push(Arc::new(disk_fs::RaidautorunApplet));
    #[cfg(feature = "resume")]
    applets.push(Arc::new(disk_fs::ResumeApplet));
    #[cfg(feature = "linux32")]
    applets.push(Arc::new(sys_arch::Linux32Applet));
    #[cfg(feature = "linux64")]
    applets.push(Arc::new(sys_arch::Linux64Applet));
    #[cfg(feature = "setarch")]
    applets.push(Arc::new(sys_arch::SetarchApplet));
    #[cfg(feature = "pivot_root")]
    applets.push(Arc::new(sys_arch::PivotRootApplet));
    #[cfg(feature = "switch_root")]
    applets.push(Arc::new(sys_arch::SwitchRootApplet));
    #[cfg(feature = "run_init")]
    applets.push(Arc::new(sys_arch::RunInitApplet));
    #[cfg(feature = "adjtimex")]
    applets.push(Arc::new(sys_arch::AdjtimexApplet));
    #[cfg(feature = "hwclock")]
    applets.push(Arc::new(sys_arch::HwclockApplet));
    #[cfg(feature = "rtcwake")]
    applets.push(Arc::new(sys_arch::RtcwakeApplet));
    #[cfg(feature = "seedrng")]
    applets.push(Arc::new(sys_arch::SeedrngApplet));
    #[cfg(feature = "smemcap")]
    applets.push(Arc::new(sys_arch::SmemcapApplet));
    #[cfg(feature = "readprofile")]
    applets.push(Arc::new(sys_arch::ReadprofileApplet));
    #[cfg(feature = "rdev")]
    applets.push(Arc::new(sys_arch::RdevApplet));
    #[cfg(feature = "fallocate")]
    applets.push(Arc::new(sys_arch::FallocateApplet));
    #[cfg(feature = "run_parts")]
    applets.push(Arc::new(sys_arch::RunPartsApplet));
    #[cfg(feature = "sysctl")]
    applets.push(Arc::new(sys_control::SysctlApplet));
    #[cfg(feature = "stty")]
    applets.push(Arc::new(sys_control::SttyApplet));
    #[cfg(feature = "unshare")]
    applets.push(Arc::new(sys_control::UnshareApplet));
    #[cfg(feature = "nsenter")]
    applets.push(Arc::new(sys_control::NsenterApplet));
    #[cfg(feature = "lpd")]
    applets.push(Arc::new(sys_control::LpdApplet));
    #[cfg(feature = "lpq")]
    applets.push(Arc::new(sys_control::LpqApplet));
    #[cfg(feature = "lpr")]
    applets.push(Arc::new(sys_control::LprApplet));
    #[cfg(feature = "svok")]
    applets.push(Arc::new(sys_control::SvokApplet));
    #[cfg(feature = "dumpleases")]
    applets.push(Arc::new(sys_control::DumpleasesApplet));
    #[cfg(feature = "mkswap")]
    applets.push(Arc::new(sys_control::MkswapApplet));
    #[cfg(feature = "swapon")]
    applets.push(Arc::new(sys_control::SwaponApplet));
    #[cfg(feature = "swapoff")]
    applets.push(Arc::new(sys_control::SwapoffApplet));
    #[cfg(feature = "fsfreeze")]
    applets.push(Arc::new(sys_control::FsfreezeApplet));
    #[cfg(feature = "ttysize")]
    applets.push(Arc::new(sys_control::TtysizeApplet));
    #[cfg(feature = "openvt")]
    applets.push(Arc::new(sys_control::OpenvtApplet));
    #[cfg(feature = "watch")]
    applets.push(Arc::new(sys_control::WatchApplet));
    #[cfg(feature = "setlogcons")]
    applets.push(Arc::new(sys_control::SetlogconsApplet));
    #[cfg(feature = "setserial")]
    applets.push(Arc::new(sys_control::SetserialApplet));
    #[cfg(feature = "partprobe")]
    applets.push(Arc::new(sys_control::PartprobeApplet));
    #[cfg(feature = "nbd_client")]
    applets.push(Arc::new(sys_control::NbdClientApplet));
    #[cfg(feature = "mount")]
    applets.push(Arc::new(util::MountApplet));
    #[cfg(feature = "umount")]
    applets.push(Arc::new(util::UmountApplet));
    #[cfg(feature = "dmesg")]
    applets.push(Arc::new(util::DmesgApplet));
    #[cfg(feature = "lsblk")]
    applets.push(Arc::new(util::LsblkApplet));
    #[cfg(feature = "flock")]
    applets.push(Arc::new(util::FlockApplet));
    #[cfg(feature = "test")]
    applets.push(Arc::new(util::TestApplet));
    #[cfg(feature = "test")]
    applets.push(Arc::new(util::LBracketApplet));
}
