use std::convert::Into;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

pub mod client;
mod http_helper;
pub mod service;

#[cfg(test)]
mod http_helper_test;
mod socket_helper;

#[derive(PartialEq)]
/// The Multicast Address in use
///
/// This uses officially assigned addresses by IANA
pub enum MulticastAddr {
    /// The IPv4 Multicast address: `239.255.255.250`
    ///
    /// This is referred to as `site-local`
    V4,

    /// The IPv6 Multicast address: `ff02::c`
    ///
    /// This is referred to as `link-local`
    V6LinkLocal,

    /// The IPv6 Multicast address: `ff05::c`
    ///
    /// This is referred to as `site-local`
    V6SiteLocal,

    /// This is used only for test purposes or a local setup
    /// 
    /// Multicast is not redirected to the originating host, so we cannot run a Service and a Client on the same device
    Loopback,
}

impl MulticastAddr {
    /// Returns an [IpAddr] for the given enum value
    pub fn get_ip(&self) -> IpAddr {
        match self {
            MulticastAddr::V4 => self.get_v4().unwrap().into(),
            MulticastAddr::V6LinkLocal => self.get_v6().unwrap().into(),
            MulticastAddr::V6SiteLocal => self.get_v6().unwrap().into(),
            MulticastAddr::Loopback => self.get_v4().unwrap().into(),
        }
    }

    /// Returns `true` if the enum value is IPv4, `false` if it's IPv6
    pub fn is_v4(&self) -> bool {
        match self {
            MulticastAddr::V4 => true,
            MulticastAddr::V6LinkLocal => false,
            MulticastAddr::V6SiteLocal => false,
            MulticastAddr::Loopback => true,
        }
    }

    /// Returns the IPv4 addr if enum is an IPv4 addr
    pub fn get_v4(&self) -> Option<Ipv4Addr> {
        match self {
            MulticastAddr::V4 => Some(Ipv4Addr::new(239, 255, 255, 250)),
            MulticastAddr::V6LinkLocal => None,
            MulticastAddr::V6SiteLocal => None,
            MulticastAddr::Loopback => Some(Ipv4Addr::new(127, 0, 0, 1)),
        }
    }

    /// Returns the IPv6 addr if enum is an IPv6 addr
    pub fn get_v6(&self) -> Option<Ipv6Addr> {
        match self {
            MulticastAddr::V4 => None,
            MulticastAddr::V6LinkLocal => Some(Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0xC)),
            MulticastAddr::V6SiteLocal => Some(Ipv6Addr::new(0xFF05, 0, 0, 0, 0, 0, 0, 0xC)),
            MulticastAddr::Loopback => None,
        }
    }
}

/// Port assigned by IANA for SSDP
pub(crate) static SSDP_PORT: u16 = 1900;
