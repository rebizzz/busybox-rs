use crate::core::Applet;
use std::sync::Arc;

pub mod config;
pub mod servers;
pub mod sockets;
pub mod tools;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "ifconfig")]
    applets.push(Arc::new(config::IfconfigApplet));
    #[cfg(feature = "route")]
    applets.push(Arc::new(config::RouteApplet));
    #[cfg(feature = "netstat")]
    applets.push(Arc::new(config::NetstatApplet));
    #[cfg(feature = "arp")]
    applets.push(Arc::new(config::ArpApplet));
    #[cfg(feature = "arping")]
    applets.push(Arc::new(config::ArpingApplet));
    #[cfg(feature = "ether_wake")]
    applets.push(Arc::new(config::EtherWakeApplet));
    #[cfg(feature = "nameif")]
    applets.push(Arc::new(config::NameifApplet));
    #[cfg(feature = "ip")]
    applets.push(Arc::new(config::IpApplet));
    #[cfg(feature = "ipaddr")]
    applets.push(Arc::new(config::IpaddrApplet));
    #[cfg(feature = "iplink")]
    applets.push(Arc::new(config::IplinkApplet));
    #[cfg(feature = "ipneigh")]
    applets.push(Arc::new(config::IpneighApplet));
    #[cfg(feature = "iproute")]
    applets.push(Arc::new(config::IprouteApplet));
    #[cfg(feature = "iprule")]
    applets.push(Arc::new(config::IpruleApplet));
    #[cfg(feature = "iptunnel")]
    applets.push(Arc::new(config::IptunnelApplet));
    #[cfg(feature = "ifup")]
    applets.push(Arc::new(config::IfupApplet));
    #[cfg(feature = "ifdown")]
    applets.push(Arc::new(config::IfdownApplet));
    #[cfg(feature = "ifenslave")]
    applets.push(Arc::new(config::IfenslaveApplet));
    #[cfg(feature = "ifplugd")]
    applets.push(Arc::new(config::IfplugdApplet));
    #[cfg(feature = "vconfig")]
    applets.push(Arc::new(config::VconfigApplet));
    #[cfg(feature = "tunctl")]
    applets.push(Arc::new(config::TunctlApplet));
    #[cfg(feature = "slattach")]
    applets.push(Arc::new(config::SlattachApplet));
    #[cfg(feature = "brctl")]
    applets.push(Arc::new(config::BrctlApplet));
    #[cfg(feature = "httpd")]
    applets.push(Arc::new(servers::HttpdApplet));
    #[cfg(feature = "ftpd")]
    applets.push(Arc::new(servers::FtpdApplet));
    #[cfg(feature = "ftpget")]
    applets.push(Arc::new(servers::FtpgetApplet));
    #[cfg(feature = "ftpput")]
    applets.push(Arc::new(servers::FtpputApplet));
    #[cfg(feature = "tftp")]
    applets.push(Arc::new(servers::TftpApplet));
    #[cfg(feature = "tftpd")]
    applets.push(Arc::new(servers::TftpdApplet));
    #[cfg(feature = "telnet")]
    applets.push(Arc::new(servers::TelnetApplet));
    #[cfg(feature = "telnetd")]
    applets.push(Arc::new(servers::TelnetdApplet));
    #[cfg(feature = "inetd")]
    applets.push(Arc::new(servers::InetdApplet));
    #[cfg(feature = "fakeidentd")]
    applets.push(Arc::new(servers::FakeidentdApplet));
    #[cfg(feature = "dnsd")]
    applets.push(Arc::new(servers::DnsdApplet));
    #[cfg(feature = "dhcprelay")]
    applets.push(Arc::new(servers::DhcprelayApplet));
    #[cfg(feature = "udhcpc")]
    applets.push(Arc::new(servers::UdhcpcApplet));
    #[cfg(feature = "udhcpc6")]
    applets.push(Arc::new(servers::Udhcpc6Applet));
    #[cfg(feature = "udhcpd")]
    applets.push(Arc::new(servers::UdhcpdApplet));
    #[cfg(feature = "tcpsvd")]
    applets.push(Arc::new(sockets::TcpsvdApplet));
    #[cfg(feature = "udpsvd")]
    applets.push(Arc::new(sockets::UdpsvdApplet));
    #[cfg(feature = "nc")]
    applets.push(Arc::new(sockets::NcApplet));
    #[cfg(feature = "ping")]
    applets.push(Arc::new(sockets::PingApplet));
    #[cfg(feature = "ping6")]
    applets.push(Arc::new(sockets::Ping6Applet));
    #[cfg(feature = "traceroute")]
    applets.push(Arc::new(sockets::TracerouteApplet));
    #[cfg(feature = "traceroute6")]
    applets.push(Arc::new(sockets::Traceroute6Applet));
    #[cfg(feature = "whois")]
    applets.push(Arc::new(sockets::WhoisApplet));
    #[cfg(feature = "nslookup")]
    applets.push(Arc::new(sockets::NslookupApplet));
    #[cfg(feature = "ssl_client")]
    applets.push(Arc::new(sockets::SslClientApplet));
    #[cfg(feature = "ssl_server")]
    applets.push(Arc::new(sockets::SslServerApplet));
    #[cfg(feature = "zcip")]
    applets.push(Arc::new(tools::ZcipApplet));
    #[cfg(feature = "wget")]
    applets.push(Arc::new(tools::WgetApplet));
    #[cfg(feature = "sendmail")]
    applets.push(Arc::new(tools::SendmailApplet));
    #[cfg(feature = "popmaildir")]
    applets.push(Arc::new(tools::PopmaildirApplet));
    #[cfg(feature = "ntpd")]
    applets.push(Arc::new(tools::NtpdApplet));
    #[cfg(feature = "rdate")]
    applets.push(Arc::new(tools::RdateApplet));
    #[cfg(feature = "chat")]
    applets.push(Arc::new(tools::ChatApplet));
    #[cfg(feature = "microcom")]
    applets.push(Arc::new(tools::MicrocomApplet));
    #[cfg(feature = "pscan")]
    applets.push(Arc::new(tools::PscanApplet));
    #[cfg(feature = "dnsdomainname")]
    applets.push(Arc::new(tools::DnsdomainnameApplet));
    #[cfg(feature = "ipcalc")]
    applets.push(Arc::new(tools::IpcalcApplet));
    #[cfg(feature = "watchdog")]
    applets.push(Arc::new(tools::WatchdogApplet));
    #[cfg(feature = "conspy")]
    applets.push(Arc::new(tools::ConspyApplet));
    #[cfg(feature = "setconsole")]
    applets.push(Arc::new(tools::SetconsoleApplet));
    #[cfg(feature = "resize")]
    applets.push(Arc::new(tools::ResizeApplet));
}
