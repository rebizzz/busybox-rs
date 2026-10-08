#[cfg(any(
    feature = "arch",
    feature = "clear",
    feature = "echo",
    feature = "false",
    feature = "nproc",
    feature = "printenv",
    feature = "pwd",
    feature = "reset",
    feature = "sleep",
    feature = "sync",
    feature = "true",
    feature = "whoami",
    feature = "yes"
))]
pub mod core_cmds;

#[cfg(feature = "cut")]
pub mod cut_cmd;

#[cfg(feature = "uuencode")]
pub mod encode_cmds;

#[cfg(feature = "fold")]
pub mod fold_cmd;

#[cfg(any(
    feature = "basename",
    feature = "cp",
    feature = "dirname",
    feature = "link",
    feature = "ls",
    feature = "mkdir",
    feature = "mv",
    feature = "rm",
    feature = "rmdir",
    feature = "touch",
    feature = "unlink",
    feature = "which"
))]
pub mod fs_cmds;

#[cfg(any(feature = "grep", feature = "egrep", feature = "fgrep"))]
pub mod grep_cmd;

#[cfg(any(feature = "factor", feature = "seq", feature = "tsort"))]
pub mod number_cmds;

#[cfg(feature = "rev")]
pub mod rev_cmd;

#[cfg(any(feature = "sh", feature = "xargs"))]
pub mod shell_cmds;

#[cfg(feature = "sort")]
pub mod sort_cmd;

#[cfg(any(
    feature = "comm",
    feature = "strings",
    feature = "tee",
    feature = "uniq"
))]
pub mod stream_cmds;

#[cfg(any(feature = "expand", feature = "unexpand"))]
pub mod tab_cmds;

#[cfg(any(feature = "cat", feature = "head", feature = "tail", feature = "wc"))]
pub mod text_cmds;

#[cfg(feature = "tr")]
pub mod tr_cmd;

#[cfg(feature = "uudecode")]
pub mod uudecode_cmd;

#[cfg(feature = "xxd")]
pub mod xxd_cmd;

use crate::core::Applet;
use std::sync::Arc;

#[allow(clippy::vec_init_then_push)]
pub fn get_applets() -> Vec<Arc<dyn Applet>> {
    let mut applets: Vec<Arc<dyn Applet>> = Vec::new();

    #[cfg(feature = "arch")]
    applets.push(Arc::new(core_cmds::ArchApplet));
    #[cfg(feature = "basename")]
    applets.push(Arc::new(fs_cmds::BasenameApplet));
    #[cfg(feature = "cat")]
    applets.push(Arc::new(text_cmds::CatApplet));
    #[cfg(feature = "clear")]
    applets.push(Arc::new(core_cmds::ClearApplet));
    #[cfg(feature = "comm")]
    applets.push(Arc::new(stream_cmds::CommApplet));
    #[cfg(feature = "cp")]
    applets.push(Arc::new(fs_cmds::CpApplet));
    #[cfg(feature = "cut")]
    applets.push(Arc::new(cut_cmd::CutApplet));
    #[cfg(feature = "dirname")]
    applets.push(Arc::new(fs_cmds::DirnameApplet));
    #[cfg(feature = "echo")]
    applets.push(Arc::new(core_cmds::EchoApplet));
    #[cfg(feature = "egrep")]
    applets.push(Arc::new(grep_cmd::EgrepApplet));
    #[cfg(feature = "expand")]
    applets.push(Arc::new(tab_cmds::ExpandApplet));
    #[cfg(feature = "factor")]
    applets.push(Arc::new(number_cmds::FactorApplet));
    #[cfg(feature = "false")]
    applets.push(Arc::new(core_cmds::FalseApplet));
    #[cfg(feature = "fgrep")]
    applets.push(Arc::new(grep_cmd::FgrepApplet));
    #[cfg(feature = "fold")]
    applets.push(Arc::new(fold_cmd::FoldApplet));
    #[cfg(feature = "grep")]
    applets.push(Arc::new(grep_cmd::GrepApplet));
    #[cfg(feature = "head")]
    applets.push(Arc::new(text_cmds::HeadApplet));
    #[cfg(feature = "link")]
    applets.push(Arc::new(fs_cmds::LinkApplet));
    #[cfg(feature = "ls")]
    applets.push(Arc::new(fs_cmds::LsApplet));
    #[cfg(feature = "mkdir")]
    applets.push(Arc::new(fs_cmds::MkdirApplet));
    #[cfg(feature = "mv")]
    applets.push(Arc::new(fs_cmds::MvApplet));
    #[cfg(feature = "nproc")]
    applets.push(Arc::new(core_cmds::NprocApplet));
    #[cfg(feature = "printenv")]
    applets.push(Arc::new(core_cmds::PrintenvApplet));
    #[cfg(feature = "pwd")]
    applets.push(Arc::new(core_cmds::PwdApplet));
    #[cfg(feature = "reset")]
    applets.push(Arc::new(core_cmds::ResetApplet));
    #[cfg(feature = "rev")]
    applets.push(Arc::new(rev_cmd::RevApplet));
    #[cfg(feature = "rm")]
    applets.push(Arc::new(fs_cmds::RmApplet));
    #[cfg(feature = "rmdir")]
    applets.push(Arc::new(fs_cmds::RmdirApplet));
    #[cfg(feature = "seq")]
    applets.push(Arc::new(number_cmds::SeqApplet));
    #[cfg(feature = "sh")]
    applets.push(Arc::new(shell_cmds::ShApplet));
    #[cfg(feature = "sleep")]
    applets.push(Arc::new(core_cmds::SleepApplet));
    #[cfg(feature = "sort")]
    applets.push(Arc::new(sort_cmd::SortApplet));
    #[cfg(feature = "tail")]
    applets.push(Arc::new(text_cmds::TailApplet));
    #[cfg(feature = "strings")]
    applets.push(Arc::new(stream_cmds::StringsApplet));
    #[cfg(feature = "sync")]
    applets.push(Arc::new(core_cmds::SyncApplet));
    #[cfg(feature = "tee")]
    applets.push(Arc::new(stream_cmds::TeeApplet));
    #[cfg(feature = "touch")]
    applets.push(Arc::new(fs_cmds::TouchApplet));
    #[cfg(feature = "tr")]
    applets.push(Arc::new(tr_cmd::TrApplet));
    #[cfg(feature = "true")]
    applets.push(Arc::new(core_cmds::TrueApplet));
    #[cfg(feature = "tsort")]
    applets.push(Arc::new(number_cmds::TsortApplet));
    #[cfg(feature = "unexpand")]
    applets.push(Arc::new(tab_cmds::UnexpandApplet));
    #[cfg(feature = "uniq")]
    applets.push(Arc::new(stream_cmds::UniqApplet));
    #[cfg(feature = "unlink")]
    applets.push(Arc::new(fs_cmds::UnlinkApplet));
    #[cfg(feature = "uudecode")]
    applets.push(Arc::new(uudecode_cmd::UudecodeApplet));
    #[cfg(feature = "uuencode")]
    applets.push(Arc::new(encode_cmds::UuencodeApplet));
    #[cfg(feature = "wc")]
    applets.push(Arc::new(text_cmds::WcApplet));
    #[cfg(feature = "which")]
    applets.push(Arc::new(fs_cmds::WhichApplet));
    #[cfg(feature = "whoami")]
    applets.push(Arc::new(core_cmds::WhoamiApplet));
    #[cfg(feature = "xargs")]
    applets.push(Arc::new(shell_cmds::XargsApplet));
    #[cfg(feature = "xxd")]
    applets.push(Arc::new(xxd_cmd::XxdApplet));
    #[cfg(feature = "yes")]
    applets.push(Arc::new(core_cmds::YesApplet));

    applets
}
