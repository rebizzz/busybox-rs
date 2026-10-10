use crate::core::{Applet, applet::AppletEntry};

#[cfg(feature = "test")]
fn run_lbracket(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::lbracket::LBracketApplet.run(args)
}

#[cfg(feature = "test_extended")]
fn run_double_lbracket(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::shell::double_lbracket::DoubleLBracketApplet.run(args)
}

#[cfg(feature = "acpid")]
fn run_acpid(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::AcpidApplet.run(args)
}

#[cfg(feature = "add_shell")]
fn run_add_shell(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::AddShellApplet.run(args)
}

#[cfg(feature = "addgroup")]
fn run_addgroup(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::AddgroupApplet.run(args)
}

#[cfg(feature = "adduser")]
fn run_adduser(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::AdduserApplet.run(args)
}

#[cfg(feature = "adjtimex")]
fn run_adjtimex(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::adjtimex::AdjtimexApplet.run(args)
}

#[cfg(feature = "arch")]
fn run_arch(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::arch::ArchApplet.run(args)
}

#[cfg(feature = "arp")]
fn run_arp(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::arp::ArpApplet.run(args)
}

#[cfg(feature = "arping")]
fn run_arping(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::arping::ArpingApplet.run(args)
}

#[cfg(feature = "ascii")]
fn run_ascii(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::ascii::AsciiApplet.run(args)
}

#[cfg(feature = "ash")]
fn run_ash(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::shell::ash::AshApplet.run(args)
}

#[cfg(feature = "awk")]
fn run_awk(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::awk::AwkApplet.run(args)
}

#[cfg(feature = "base32")]
fn run_base32(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::base32::Base32Applet.run(args)
}

#[cfg(feature = "base64")]
fn run_base64(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::base64::Base64Applet.run(args)
}

#[cfg(feature = "basename")]
fn run_basename(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::basename::BasenameApplet.run(args)
}

#[cfg(feature = "bc")]
fn run_bc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::bc::BcApplet.run(args)
}

#[cfg(feature = "beep")]
fn run_beep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::BeepApplet.run(args)
}

#[cfg(feature = "blkdiscard")]
fn run_blkdiscard(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::BlkdiscardApplet.run(args)
}

#[cfg(feature = "blkid")]
fn run_blkid(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::blkid::BlkidApplet.run(args)
}

#[cfg(feature = "blockdev")]
fn run_blockdev(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::BlockdevApplet.run(args)
}

#[cfg(feature = "bootchartd")]
fn run_bootchartd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::BootchartdApplet.run(args)
}

#[cfg(feature = "brctl")]
fn run_brctl(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::brctl::BrctlApplet.run(args)
}

#[cfg(feature = "bunzip2")]
fn run_bunzip2(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::bunzip2::Bunzip2Applet.run(args)
}

#[cfg(feature = "bzcat")]
fn run_bzcat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::bzcat::BzcatApplet.run(args)
}

#[cfg(feature = "bzip2")]
fn run_bzip2(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::bzip2::Bzip2Applet.run(args)
}

#[cfg(feature = "cal")]
fn run_cal(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::cal::CalApplet.run(args)
}

#[cfg(feature = "cat")]
fn run_cat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::cat::CatApplet.run(args)
}

#[cfg(feature = "chat")]
fn run_chat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::chat::ChatApplet.run(args)
}

#[cfg(feature = "chattr")]
fn run_chattr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::chattr::ChattrApplet.run(args)
}

#[cfg(feature = "chgrp")]
fn run_chgrp(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::chgrp::ChgrpApplet.run(args)
}

#[cfg(feature = "chmod")]
fn run_chmod(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::chmod::ChmodApplet.run(args)
}

#[cfg(feature = "chown")]
fn run_chown(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::chown::ChownApplet.run(args)
}

#[cfg(feature = "chpasswd")]
fn run_chpasswd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::ChpasswdApplet.run(args)
}

#[cfg(feature = "chpst")]
fn run_chpst(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::chpst::ChpstApplet.run(args)
}

#[cfg(feature = "chroot")]
fn run_chroot(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::chroot::ChrootApplet.run(args)
}

#[cfg(feature = "chrt")]
fn run_chrt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::chrt::ChrtApplet.run(args)
}

#[cfg(feature = "chvt")]
fn run_chvt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::ChvtApplet.run(args)
}

#[cfg(feature = "cksum")]
fn run_cksum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::cksum::CksumApplet.run(args)
}

#[cfg(feature = "clear")]
fn run_clear(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::clear::ClearApplet.run(args)
}

#[cfg(feature = "cmp")]
fn run_cmp(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::cmp::CmpApplet.run(args)
}

#[cfg(feature = "comm")]
fn run_comm(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::comm::CommApplet.run(args)
}

#[cfg(feature = "conspy")]
fn run_conspy(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::conspy::ConspyApplet.run(args)
}

#[cfg(feature = "cp")]
fn run_cp(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::cp::CpApplet.run(args)
}

#[cfg(feature = "cpio")]
fn run_cpio(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::cpio::CpioApplet.run(args)
}

#[cfg(feature = "crc32")]
fn run_crc32(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::crc32::Crc32Applet.run(args)
}

#[cfg(feature = "crond")]
fn run_crond(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::CrondApplet.run(args)
}

#[cfg(feature = "crontab")]
fn run_crontab(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::CrontabApplet.run(args)
}

#[cfg(feature = "cryptpw")]
fn run_cryptpw(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::CryptpwApplet.run(args)
}

#[cfg(feature = "cttyhack")]
fn run_cttyhack(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::cttyhack::CttyhackApplet.run(args)
}

#[cfg(feature = "cut")]
fn run_cut(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::cut::CutApplet.run(args)
}

#[cfg(feature = "date")]
fn run_date(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::date::DateApplet.run(args)
}

#[cfg(feature = "dc")]
fn run_dc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::dc::DcApplet.run(args)
}

#[cfg(feature = "dd")]
fn run_dd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::dd::DdApplet.run(args)
}

#[cfg(feature = "deallocvt")]
fn run_deallocvt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::DeallocvtApplet.run(args)
}

#[cfg(feature = "delgroup")]
fn run_delgroup(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::DelgroupApplet.run(args)
}

#[cfg(feature = "deluser")]
fn run_deluser(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::DeluserApplet.run(args)
}

#[cfg(feature = "depmod")]
fn run_depmod(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::DepmodApplet.run(args)
}

#[cfg(feature = "devmem")]
fn run_devmem(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::DevmemApplet.run(args)
}

#[cfg(feature = "df")]
fn run_df(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::df::DfApplet.run(args)
}

#[cfg(feature = "dhcprelay")]
fn run_dhcprelay(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::dhcprelay::DhcprelayApplet.run(args)
}

#[cfg(feature = "diff")]
fn run_diff(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::diff::DiffApplet.run(args)
}

#[cfg(feature = "dirname")]
fn run_dirname(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::dirname::DirnameApplet.run(args)
}

#[cfg(feature = "dmesg")]
fn run_dmesg(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::dmesg::DmesgApplet.run(args)
}

#[cfg(feature = "dnsd")]
fn run_dnsd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::dnsd::DnsdApplet.run(args)
}

#[cfg(feature = "dnsdomainname")]
fn run_dnsdomainname(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::dnsdomainname::DnsdomainnameApplet.run(args)
}

#[cfg(feature = "dos2unix")]
fn run_dos2unix(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::dos2unix::Dos2unixApplet.run(args)
}

#[cfg(feature = "dpkg")]
fn run_dpkg(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::dpkg::DpkgApplet.run(args)
}

#[cfg(feature = "dpkg_deb")]
fn run_dpkg_deb(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::dpkg_deb::DpkgDebApplet.run(args)
}

#[cfg(feature = "du")]
fn run_du(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::du::DuApplet.run(args)
}

#[cfg(feature = "dumpkmap")]
fn run_dumpkmap(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::DumpkmapApplet.run(args)
}

#[cfg(feature = "dumpleases")]
fn run_dumpleases(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::dumpleases::DumpleasesApplet.run(args)
}

#[cfg(feature = "echo")]
fn run_echo(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::echo::EchoApplet.run(args)
}

#[cfg(feature = "ed")]
fn run_ed(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::ed::EdApplet.run(args)
}

#[cfg(feature = "egrep")]
fn run_egrep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::findutils::grep::EgrepApplet.run(args)
}

#[cfg(feature = "eject")]
fn run_eject(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::EjectApplet.run(args)
}

#[cfg(feature = "env")]
fn run_env(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::env::EnvApplet.run(args)
}

#[cfg(feature = "envdir")]
fn run_envdir(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::envdir::EnvdirApplet.run(args)
}

#[cfg(feature = "envuidgid")]
fn run_envuidgid(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::envuidgid::EnvuidgidApplet.run(args)
}

#[cfg(feature = "ether_wake")]
fn run_ether_wake(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ether_wake::EtherWakeApplet.run(args)
}

#[cfg(feature = "expand")]
fn run_expand(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::expand::ExpandApplet.run(args)
}

#[cfg(feature = "expr")]
fn run_expr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::expr::ExprApplet.run(args)
}

#[cfg(feature = "factor")]
fn run_factor(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::factor::FactorApplet.run(args)
}

#[cfg(feature = "fakeidentd")]
fn run_fakeidentd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::fakeidentd::FakeidentdApplet.run(args)
}

#[cfg(feature = "fallocate")]
fn run_fallocate(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fallocate::FallocateApplet.run(args)
}

#[cfg(feature = "false")]
fn run_false(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::r#false::FalseApplet.run(args)
}

#[cfg(feature = "fatattr")]
fn run_fatattr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::fatattr::FatattrApplet.run(args)
}

#[cfg(feature = "fatlabel")]
fn run_fatlabel(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fatlabel::FatlabelApplet.run(args)
}

#[cfg(feature = "fbset")]
fn run_fbset(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::FbsetApplet.run(args)
}

#[cfg(feature = "fbsplash")]
fn run_fbsplash(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::FbsplashApplet.run(args)
}

#[cfg(feature = "fdflush")]
fn run_fdflush(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::FdflushApplet.run(args)
}

#[cfg(feature = "fdformat")]
fn run_fdformat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::FdformatApplet.run(args)
}

#[cfg(feature = "fdisk")]
fn run_fdisk(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fdisk::FdiskApplet.run(args)
}

#[cfg(feature = "fgconsole")]
fn run_fgconsole(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::FgconsoleApplet.run(args)
}

#[cfg(feature = "fgrep")]
fn run_fgrep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::findutils::grep::FgrepApplet.run(args)
}

#[cfg(feature = "find")]
fn run_find(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::findutils::find::FindApplet.run(args)
}

#[cfg(feature = "findfs")]
fn run_findfs(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::findfs::FindfsApplet.run(args)
}

#[cfg(feature = "flock")]
fn run_flock(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::flock::FlockApplet.run(args)
}

#[cfg(feature = "fold")]
fn run_fold(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::fold::FoldApplet.run(args)
}

#[cfg(feature = "free")]
fn run_free(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::free::FreeApplet.run(args)
}

#[cfg(feature = "freeramdisk")]
fn run_freeramdisk(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::FreeramdiskApplet.run(args)
}

#[cfg(feature = "fsck")]
fn run_fsck(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fsck::FsckApplet.run(args)
}

#[cfg(feature = "fsck_minix")]
fn run_fsck_minix(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fsck_minix::FsckMinixApplet.run(args)
}

#[cfg(feature = "fsfreeze")]
fn run_fsfreeze(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fsfreeze::FsfreezeApplet.run(args)
}

#[cfg(feature = "fstrim")]
fn run_fstrim(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::fstrim::FstrimApplet.run(args)
}

#[cfg(feature = "fsync")]
fn run_fsync(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::FsyncApplet.run(args)
}

#[cfg(feature = "ftpd")]
fn run_ftpd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ftpd::FtpdApplet.run(args)
}

#[cfg(feature = "ftpget")]
fn run_ftpget(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ftpget::FtpgetApplet.run(args)
}

#[cfg(feature = "ftpput")]
fn run_ftpput(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ftpput::FtpputApplet.run(args)
}

#[cfg(feature = "fuser")]
fn run_fuser(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::fuser::FuserApplet.run(args)
}

#[cfg(feature = "getfattr")]
fn run_getfattr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::getfattr::GetfattrApplet.run(args)
}

#[cfg(feature = "getopt")]
fn run_getopt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::getopt::GetoptApplet.run(args)
}

#[cfg(feature = "getty")]
fn run_getty(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::GettyApplet.run(args)
}

#[cfg(feature = "grep")]
fn run_grep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::findutils::grep::GrepApplet.run(args)
}

#[cfg(feature = "groups")]
fn run_groups(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::groups::GroupsApplet.run(args)
}

#[cfg(feature = "gunzip")]
fn run_gunzip(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::gunzip::GunzipApplet.run(args)
}

#[cfg(feature = "gzip")]
fn run_gzip(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::gzip::GzipApplet.run(args)
}

#[cfg(feature = "halt")]
fn run_halt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::HaltApplet.run(args)
}

#[cfg(feature = "hd")]
fn run_hd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::hd::HdApplet.run(args)
}

#[cfg(feature = "hdparm")]
fn run_hdparm(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::HdparmApplet.run(args)
}

#[cfg(feature = "head")]
fn run_head(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::head::HeadApplet.run(args)
}

#[cfg(feature = "hexdump")]
fn run_hexdump(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::hexdump::HexdumpApplet.run(args)
}

#[cfg(feature = "hexedit")]
fn run_hexedit(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::hexedit::HexeditApplet.run(args)
}

#[cfg(feature = "hostid")]
fn run_hostid(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::hostid::HostidApplet.run(args)
}

#[cfg(feature = "hostname")]
fn run_hostname(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::hostname::HostnameApplet.run(args)
}

#[cfg(feature = "httpd")]
fn run_httpd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::httpd::HttpdApplet.run(args)
}

#[cfg(feature = "hush")]
fn run_hush(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::shell::hush::HushApplet.run(args)
}

#[cfg(feature = "hwclock")]
fn run_hwclock(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::hwclock::HwclockApplet.run(args)
}

#[cfg(feature = "i2cdetect")]
fn run_i2cdetect(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::I2cdetectApplet.run(args)
}

#[cfg(feature = "i2cdump")]
fn run_i2cdump(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::I2cdumpApplet.run(args)
}

#[cfg(feature = "i2cget")]
fn run_i2cget(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::I2cgetApplet.run(args)
}

#[cfg(feature = "i2cset")]
fn run_i2cset(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::I2csetApplet.run(args)
}

#[cfg(feature = "i2ctransfer")]
fn run_i2ctransfer(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::I2ctransferApplet.run(args)
}

#[cfg(feature = "id")]
fn run_id(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::id::IdApplet.run(args)
}

#[cfg(feature = "ifconfig")]
fn run_ifconfig(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ifconfig::IfconfigApplet.run(args)
}

#[cfg(feature = "ifdown")]
fn run_ifdown(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ifdown::IfdownApplet.run(args)
}

#[cfg(feature = "ifenslave")]
fn run_ifenslave(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ifenslave::IfenslaveApplet.run(args)
}

#[cfg(feature = "ifplugd")]
fn run_ifplugd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ifplugd::IfplugdApplet.run(args)
}

#[cfg(feature = "ifup")]
fn run_ifup(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ifup::IfupApplet.run(args)
}

#[cfg(feature = "inetd")]
fn run_inetd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::inetd::InetdApplet.run(args)
}

#[cfg(feature = "init")]
fn run_init(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::InitApplet.run(args)
}

#[cfg(feature = "insmod")]
fn run_insmod(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::InsmodApplet.run(args)
}

#[cfg(feature = "install")]
fn run_install(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::install::InstallApplet.run(args)
}

#[cfg(feature = "ionice")]
fn run_ionice(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::ionice::IoniceApplet.run(args)
}

#[cfg(feature = "iostat")]
fn run_iostat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::iostat::IostatApplet.run(args)
}

#[cfg(feature = "ip")]
fn run_ip(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ip::IpApplet.run(args)
}

#[cfg(feature = "ipaddr")]
fn run_ipaddr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ipaddr::IpaddrApplet.run(args)
}

#[cfg(feature = "ipcalc")]
fn run_ipcalc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ipcalc::IpcalcApplet.run(args)
}

#[cfg(feature = "ipcrm")]
fn run_ipcrm(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::ipcrm::IpcrmApplet.run(args)
}

#[cfg(feature = "ipcs")]
fn run_ipcs(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::ipcs::IpcsApplet.run(args)
}

#[cfg(feature = "iplink")]
fn run_iplink(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::iplink::IplinkApplet.run(args)
}

#[cfg(feature = "ipneigh")]
fn run_ipneigh(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ipneigh::IpneighApplet.run(args)
}

#[cfg(feature = "iproute")]
fn run_iproute(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::iproute::IprouteApplet.run(args)
}

#[cfg(feature = "iprule")]
fn run_iprule(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::iprule::IpruleApplet.run(args)
}

#[cfg(feature = "iptunnel")]
fn run_iptunnel(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::iptunnel::IptunnelApplet.run(args)
}

#[cfg(feature = "join")]
fn run_join(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::join::JoinApplet.run(args)
}

#[cfg(feature = "kbd_mode")]
fn run_kbd_mode(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::KbdModeApplet.run(args)
}

#[cfg(feature = "kill")]
fn run_kill(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::kill::KillApplet.run(args)
}

#[cfg(feature = "killall")]
fn run_killall(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::killall::KillallApplet.run(args)
}

#[cfg(feature = "killall5")]
fn run_killall5(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::killall5::Killall5Applet.run(args)
}

#[cfg(feature = "klogd")]
fn run_klogd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::KlogdApplet.run(args)
}

#[cfg(feature = "last")]
fn run_last(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::LastApplet.run(args)
}

#[cfg(feature = "less")]
fn run_less(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::less::LessApplet.run(args)
}

#[cfg(feature = "link")]
fn run_link(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::link::LinkApplet.run(args)
}

#[cfg(feature = "linux32")]
fn run_linux32(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::linux32::Linux32Applet.run(args)
}

#[cfg(feature = "linux64")]
fn run_linux64(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::linux64::Linux64Applet.run(args)
}

#[cfg(feature = "linuxrc")]
fn run_linuxrc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::LinuxrcApplet.run(args)
}

#[cfg(feature = "ln")]
fn run_ln(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::ln::LnApplet.run(args)
}

#[cfg(feature = "loadfont")]
fn run_loadfont(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::LoadfontApplet.run(args)
}

#[cfg(feature = "loadkmap")]
fn run_loadkmap(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::LoadkmapApplet.run(args)
}

#[cfg(feature = "logger")]
fn run_logger(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::LoggerApplet.run(args)
}

#[cfg(feature = "login")]
fn run_login(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::LoginApplet.run(args)
}

#[cfg(feature = "logname")]
fn run_logname(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::logname::LognameApplet.run(args)
}

#[cfg(feature = "logread")]
fn run_logread(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::LogreadApplet.run(args)
}

#[cfg(feature = "losetup")]
fn run_losetup(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::losetup::LosetupApplet.run(args)
}

#[cfg(feature = "lpd")]
fn run_lpd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::lpd::LpdApplet.run(args)
}

#[cfg(feature = "lpq")]
fn run_lpq(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::lpq::LpqApplet.run(args)
}

#[cfg(feature = "lpr")]
fn run_lpr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::lpr::LprApplet.run(args)
}

#[cfg(feature = "ls")]
fn run_ls(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::ls::LsApplet.run(args)
}

#[cfg(feature = "lsattr")]
fn run_lsattr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::lsattr::LsattrApplet.run(args)
}

#[cfg(feature = "lsblk")]
fn run_lsblk(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::lsblk::LsblkApplet.run(args)
}

#[cfg(feature = "lsmod")]
fn run_lsmod(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::LsmodApplet.run(args)
}

#[cfg(feature = "lsof")]
fn run_lsof(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::LsofApplet.run(args)
}

#[cfg(feature = "lspci")]
fn run_lspci(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::LspciApplet.run(args)
}

#[cfg(feature = "lsscsi")]
fn run_lsscsi(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::LsscsiApplet.run(args)
}

#[cfg(feature = "lsusb")]
fn run_lsusb(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::LsusbApplet.run(args)
}

#[cfg(feature = "lzcat")]
fn run_lzcat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::lzcat::LzcatApplet.run(args)
}

#[cfg(feature = "lzma")]
fn run_lzma(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::lzma::LzmaApplet.run(args)
}

#[cfg(feature = "lzop")]
fn run_lzop(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::lzop::LzopApplet.run(args)
}

#[cfg(feature = "makedevs")]
fn run_makedevs(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::makedevs::MakedevsApplet.run(args)
}

#[cfg(feature = "makemime")]
fn run_makemime(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::makemime::MakemimeApplet.run(args)
}

#[cfg(feature = "man")]
fn run_man(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::man::ManApplet.run(args)
}

#[cfg(feature = "md5sum")]
fn run_md5sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::md5sum::Md5SumApplet.run(args)
}

#[cfg(feature = "mdev")]
fn run_mdev(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mdev::MdevApplet.run(args)
}

#[cfg(feature = "mesg")]
fn run_mesg(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::MesgApplet.run(args)
}

#[cfg(feature = "microcom")]
fn run_microcom(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::microcom::MicrocomApplet.run(args)
}

#[cfg(feature = "mim")]
fn run_mim(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::MimApplet.run(args)
}

#[cfg(feature = "mkdir")]
fn run_mkdir(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mkdir::MkdirApplet.run(args)
}

#[cfg(feature = "mkdosfs")]
fn run_mkdosfs(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mkdosfs::MkdosfsApplet.run(args)
}

#[cfg(feature = "mke2fs")]
fn run_mke2fs(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mke2fs::Mke2fsApplet.run(args)
}

#[cfg(feature = "mkfifo")]
fn run_mkfifo(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mkfifo::MkfifoApplet.run(args)
}

#[cfg(feature = "mkfs_ext2")]
fn run_mkfs_ext2(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mkfs_ext2::MkfsExt2Applet.run(args)
}

#[cfg(feature = "mkfs_minix")]
fn run_mkfs_minix(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mkfs_minix::MkfsMinixApplet.run(args)
}

#[cfg(feature = "mkfs_vfat")]
fn run_mkfs_vfat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mkfs_vfat::MkfsVfatApplet.run(args)
}

#[cfg(feature = "mknod")]
fn run_mknod(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mknod::MknodApplet.run(args)
}

#[cfg(feature = "mkpasswd")]
fn run_mkpasswd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::MkpasswdApplet.run(args)
}

#[cfg(feature = "mkswap")]
fn run_mkswap(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mkswap::MkswapApplet.run(args)
}

#[cfg(feature = "mktemp")]
fn run_mktemp(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::mktemp::MktempApplet.run(args)
}

#[cfg(feature = "modinfo")]
fn run_modinfo(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::ModinfoApplet.run(args)
}

#[cfg(feature = "modprobe")]
fn run_modprobe(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::ModprobeApplet.run(args)
}

#[cfg(feature = "more")]
fn run_more(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::more::MoreApplet.run(args)
}

#[cfg(feature = "mount")]
fn run_mount(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::mount::MountApplet.run(args)
}

#[cfg(feature = "mountpoint")]
fn run_mountpoint(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mountpoint::MountpointApplet.run(args)
}

#[cfg(feature = "mpstat")]
fn run_mpstat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mpstat::MpstatApplet.run(args)
}

#[cfg(feature = "mt")]
fn run_mt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::mt::MtApplet.run(args)
}

#[cfg(feature = "mv")]
fn run_mv(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::mv::MvApplet.run(args)
}

#[cfg(feature = "nameif")]
fn run_nameif(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::nameif::NameifApplet.run(args)
}

#[cfg(feature = "nanddump")]
fn run_nanddump(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::nanddump::NanddumpApplet.run(args)
}

#[cfg(feature = "nandwrite")]
fn run_nandwrite(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::nandwrite::NandwriteApplet.run(args)
}

#[cfg(feature = "nbd_client")]
fn run_nbd_client(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::nbd_client::NbdClientApplet.run(args)
}

#[cfg(feature = "nc")]
fn run_nc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::nc::NcApplet.run(args)
}

#[cfg(feature = "netstat")]
fn run_netstat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::netstat::NetstatApplet.run(args)
}

#[cfg(feature = "nice")]
fn run_nice(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::nice::NiceApplet.run(args)
}

#[cfg(feature = "nl")]
fn run_nl(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::nl::NlApplet.run(args)
}

#[cfg(feature = "nmeter")]
fn run_nmeter(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::nmeter::NmeterApplet.run(args)
}

#[cfg(feature = "nohup")]
fn run_nohup(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::nohup::NohupApplet.run(args)
}

#[cfg(feature = "nologin")]
fn run_nologin(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::NologinApplet.run(args)
}

#[cfg(feature = "nproc")]
fn run_nproc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::nproc::NprocApplet.run(args)
}

#[cfg(feature = "nsenter")]
fn run_nsenter(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::nsenter::NsenterApplet.run(args)
}

#[cfg(feature = "nslookup")]
fn run_nslookup(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::nslookup::NslookupApplet.run(args)
}

#[cfg(feature = "ntpd")]
fn run_ntpd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ntpd::NtpdApplet.run(args)
}

#[cfg(feature = "od")]
fn run_od(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::od::OdApplet.run(args)
}

#[cfg(feature = "openvt")]
fn run_openvt(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::openvt::OpenvtApplet.run(args)
}

#[cfg(feature = "partprobe")]
fn run_partprobe(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::partprobe::PartprobeApplet.run(args)
}

#[cfg(feature = "passwd")]
fn run_passwd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::PasswdApplet.run(args)
}

#[cfg(feature = "paste")]
fn run_paste(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::paste::PasteApplet.run(args)
}

#[cfg(feature = "patch")]
fn run_patch(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::patch::PatchApplet.run(args)
}

#[cfg(feature = "pgrep")]
fn run_pgrep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::pgrep::PgrepApplet.run(args)
}

#[cfg(feature = "pidof")]
fn run_pidof(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::pidof::PidofApplet.run(args)
}

#[cfg(feature = "ping")]
fn run_ping(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ping::PingApplet.run(args)
}

#[cfg(feature = "ping6")]
fn run_ping6(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ping6::Ping6Applet.run(args)
}

#[cfg(feature = "pipe_progress")]
fn run_pipe_progress(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::pipe_progress::PipeProgressApplet.run(args)
}

#[cfg(feature = "pivot_root")]
fn run_pivot_root(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::pivot_root::PivotRootApplet.run(args)
}

#[cfg(feature = "pkill")]
fn run_pkill(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::pkill::PkillApplet.run(args)
}

#[cfg(feature = "pmap")]
fn run_pmap(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::pmap::PmapApplet.run(args)
}

#[cfg(feature = "popmaildir")]
fn run_popmaildir(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::popmaildir::PopmaildirApplet.run(args)
}

#[cfg(feature = "poweroff")]
fn run_poweroff(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::PoweroffApplet.run(args)
}

#[cfg(feature = "powertop")]
fn run_powertop(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::powertop::PowertopApplet.run(args)
}

#[cfg(feature = "printenv")]
fn run_printenv(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::printenv::PrintenvApplet.run(args)
}

#[cfg(feature = "printf")]
fn run_printf(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::printf::PrintfApplet.run(args)
}

#[cfg(feature = "ps")]
fn run_ps(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::ps::PsApplet.run(args)
}

#[cfg(feature = "pscan")]
fn run_pscan(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::pscan::PscanApplet.run(args)
}

#[cfg(feature = "pstree")]
fn run_pstree(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::pstree::PstreeApplet.run(args)
}

#[cfg(feature = "pwd")]
fn run_pwd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::pwd::PwdApplet.run(args)
}

#[cfg(feature = "pwdx")]
fn run_pwdx(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::pwdx::PwdxApplet.run(args)
}

#[cfg(feature = "raidautorun")]
fn run_raidautorun(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::raidautorun::RaidautorunApplet.run(args)
}

#[cfg(feature = "rdate")]
fn run_rdate(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::rdate::RdateApplet.run(args)
}

#[cfg(feature = "rdev")]
fn run_rdev(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::rdev::RdevApplet.run(args)
}

#[cfg(feature = "readahead")]
fn run_readahead(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::miscutils::hardware::ReadaheadApplet.run(args)
}

#[cfg(feature = "readlink")]
fn run_readlink(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::readlink::ReadlinkApplet.run(args)
}

#[cfg(feature = "readprofile")]
fn run_readprofile(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::readprofile::ReadprofileApplet.run(args)
}

#[cfg(feature = "realpath")]
fn run_realpath(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::realpath::RealpathApplet.run(args)
}

#[cfg(feature = "reboot")]
fn run_reboot(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::RebootApplet.run(args)
}

#[cfg(feature = "reformime")]
fn run_reformime(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::reformime::ReformimeApplet.run(args)
}

#[cfg(feature = "remove_shell")]
fn run_remove_shell(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::RemoveShellApplet.run(args)
}

#[cfg(feature = "renice")]
fn run_renice(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::renice::ReniceApplet.run(args)
}

#[cfg(feature = "reset")]
fn run_reset(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::reset::ResetApplet.run(args)
}

#[cfg(feature = "resize")]
fn run_resize(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::resize::ResizeApplet.run(args)
}

#[cfg(feature = "resume")]
fn run_resume(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::resume::ResumeApplet.run(args)
}

#[cfg(feature = "rev")]
fn run_rev(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::rev::RevApplet.run(args)
}

#[cfg(feature = "rm")]
fn run_rm(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::rm::RmApplet.run(args)
}

#[cfg(feature = "rmdir")]
fn run_rmdir(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::rmdir::RmdirApplet.run(args)
}

#[cfg(feature = "rmmod")]
fn run_rmmod(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::RmmodApplet.run(args)
}

#[cfg(feature = "route")]
fn run_route(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::route::RouteApplet.run(args)
}

#[cfg(feature = "rpm")]
fn run_rpm(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::rpm::RpmApplet.run(args)
}

#[cfg(feature = "rpm2cpio")]
fn run_rpm2cpio(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::rpm2cpio::Rpm2cpioApplet.run(args)
}

#[cfg(feature = "rtcwake")]
fn run_rtcwake(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::rtcwake::RtcwakeApplet.run(args)
}

#[cfg(feature = "run_init")]
fn run_run_init(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::run_init::RunInitApplet.run(args)
}

#[cfg(feature = "run_parts")]
fn run_run_parts(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::run_parts::RunPartsApplet.run(args)
}

#[cfg(feature = "runlevel")]
fn run_runlevel(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::RunlevelApplet.run(args)
}

#[cfg(feature = "runsv")]
fn run_runsv(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::runsv::RunsvApplet.run(args)
}

#[cfg(feature = "runsvdir")]
fn run_runsvdir(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::runsvdir::RunsvdirApplet.run(args)
}

#[cfg(feature = "rx")]
fn run_rx(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::rx::RxApplet.run(args)
}

#[cfg(feature = "script")]
fn run_script(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::script::ScriptApplet.run(args)
}

#[cfg(feature = "scriptreplay")]
fn run_scriptreplay(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::scriptreplay::ScriptreplayApplet.run(args)
}

#[cfg(feature = "sed")]
fn run_sed(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::sed::SedApplet.run(args)
}

#[cfg(feature = "seedrng")]
fn run_seedrng(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::seedrng::SeedrngApplet.run(args)
}

#[cfg(feature = "sendmail")]
fn run_sendmail(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::sendmail::SendmailApplet.run(args)
}

#[cfg(feature = "seq")]
fn run_seq(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::seq::SeqApplet.run(args)
}

#[cfg(feature = "setarch")]
fn run_setarch(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::setarch::SetarchApplet.run(args)
}

#[cfg(feature = "setconsole")]
fn run_setconsole(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::setconsole::SetconsoleApplet.run(args)
}

#[cfg(feature = "setfattr")]
fn run_setfattr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::setfattr::SetfattrApplet.run(args)
}

#[cfg(feature = "setfont")]
fn run_setfont(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::SetfontApplet.run(args)
}

#[cfg(feature = "setkeycodes")]
fn run_setkeycodes(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::SetkeycodesApplet.run(args)
}

#[cfg(feature = "setlogcons")]
fn run_setlogcons(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::setlogcons::SetlogconsApplet.run(args)
}

#[cfg(feature = "setpriv")]
fn run_setpriv(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::setpriv::SetprivApplet.run(args)
}

#[cfg(feature = "setserial")]
fn run_setserial(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::setserial::SetserialApplet.run(args)
}

#[cfg(feature = "setsid")]
fn run_setsid(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::setsid::SetsidApplet.run(args)
}

#[cfg(feature = "setuidgid")]
fn run_setuidgid(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::setuidgid::SetuidgidApplet.run(args)
}

#[cfg(feature = "sh")]
fn run_sh(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::shell::shell::ShApplet.run(args)
}

#[cfg(feature = "sha1sum")]
fn run_sha1sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sha1sum::Sha1SumApplet.run(args)
}

#[cfg(feature = "sha256sum")]
fn run_sha256sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sha256sum::Sha256SumApplet.run(args)
}

#[cfg(feature = "sha384sum")]
fn run_sha384sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sha384sum::Sha384sumApplet.run(args)
}

#[cfg(feature = "sha3sum")]
fn run_sha3sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sha3sum::Sha3sumApplet.run(args)
}

#[cfg(feature = "sha512sum")]
fn run_sha512sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sha512sum::Sha512SumApplet.run(args)
}

#[cfg(feature = "showkey")]
fn run_showkey(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::console_tools::console::ShowkeyApplet.run(args)
}

#[cfg(feature = "shred")]
fn run_shred(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::shred::ShredApplet.run(args)
}

#[cfg(feature = "shuf")]
fn run_shuf(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::shuf::ShufApplet.run(args)
}

#[cfg(feature = "slattach")]
fn run_slattach(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::slattach::SlattachApplet.run(args)
}

#[cfg(feature = "sleep")]
fn run_sleep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sleep::SleepApplet.run(args)
}

#[cfg(feature = "smemcap")]
fn run_smemcap(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::smemcap::SmemcapApplet.run(args)
}

#[cfg(feature = "softlimit")]
fn run_softlimit(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::softlimit::SoftlimitApplet.run(args)
}

#[cfg(feature = "sort")]
fn run_sort(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sort::SortApplet.run(args)
}

#[cfg(feature = "split")]
fn run_split(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::split::SplitApplet.run(args)
}

#[cfg(feature = "ssl_client")]
fn run_ssl_client(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ssl_client::SslClientApplet.run(args)
}

#[cfg(feature = "ssl_server")]
fn run_ssl_server(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::ssl_server::SslServerApplet.run(args)
}

#[cfg(feature = "start_stop_daemon")]
fn run_start_stop_daemon(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::start_stop_daemon::StartStopDaemonApplet.run(args)
}

#[cfg(feature = "stat")]
fn run_stat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::stat::StatApplet.run(args)
}

#[cfg(feature = "strings")]
fn run_strings(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::strings::StringsApplet.run(args)
}

#[cfg(feature = "stty")]
fn run_stty(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::stty::SttyApplet.run(args)
}

#[cfg(feature = "su")]
fn run_su(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::SuApplet.run(args)
}

#[cfg(feature = "sulogin")]
fn run_sulogin(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::SuloginApplet.run(args)
}

#[cfg(feature = "sum")]
fn run_sum(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sum::SumApplet.run(args)
}

#[cfg(feature = "sv")]
fn run_sv(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::sv::SvApplet.run(args)
}

#[cfg(feature = "svc")]
fn run_svc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::svc::SvcApplet.run(args)
}

#[cfg(feature = "svlogd")]
fn run_svlogd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::svlogd::SvlogdApplet.run(args)
}

#[cfg(feature = "svok")]
fn run_svok(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::svok::SvokApplet.run(args)
}

#[cfg(feature = "swaplabel")]
fn run_swaplabel(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::swaplabel::SwaplabelApplet.run(args)
}

#[cfg(feature = "swapoff")]
fn run_swapoff(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::swapoff::SwapoffApplet.run(args)
}

#[cfg(feature = "swapon")]
fn run_swapon(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::swapon::SwaponApplet.run(args)
}

#[cfg(feature = "switch_root")]
fn run_switch_root(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::switch_root::SwitchRootApplet.run(args)
}

#[cfg(feature = "sync")]
fn run_sync(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::sync::SyncApplet.run(args)
}

#[cfg(feature = "sysctl")]
fn run_sysctl(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::sysctl::SysctlApplet.run(args)
}

#[cfg(feature = "syslogd")]
fn run_syslogd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::SyslogdApplet.run(args)
}

#[cfg(feature = "tac")]
fn run_tac(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::tac::TacApplet.run(args)
}

#[cfg(feature = "tail")]
fn run_tail(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::tail::TailApplet.run(args)
}

#[cfg(feature = "tar")]
fn run_tar(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::tar::TarApplet.run(args)
}

#[cfg(feature = "taskset")]
fn run_taskset(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::taskset::TasksetApplet.run(args)
}

#[cfg(feature = "tcpsvd")]
fn run_tcpsvd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::tcpsvd::TcpsvdApplet.run(args)
}

#[cfg(feature = "tee")]
fn run_tee(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::tee::TeeApplet.run(args)
}

#[cfg(feature = "telnet")]
fn run_telnet(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::telnet::TelnetApplet.run(args)
}

#[cfg(feature = "telnetd")]
fn run_telnetd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::telnetd::TelnetdApplet.run(args)
}

#[cfg(feature = "test")]
fn run_test(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::test::TestApplet.run(args)
}

#[cfg(feature = "tftp")]
fn run_tftp(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::tftp::TftpApplet.run(args)
}

#[cfg(feature = "tftpd")]
fn run_tftpd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::tftpd::TftpdApplet.run(args)
}

#[cfg(feature = "time")]
fn run_time(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::time::TimeApplet.run(args)
}

#[cfg(feature = "timeout")]
fn run_timeout(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::timeout::TimeoutApplet.run(args)
}

#[cfg(feature = "top")]
fn run_top(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::top::TopApplet.run(args)
}

#[cfg(feature = "touch")]
fn run_touch(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::touch::TouchApplet.run(args)
}

#[cfg(feature = "tr")]
fn run_tr(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::tr::TrApplet.run(args)
}

#[cfg(feature = "traceroute")]
fn run_traceroute(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::traceroute::TracerouteApplet.run(args)
}

#[cfg(feature = "traceroute6")]
fn run_traceroute6(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::traceroute6::Traceroute6Applet.run(args)
}

#[cfg(feature = "tree")]
fn run_tree(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::modutils::modules::TreeApplet.run(args)
}

#[cfg(feature = "true")]
fn run_true(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::r#true::TrueApplet.run(args)
}

#[cfg(feature = "truncate")]
fn run_truncate(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::truncate::TruncateApplet.run(args)
}

#[cfg(feature = "ts")]
fn run_ts(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::ts::TsApplet.run(args)
}

#[cfg(feature = "tsort")]
fn run_tsort(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::tsort::TsortApplet.run(args)
}

#[cfg(feature = "tty")]
fn run_tty(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::tty::TtyApplet.run(args)
}

#[cfg(feature = "ttysize")]
fn run_ttysize(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::ttysize::TtysizeApplet.run(args)
}

#[cfg(feature = "tunctl")]
fn run_tunctl(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::tunctl::TunctlApplet.run(args)
}

#[cfg(feature = "ubiattach")]
fn run_ubiattach(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubiattach::UbiattachApplet.run(args)
}

#[cfg(feature = "ubidetach")]
fn run_ubidetach(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubidetach::UbidetachApplet.run(args)
}

#[cfg(feature = "ubimkvol")]
fn run_ubimkvol(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubimkvol::UbimkvolApplet.run(args)
}

#[cfg(feature = "ubirename")]
fn run_ubirename(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubirename::UbirenameApplet.run(args)
}

#[cfg(feature = "ubirmvol")]
fn run_ubirmvol(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubirmvol::UbirmvolApplet.run(args)
}

#[cfg(feature = "ubirsvol")]
fn run_ubirsvol(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubirsvol::UbirsvolApplet.run(args)
}

#[cfg(feature = "ubiupdatevol")]
fn run_ubiupdatevol(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::ubiupdatevol::UbiupdatevolApplet.run(args)
}

#[cfg(feature = "udhcpc")]
fn run_udhcpc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::udhcpc::UdhcpcApplet.run(args)
}

#[cfg(feature = "udhcpc6")]
fn run_udhcpc6(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::udhcpc6::Udhcpc6Applet.run(args)
}

#[cfg(feature = "udhcpd")]
fn run_udhcpd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::udhcpd::UdhcpdApplet.run(args)
}

#[cfg(feature = "udpsvd")]
fn run_udpsvd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::udpsvd::UdpsvdApplet.run(args)
}

#[cfg(feature = "uevent")]
fn run_uevent(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::uevent::UeventApplet.run(args)
}

#[cfg(feature = "umount")]
fn run_umount(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::umount::UmountApplet.run(args)
}

#[cfg(feature = "uname")]
fn run_uname(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::uname::UnameApplet.run(args)
}

#[cfg(feature = "unexpand")]
fn run_unexpand(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::unexpand::UnexpandApplet.run(args)
}

#[cfg(feature = "uniq")]
fn run_uniq(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::uniq::UniqApplet.run(args)
}

#[cfg(feature = "unix2dos")]
fn run_unix2dos(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::unix2dos::Unix2dosApplet.run(args)
}

#[cfg(feature = "unlink")]
fn run_unlink(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::unlink::UnlinkApplet.run(args)
}

#[cfg(feature = "unlzma")]
fn run_unlzma(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::unlzma::UnlzmaApplet.run(args)
}

#[cfg(feature = "unshare")]
fn run_unshare(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::unshare::UnshareApplet.run(args)
}

#[cfg(feature = "unxz")]
fn run_unxz(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::unxz::UnxzApplet.run(args)
}

#[cfg(feature = "unzip")]
fn run_unzip(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::unzip::UnzipApplet.run(args)
}

#[cfg(feature = "uptime")]
fn run_uptime(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::uptime::UptimeApplet.run(args)
}

#[cfg(feature = "users")]
fn run_users(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::users::UsersApplet.run(args)
}

#[cfg(feature = "usleep")]
fn run_usleep(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::usleep::UsleepApplet.run(args)
}

#[cfg(feature = "uudecode")]
fn run_uudecode(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::uudecode::UudecodeApplet.run(args)
}

#[cfg(feature = "uuencode")]
fn run_uuencode(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::encode::UuencodeApplet.run(args)
}

#[cfg(feature = "uuidgen")]
fn run_uuidgen(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::uuidgen::UuidgenApplet.run(args)
}

#[cfg(feature = "vconfig")]
fn run_vconfig(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::vconfig::VconfigApplet.run(args)
}

#[cfg(feature = "vi")]
fn run_vi(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::editors::vi::ViApplet.run(args)
}

#[cfg(feature = "vlock")]
fn run_vlock(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::loginutils::login::VlockApplet.run(args)
}

#[cfg(feature = "vmstat")]
fn run_vmstat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::procps::vmstat::VmstatApplet.run(args)
}

#[cfg(feature = "volname")]
fn run_volname(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::volname::VolnameApplet.run(args)
}

#[cfg(feature = "w")]
fn run_w(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::w::WApplet.run(args)
}

#[cfg(feature = "wall")]
fn run_wall(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::init::init::WallApplet.run(args)
}

#[cfg(feature = "watch")]
fn run_watch(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::util_linux::watch::WatchApplet.run(args)
}

#[cfg(feature = "watchdog")]
fn run_watchdog(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::watchdog::WatchdogApplet.run(args)
}

#[cfg(feature = "wc")]
fn run_wc(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::wc::WcApplet.run(args)
}

#[cfg(feature = "wget")]
fn run_wget(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::wget::WgetApplet.run(args)
}

#[cfg(feature = "which")]
fn run_which(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::which::WhichApplet.run(args)
}

#[cfg(feature = "who")]
fn run_who(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::who::WhoApplet.run(args)
}

#[cfg(feature = "whoami")]
fn run_whoami(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::whoami::WhoamiApplet.run(args)
}

#[cfg(feature = "whois")]
fn run_whois(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::whois::WhoisApplet.run(args)
}

#[cfg(feature = "xargs")]
fn run_xargs(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::findutils::xargs::XargsApplet.run(args)
}

#[cfg(feature = "xxd")]
fn run_xxd(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::xxd::XxdApplet.run(args)
}

#[cfg(feature = "xz")]
fn run_xz(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::xz::XzApplet.run(args)
}

#[cfg(feature = "xzcat")]
fn run_xzcat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::xzcat::XzcatApplet.run(args)
}

#[cfg(feature = "yes")]
fn run_yes(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::coreutils::yes::YesApplet.run(args)
}

#[cfg(feature = "zcat")]
fn run_zcat(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::archival::zcat::ZcatApplet.run(args)
}

#[cfg(feature = "zcip")]
fn run_zcip(args: &[std::ffi::OsString]) -> crate::core::Result<i32> {
    crate::applets::networking::zcip::ZcipApplet.run(args)
}

pub static APPLETS: &[AppletEntry] = &[
    #[cfg(feature = "test")]
    AppletEntry {
        name: "[",
        run: run_lbracket,
    },
    #[cfg(feature = "test_extended")]
    AppletEntry {
        name: "[[",
        run: run_double_lbracket,
    },
    #[cfg(feature = "acpid")]
    AppletEntry {
        name: "acpid",
        run: run_acpid,
    },
    #[cfg(feature = "add_shell")]
    AppletEntry {
        name: "add-shell",
        run: run_add_shell,
    },
    #[cfg(feature = "addgroup")]
    AppletEntry {
        name: "addgroup",
        run: run_addgroup,
    },
    #[cfg(feature = "adduser")]
    AppletEntry {
        name: "adduser",
        run: run_adduser,
    },
    #[cfg(feature = "adjtimex")]
    AppletEntry {
        name: "adjtimex",
        run: run_adjtimex,
    },
    #[cfg(feature = "arch")]
    AppletEntry {
        name: "arch",
        run: run_arch,
    },
    #[cfg(feature = "arp")]
    AppletEntry {
        name: "arp",
        run: run_arp,
    },
    #[cfg(feature = "arping")]
    AppletEntry {
        name: "arping",
        run: run_arping,
    },
    #[cfg(feature = "ascii")]
    AppletEntry {
        name: "ascii",
        run: run_ascii,
    },
    #[cfg(feature = "ash")]
    AppletEntry {
        name: "ash",
        run: run_ash,
    },
    #[cfg(feature = "awk")]
    AppletEntry {
        name: "awk",
        run: run_awk,
    },
    #[cfg(feature = "base32")]
    AppletEntry {
        name: "base32",
        run: run_base32,
    },
    #[cfg(feature = "base64")]
    AppletEntry {
        name: "base64",
        run: run_base64,
    },
    #[cfg(feature = "basename")]
    AppletEntry {
        name: "basename",
        run: run_basename,
    },
    #[cfg(feature = "bc")]
    AppletEntry {
        name: "bc",
        run: run_bc,
    },
    #[cfg(feature = "beep")]
    AppletEntry {
        name: "beep",
        run: run_beep,
    },
    #[cfg(feature = "blkdiscard")]
    AppletEntry {
        name: "blkdiscard",
        run: run_blkdiscard,
    },
    #[cfg(feature = "blkid")]
    AppletEntry {
        name: "blkid",
        run: run_blkid,
    },
    #[cfg(feature = "blockdev")]
    AppletEntry {
        name: "blockdev",
        run: run_blockdev,
    },
    #[cfg(feature = "bootchartd")]
    AppletEntry {
        name: "bootchartd",
        run: run_bootchartd,
    },
    #[cfg(feature = "brctl")]
    AppletEntry {
        name: "brctl",
        run: run_brctl,
    },
    #[cfg(feature = "bunzip2")]
    AppletEntry {
        name: "bunzip2",
        run: run_bunzip2,
    },
    #[cfg(feature = "bzcat")]
    AppletEntry {
        name: "bzcat",
        run: run_bzcat,
    },
    #[cfg(feature = "bzip2")]
    AppletEntry {
        name: "bzip2",
        run: run_bzip2,
    },
    #[cfg(feature = "cal")]
    AppletEntry {
        name: "cal",
        run: run_cal,
    },
    #[cfg(feature = "cat")]
    AppletEntry {
        name: "cat",
        run: run_cat,
    },
    #[cfg(feature = "chat")]
    AppletEntry {
        name: "chat",
        run: run_chat,
    },
    #[cfg(feature = "chattr")]
    AppletEntry {
        name: "chattr",
        run: run_chattr,
    },
    #[cfg(feature = "chgrp")]
    AppletEntry {
        name: "chgrp",
        run: run_chgrp,
    },
    #[cfg(feature = "chmod")]
    AppletEntry {
        name: "chmod",
        run: run_chmod,
    },
    #[cfg(feature = "chown")]
    AppletEntry {
        name: "chown",
        run: run_chown,
    },
    #[cfg(feature = "chpasswd")]
    AppletEntry {
        name: "chpasswd",
        run: run_chpasswd,
    },
    #[cfg(feature = "chpst")]
    AppletEntry {
        name: "chpst",
        run: run_chpst,
    },
    #[cfg(feature = "chroot")]
    AppletEntry {
        name: "chroot",
        run: run_chroot,
    },
    #[cfg(feature = "chrt")]
    AppletEntry {
        name: "chrt",
        run: run_chrt,
    },
    #[cfg(feature = "chvt")]
    AppletEntry {
        name: "chvt",
        run: run_chvt,
    },
    #[cfg(feature = "cksum")]
    AppletEntry {
        name: "cksum",
        run: run_cksum,
    },
    #[cfg(feature = "clear")]
    AppletEntry {
        name: "clear",
        run: run_clear,
    },
    #[cfg(feature = "cmp")]
    AppletEntry {
        name: "cmp",
        run: run_cmp,
    },
    #[cfg(feature = "comm")]
    AppletEntry {
        name: "comm",
        run: run_comm,
    },
    #[cfg(feature = "conspy")]
    AppletEntry {
        name: "conspy",
        run: run_conspy,
    },
    #[cfg(feature = "cp")]
    AppletEntry {
        name: "cp",
        run: run_cp,
    },
    #[cfg(feature = "cpio")]
    AppletEntry {
        name: "cpio",
        run: run_cpio,
    },
    #[cfg(feature = "crc32")]
    AppletEntry {
        name: "crc32",
        run: run_crc32,
    },
    #[cfg(feature = "crond")]
    AppletEntry {
        name: "crond",
        run: run_crond,
    },
    #[cfg(feature = "crontab")]
    AppletEntry {
        name: "crontab",
        run: run_crontab,
    },
    #[cfg(feature = "cryptpw")]
    AppletEntry {
        name: "cryptpw",
        run: run_cryptpw,
    },
    #[cfg(feature = "cttyhack")]
    AppletEntry {
        name: "cttyhack",
        run: run_cttyhack,
    },
    #[cfg(feature = "cut")]
    AppletEntry {
        name: "cut",
        run: run_cut,
    },
    #[cfg(feature = "date")]
    AppletEntry {
        name: "date",
        run: run_date,
    },
    #[cfg(feature = "dc")]
    AppletEntry {
        name: "dc",
        run: run_dc,
    },
    #[cfg(feature = "dd")]
    AppletEntry {
        name: "dd",
        run: run_dd,
    },
    #[cfg(feature = "deallocvt")]
    AppletEntry {
        name: "deallocvt",
        run: run_deallocvt,
    },
    #[cfg(feature = "delgroup")]
    AppletEntry {
        name: "delgroup",
        run: run_delgroup,
    },
    #[cfg(feature = "deluser")]
    AppletEntry {
        name: "deluser",
        run: run_deluser,
    },
    #[cfg(feature = "depmod")]
    AppletEntry {
        name: "depmod",
        run: run_depmod,
    },
    #[cfg(feature = "devmem")]
    AppletEntry {
        name: "devmem",
        run: run_devmem,
    },
    #[cfg(feature = "df")]
    AppletEntry {
        name: "df",
        run: run_df,
    },
    #[cfg(feature = "dhcprelay")]
    AppletEntry {
        name: "dhcprelay",
        run: run_dhcprelay,
    },
    #[cfg(feature = "diff")]
    AppletEntry {
        name: "diff",
        run: run_diff,
    },
    #[cfg(feature = "dirname")]
    AppletEntry {
        name: "dirname",
        run: run_dirname,
    },
    #[cfg(feature = "dmesg")]
    AppletEntry {
        name: "dmesg",
        run: run_dmesg,
    },
    #[cfg(feature = "dnsd")]
    AppletEntry {
        name: "dnsd",
        run: run_dnsd,
    },
    #[cfg(feature = "dnsdomainname")]
    AppletEntry {
        name: "dnsdomainname",
        run: run_dnsdomainname,
    },
    #[cfg(feature = "dos2unix")]
    AppletEntry {
        name: "dos2unix",
        run: run_dos2unix,
    },
    #[cfg(feature = "dpkg")]
    AppletEntry {
        name: "dpkg",
        run: run_dpkg,
    },
    #[cfg(feature = "dpkg_deb")]
    AppletEntry {
        name: "dpkg-deb",
        run: run_dpkg_deb,
    },
    #[cfg(feature = "du")]
    AppletEntry {
        name: "du",
        run: run_du,
    },
    #[cfg(feature = "dumpkmap")]
    AppletEntry {
        name: "dumpkmap",
        run: run_dumpkmap,
    },
    #[cfg(feature = "dumpleases")]
    AppletEntry {
        name: "dumpleases",
        run: run_dumpleases,
    },
    #[cfg(feature = "echo")]
    AppletEntry {
        name: "echo",
        run: run_echo,
    },
    #[cfg(feature = "ed")]
    AppletEntry {
        name: "ed",
        run: run_ed,
    },
    #[cfg(feature = "egrep")]
    AppletEntry {
        name: "egrep",
        run: run_egrep,
    },
    #[cfg(feature = "eject")]
    AppletEntry {
        name: "eject",
        run: run_eject,
    },
    #[cfg(feature = "env")]
    AppletEntry {
        name: "env",
        run: run_env,
    },
    #[cfg(feature = "envdir")]
    AppletEntry {
        name: "envdir",
        run: run_envdir,
    },
    #[cfg(feature = "envuidgid")]
    AppletEntry {
        name: "envuidgid",
        run: run_envuidgid,
    },
    #[cfg(feature = "ether_wake")]
    AppletEntry {
        name: "ether-wake",
        run: run_ether_wake,
    },
    #[cfg(feature = "expand")]
    AppletEntry {
        name: "expand",
        run: run_expand,
    },
    #[cfg(feature = "expr")]
    AppletEntry {
        name: "expr",
        run: run_expr,
    },
    #[cfg(feature = "factor")]
    AppletEntry {
        name: "factor",
        run: run_factor,
    },
    #[cfg(feature = "fakeidentd")]
    AppletEntry {
        name: "fakeidentd",
        run: run_fakeidentd,
    },
    #[cfg(feature = "fallocate")]
    AppletEntry {
        name: "fallocate",
        run: run_fallocate,
    },
    #[cfg(feature = "false")]
    AppletEntry {
        name: "false",
        run: run_false,
    },
    #[cfg(feature = "fatattr")]
    AppletEntry {
        name: "fatattr",
        run: run_fatattr,
    },
    #[cfg(feature = "fatlabel")]
    AppletEntry {
        name: "fatlabel",
        run: run_fatlabel,
    },
    #[cfg(feature = "fbset")]
    AppletEntry {
        name: "fbset",
        run: run_fbset,
    },
    #[cfg(feature = "fbsplash")]
    AppletEntry {
        name: "fbsplash",
        run: run_fbsplash,
    },
    #[cfg(feature = "fdflush")]
    AppletEntry {
        name: "fdflush",
        run: run_fdflush,
    },
    #[cfg(feature = "fdformat")]
    AppletEntry {
        name: "fdformat",
        run: run_fdformat,
    },
    #[cfg(feature = "fdisk")]
    AppletEntry {
        name: "fdisk",
        run: run_fdisk,
    },
    #[cfg(feature = "fgconsole")]
    AppletEntry {
        name: "fgconsole",
        run: run_fgconsole,
    },
    #[cfg(feature = "fgrep")]
    AppletEntry {
        name: "fgrep",
        run: run_fgrep,
    },
    #[cfg(feature = "find")]
    AppletEntry {
        name: "find",
        run: run_find,
    },
    #[cfg(feature = "findfs")]
    AppletEntry {
        name: "findfs",
        run: run_findfs,
    },
    #[cfg(feature = "flock")]
    AppletEntry {
        name: "flock",
        run: run_flock,
    },
    #[cfg(feature = "fold")]
    AppletEntry {
        name: "fold",
        run: run_fold,
    },
    #[cfg(feature = "free")]
    AppletEntry {
        name: "free",
        run: run_free,
    },
    #[cfg(feature = "freeramdisk")]
    AppletEntry {
        name: "freeramdisk",
        run: run_freeramdisk,
    },
    #[cfg(feature = "fsck")]
    AppletEntry {
        name: "fsck",
        run: run_fsck,
    },
    #[cfg(feature = "fsck_minix")]
    AppletEntry {
        name: "fsck.minix",
        run: run_fsck_minix,
    },
    #[cfg(feature = "fsfreeze")]
    AppletEntry {
        name: "fsfreeze",
        run: run_fsfreeze,
    },
    #[cfg(feature = "fstrim")]
    AppletEntry {
        name: "fstrim",
        run: run_fstrim,
    },
    #[cfg(feature = "fsync")]
    AppletEntry {
        name: "fsync",
        run: run_fsync,
    },
    #[cfg(feature = "ftpd")]
    AppletEntry {
        name: "ftpd",
        run: run_ftpd,
    },
    #[cfg(feature = "ftpget")]
    AppletEntry {
        name: "ftpget",
        run: run_ftpget,
    },
    #[cfg(feature = "ftpput")]
    AppletEntry {
        name: "ftpput",
        run: run_ftpput,
    },
    #[cfg(feature = "fuser")]
    AppletEntry {
        name: "fuser",
        run: run_fuser,
    },
    #[cfg(feature = "getfattr")]
    AppletEntry {
        name: "getfattr",
        run: run_getfattr,
    },
    #[cfg(feature = "getopt")]
    AppletEntry {
        name: "getopt",
        run: run_getopt,
    },
    #[cfg(feature = "getty")]
    AppletEntry {
        name: "getty",
        run: run_getty,
    },
    #[cfg(feature = "grep")]
    AppletEntry {
        name: "grep",
        run: run_grep,
    },
    #[cfg(feature = "groups")]
    AppletEntry {
        name: "groups",
        run: run_groups,
    },
    #[cfg(feature = "gunzip")]
    AppletEntry {
        name: "gunzip",
        run: run_gunzip,
    },
    #[cfg(feature = "gzip")]
    AppletEntry {
        name: "gzip",
        run: run_gzip,
    },
    #[cfg(feature = "halt")]
    AppletEntry {
        name: "halt",
        run: run_halt,
    },
    #[cfg(feature = "hd")]
    AppletEntry {
        name: "hd",
        run: run_hd,
    },
    #[cfg(feature = "hdparm")]
    AppletEntry {
        name: "hdparm",
        run: run_hdparm,
    },
    #[cfg(feature = "head")]
    AppletEntry {
        name: "head",
        run: run_head,
    },
    #[cfg(feature = "hexdump")]
    AppletEntry {
        name: "hexdump",
        run: run_hexdump,
    },
    #[cfg(feature = "hexedit")]
    AppletEntry {
        name: "hexedit",
        run: run_hexedit,
    },
    #[cfg(feature = "hostid")]
    AppletEntry {
        name: "hostid",
        run: run_hostid,
    },
    #[cfg(feature = "hostname")]
    AppletEntry {
        name: "hostname",
        run: run_hostname,
    },
    #[cfg(feature = "httpd")]
    AppletEntry {
        name: "httpd",
        run: run_httpd,
    },
    #[cfg(feature = "hush")]
    AppletEntry {
        name: "hush",
        run: run_hush,
    },
    #[cfg(feature = "hwclock")]
    AppletEntry {
        name: "hwclock",
        run: run_hwclock,
    },
    #[cfg(feature = "i2cdetect")]
    AppletEntry {
        name: "i2cdetect",
        run: run_i2cdetect,
    },
    #[cfg(feature = "i2cdump")]
    AppletEntry {
        name: "i2cdump",
        run: run_i2cdump,
    },
    #[cfg(feature = "i2cget")]
    AppletEntry {
        name: "i2cget",
        run: run_i2cget,
    },
    #[cfg(feature = "i2cset")]
    AppletEntry {
        name: "i2cset",
        run: run_i2cset,
    },
    #[cfg(feature = "i2ctransfer")]
    AppletEntry {
        name: "i2ctransfer",
        run: run_i2ctransfer,
    },
    #[cfg(feature = "id")]
    AppletEntry {
        name: "id",
        run: run_id,
    },
    #[cfg(feature = "ifconfig")]
    AppletEntry {
        name: "ifconfig",
        run: run_ifconfig,
    },
    #[cfg(feature = "ifdown")]
    AppletEntry {
        name: "ifdown",
        run: run_ifdown,
    },
    #[cfg(feature = "ifenslave")]
    AppletEntry {
        name: "ifenslave",
        run: run_ifenslave,
    },
    #[cfg(feature = "ifplugd")]
    AppletEntry {
        name: "ifplugd",
        run: run_ifplugd,
    },
    #[cfg(feature = "ifup")]
    AppletEntry {
        name: "ifup",
        run: run_ifup,
    },
    #[cfg(feature = "inetd")]
    AppletEntry {
        name: "inetd",
        run: run_inetd,
    },
    #[cfg(feature = "init")]
    AppletEntry {
        name: "init",
        run: run_init,
    },
    #[cfg(feature = "insmod")]
    AppletEntry {
        name: "insmod",
        run: run_insmod,
    },
    #[cfg(feature = "install")]
    AppletEntry {
        name: "install",
        run: run_install,
    },
    #[cfg(feature = "ionice")]
    AppletEntry {
        name: "ionice",
        run: run_ionice,
    },
    #[cfg(feature = "iostat")]
    AppletEntry {
        name: "iostat",
        run: run_iostat,
    },
    #[cfg(feature = "ip")]
    AppletEntry {
        name: "ip",
        run: run_ip,
    },
    #[cfg(feature = "ipaddr")]
    AppletEntry {
        name: "ipaddr",
        run: run_ipaddr,
    },
    #[cfg(feature = "ipcalc")]
    AppletEntry {
        name: "ipcalc",
        run: run_ipcalc,
    },
    #[cfg(feature = "ipcrm")]
    AppletEntry {
        name: "ipcrm",
        run: run_ipcrm,
    },
    #[cfg(feature = "ipcs")]
    AppletEntry {
        name: "ipcs",
        run: run_ipcs,
    },
    #[cfg(feature = "iplink")]
    AppletEntry {
        name: "iplink",
        run: run_iplink,
    },
    #[cfg(feature = "ipneigh")]
    AppletEntry {
        name: "ipneigh",
        run: run_ipneigh,
    },
    #[cfg(feature = "iproute")]
    AppletEntry {
        name: "iproute",
        run: run_iproute,
    },
    #[cfg(feature = "iprule")]
    AppletEntry {
        name: "iprule",
        run: run_iprule,
    },
    #[cfg(feature = "iptunnel")]
    AppletEntry {
        name: "iptunnel",
        run: run_iptunnel,
    },
    #[cfg(feature = "join")]
    AppletEntry {
        name: "join",
        run: run_join,
    },
    #[cfg(feature = "kbd_mode")]
    AppletEntry {
        name: "kbd_mode",
        run: run_kbd_mode,
    },
    #[cfg(feature = "kill")]
    AppletEntry {
        name: "kill",
        run: run_kill,
    },
    #[cfg(feature = "killall")]
    AppletEntry {
        name: "killall",
        run: run_killall,
    },
    #[cfg(feature = "killall5")]
    AppletEntry {
        name: "killall5",
        run: run_killall5,
    },
    #[cfg(feature = "klogd")]
    AppletEntry {
        name: "klogd",
        run: run_klogd,
    },
    #[cfg(feature = "last")]
    AppletEntry {
        name: "last",
        run: run_last,
    },
    #[cfg(feature = "less")]
    AppletEntry {
        name: "less",
        run: run_less,
    },
    #[cfg(feature = "link")]
    AppletEntry {
        name: "link",
        run: run_link,
    },
    #[cfg(feature = "linux32")]
    AppletEntry {
        name: "linux32",
        run: run_linux32,
    },
    #[cfg(feature = "linux64")]
    AppletEntry {
        name: "linux64",
        run: run_linux64,
    },
    #[cfg(feature = "linuxrc")]
    AppletEntry {
        name: "linuxrc",
        run: run_linuxrc,
    },
    #[cfg(feature = "ln")]
    AppletEntry {
        name: "ln",
        run: run_ln,
    },
    #[cfg(feature = "loadfont")]
    AppletEntry {
        name: "loadfont",
        run: run_loadfont,
    },
    #[cfg(feature = "loadkmap")]
    AppletEntry {
        name: "loadkmap",
        run: run_loadkmap,
    },
    #[cfg(feature = "logger")]
    AppletEntry {
        name: "logger",
        run: run_logger,
    },
    #[cfg(feature = "login")]
    AppletEntry {
        name: "login",
        run: run_login,
    },
    #[cfg(feature = "logname")]
    AppletEntry {
        name: "logname",
        run: run_logname,
    },
    #[cfg(feature = "logread")]
    AppletEntry {
        name: "logread",
        run: run_logread,
    },
    #[cfg(feature = "losetup")]
    AppletEntry {
        name: "losetup",
        run: run_losetup,
    },
    #[cfg(feature = "lpd")]
    AppletEntry {
        name: "lpd",
        run: run_lpd,
    },
    #[cfg(feature = "lpq")]
    AppletEntry {
        name: "lpq",
        run: run_lpq,
    },
    #[cfg(feature = "lpr")]
    AppletEntry {
        name: "lpr",
        run: run_lpr,
    },
    #[cfg(feature = "ls")]
    AppletEntry {
        name: "ls",
        run: run_ls,
    },
    #[cfg(feature = "lsattr")]
    AppletEntry {
        name: "lsattr",
        run: run_lsattr,
    },
    #[cfg(feature = "lsblk")]
    AppletEntry {
        name: "lsblk",
        run: run_lsblk,
    },
    #[cfg(feature = "lsmod")]
    AppletEntry {
        name: "lsmod",
        run: run_lsmod,
    },
    #[cfg(feature = "lsof")]
    AppletEntry {
        name: "lsof",
        run: run_lsof,
    },
    #[cfg(feature = "lspci")]
    AppletEntry {
        name: "lspci",
        run: run_lspci,
    },
    #[cfg(feature = "lsscsi")]
    AppletEntry {
        name: "lsscsi",
        run: run_lsscsi,
    },
    #[cfg(feature = "lsusb")]
    AppletEntry {
        name: "lsusb",
        run: run_lsusb,
    },
    #[cfg(feature = "lzcat")]
    AppletEntry {
        name: "lzcat",
        run: run_lzcat,
    },
    #[cfg(feature = "lzma")]
    AppletEntry {
        name: "lzma",
        run: run_lzma,
    },
    #[cfg(feature = "lzop")]
    AppletEntry {
        name: "lzop",
        run: run_lzop,
    },
    #[cfg(feature = "makedevs")]
    AppletEntry {
        name: "makedevs",
        run: run_makedevs,
    },
    #[cfg(feature = "makemime")]
    AppletEntry {
        name: "makemime",
        run: run_makemime,
    },
    #[cfg(feature = "man")]
    AppletEntry {
        name: "man",
        run: run_man,
    },
    #[cfg(feature = "md5sum")]
    AppletEntry {
        name: "md5sum",
        run: run_md5sum,
    },
    #[cfg(feature = "mdev")]
    AppletEntry {
        name: "mdev",
        run: run_mdev,
    },
    #[cfg(feature = "mesg")]
    AppletEntry {
        name: "mesg",
        run: run_mesg,
    },
    #[cfg(feature = "microcom")]
    AppletEntry {
        name: "microcom",
        run: run_microcom,
    },
    #[cfg(feature = "mim")]
    AppletEntry {
        name: "mim",
        run: run_mim,
    },
    #[cfg(feature = "mkdir")]
    AppletEntry {
        name: "mkdir",
        run: run_mkdir,
    },
    #[cfg(feature = "mkdosfs")]
    AppletEntry {
        name: "mkdosfs",
        run: run_mkdosfs,
    },
    #[cfg(feature = "mke2fs")]
    AppletEntry {
        name: "mke2fs",
        run: run_mke2fs,
    },
    #[cfg(feature = "mkfifo")]
    AppletEntry {
        name: "mkfifo",
        run: run_mkfifo,
    },
    #[cfg(feature = "mkfs_ext2")]
    AppletEntry {
        name: "mkfs.ext2",
        run: run_mkfs_ext2,
    },
    #[cfg(feature = "mkfs_minix")]
    AppletEntry {
        name: "mkfs.minix",
        run: run_mkfs_minix,
    },
    #[cfg(feature = "mkfs_vfat")]
    AppletEntry {
        name: "mkfs.vfat",
        run: run_mkfs_vfat,
    },
    #[cfg(feature = "mknod")]
    AppletEntry {
        name: "mknod",
        run: run_mknod,
    },
    #[cfg(feature = "mkpasswd")]
    AppletEntry {
        name: "mkpasswd",
        run: run_mkpasswd,
    },
    #[cfg(feature = "mkswap")]
    AppletEntry {
        name: "mkswap",
        run: run_mkswap,
    },
    #[cfg(feature = "mktemp")]
    AppletEntry {
        name: "mktemp",
        run: run_mktemp,
    },
    #[cfg(feature = "modinfo")]
    AppletEntry {
        name: "modinfo",
        run: run_modinfo,
    },
    #[cfg(feature = "modprobe")]
    AppletEntry {
        name: "modprobe",
        run: run_modprobe,
    },
    #[cfg(feature = "more")]
    AppletEntry {
        name: "more",
        run: run_more,
    },
    #[cfg(feature = "mount")]
    AppletEntry {
        name: "mount",
        run: run_mount,
    },
    #[cfg(feature = "mountpoint")]
    AppletEntry {
        name: "mountpoint",
        run: run_mountpoint,
    },
    #[cfg(feature = "mpstat")]
    AppletEntry {
        name: "mpstat",
        run: run_mpstat,
    },
    #[cfg(feature = "mt")]
    AppletEntry {
        name: "mt",
        run: run_mt,
    },
    #[cfg(feature = "mv")]
    AppletEntry {
        name: "mv",
        run: run_mv,
    },
    #[cfg(feature = "nameif")]
    AppletEntry {
        name: "nameif",
        run: run_nameif,
    },
    #[cfg(feature = "nanddump")]
    AppletEntry {
        name: "nanddump",
        run: run_nanddump,
    },
    #[cfg(feature = "nandwrite")]
    AppletEntry {
        name: "nandwrite",
        run: run_nandwrite,
    },
    #[cfg(feature = "nbd_client")]
    AppletEntry {
        name: "nbd-client",
        run: run_nbd_client,
    },
    #[cfg(feature = "nc")]
    AppletEntry {
        name: "nc",
        run: run_nc,
    },
    #[cfg(feature = "netstat")]
    AppletEntry {
        name: "netstat",
        run: run_netstat,
    },
    #[cfg(feature = "nice")]
    AppletEntry {
        name: "nice",
        run: run_nice,
    },
    #[cfg(feature = "nl")]
    AppletEntry {
        name: "nl",
        run: run_nl,
    },
    #[cfg(feature = "nmeter")]
    AppletEntry {
        name: "nmeter",
        run: run_nmeter,
    },
    #[cfg(feature = "nohup")]
    AppletEntry {
        name: "nohup",
        run: run_nohup,
    },
    #[cfg(feature = "nologin")]
    AppletEntry {
        name: "nologin",
        run: run_nologin,
    },
    #[cfg(feature = "nproc")]
    AppletEntry {
        name: "nproc",
        run: run_nproc,
    },
    #[cfg(feature = "nsenter")]
    AppletEntry {
        name: "nsenter",
        run: run_nsenter,
    },
    #[cfg(feature = "nslookup")]
    AppletEntry {
        name: "nslookup",
        run: run_nslookup,
    },
    #[cfg(feature = "ntpd")]
    AppletEntry {
        name: "ntpd",
        run: run_ntpd,
    },
    #[cfg(feature = "od")]
    AppletEntry {
        name: "od",
        run: run_od,
    },
    #[cfg(feature = "openvt")]
    AppletEntry {
        name: "openvt",
        run: run_openvt,
    },
    #[cfg(feature = "partprobe")]
    AppletEntry {
        name: "partprobe",
        run: run_partprobe,
    },
    #[cfg(feature = "passwd")]
    AppletEntry {
        name: "passwd",
        run: run_passwd,
    },
    #[cfg(feature = "paste")]
    AppletEntry {
        name: "paste",
        run: run_paste,
    },
    #[cfg(feature = "patch")]
    AppletEntry {
        name: "patch",
        run: run_patch,
    },
    #[cfg(feature = "pgrep")]
    AppletEntry {
        name: "pgrep",
        run: run_pgrep,
    },
    #[cfg(feature = "pidof")]
    AppletEntry {
        name: "pidof",
        run: run_pidof,
    },
    #[cfg(feature = "ping")]
    AppletEntry {
        name: "ping",
        run: run_ping,
    },
    #[cfg(feature = "ping6")]
    AppletEntry {
        name: "ping6",
        run: run_ping6,
    },
    #[cfg(feature = "pipe_progress")]
    AppletEntry {
        name: "pipe_progress",
        run: run_pipe_progress,
    },
    #[cfg(feature = "pivot_root")]
    AppletEntry {
        name: "pivot_root",
        run: run_pivot_root,
    },
    #[cfg(feature = "pkill")]
    AppletEntry {
        name: "pkill",
        run: run_pkill,
    },
    #[cfg(feature = "pmap")]
    AppletEntry {
        name: "pmap",
        run: run_pmap,
    },
    #[cfg(feature = "popmaildir")]
    AppletEntry {
        name: "popmaildir",
        run: run_popmaildir,
    },
    #[cfg(feature = "poweroff")]
    AppletEntry {
        name: "poweroff",
        run: run_poweroff,
    },
    #[cfg(feature = "powertop")]
    AppletEntry {
        name: "powertop",
        run: run_powertop,
    },
    #[cfg(feature = "printenv")]
    AppletEntry {
        name: "printenv",
        run: run_printenv,
    },
    #[cfg(feature = "printf")]
    AppletEntry {
        name: "printf",
        run: run_printf,
    },
    #[cfg(feature = "ps")]
    AppletEntry {
        name: "ps",
        run: run_ps,
    },
    #[cfg(feature = "pscan")]
    AppletEntry {
        name: "pscan",
        run: run_pscan,
    },
    #[cfg(feature = "pstree")]
    AppletEntry {
        name: "pstree",
        run: run_pstree,
    },
    #[cfg(feature = "pwd")]
    AppletEntry {
        name: "pwd",
        run: run_pwd,
    },
    #[cfg(feature = "pwdx")]
    AppletEntry {
        name: "pwdx",
        run: run_pwdx,
    },
    #[cfg(feature = "raidautorun")]
    AppletEntry {
        name: "raidautorun",
        run: run_raidautorun,
    },
    #[cfg(feature = "rdate")]
    AppletEntry {
        name: "rdate",
        run: run_rdate,
    },
    #[cfg(feature = "rdev")]
    AppletEntry {
        name: "rdev",
        run: run_rdev,
    },
    #[cfg(feature = "readahead")]
    AppletEntry {
        name: "readahead",
        run: run_readahead,
    },
    #[cfg(feature = "readlink")]
    AppletEntry {
        name: "readlink",
        run: run_readlink,
    },
    #[cfg(feature = "readprofile")]
    AppletEntry {
        name: "readprofile",
        run: run_readprofile,
    },
    #[cfg(feature = "realpath")]
    AppletEntry {
        name: "realpath",
        run: run_realpath,
    },
    #[cfg(feature = "reboot")]
    AppletEntry {
        name: "reboot",
        run: run_reboot,
    },
    #[cfg(feature = "reformime")]
    AppletEntry {
        name: "reformime",
        run: run_reformime,
    },
    #[cfg(feature = "remove_shell")]
    AppletEntry {
        name: "remove-shell",
        run: run_remove_shell,
    },
    #[cfg(feature = "renice")]
    AppletEntry {
        name: "renice",
        run: run_renice,
    },
    #[cfg(feature = "reset")]
    AppletEntry {
        name: "reset",
        run: run_reset,
    },
    #[cfg(feature = "resize")]
    AppletEntry {
        name: "resize",
        run: run_resize,
    },
    #[cfg(feature = "resume")]
    AppletEntry {
        name: "resume",
        run: run_resume,
    },
    #[cfg(feature = "rev")]
    AppletEntry {
        name: "rev",
        run: run_rev,
    },
    #[cfg(feature = "rm")]
    AppletEntry {
        name: "rm",
        run: run_rm,
    },
    #[cfg(feature = "rmdir")]
    AppletEntry {
        name: "rmdir",
        run: run_rmdir,
    },
    #[cfg(feature = "rmmod")]
    AppletEntry {
        name: "rmmod",
        run: run_rmmod,
    },
    #[cfg(feature = "route")]
    AppletEntry {
        name: "route",
        run: run_route,
    },
    #[cfg(feature = "rpm")]
    AppletEntry {
        name: "rpm",
        run: run_rpm,
    },
    #[cfg(feature = "rpm2cpio")]
    AppletEntry {
        name: "rpm2cpio",
        run: run_rpm2cpio,
    },
    #[cfg(feature = "rtcwake")]
    AppletEntry {
        name: "rtcwake",
        run: run_rtcwake,
    },
    #[cfg(feature = "run_init")]
    AppletEntry {
        name: "run-init",
        run: run_run_init,
    },
    #[cfg(feature = "run_parts")]
    AppletEntry {
        name: "run-parts",
        run: run_run_parts,
    },
    #[cfg(feature = "runlevel")]
    AppletEntry {
        name: "runlevel",
        run: run_runlevel,
    },
    #[cfg(feature = "runsv")]
    AppletEntry {
        name: "runsv",
        run: run_runsv,
    },
    #[cfg(feature = "runsvdir")]
    AppletEntry {
        name: "runsvdir",
        run: run_runsvdir,
    },
    #[cfg(feature = "rx")]
    AppletEntry {
        name: "rx",
        run: run_rx,
    },
    #[cfg(feature = "script")]
    AppletEntry {
        name: "script",
        run: run_script,
    },
    #[cfg(feature = "scriptreplay")]
    AppletEntry {
        name: "scriptreplay",
        run: run_scriptreplay,
    },
    #[cfg(feature = "sed")]
    AppletEntry {
        name: "sed",
        run: run_sed,
    },
    #[cfg(feature = "seedrng")]
    AppletEntry {
        name: "seedrng",
        run: run_seedrng,
    },
    #[cfg(feature = "sendmail")]
    AppletEntry {
        name: "sendmail",
        run: run_sendmail,
    },
    #[cfg(feature = "seq")]
    AppletEntry {
        name: "seq",
        run: run_seq,
    },
    #[cfg(feature = "setarch")]
    AppletEntry {
        name: "setarch",
        run: run_setarch,
    },
    #[cfg(feature = "setconsole")]
    AppletEntry {
        name: "setconsole",
        run: run_setconsole,
    },
    #[cfg(feature = "setfattr")]
    AppletEntry {
        name: "setfattr",
        run: run_setfattr,
    },
    #[cfg(feature = "setfont")]
    AppletEntry {
        name: "setfont",
        run: run_setfont,
    },
    #[cfg(feature = "setkeycodes")]
    AppletEntry {
        name: "setkeycodes",
        run: run_setkeycodes,
    },
    #[cfg(feature = "setlogcons")]
    AppletEntry {
        name: "setlogcons",
        run: run_setlogcons,
    },
    #[cfg(feature = "setpriv")]
    AppletEntry {
        name: "setpriv",
        run: run_setpriv,
    },
    #[cfg(feature = "setserial")]
    AppletEntry {
        name: "setserial",
        run: run_setserial,
    },
    #[cfg(feature = "setsid")]
    AppletEntry {
        name: "setsid",
        run: run_setsid,
    },
    #[cfg(feature = "setuidgid")]
    AppletEntry {
        name: "setuidgid",
        run: run_setuidgid,
    },
    #[cfg(feature = "sh")]
    AppletEntry {
        name: "sh",
        run: run_sh,
    },
    #[cfg(feature = "sha1sum")]
    AppletEntry {
        name: "sha1sum",
        run: run_sha1sum,
    },
    #[cfg(feature = "sha256sum")]
    AppletEntry {
        name: "sha256sum",
        run: run_sha256sum,
    },
    #[cfg(feature = "sha384sum")]
    AppletEntry {
        name: "sha384sum",
        run: run_sha384sum,
    },
    #[cfg(feature = "sha3sum")]
    AppletEntry {
        name: "sha3sum",
        run: run_sha3sum,
    },
    #[cfg(feature = "sha512sum")]
    AppletEntry {
        name: "sha512sum",
        run: run_sha512sum,
    },
    #[cfg(feature = "showkey")]
    AppletEntry {
        name: "showkey",
        run: run_showkey,
    },
    #[cfg(feature = "shred")]
    AppletEntry {
        name: "shred",
        run: run_shred,
    },
    #[cfg(feature = "shuf")]
    AppletEntry {
        name: "shuf",
        run: run_shuf,
    },
    #[cfg(feature = "slattach")]
    AppletEntry {
        name: "slattach",
        run: run_slattach,
    },
    #[cfg(feature = "sleep")]
    AppletEntry {
        name: "sleep",
        run: run_sleep,
    },
    #[cfg(feature = "smemcap")]
    AppletEntry {
        name: "smemcap",
        run: run_smemcap,
    },
    #[cfg(feature = "softlimit")]
    AppletEntry {
        name: "softlimit",
        run: run_softlimit,
    },
    #[cfg(feature = "sort")]
    AppletEntry {
        name: "sort",
        run: run_sort,
    },
    #[cfg(feature = "split")]
    AppletEntry {
        name: "split",
        run: run_split,
    },
    #[cfg(feature = "ssl_client")]
    AppletEntry {
        name: "ssl_client",
        run: run_ssl_client,
    },
    #[cfg(feature = "ssl_server")]
    AppletEntry {
        name: "ssl_server",
        run: run_ssl_server,
    },
    #[cfg(feature = "start_stop_daemon")]
    AppletEntry {
        name: "start-stop-daemon",
        run: run_start_stop_daemon,
    },
    #[cfg(feature = "stat")]
    AppletEntry {
        name: "stat",
        run: run_stat,
    },
    #[cfg(feature = "strings")]
    AppletEntry {
        name: "strings",
        run: run_strings,
    },
    #[cfg(feature = "stty")]
    AppletEntry {
        name: "stty",
        run: run_stty,
    },
    #[cfg(feature = "su")]
    AppletEntry {
        name: "su",
        run: run_su,
    },
    #[cfg(feature = "sulogin")]
    AppletEntry {
        name: "sulogin",
        run: run_sulogin,
    },
    #[cfg(feature = "sum")]
    AppletEntry {
        name: "sum",
        run: run_sum,
    },
    #[cfg(feature = "sv")]
    AppletEntry {
        name: "sv",
        run: run_sv,
    },
    #[cfg(feature = "svc")]
    AppletEntry {
        name: "svc",
        run: run_svc,
    },
    #[cfg(feature = "svlogd")]
    AppletEntry {
        name: "svlogd",
        run: run_svlogd,
    },
    #[cfg(feature = "svok")]
    AppletEntry {
        name: "svok",
        run: run_svok,
    },
    #[cfg(feature = "swaplabel")]
    AppletEntry {
        name: "swaplabel",
        run: run_swaplabel,
    },
    #[cfg(feature = "swapoff")]
    AppletEntry {
        name: "swapoff",
        run: run_swapoff,
    },
    #[cfg(feature = "swapon")]
    AppletEntry {
        name: "swapon",
        run: run_swapon,
    },
    #[cfg(feature = "switch_root")]
    AppletEntry {
        name: "switch_root",
        run: run_switch_root,
    },
    #[cfg(feature = "sync")]
    AppletEntry {
        name: "sync",
        run: run_sync,
    },
    #[cfg(feature = "sysctl")]
    AppletEntry {
        name: "sysctl",
        run: run_sysctl,
    },
    #[cfg(feature = "syslogd")]
    AppletEntry {
        name: "syslogd",
        run: run_syslogd,
    },
    #[cfg(feature = "tac")]
    AppletEntry {
        name: "tac",
        run: run_tac,
    },
    #[cfg(feature = "tail")]
    AppletEntry {
        name: "tail",
        run: run_tail,
    },
    #[cfg(feature = "tar")]
    AppletEntry {
        name: "tar",
        run: run_tar,
    },
    #[cfg(feature = "taskset")]
    AppletEntry {
        name: "taskset",
        run: run_taskset,
    },
    #[cfg(feature = "tcpsvd")]
    AppletEntry {
        name: "tcpsvd",
        run: run_tcpsvd,
    },
    #[cfg(feature = "tee")]
    AppletEntry {
        name: "tee",
        run: run_tee,
    },
    #[cfg(feature = "telnet")]
    AppletEntry {
        name: "telnet",
        run: run_telnet,
    },
    #[cfg(feature = "telnetd")]
    AppletEntry {
        name: "telnetd",
        run: run_telnetd,
    },
    #[cfg(feature = "test")]
    AppletEntry {
        name: "test",
        run: run_test,
    },
    #[cfg(feature = "tftp")]
    AppletEntry {
        name: "tftp",
        run: run_tftp,
    },
    #[cfg(feature = "tftpd")]
    AppletEntry {
        name: "tftpd",
        run: run_tftpd,
    },
    #[cfg(feature = "time")]
    AppletEntry {
        name: "time",
        run: run_time,
    },
    #[cfg(feature = "timeout")]
    AppletEntry {
        name: "timeout",
        run: run_timeout,
    },
    #[cfg(feature = "top")]
    AppletEntry {
        name: "top",
        run: run_top,
    },
    #[cfg(feature = "touch")]
    AppletEntry {
        name: "touch",
        run: run_touch,
    },
    #[cfg(feature = "tr")]
    AppletEntry {
        name: "tr",
        run: run_tr,
    },
    #[cfg(feature = "traceroute")]
    AppletEntry {
        name: "traceroute",
        run: run_traceroute,
    },
    #[cfg(feature = "traceroute6")]
    AppletEntry {
        name: "traceroute6",
        run: run_traceroute6,
    },
    #[cfg(feature = "tree")]
    AppletEntry {
        name: "tree",
        run: run_tree,
    },
    #[cfg(feature = "true")]
    AppletEntry {
        name: "true",
        run: run_true,
    },
    #[cfg(feature = "truncate")]
    AppletEntry {
        name: "truncate",
        run: run_truncate,
    },
    #[cfg(feature = "ts")]
    AppletEntry {
        name: "ts",
        run: run_ts,
    },
    #[cfg(feature = "tsort")]
    AppletEntry {
        name: "tsort",
        run: run_tsort,
    },
    #[cfg(feature = "tty")]
    AppletEntry {
        name: "tty",
        run: run_tty,
    },
    #[cfg(feature = "ttysize")]
    AppletEntry {
        name: "ttysize",
        run: run_ttysize,
    },
    #[cfg(feature = "tunctl")]
    AppletEntry {
        name: "tunctl",
        run: run_tunctl,
    },
    #[cfg(feature = "ubiattach")]
    AppletEntry {
        name: "ubiattach",
        run: run_ubiattach,
    },
    #[cfg(feature = "ubidetach")]
    AppletEntry {
        name: "ubidetach",
        run: run_ubidetach,
    },
    #[cfg(feature = "ubimkvol")]
    AppletEntry {
        name: "ubimkvol",
        run: run_ubimkvol,
    },
    #[cfg(feature = "ubirename")]
    AppletEntry {
        name: "ubirename",
        run: run_ubirename,
    },
    #[cfg(feature = "ubirmvol")]
    AppletEntry {
        name: "ubirmvol",
        run: run_ubirmvol,
    },
    #[cfg(feature = "ubirsvol")]
    AppletEntry {
        name: "ubirsvol",
        run: run_ubirsvol,
    },
    #[cfg(feature = "ubiupdatevol")]
    AppletEntry {
        name: "ubiupdatevol",
        run: run_ubiupdatevol,
    },
    #[cfg(feature = "udhcpc")]
    AppletEntry {
        name: "udhcpc",
        run: run_udhcpc,
    },
    #[cfg(feature = "udhcpc6")]
    AppletEntry {
        name: "udhcpc6",
        run: run_udhcpc6,
    },
    #[cfg(feature = "udhcpd")]
    AppletEntry {
        name: "udhcpd",
        run: run_udhcpd,
    },
    #[cfg(feature = "udpsvd")]
    AppletEntry {
        name: "udpsvd",
        run: run_udpsvd,
    },
    #[cfg(feature = "uevent")]
    AppletEntry {
        name: "uevent",
        run: run_uevent,
    },
    #[cfg(feature = "umount")]
    AppletEntry {
        name: "umount",
        run: run_umount,
    },
    #[cfg(feature = "uname")]
    AppletEntry {
        name: "uname",
        run: run_uname,
    },
    #[cfg(feature = "unexpand")]
    AppletEntry {
        name: "unexpand",
        run: run_unexpand,
    },
    #[cfg(feature = "uniq")]
    AppletEntry {
        name: "uniq",
        run: run_uniq,
    },
    #[cfg(feature = "unix2dos")]
    AppletEntry {
        name: "unix2dos",
        run: run_unix2dos,
    },
    #[cfg(feature = "unlink")]
    AppletEntry {
        name: "unlink",
        run: run_unlink,
    },
    #[cfg(feature = "unlzma")]
    AppletEntry {
        name: "unlzma",
        run: run_unlzma,
    },
    #[cfg(feature = "unshare")]
    AppletEntry {
        name: "unshare",
        run: run_unshare,
    },
    #[cfg(feature = "unxz")]
    AppletEntry {
        name: "unxz",
        run: run_unxz,
    },
    #[cfg(feature = "unzip")]
    AppletEntry {
        name: "unzip",
        run: run_unzip,
    },
    #[cfg(feature = "uptime")]
    AppletEntry {
        name: "uptime",
        run: run_uptime,
    },
    #[cfg(feature = "users")]
    AppletEntry {
        name: "users",
        run: run_users,
    },
    #[cfg(feature = "usleep")]
    AppletEntry {
        name: "usleep",
        run: run_usleep,
    },
    #[cfg(feature = "uudecode")]
    AppletEntry {
        name: "uudecode",
        run: run_uudecode,
    },
    #[cfg(feature = "uuencode")]
    AppletEntry {
        name: "uuencode",
        run: run_uuencode,
    },
    #[cfg(feature = "uuidgen")]
    AppletEntry {
        name: "uuidgen",
        run: run_uuidgen,
    },
    #[cfg(feature = "vconfig")]
    AppletEntry {
        name: "vconfig",
        run: run_vconfig,
    },
    #[cfg(feature = "vi")]
    AppletEntry {
        name: "vi",
        run: run_vi,
    },
    #[cfg(feature = "vlock")]
    AppletEntry {
        name: "vlock",
        run: run_vlock,
    },
    #[cfg(feature = "vmstat")]
    AppletEntry {
        name: "vmstat",
        run: run_vmstat,
    },
    #[cfg(feature = "volname")]
    AppletEntry {
        name: "volname",
        run: run_volname,
    },
    #[cfg(feature = "w")]
    AppletEntry {
        name: "w",
        run: run_w,
    },
    #[cfg(feature = "wall")]
    AppletEntry {
        name: "wall",
        run: run_wall,
    },
    #[cfg(feature = "watch")]
    AppletEntry {
        name: "watch",
        run: run_watch,
    },
    #[cfg(feature = "watchdog")]
    AppletEntry {
        name: "watchdog",
        run: run_watchdog,
    },
    #[cfg(feature = "wc")]
    AppletEntry {
        name: "wc",
        run: run_wc,
    },
    #[cfg(feature = "wget")]
    AppletEntry {
        name: "wget",
        run: run_wget,
    },
    #[cfg(feature = "which")]
    AppletEntry {
        name: "which",
        run: run_which,
    },
    #[cfg(feature = "who")]
    AppletEntry {
        name: "who",
        run: run_who,
    },
    #[cfg(feature = "whoami")]
    AppletEntry {
        name: "whoami",
        run: run_whoami,
    },
    #[cfg(feature = "whois")]
    AppletEntry {
        name: "whois",
        run: run_whois,
    },
    #[cfg(feature = "xargs")]
    AppletEntry {
        name: "xargs",
        run: run_xargs,
    },
    #[cfg(feature = "xxd")]
    AppletEntry {
        name: "xxd",
        run: run_xxd,
    },
    #[cfg(feature = "xz")]
    AppletEntry {
        name: "xz",
        run: run_xz,
    },
    #[cfg(feature = "xzcat")]
    AppletEntry {
        name: "xzcat",
        run: run_xzcat,
    },
    #[cfg(feature = "yes")]
    AppletEntry {
        name: "yes",
        run: run_yes,
    },
    #[cfg(feature = "zcat")]
    AppletEntry {
        name: "zcat",
        run: run_zcat,
    },
    #[cfg(feature = "zcip")]
    AppletEntry {
        name: "zcip",
        run: run_zcip,
    },
];

pub fn find_applet(name: &str) -> Option<&'static AppletEntry> {
    APPLETS
        .binary_search_by_key(&name, |a| a.name)
        .ok()
        .map(|idx| &APPLETS[idx])
}
