use crate::core::Applet;
use std::sync::Arc;

pub mod hardware;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "fdflush")]
    applets.push(Arc::new(hardware::FdflushApplet));
    #[cfg(feature = "fdformat")]
    applets.push(Arc::new(hardware::FdformatApplet));
    #[cfg(feature = "blockdev")]
    applets.push(Arc::new(hardware::BlockdevApplet));
    #[cfg(feature = "blkdiscard")]
    applets.push(Arc::new(hardware::BlkdiscardApplet));
    #[cfg(feature = "freeramdisk")]
    applets.push(Arc::new(hardware::FreeramdiskApplet));
    #[cfg(feature = "readahead")]
    applets.push(Arc::new(hardware::ReadaheadApplet));
    #[cfg(feature = "fsync")]
    applets.push(Arc::new(hardware::FsyncApplet));
    #[cfg(feature = "devmem")]
    applets.push(Arc::new(hardware::DevmemApplet));
    #[cfg(feature = "i2cdetect")]
    applets.push(Arc::new(hardware::I2cdetectApplet));
    #[cfg(feature = "i2cdump")]
    applets.push(Arc::new(hardware::I2cdumpApplet));
    #[cfg(feature = "i2cget")]
    applets.push(Arc::new(hardware::I2cgetApplet));
    #[cfg(feature = "i2cset")]
    applets.push(Arc::new(hardware::I2csetApplet));
    #[cfg(feature = "i2ctransfer")]
    applets.push(Arc::new(hardware::I2ctransferApplet));
    #[cfg(feature = "lspci")]
    applets.push(Arc::new(hardware::LspciApplet));
    #[cfg(feature = "lsusb")]
    applets.push(Arc::new(hardware::LsusbApplet));
}
