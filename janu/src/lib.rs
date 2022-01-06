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

//! The crate of the janu API.
//!
//! See the [Janu] struct for details.
//!
//! # Quick start examples
//!
//! ### Put a key/value into janu
//! ```
//! use janu::*;
//! use std::convert::TryInto;
//!
//! #[async_std::main]
//! async fn main() {
//!     let janu = Janu::new(net::config::default()).await.unwrap();
//!     let workspace = janu.workspace(None).await.unwrap();
//!     workspace.put(
//!         &"/demo/example/hello".try_into().unwrap(),
//!         "Hello World!".into()
//!     ).await.unwrap();
//!     janu.close().await.unwrap();
//! }
//! ```
//!
//! ### Subscribe for keys/values changes from janu
//! ```no_run
//! use janu::*;
//! use futures::prelude::*;
//! use std::convert::TryInto;
//!
//! #[async_std::main]
//! async fn main() {
//!     let janu = Janu::new(net::config::default()).await.unwrap();
//!     let workspace = janu.workspace(None).await.unwrap();
//!     let mut change_stream =
//!         workspace.subscribe(&"/demo/example/**".try_into().unwrap()).await.unwrap();
//!     while let Some(change) = change_stream.next().await {
//!         println!(">> {:?} for {} : {:?} at {}",
//!             change.kind, change.path, change.value, change.timestamp
//!         )
//!     }
//!     change_stream.close().await.unwrap();
//!     janu.close().await.unwrap();
//! }
//! ```
//!
//! ### Get keys/values from janu
//! ```no_run
//! use janu::*;
//! use futures::prelude::*;
//! use std::convert::TryInto;
//!
//! #[async_std::main]
//! async fn main() {
//!     let janu = Janu::new(net::config::default()).await.unwrap();
//!     let workspace = janu.workspace(None).await.unwrap();
//!     let mut data_stream = workspace.get(&"/demo/example/**".try_into().unwrap()).await.unwrap();
//!     while let Some(data) = data_stream.next().await {
//!         println!(">> {} : {:?} at {}",
//!             data.path, data.value, data.timestamp
//!         )
//!     }
//!     janu.close().await.unwrap();
//! }
//! ```
#![doc(
    html_logo_url = "http://janu.io/img/janu-dragon.png",
    html_favicon_url = "http://janu.io/favicon-32x32.png",
    html_root_url = "https://eclipse-janu.github.io/janu/janu/"
)]
#![recursion_limit = "256"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate janu_util;

extern crate async_std;
extern crate uuid;

use log::debug;

pub mod net;

use net::info::ZN_INFO_ROUTER_PID_KEY;
use net::runtime::Runtime;
use net::Session;
pub use net::{zready, ZError, ZErrorKind, ZFuture, ZPinBoxFuture, ZReady, ZResult};

mod workspace;
pub use workspace::*;

mod path;
pub use path::{path, Path};
mod pathexpr;
pub use pathexpr::{pathexpr, PathExpr};
mod selector;
pub use selector::{selector, Selector};
mod values;
pub use values::*;

// pub mod config;
pub mod utils;

pub use net::protocol::core::{Timestamp, TimestampId};
pub use janu_util::properties::config::ConfigProperties;
pub use janu_util::properties::Properties;
pub use janu_util::sync::zpinbox;

/// The janu client API.
pub struct Janu {
    session: Session,
}

impl Janu {
    /// Creates a janu API, establishing a janu-net session with discovered peers and/or routers.
    ///
    /// # Arguments
    ///
    /// * `config` - The [ConfigProperties](net::config::ConfigProperties) for the janu session
    ///
    /// # Examples
    /// ```
    /// # async_std::task::block_on(async {
    /// use janu::*;
    ///
    /// let janu = Janu::new(net::config::default()).await.unwrap();
    /// # })
    /// ```
    ///
    /// # Configuration Properties
    ///
    /// [ConfigProperties](net::config::ConfigProperties) are a set of key/value (`u64`/`String`) pairs.
    /// Constants for the accepted keys can be found in the [config](net::config) module.
    /// Multiple values are coma separated.
    ///
    /// # Examples
    /// ```
    /// # async_std::task::block_on(async {
    /// use janu::*;
    ///
    /// let mut config = net::config::peer();
    /// config.insert(net::config::ZN_LOCAL_ROUTING_KEY, "false".to_string());
    /// config.insert(net::config::ZN_PEER_KEY, "tcp/10.10.10.10:7447,tcp/11.11.11.11:7447".to_string());
    ///
    /// let janu = Janu::new(config).await.unwrap();
    /// # })
    /// ```
    ///
    /// [ConfigProperties](net::config::ConfigProperties) can be built set of key/value (`String`/`String`) set
    /// of [Properties](Properties).
    ///
    /// # Examples
    /// ```
    /// # async_std::task::block_on(async {
    /// use janu::*;
    ///
    /// let mut config = Properties::default();
    /// config.insert("local_routing".to_string(), "false".to_string());
    /// config.insert("peer".to_string(), "tcp/10.10.10.10:7447,tcp/11.11.11.11:7447".to_string());
    ///
    /// let janu = Janu::new(config.into()).await.unwrap();
    /// # })
    /// ```
    pub fn new(config: ConfigProperties) -> impl ZFuture<Output = ZResult<Janu>> {
        zpinbox(async {
            Ok(Janu {
                session: net::open(config).await?,
            })
        })
    }

    /// Creates a Janu API with an existing Runtime.
    /// This operation is used by the plugins to share the same Runtime than the router.
    #[doc(hidden)]
    pub fn init(runtime: Runtime) -> impl ZFuture<Output = Janu> {
        zpinbox(async {
            Janu {
                session: Session::init(runtime, true, vec![], vec![]).await,
            }
        })
    }

    /// Returns the janu-net [Session](net::Session) used by this janu session.
    /// This is for advanced use cases requiring fine usage of the janu-net API.
    #[inline(always)]
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Returns the PeerId of the janu router this janu API is connected to (if any).
    /// This calls [Session::info()](net::Session::info) and returns the first router pid from
    /// the ZN_INFO_ROUTER_PID_KEY property.
    pub fn router_pid(&self) -> impl ZFuture<Output = Option<String>> {
        zready(
            match self.session().info().wait().remove(&ZN_INFO_ROUTER_PID_KEY) {
                None => None,
                Some(s) if s.is_empty() => None,
                Some(s) if !s.contains(',') => Some(s),
                Some(s) => Some(s.split(',').next().unwrap().to_string()),
            },
        )
    }

    /// Creates a [`Workspace`] with an optional [`Path`] as `prefix`.
    /// All relative [`Path`] or [`Selector`] used with this Workspace will be relative to the
    /// specified prefix. Not specifying a prefix is equivalent to specifying "/" as prefix,
    /// meaning in this case that all relative paths/selectors will be prependend with "/".
    ///
    /// # Examples
    /// ```
    /// # async_std::task::block_on(async {
    /// use janu::*;
    /// use std::convert::TryInto;
    ///
    /// let janu = Janu::new(net::config::default()).await.unwrap();
    /// let workspace = janu.workspace(Some("/demo/example".try_into().unwrap())).await.unwrap();
    /// // The following it equivalent to a PUT on "/demo/example/hello".
    /// workspace.put(
    ///     &"hello".try_into().unwrap(),
    ///     "Hello World!".into()
    /// ).await.unwrap();
    /// # })
    /// ```
    pub fn workspace(&self, prefix: Option<Path>) -> impl ZFuture<Output = ZResult<Workspace<'_>>> {
        debug!("New workspace with prefix: {:?}", prefix);
        Workspace::new(self, prefix)
    }

    /// Closes the janu API and the associated janu-net session.
    ///
    /// Note that on drop, the janu-net session is also automatically closed.
    /// But you may want to use this function to handle errors or
    /// close the session synchronously.
    ///
    /// # Examples
    /// ```
    /// # async_std::task::block_on(async {
    /// use janu::*;
    ///
    /// let janu = Janu::new(net::config::default()).await.unwrap();
    /// janu.close();
    /// # })
    /// ```
    pub fn close(self) -> impl ZFuture<Output = ZResult<()>> {
        self.session.close()
    }
}

impl From<Session> for Janu {
    fn from(session: Session) -> Self {
        Janu { session }
    }
}

impl From<&Session> for Janu {
    fn from(s: &Session) -> Self {
        Janu { session: s.clone() }
    }
}
