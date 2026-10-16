use crate::core::Applet;
use std::sync::Arc;

pub mod common;
pub mod ifconfig;
pub mod route;
pub mod netstat;
pub mod arp;
pub mod arping;
pub mod ether_wake;
pub mod nameif;
pub mod ip;
pub mod ipaddr;
pub mod iplink;
pub mod ipneigh;
pub mod iproute;
pub mod iprule;
pub mod iptunnel;
pub mod ifup;
pub mod ifdown;
pub mod ifenslave;
pub mod ifplugd;
pub mod vconfig;
pub mod tunctl;
pub mod slattach;
pub mod brctl;
pub mod httpd;
pub mod ftpd;
pub mod ftpget;
pub mod ftpput;
pub mod tftp;
pub mod tftpd;
pub mod telnet;
pub mod telnetd;
pub mod inetd;
pub mod fakeidentd;
pub mod dnsd;
pub mod dhcprelay;
pub mod udhcpc;
pub mod udhcpc6;
pub mod udhcpd;
pub mod tcpsvd;
pub mod udpsvd;
pub mod nc;
pub mod ping;
pub mod ping6;
pub mod traceroute;
pub mod traceroute6;
pub mod whois;
pub mod nslookup;
pub mod ssl_client;
pub mod ssl_server;
pub mod zcip;
pub mod wget;
pub mod sendmail;
pub mod popmaildir;
pub mod ntpd;
pub mod rdate;
pub mod chat;
pub mod microcom;
pub mod pscan;
pub mod dnsdomainname;
pub mod ipcalc;
pub mod watchdog;
pub mod conspy;
pub mod setconsole;
pub mod resize;

pub fn register(applets: &mut Vec<Arc<dyn Applet>>) {
    let _ = applets;
    #[cfg(feature = "ifconfig")]
    applets.push(Arc::new(ifconfig::IfconfigApplet));
    #[cfg(feature = "route")]
    applets.push(Arc::new(route::RouteApplet));
    #[cfg(feature = "netstat")]
    applets.push(Arc::new(netstat::NetstatApplet));
    #[cfg(feature = "arp")]
    applets.push(Arc::new(arp::ArpApplet));
    #[cfg(feature = "arping")]
    applets.push(Arc::new(arping::ArpingApplet));
    #[cfg(feature = "ether_wake")]
    applets.push(Arc::new(ether_wake::EtherWakeApplet));
    #[cfg(feature = "nameif")]
    applets.push(Arc::new(nameif::NameifApplet));
    #[cfg(feature = "ip")]
    applets.push(Arc::new(ip::IpApplet));
    #[cfg(feature = "ipaddr")]
    applets.push(Arc::new(ipaddr::IpaddrApplet));
    #[cfg(feature = "iplink")]
    applets.push(Arc::new(iplink::IplinkApplet));
    #[cfg(feature = "ipneigh")]
    applets.push(Arc::new(ipneigh::IpneighApplet));
    #[cfg(feature = "iproute")]
    applets.push(Arc::new(iproute::IprouteApplet));
    #[cfg(feature = "iprule")]
    applets.push(Arc::new(iprule::IpruleApplet));
    #[cfg(feature = "iptunnel")]
    applets.push(Arc::new(iptunnel::IptunnelApplet));
    #[cfg(feature = "ifup")]
    applets.push(Arc::new(ifup::IfupApplet));
    #[cfg(feature = "ifdown")]
    applets.push(Arc::new(ifdown::IfdownApplet));
    #[cfg(feature = "ifenslave")]
    applets.push(Arc::new(ifenslave::IfenslaveApplet));
    #[cfg(feature = "ifplugd")]
    applets.push(Arc::new(ifplugd::IfplugdApplet));
    #[cfg(feature = "vconfig")]
    applets.push(Arc::new(vconfig::VconfigApplet));
    #[cfg(feature = "tunctl")]
    applets.push(Arc::new(tunctl::TunctlApplet));
    #[cfg(feature = "slattach")]
    applets.push(Arc::new(slattach::SlattachApplet));
    #[cfg(feature = "brctl")]
    applets.push(Arc::new(brctl::BrctlApplet));
    #[cfg(feature = "httpd")]
    applets.push(Arc::new(httpd::HttpdApplet));
    #[cfg(feature = "ftpd")]
    applets.push(Arc::new(ftpd::FtpdApplet));
    #[cfg(feature = "ftpget")]
    applets.push(Arc::new(ftpget::FtpgetApplet));
    #[cfg(feature = "ftpput")]
    applets.push(Arc::new(ftpput::FtpputApplet));
    #[cfg(feature = "tftp")]
    applets.push(Arc::new(tftp::TftpApplet));
    #[cfg(feature = "tftpd")]
    applets.push(Arc::new(tftpd::TftpdApplet));
    #[cfg(feature = "telnet")]
    applets.push(Arc::new(telnet::TelnetApplet));
    #[cfg(feature = "telnetd")]
    applets.push(Arc::new(telnetd::TelnetdApplet));
    #[cfg(feature = "inetd")]
    applets.push(Arc::new(inetd::InetdApplet));
    #[cfg(feature = "fakeidentd")]
    applets.push(Arc::new(fakeidentd::FakeidentdApplet));
    #[cfg(feature = "dnsd")]
    applets.push(Arc::new(dnsd::DnsdApplet));
    #[cfg(feature = "dhcprelay")]
    applets.push(Arc::new(dhcprelay::DhcprelayApplet));
    #[cfg(feature = "udhcpc")]
    applets.push(Arc::new(udhcpc::UdhcpcApplet));
    #[cfg(feature = "udhcpc6")]
    applets.push(Arc::new(udhcpc6::Udhcpc6Applet));
    #[cfg(feature = "udhcpd")]
    applets.push(Arc::new(udhcpd::UdhcpdApplet));
    #[cfg(feature = "tcpsvd")]
    applets.push(Arc::new(tcpsvd::TcpsvdApplet));
    #[cfg(feature = "udpsvd")]
    applets.push(Arc::new(udpsvd::UdpsvdApplet));
    #[cfg(feature = "nc")]
    applets.push(Arc::new(nc::NcApplet));
    #[cfg(feature = "ping")]
    applets.push(Arc::new(ping::PingApplet));
    #[cfg(feature = "ping6")]
    applets.push(Arc::new(ping6::Ping6Applet));
    #[cfg(feature = "traceroute")]
    applets.push(Arc::new(traceroute::TracerouteApplet));
    #[cfg(feature = "traceroute6")]
    applets.push(Arc::new(traceroute6::Traceroute6Applet));
    #[cfg(feature = "whois")]
    applets.push(Arc::new(whois::WhoisApplet));
    #[cfg(feature = "nslookup")]
    applets.push(Arc::new(nslookup::NslookupApplet));
    #[cfg(feature = "ssl_client")]
    applets.push(Arc::new(ssl_client::SslClientApplet));
    #[cfg(feature = "ssl_server")]
    applets.push(Arc::new(ssl_server::SslServerApplet));
    #[cfg(feature = "zcip")]
    applets.push(Arc::new(zcip::ZcipApplet));
    #[cfg(feature = "wget")]
    applets.push(Arc::new(wget::WgetApplet));
    #[cfg(feature = "sendmail")]
    applets.push(Arc::new(sendmail::SendmailApplet));
    #[cfg(feature = "popmaildir")]
    applets.push(Arc::new(popmaildir::PopmaildirApplet));
    #[cfg(feature = "ntpd")]
    applets.push(Arc::new(ntpd::NtpdApplet));
    #[cfg(feature = "rdate")]
    applets.push(Arc::new(rdate::RdateApplet));
    #[cfg(feature = "chat")]
    applets.push(Arc::new(chat::ChatApplet));
    #[cfg(feature = "microcom")]
    applets.push(Arc::new(microcom::MicrocomApplet));
    #[cfg(feature = "pscan")]
    applets.push(Arc::new(pscan::PscanApplet));
    #[cfg(feature = "dnsdomainname")]
    applets.push(Arc::new(dnsdomainname::DnsdomainnameApplet));
    #[cfg(feature = "ipcalc")]
    applets.push(Arc::new(ipcalc::IpcalcApplet));
    #[cfg(feature = "watchdog")]
    applets.push(Arc::new(watchdog::WatchdogApplet));
    #[cfg(feature = "conspy")]
    applets.push(Arc::new(conspy::ConspyApplet));
    #[cfg(feature = "setconsole")]
    applets.push(Arc::new(setconsole::SetconsoleApplet));
    #[cfg(feature = "resize")]
    applets.push(Arc::new(resize::ResizeApplet));
}
