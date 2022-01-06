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
pub mod authenticator;
pub(crate) mod establishment;
pub(crate) mod link;
pub(crate) mod manager;
pub(crate) mod rx;
pub(crate) mod transport;
pub(crate) mod tx;

use super::common;
use super::protocol;
use super::protocol::core::{PeerId, WhatAmI, ZInt};
use super::protocol::proto::{tmsg, JanuMessage};
use super::{TransportPeer, TransportPeerEventHandler};
use crate::net::link::Link;
pub use manager::*;
use std::fmt;
use std::sync::{Arc, Weak};
use transport::TransportUnicastInner;
use janu_util::core::{ZError, ZErrorKind, ZResult};
use janu_util::zerror2;

/*************************************/
/*        TRANSPORT UNICAST          */
/*************************************/
#[cfg(feature = "stats")]
#[derive(Clone, Copy, Debug)]
pub struct TransportStatsUnicast {
    tx_msgs: usize,
    tx_bytes: usize,
    rx_msgs: usize,
    rx_bytes: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct TransportConfigUnicast {
    pub(crate) peer: PeerId,
    pub(crate) whatami: WhatAmI,
    pub(crate) sn_resolution: ZInt,
    pub(crate) initial_sn_tx: ZInt,
    pub(crate) initial_sn_rx: ZInt,
    pub(crate) is_shm: bool,
    pub(crate) is_qos: bool,
}

/// [`TransportUnicast`] is the transport handler returned
/// when opening a new unicast transport
#[derive(Clone)]
pub struct TransportUnicast(Weak<TransportUnicastInner>);

impl TransportUnicast {
    #[inline(always)]
    pub(super) fn get_transport(&self) -> ZResult<Arc<TransportUnicastInner>> {
        self.0.upgrade().ok_or_else(|| {
            zerror2!(ZErrorKind::InvalidReference {
                descr: "Transport unicast closed".to_string()
            })
        })
    }

    #[inline(always)]
    pub fn get_pid(&self) -> ZResult<PeerId> {
        let transport = self.get_transport()?;
        Ok(transport.get_pid())
    }

    #[inline(always)]
    pub fn get_whatami(&self) -> ZResult<WhatAmI> {
        let transport = self.get_transport()?;
        Ok(transport.get_whatami())
    }

    #[inline(always)]
    pub fn get_sn_resolution(&self) -> ZResult<ZInt> {
        let transport = self.get_transport()?;
        Ok(transport.get_sn_resolution())
    }

    #[inline(always)]
    pub fn is_shm(&self) -> ZResult<bool> {
        let transport = self.get_transport()?;
        Ok(transport.is_shm())
    }

    #[inline(always)]
    pub fn is_qos(&self) -> ZResult<bool> {
        let transport = self.get_transport()?;
        Ok(transport.is_qos())
    }

    #[inline(always)]
    pub fn get_callback(&self) -> ZResult<Option<Arc<dyn TransportPeerEventHandler>>> {
        let transport = self.get_transport()?;
        Ok(transport.get_callback())
    }

    pub fn get_peer(&self) -> ZResult<TransportPeer> {
        let transport = self.get_transport()?;
        let tp = TransportPeer {
            pid: transport.get_pid(),
            whatami: transport.get_whatami(),
            is_qos: transport.is_qos(),
            is_shm: transport.is_shm(),
            links: transport
                .get_links()
                .into_iter()
                .map(|l| l.into())
                .collect(),
        };
        Ok(tp)
    }

    #[inline(always)]
    pub fn get_links(&self) -> ZResult<Vec<Link>> {
        let transport = self.get_transport()?;
        Ok(transport
            .get_links()
            .into_iter()
            .map(|l| l.into())
            .collect())
    }

    #[inline(always)]
    pub fn schedule(&self, message: JanuMessage) -> ZResult<()> {
        let transport = self.get_transport()?;
        transport.schedule(message);
        Ok(())
    }

    #[inline(always)]
    pub async fn close_link(&self, link: &Link) -> ZResult<()> {
        let transport = self.get_transport()?;
        let link = transport
            .get_links()
            .into_iter()
            .find(|l| l.get_src() == link.src && l.get_dst() == link.dst)
            .ok_or_else(|| {
                zerror2!(ZErrorKind::InvalidLink {
                    descr: "Invalid link".to_string()
                })
            })?;
        transport
            .close_link(&link, tmsg::close_reason::GENERIC)
            .await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn close(&self) -> ZResult<()> {
        // Return Ok if the transport has already been closed
        match self.get_transport() {
            Ok(transport) => transport.close(tmsg::close_reason::GENERIC).await,
            Err(_) => Ok(()),
        }
    }

    #[inline(always)]
    pub fn handle_message(&self, message: JanuMessage) -> ZResult<()> {
        self.schedule(message)
    }

    #[cfg(feature = "stats")]
    pub fn get_stats(&self) -> ZResult<TransportStatsUnicast> {
        let transport = self.get_transport()?;
        let stats = TransportStatsUnicast {
            tx_msgs: transport.stats.get_tx_msgs(),
            tx_bytes: transport.stats.get_tx_bytes(),
            rx_msgs: transport.stats.get_rx_msgs(),
            rx_bytes: transport.stats.get_rx_bytes(),
        };
        Ok(stats)
    }
}

impl From<&Arc<TransportUnicastInner>> for TransportUnicast {
    fn from(s: &Arc<TransportUnicastInner>) -> TransportUnicast {
        TransportUnicast(Arc::downgrade(s))
    }
}

impl Eq for TransportUnicast {}

impl PartialEq for TransportUnicast {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl fmt::Debug for TransportUnicast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.get_transport() {
            Ok(transport) => f
                .debug_struct("Transport Unicast")
                .field("pid", &transport.get_pid())
                .field("whatami", &transport.get_whatami())
                .field("sn_resolution", &transport.get_sn_resolution())
                .field("is_qos", &transport.is_qos())
                .field("is_shm", &transport.is_shm())
                .finish(),
            Err(e) => {
                write!(f, "{}", e)
            }
        }
    }
}
