use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod bc;
pub mod dc;
pub mod less;
pub mod man;
pub mod more;
pub mod fdflush;
pub mod fdformat;
pub mod blockdev;
pub mod blkdiscard;
pub mod freeramdisk;
pub mod readahead;
pub mod fsync;
pub mod devmem;
pub mod i2cdetect;
pub mod i2cdump;
pub mod i2cget;
pub mod i2cset;
pub mod i2ctransfer;
pub mod lspci;
pub mod lsusb;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "bc")]
    applets.push(Arc::new(bc::BcApplet));
    #[cfg(feature = "dc")]
    applets.push(Arc::new(dc::DcApplet));
    #[cfg(feature = "less")]
    applets.push(Arc::new(less::LessApplet));
    #[cfg(feature = "man")]
    applets.push(Arc::new(man::ManApplet));
    #[cfg(feature = "more")]
    applets.push(Arc::new(more::MoreApplet));
    #[cfg(feature = "fdflush")]
    applets.push(Arc::new(fdflush::FdflushApplet));
    #[cfg(feature = "fdformat")]
    applets.push(Arc::new(fdformat::FdformatApplet));
    #[cfg(feature = "blockdev")]
    applets.push(Arc::new(blockdev::BlockdevApplet));
    #[cfg(feature = "blkdiscard")]
    applets.push(Arc::new(blkdiscard::BlkdiscardApplet));
    #[cfg(feature = "freeramdisk")]
    applets.push(Arc::new(freeramdisk::FreeramdiskApplet));
    #[cfg(feature = "readahead")]
    applets.push(Arc::new(readahead::ReadaheadApplet));
    #[cfg(feature = "fsync")]
    applets.push(Arc::new(fsync::FsyncApplet));
    #[cfg(feature = "devmem")]
    applets.push(Arc::new(devmem::DevmemApplet));
    #[cfg(feature = "i2cdetect")]
    applets.push(Arc::new(i2cdetect::I2cdetectApplet));
    #[cfg(feature = "i2cdump")]
    applets.push(Arc::new(i2cdump::I2cdumpApplet));
    #[cfg(feature = "i2cget")]
    applets.push(Arc::new(i2cget::I2cgetApplet));
    #[cfg(feature = "i2cset")]
    applets.push(Arc::new(i2cset::I2csetApplet));
    #[cfg(feature = "i2ctransfer")]
    applets.push(Arc::new(i2ctransfer::I2ctransferApplet));
    #[cfg(feature = "lspci")]
    applets.push(Arc::new(lspci::LspciApplet));
    #[cfg(feature = "lsusb")]
    applets.push(Arc::new(lsusb::LsusbApplet));
}
