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
use clap::{App, Arg};
use std::convert::TryFrom;
use janu::net::*;
use janu::Properties;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let config: Properties = parse_args();

    println!("Opening session...");
    let session = open(config.into()).await.unwrap();

    let info: Properties = session.info().await.into();
    for (key, value) in info.iter() {
        println!("{} : {}", key, value);
    }
}

fn parse_args() -> Properties {
    let args = App::new("janu-net info example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE] 'The janu session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...  'Peer locators used to initiate the janu session.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listener=[LOCATOR]...   'Locators to listen on.'",
        ))
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Properties::try_from(std::path::Path::new(conf_file)).unwrap()
    } else {
        Properties::default()
    };
    for key in ["mode", "peer", "listener"].iter() {
        if let Some(value) = args.values_of(key) {
            config.insert(key.to_string(), value.collect::<Vec<&str>>().join(","));
        }
    }
    if args.is_present("no-multicast-scouting") {
        config.insert("multicast_scouting".to_string(), "false".to_string());
    }

    config
}
