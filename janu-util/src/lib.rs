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
#[macro_use]
extern crate lazy_static;
use std::path::{Path, PathBuf};
pub mod collections;
pub mod core;
pub mod crypto;
pub mod ffi;
mod lib_loader;
pub mod net;
pub mod properties;
pub mod sync;
pub use crate::core::macros::*;
pub use lib_loader::*;

/// the "JANU_HOME" environement variable name
pub const JANU_HOME_ENV_VAR: &str = "JANU_HOME";

const DEFAULT_JANU_HOME_DIRNAME: &str = ".janu";

/// Return the path to the ${JANU_HOME} directory (~/.janu by default).
pub fn janu_home() -> &'static Path {
    lazy_static! {
        static ref ROOT: PathBuf = {
            if let Some(dir) = std::env::var_os(JANU_HOME_ENV_VAR) {
                PathBuf::from(dir)
            } else {
                match home::home_dir() {
                    Some(mut dir) => {
                        dir.push(DEFAULT_JANU_HOME_DIRNAME);
                        dir
                    }
                    None => PathBuf::from(DEFAULT_JANU_HOME_DIRNAME),
                }
            }
        };
    }
    ROOT.as_path()
}
