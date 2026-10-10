use crate::core::Applet;
use std::sync::Arc;

pub mod cal;
pub mod checksums;
pub mod core;
pub mod crypto_attr;
pub mod cut;
pub mod encode;
pub mod fold;
pub mod fs;
pub mod link;
pub mod numbers;
pub mod perms;
pub mod printf;
pub mod process_misc;
pub mod rev;
pub mod sort;
pub mod stream;
pub mod tabs;
pub mod text;
pub mod text2;
pub mod tr;
pub mod uudecode;
pub mod xxd;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "cal")]
    applets.push(Arc::new(cal::CalApplet));
    #[cfg(feature = "md5sum")]
    applets.push(Arc::new(checksums::Md5SumApplet));
    #[cfg(feature = "sha1sum")]
    applets.push(Arc::new(checksums::Sha1SumApplet));
    #[cfg(feature = "sha256sum")]
    applets.push(Arc::new(checksums::Sha256SumApplet));
    #[cfg(feature = "sha512sum")]
    applets.push(Arc::new(checksums::Sha512SumApplet));
    #[cfg(feature = "sum")]
    applets.push(Arc::new(checksums::SumApplet));
    #[cfg(feature = "true")]
    applets.push(Arc::new(core::TrueApplet));
    #[cfg(feature = "false")]
    applets.push(Arc::new(core::FalseApplet));
    #[cfg(feature = "pwd")]
    applets.push(Arc::new(core::PwdApplet));
    #[cfg(feature = "echo")]
    applets.push(Arc::new(core::EchoApplet));
    #[cfg(feature = "printenv")]
    applets.push(Arc::new(core::PrintenvApplet));
    #[cfg(feature = "sleep")]
    applets.push(Arc::new(core::SleepApplet));
    #[cfg(feature = "yes")]
    applets.push(Arc::new(core::YesApplet));
    #[cfg(feature = "whoami")]
    applets.push(Arc::new(core::WhoamiApplet));
    #[cfg(feature = "arch")]
    applets.push(Arc::new(core::ArchApplet));
    #[cfg(feature = "nproc")]
    applets.push(Arc::new(core::NprocApplet));
    #[cfg(feature = "sync")]
    applets.push(Arc::new(core::SyncApplet));
    #[cfg(feature = "clear")]
    applets.push(Arc::new(core::ClearApplet));
    #[cfg(feature = "reset")]
    applets.push(Arc::new(core::ResetApplet));
    #[cfg(feature = "sha384sum")]
    applets.push(Arc::new(crypto_attr::Sha384sumApplet));
    #[cfg(feature = "sha3sum")]
    applets.push(Arc::new(crypto_attr::Sha3sumApplet));
    #[cfg(feature = "base32")]
    applets.push(Arc::new(crypto_attr::Base32Applet));
    #[cfg(feature = "base64")]
    applets.push(Arc::new(crypto_attr::Base64Applet));
    #[cfg(feature = "crc32")]
    applets.push(Arc::new(crypto_attr::Crc32Applet));
    #[cfg(feature = "makemime")]
    applets.push(Arc::new(crypto_attr::MakemimeApplet));
    #[cfg(feature = "reformime")]
    applets.push(Arc::new(crypto_attr::ReformimeApplet));
    #[cfg(feature = "ascii")]
    applets.push(Arc::new(crypto_attr::AsciiApplet));
    #[cfg(feature = "hd")]
    applets.push(Arc::new(crypto_attr::HdApplet));
    #[cfg(feature = "hexdump")]
    applets.push(Arc::new(crypto_attr::HexdumpApplet));
    #[cfg(feature = "hexedit")]
    applets.push(Arc::new(crypto_attr::HexeditApplet));
    #[cfg(feature = "volname")]
    applets.push(Arc::new(crypto_attr::VolnameApplet));
    #[cfg(feature = "blkid")]
    applets.push(Arc::new(crypto_attr::BlkidApplet));
    #[cfg(feature = "findfs")]
    applets.push(Arc::new(crypto_attr::FindfsApplet));
    #[cfg(feature = "lsattr")]
    applets.push(Arc::new(crypto_attr::LsattrApplet));
    #[cfg(feature = "chattr")]
    applets.push(Arc::new(crypto_attr::ChattrApplet));
    #[cfg(feature = "getfattr")]
    applets.push(Arc::new(crypto_attr::GetfattrApplet));
    #[cfg(feature = "setfattr")]
    applets.push(Arc::new(crypto_attr::SetfattrApplet));
    #[cfg(feature = "fatattr")]
    applets.push(Arc::new(crypto_attr::FatattrApplet));
    #[cfg(feature = "losetup")]
    applets.push(Arc::new(crypto_attr::LosetupApplet));
    #[cfg(feature = "mountpoint")]
    applets.push(Arc::new(crypto_attr::MountpointApplet));
    #[cfg(feature = "mdev")]
    applets.push(Arc::new(crypto_attr::MdevApplet));
    #[cfg(feature = "uevent")]
    applets.push(Arc::new(crypto_attr::UeventApplet));
    #[cfg(feature = "makedevs")]
    applets.push(Arc::new(crypto_attr::MakedevsApplet));
    #[cfg(feature = "swaplabel")]
    applets.push(Arc::new(crypto_attr::SwaplabelApplet));
    #[cfg(feature = "uuidgen")]
    applets.push(Arc::new(crypto_attr::UuidgenApplet));
    #[cfg(feature = "ipcrm")]
    applets.push(Arc::new(crypto_attr::IpcrmApplet));
    #[cfg(feature = "ipcs")]
    applets.push(Arc::new(crypto_attr::IpcsApplet));
    #[cfg(feature = "iostat")]
    applets.push(Arc::new(crypto_attr::IostatApplet));
    #[cfg(feature = "mpstat")]
    applets.push(Arc::new(crypto_attr::MpstatApplet));
    #[cfg(feature = "cut")]
    applets.push(Arc::new(cut::CutApplet));
    #[cfg(feature = "uuencode")]
    applets.push(Arc::new(encode::UuencodeApplet));
    #[cfg(feature = "fold")]
    applets.push(Arc::new(fold::FoldApplet));
    #[cfg(feature = "ls")]
    applets.push(Arc::new(fs::LsApplet));
    #[cfg(feature = "cp")]
    applets.push(Arc::new(fs::CpApplet));
    #[cfg(feature = "mv")]
    applets.push(Arc::new(fs::MvApplet));
    #[cfg(feature = "rm")]
    applets.push(Arc::new(fs::RmApplet));
    #[cfg(feature = "mkdir")]
    applets.push(Arc::new(fs::MkdirApplet));
    #[cfg(feature = "rmdir")]
    applets.push(Arc::new(fs::RmdirApplet));
    #[cfg(feature = "touch")]
    applets.push(Arc::new(fs::TouchApplet));
    #[cfg(feature = "link")]
    applets.push(Arc::new(fs::LinkApplet));
    #[cfg(feature = "unlink")]
    applets.push(Arc::new(fs::UnlinkApplet));
    #[cfg(feature = "dirname")]
    applets.push(Arc::new(fs::DirnameApplet));
    #[cfg(feature = "basename")]
    applets.push(Arc::new(fs::BasenameApplet));
    #[cfg(feature = "which")]
    applets.push(Arc::new(fs::WhichApplet));
    #[cfg(feature = "readlink")]
    applets.push(Arc::new(link::ReadlinkApplet));
    #[cfg(feature = "realpath")]
    applets.push(Arc::new(link::RealpathApplet));
    #[cfg(feature = "tsort")]
    applets.push(Arc::new(numbers::TsortApplet));
    #[cfg(feature = "seq")]
    applets.push(Arc::new(numbers::SeqApplet));
    #[cfg(feature = "factor")]
    applets.push(Arc::new(numbers::FactorApplet));
    #[cfg(feature = "chmod")]
    applets.push(Arc::new(perms::ChmodApplet));
    #[cfg(feature = "chown")]
    applets.push(Arc::new(perms::ChownApplet));
    #[cfg(feature = "chgrp")]
    applets.push(Arc::new(perms::ChgrpApplet));
    #[cfg(feature = "ln")]
    applets.push(Arc::new(perms::LnApplet));
    #[cfg(feature = "stat")]
    applets.push(Arc::new(perms::StatApplet));
    #[cfg(feature = "du")]
    applets.push(Arc::new(perms::DuApplet));
    #[cfg(feature = "df")]
    applets.push(Arc::new(perms::DfApplet));
    #[cfg(feature = "printf")]
    applets.push(Arc::new(printf::PrintfApplet));
    #[cfg(feature = "nice")]
    applets.push(Arc::new(process_misc::NiceApplet));
    #[cfg(feature = "nohup")]
    applets.push(Arc::new(process_misc::NohupApplet));
    #[cfg(feature = "shred")]
    applets.push(Arc::new(process_misc::ShredApplet));
    #[cfg(feature = "usleep")]
    applets.push(Arc::new(process_misc::UsleepApplet));
    #[cfg(feature = "time")]
    applets.push(Arc::new(process_misc::TimeApplet));
    #[cfg(feature = "users")]
    applets.push(Arc::new(process_misc::UsersApplet));
    #[cfg(feature = "who")]
    applets.push(Arc::new(process_misc::WhoApplet));
    #[cfg(feature = "tty")]
    applets.push(Arc::new(process_misc::TtyApplet));
    #[cfg(feature = "dd")]
    applets.push(Arc::new(process_misc::DdApplet));
    #[cfg(feature = "fuser")]
    applets.push(Arc::new(process_misc::FuserApplet));
    #[cfg(feature = "pstree")]
    applets.push(Arc::new(process_misc::PstreeApplet));
    #[cfg(feature = "w")]
    applets.push(Arc::new(process_misc::WApplet));
    #[cfg(feature = "powertop")]
    applets.push(Arc::new(process_misc::PowertopApplet));
    #[cfg(feature = "nmeter")]
    applets.push(Arc::new(process_misc::NmeterApplet));
    #[cfg(feature = "mkfifo")]
    applets.push(Arc::new(process_misc::MkfifoApplet));
    #[cfg(feature = "mknod")]
    applets.push(Arc::new(process_misc::MknodApplet));
    #[cfg(feature = "rev")]
    applets.push(Arc::new(rev::RevApplet));
    #[cfg(feature = "sort")]
    applets.push(Arc::new(sort::SortApplet));
    #[cfg(feature = "tee")]
    applets.push(Arc::new(stream::TeeApplet));
    #[cfg(feature = "strings")]
    applets.push(Arc::new(stream::StringsApplet));
    #[cfg(feature = "comm")]
    applets.push(Arc::new(stream::CommApplet));
    #[cfg(feature = "uniq")]
    applets.push(Arc::new(stream::UniqApplet));
    #[cfg(feature = "expand")]
    applets.push(Arc::new(tabs::ExpandApplet));
    #[cfg(feature = "unexpand")]
    applets.push(Arc::new(tabs::UnexpandApplet));
    #[cfg(feature = "cat")]
    applets.push(Arc::new(text::CatApplet));
    #[cfg(feature = "head")]
    applets.push(Arc::new(text::HeadApplet));
    #[cfg(feature = "wc")]
    applets.push(Arc::new(text::WcApplet));
    #[cfg(feature = "tail")]
    applets.push(Arc::new(text::TailApplet));
    #[cfg(feature = "paste")]
    applets.push(Arc::new(text2::PasteApplet));
    #[cfg(feature = "nl")]
    applets.push(Arc::new(text2::NlApplet));
    #[cfg(feature = "od")]
    applets.push(Arc::new(text2::OdApplet));
    #[cfg(feature = "dos2unix")]
    applets.push(Arc::new(text2::Dos2unixApplet));
    #[cfg(feature = "unix2dos")]
    applets.push(Arc::new(text2::Unix2dosApplet));
    #[cfg(feature = "tr")]
    applets.push(Arc::new(tr::TrApplet));
    #[cfg(feature = "uudecode")]
    applets.push(Arc::new(uudecode::UudecodeApplet));
    #[cfg(feature = "xxd")]
    applets.push(Arc::new(xxd::XxdApplet));
}
