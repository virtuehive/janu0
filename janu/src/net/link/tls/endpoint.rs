//
// Copyright (c) 2017, 2020 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK janu team, <janu@adlink-labs.tech>
//
use super::config::*;
use super::*;
pub use async_rustls::rustls::*;
pub use async_rustls::webpki::*;
use async_std::net::{SocketAddr, ToSocketAddrs};
use std::fmt;
use std::str::FromStr;
use janu_util::core::{ZError, ZErrorKind, ZResult};
use janu_util::properties::config::*;
use janu_util::properties::Properties;
use janu_util::{zerror, zerror2};

#[allow(unreachable_patterns)]
pub(super) async fn get_tls_addr(address: &LocatorAddress) -> ZResult<SocketAddr> {
    match &address {
        LocatorAddress::Tls(addr) => match addr {
            LocatorTls::SocketAddr(addr) => Ok(*addr),
            LocatorTls::DnsName(addr) => match addr.to_socket_addrs().await {
                Ok(mut addr_iter) => {
                    if let Some(addr) = addr_iter.next() {
                        Ok(addr)
                    } else {
                        let e = format!("Couldn't resolve TLS locator address: {}", addr);
                        zerror!(ZErrorKind::InvalidLocator { descr: e })
                    }
                }
                Err(e) => {
                    let e = format!("{}: {}", e, addr);
                    zerror!(ZErrorKind::InvalidLocator { descr: e })
                }
            },
        },
        _ => {
            let e = format!("Not a TLS locator address: {}", address);
            return zerror!(ZErrorKind::InvalidLocator { descr: e });
        }
    }
}

#[allow(unreachable_patterns)]
pub(super) async fn get_tls_dns(address: &LocatorAddress) -> ZResult<DNSName> {
    match &address {
        LocatorAddress::Tls(addr) => match addr {
            LocatorTls::SocketAddr(addr) => {
                let e = format!("Couldn't get domain from SocketAddr: {}", addr);
                zerror!(ZErrorKind::InvalidLocator { descr: e })
            }
            LocatorTls::DnsName(addr) => {
                // Separate the domain from the port.
                // E.g. janu.io:7447 returns (janu.io, 7447).
                let split: Vec<&str> = addr.split(':').collect();
                match split.get(0) {
                    Some(dom) => {
                        let domain = DNSNameRef::try_from_ascii_str(dom).map_err(|e| {
                            let e = e.to_string();
                            zerror2!(ZErrorKind::InvalidLocator { descr: e })
                        })?;
                        Ok(domain.to_owned())
                    }
                    None => {
                        let e = format!("Couldn't get domain for: {}", addr);
                        zerror!(ZErrorKind::InvalidLocator { descr: e })
                    }
                }
            }
        },
        _ => {
            let e = format!("Not a TLS locator address: {}", address);
            return zerror!(ZErrorKind::InvalidLocator { descr: e });
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LocatorTls {
    SocketAddr(SocketAddr),
    DnsName(String),
}

impl LocatorTls {
    pub fn is_multicast(&self) -> bool {
        false
    }
}

impl FromStr for LocatorTls {
    type Err = ZError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(addr) => Ok(LocatorTls::SocketAddr(addr)),
            Err(_) => Ok(LocatorTls::DnsName(s.to_string())),
        }
    }
}

impl fmt::Display for LocatorTls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocatorTls::SocketAddr(addr) => write!(f, "{}", addr)?,
            LocatorTls::DnsName(addr) => write!(f, "{}", addr)?,
        }
        Ok(())
    }
}

/*************************************/
/*          LOCATOR CONFIG           */
/*************************************/
#[derive(Clone)]
pub struct LocatorConfigTls;

impl LocatorConfigTls {
    pub fn from_config(config: &ConfigProperties) -> ZResult<Option<Properties>> {
        let mut properties = Properties::default();

        if let Some(tls_ca_certificate) = config.get(&ZN_TLS_ROOT_CA_CERTIFICATE_KEY) {
            properties.insert(
                TLS_ROOT_CA_CERTIFICATE_FILE.into(),
                tls_ca_certificate.into(),
            );
        }
        if let Some(tls_server_private_key) = config.get(&ZN_TLS_SERVER_PRIVATE_KEY_KEY) {
            properties.insert(
                TLS_SERVER_PRIVATE_KEY_FILE.into(),
                tls_server_private_key.into(),
            );
        }
        if let Some(tls_server_certificate) = config.get(&ZN_TLS_SERVER_CERTIFICATE_KEY) {
            properties.insert(
                TLS_SERVER_CERTIFICATE_FILE.into(),
                tls_server_certificate.into(),
            );
        }

        if properties.is_empty() {
            Ok(None)
        } else {
            Ok(Some(properties))
        }
    }
}
