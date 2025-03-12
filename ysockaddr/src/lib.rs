#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

//-----------------------------------------------
// core::net::SocketAddr was added in 1.80 ?
//-----------------------------------------------

#[cfg(not(feature = "std"))]
use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

#[cfg(feature = "std")]
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

//-----------------------------------------------
// Conversions
//-----------------------------------------------

/// Converted C/FFI SockAddr
#[derive(Debug, Clone)]
pub enum YSockAddrC {
    /// IPv4
    V4(libc::sockaddr_in, libc::socklen_t),
    /// IPv6
    V6(libc::sockaddr_in6, libc::socklen_t),
}

impl YSockAddrC {
    /// Cast to C/FFI sockaddr + len
    #[inline]
    pub fn as_c_sockaddr_len(&self) -> (*const libc::sockaddr, libc::socklen_t) {
        match *self {
            Self::V4(sa4_in, len) => (core::ptr::addr_of!(sa4_in) as *const libc::sockaddr, len),
            Self::V6(sa6_in, len) => (core::ptr::addr_of!(sa6_in) as *const libc::sockaddr, len),
        }
    }
}

impl From<SocketAddr> for YSockAddrC {
    #[inline]
    fn from(sa: SocketAddr) -> YSockAddrC {
        match sa {
            SocketAddr::V4(sa_v4) => YSockAddrC::V4(
                libc::sockaddr_in {
                    sin_family: libc::AF_INET as u16,
                    sin_port: sa_v4.port().to_be(),
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(sa_v4.ip().octets()),
                    },
                    sin_zero: [0, 0, 0, 0, 0, 0, 0, 0],
                },
                size_of::<libc::sockaddr_in>() as u32,
            ),
            SocketAddr::V6(sa_v6) => YSockAddrC::V6(
                libc::sockaddr_in6 {
                    sin6_family: libc::AF_INET6 as u16,
                    sin6_port: sa_v6.port().to_be(),
                    sin6_flowinfo: sa_v6.flowinfo(),
                    sin6_addr: libc::in6_addr {
                        s6_addr: sa_v6.ip().octets(),
                    },
                    sin6_scope_id: sa_v6.scope_id(),
                },
                size_of::<libc::sockaddr_in6>() as u32,
            ),
        }
    }
}

/// Immutable Raw SockAddr
#[derive(Debug, Clone)]
pub enum YSockAddrCrawImm {
    /// IPv4
    V4(*const libc::sockaddr_in, libc::socklen_t),
    /// IPv6
    V6(*const libc::sockaddr_in6, libc::socklen_t),
}

/// Mutable Raw SockAddr
#[derive(Debug, Clone)]
pub enum YSockAddrCrawMut {
    /// IPv4
    V4(*mut libc::sockaddr_in, libc::socklen_t),
    /// IPv6
    V6(*mut libc::sockaddr_in6, libc::socklen_t),
}

/// YSockAddr Rust convenience type and allows conversion into from C equiv.
#[derive(Debug, Clone)]
pub struct YSockAddrR(SocketAddr);

impl YSockAddrR {
    /// From Rust [`net::SockAddr`]
    #[inline]
    pub fn from_sockaddr(sa: SocketAddr) -> Self {
        Self(sa)
    }
    /// Inner Rust [`net::SockAddr`] representation
    #[inline]
    pub fn as_sockaddr(&self) -> SocketAddr {
        self.0
    }
    /// To C / FFI
    #[inline]
    pub fn as_c(&self) -> YSockAddrC {
        self.0.into()
    }
}

impl From<YSockAddrC> for YSockAddrR {
    #[inline]
    fn from(sa: YSockAddrC) -> YSockAddrR {
        match sa {
            YSockAddrC::V4(c_sa4, _) => {
                let ip4 = Ipv4Addr::from_bits(u32::from_be(c_sa4.sin_addr.s_addr));
                Self(SocketAddr::V4(SocketAddrV4::new(ip4, c_sa4.sin_port)))
            }
            YSockAddrC::V6(c_sa6, _) => {
                let in6_bits = u128::from_be_bytes(c_sa6.sin6_addr.s6_addr);
                let ip6 = Ipv6Addr::from_bits(in6_bits);
                Self(SocketAddr::V6(SocketAddrV6::new(
                    ip6,
                    c_sa6.sin6_port,
                    c_sa6.sin6_flowinfo,
                    c_sa6.sin6_scope_id,
                )))
            }
        }
    }
}
