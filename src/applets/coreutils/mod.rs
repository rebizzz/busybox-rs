use crate::core::Applet;
use std::sync::Arc;

pub mod cal;
#[macro_use]
pub mod common;
pub mod r#true;
pub mod r#false;
pub mod pwd;
pub mod echo;
pub mod printenv;
pub mod sleep;
pub mod yes;
pub mod whoami;
pub mod arch;
pub mod nproc;
pub mod sync;
pub mod clear;
pub mod reset;
pub mod tsort;
pub mod seq;
pub mod factor;
pub mod tee;
pub mod strings;
pub mod comm;
pub mod uniq;
pub mod expand;
pub mod unexpand;
pub mod md5sum;
pub mod sha1sum;
pub mod sha256sum;
pub mod sha512sum;
pub mod sum;
pub mod chmod;
pub mod chown;
pub mod chgrp;
pub mod ln;
pub mod stat;
pub mod du;
pub mod df;
pub mod paste;
pub mod nl;
pub mod od;
pub mod dos2unix;
pub mod unix2dos;
pub mod ascii;
pub mod base32;
pub mod base64;
pub mod blkid;
pub mod chattr;
pub mod crc32;
pub mod fatattr;
pub mod findfs;
pub mod getfattr;
pub mod hd;
pub mod hexdump;
pub mod hexedit;
pub mod iostat;
pub mod ipcrm;
pub mod ipcs;
pub mod losetup;
pub mod lsattr;
pub mod makedevs;
pub mod makemime;
pub mod mdev;
pub mod mountpoint;
pub mod mpstat;
pub mod reformime;
pub mod setfattr;
pub mod sha384sum;
pub mod sha3sum;
pub mod swaplabel;
pub mod uevent;
pub mod uuidgen;
pub mod volname;
pub mod cut;
pub mod encode;
pub mod fold;
pub mod basename;
pub mod cp;
pub mod dirname;
pub mod link;
pub mod ls;
pub mod mkdir;
pub mod mv;
pub mod readlink;
pub mod realpath;
pub mod rm;
pub mod rmdir;
pub mod touch;
pub mod unlink;
pub mod which;
pub mod printf;
pub mod dd;
pub mod fuser;
pub mod mkfifo;
pub mod mknod;
pub mod nice;
pub mod nmeter;
pub mod nohup;
pub mod powertop;
pub mod pstree;
pub mod shred;
pub mod time;
pub mod tty;
pub mod users;
pub mod usleep;
pub mod w;
pub mod who;
pub mod rev;
pub mod sort;
pub mod tr;
pub mod uudecode;
pub mod xxd;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "cal")]
    applets.push(Arc::new(cal::CalApplet));
    #[cfg(feature = "md5sum")]
    applets.push(Arc::new(md5sum::Md5SumApplet));
    #[cfg(feature = "sha1sum")]
    applets.push(Arc::new(sha1sum::Sha1SumApplet));
    #[cfg(feature = "sha256sum")]
    applets.push(Arc::new(sha256sum::Sha256SumApplet));
    #[cfg(feature = "sha512sum")]
    applets.push(Arc::new(sha512sum::Sha512SumApplet));
    #[cfg(feature = "sum")]
    applets.push(Arc::new(sum::SumApplet));
    #[cfg(feature = "true")]
    applets.push(Arc::new(r#true::TrueApplet));
    #[cfg(feature = "false")]
    applets.push(Arc::new(r#false::FalseApplet));
    #[cfg(feature = "pwd")]
    applets.push(Arc::new(pwd::PwdApplet));
    #[cfg(feature = "echo")]
    applets.push(Arc::new(echo::EchoApplet));
    #[cfg(feature = "printenv")]
    applets.push(Arc::new(printenv::PrintenvApplet));
    #[cfg(feature = "sleep")]
    applets.push(Arc::new(sleep::SleepApplet));
    #[cfg(feature = "yes")]
    applets.push(Arc::new(yes::YesApplet));
    #[cfg(feature = "whoami")]
    applets.push(Arc::new(whoami::WhoamiApplet));
    #[cfg(feature = "arch")]
    applets.push(Arc::new(arch::ArchApplet));
    #[cfg(feature = "nproc")]
    applets.push(Arc::new(nproc::NprocApplet));
    #[cfg(feature = "sync")]
    applets.push(Arc::new(sync::SyncApplet));
    #[cfg(feature = "clear")]
    applets.push(Arc::new(clear::ClearApplet));
    #[cfg(feature = "reset")]
    applets.push(Arc::new(reset::ResetApplet));
    #[cfg(feature = "sha384sum")]
    applets.push(Arc::new(sha384sum::Sha384sumApplet));
    #[cfg(feature = "sha3sum")]
    applets.push(Arc::new(sha3sum::Sha3sumApplet));
    #[cfg(feature = "base32")]
    applets.push(Arc::new(base32::Base32Applet));
    #[cfg(feature = "base64")]
    applets.push(Arc::new(base64::Base64Applet));
    #[cfg(feature = "crc32")]
    applets.push(Arc::new(crc32::Crc32Applet));
    #[cfg(feature = "makemime")]
    applets.push(Arc::new(makemime::MakemimeApplet));
    #[cfg(feature = "reformime")]
    applets.push(Arc::new(reformime::ReformimeApplet));
    #[cfg(feature = "ascii")]
    applets.push(Arc::new(ascii::AsciiApplet));
    #[cfg(feature = "hd")]
    applets.push(Arc::new(hd::HdApplet));
    #[cfg(feature = "hexdump")]
    applets.push(Arc::new(hexdump::HexdumpApplet));
    #[cfg(feature = "hexedit")]
    applets.push(Arc::new(hexedit::HexeditApplet));
    #[cfg(feature = "volname")]
    applets.push(Arc::new(volname::VolnameApplet));
    #[cfg(feature = "blkid")]
    applets.push(Arc::new(blkid::BlkidApplet));
    #[cfg(feature = "findfs")]
    applets.push(Arc::new(findfs::FindfsApplet));
    #[cfg(feature = "lsattr")]
    applets.push(Arc::new(lsattr::LsattrApplet));
    #[cfg(feature = "chattr")]
    applets.push(Arc::new(chattr::ChattrApplet));
    #[cfg(feature = "getfattr")]
    applets.push(Arc::new(getfattr::GetfattrApplet));
    #[cfg(feature = "setfattr")]
    applets.push(Arc::new(setfattr::SetfattrApplet));
    #[cfg(feature = "fatattr")]
    applets.push(Arc::new(fatattr::FatattrApplet));
    #[cfg(feature = "losetup")]
    applets.push(Arc::new(losetup::LosetupApplet));
    #[cfg(feature = "mountpoint")]
    applets.push(Arc::new(mountpoint::MountpointApplet));
    #[cfg(feature = "mdev")]
    applets.push(Arc::new(mdev::MdevApplet));
    #[cfg(feature = "uevent")]
    applets.push(Arc::new(uevent::UeventApplet));
    #[cfg(feature = "makedevs")]
    applets.push(Arc::new(makedevs::MakedevsApplet));
    #[cfg(feature = "swaplabel")]
    applets.push(Arc::new(swaplabel::SwaplabelApplet));
    #[cfg(feature = "uuidgen")]
    applets.push(Arc::new(uuidgen::UuidgenApplet));
    #[cfg(feature = "ipcrm")]
    applets.push(Arc::new(ipcrm::IpcrmApplet));
    #[cfg(feature = "ipcs")]
    applets.push(Arc::new(ipcs::IpcsApplet));
    #[cfg(feature = "iostat")]
    applets.push(Arc::new(iostat::IostatApplet));
    #[cfg(feature = "mpstat")]
    applets.push(Arc::new(mpstat::MpstatApplet));
    #[cfg(feature = "cut")]
    applets.push(Arc::new(cut::CutApplet));
    #[cfg(feature = "uuencode")]
    applets.push(Arc::new(encode::UuencodeApplet));
    #[cfg(feature = "fold")]
    applets.push(Arc::new(fold::FoldApplet));
    #[cfg(feature = "ls")]
    applets.push(Arc::new(ls::LsApplet));
    #[cfg(feature = "cp")]
    applets.push(Arc::new(cp::CpApplet));
    #[cfg(feature = "mv")]
    applets.push(Arc::new(mv::MvApplet));
    #[cfg(feature = "rm")]
    applets.push(Arc::new(rm::RmApplet));
    #[cfg(feature = "mkdir")]
    applets.push(Arc::new(mkdir::MkdirApplet));
    #[cfg(feature = "rmdir")]
    applets.push(Arc::new(rmdir::RmdirApplet));
    #[cfg(feature = "touch")]
    applets.push(Arc::new(touch::TouchApplet));
    #[cfg(feature = "link")]
    applets.push(Arc::new(link::LinkApplet));
    #[cfg(feature = "unlink")]
    applets.push(Arc::new(unlink::UnlinkApplet));
    #[cfg(feature = "dirname")]
    applets.push(Arc::new(dirname::DirnameApplet));
    #[cfg(feature = "basename")]
    applets.push(Arc::new(basename::BasenameApplet));
    #[cfg(feature = "which")]
    applets.push(Arc::new(which::WhichApplet));
    #[cfg(feature = "readlink")]
    applets.push(Arc::new(readlink::ReadlinkApplet));
    #[cfg(feature = "realpath")]
    applets.push(Arc::new(realpath::RealpathApplet));
    #[cfg(feature = "tsort")]
    applets.push(Arc::new(tsort::TsortApplet));
    #[cfg(feature = "seq")]
    applets.push(Arc::new(seq::SeqApplet));
    #[cfg(feature = "factor")]
    applets.push(Arc::new(factor::FactorApplet));
    #[cfg(feature = "chmod")]
    applets.push(Arc::new(chmod::ChmodApplet));
    #[cfg(feature = "chown")]
    applets.push(Arc::new(chown::ChownApplet));
    #[cfg(feature = "chgrp")]
    applets.push(Arc::new(chgrp::ChgrpApplet));
    #[cfg(feature = "ln")]
    applets.push(Arc::new(ln::LnApplet));
    #[cfg(feature = "stat")]
    applets.push(Arc::new(stat::StatApplet));
    #[cfg(feature = "du")]
    applets.push(Arc::new(du::DuApplet));
    #[cfg(feature = "df")]
    applets.push(Arc::new(df::DfApplet));
    #[cfg(feature = "printf")]
    applets.push(Arc::new(printf::PrintfApplet));
    #[cfg(feature = "nice")]
    applets.push(Arc::new(nice::NiceApplet));
    #[cfg(feature = "nohup")]
    applets.push(Arc::new(nohup::NohupApplet));
    #[cfg(feature = "shred")]
    applets.push(Arc::new(shred::ShredApplet));
    #[cfg(feature = "usleep")]
    applets.push(Arc::new(usleep::UsleepApplet));
    #[cfg(feature = "time")]
    applets.push(Arc::new(time::TimeApplet));
    #[cfg(feature = "users")]
    applets.push(Arc::new(users::UsersApplet));
    #[cfg(feature = "who")]
    applets.push(Arc::new(who::WhoApplet));
    #[cfg(feature = "tty")]
    applets.push(Arc::new(tty::TtyApplet));
    #[cfg(feature = "dd")]
    applets.push(Arc::new(dd::DdApplet));
    #[cfg(feature = "fuser")]
    applets.push(Arc::new(fuser::FuserApplet));
    #[cfg(feature = "pstree")]
    applets.push(Arc::new(pstree::PstreeApplet));
    #[cfg(feature = "w")]
    applets.push(Arc::new(w::WApplet));
    #[cfg(feature = "powertop")]
    applets.push(Arc::new(powertop::PowertopApplet));
    #[cfg(feature = "nmeter")]
    applets.push(Arc::new(nmeter::NmeterApplet));
    #[cfg(feature = "mkfifo")]
    applets.push(Arc::new(mkfifo::MkfifoApplet));
    #[cfg(feature = "mknod")]
    applets.push(Arc::new(mknod::MknodApplet));
    #[cfg(feature = "rev")]
    applets.push(Arc::new(rev::RevApplet));
    #[cfg(feature = "sort")]
    applets.push(Arc::new(sort::SortApplet));
    #[cfg(feature = "tee")]
    applets.push(Arc::new(tee::TeeApplet));
    #[cfg(feature = "strings")]
    applets.push(Arc::new(strings::StringsApplet));
    #[cfg(feature = "comm")]
    applets.push(Arc::new(comm::CommApplet));
    #[cfg(feature = "uniq")]
    applets.push(Arc::new(uniq::UniqApplet));
    #[cfg(feature = "expand")]
    applets.push(Arc::new(expand::ExpandApplet));
    #[cfg(feature = "unexpand")]
    applets.push(Arc::new(unexpand::UnexpandApplet));
    #[cfg(feature = "cat")]
    applets.push(Arc::new(cat::CatApplet));
    #[cfg(feature = "head")]
    applets.push(Arc::new(head::HeadApplet));
    #[cfg(feature = "wc")]
    applets.push(Arc::new(wc::WcApplet));
    #[cfg(feature = "tail")]
    applets.push(Arc::new(tail::TailApplet));
    #[cfg(feature = "paste")]
    applets.push(Arc::new(paste::PasteApplet));
    #[cfg(feature = "nl")]
    applets.push(Arc::new(nl::NlApplet));
    #[cfg(feature = "od")]
    applets.push(Arc::new(od::OdApplet));
    #[cfg(feature = "dos2unix")]
    applets.push(Arc::new(dos2unix::Dos2unixApplet));
    #[cfg(feature = "unix2dos")]
    applets.push(Arc::new(unix2dos::Unix2dosApplet));
    #[cfg(feature = "tr")]
    applets.push(Arc::new(tr::TrApplet));
    #[cfg(feature = "uudecode")]
    applets.push(Arc::new(uudecode::UudecodeApplet));
    #[cfg(feature = "xxd")]
    applets.push(Arc::new(xxd::XxdApplet));
}
pub mod cat;
pub mod head;
pub mod wc;
pub mod tail;
