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
use janu::net::ResKey::*;
use janu::net::*;
use janu::Properties;

fn main() {
    // initiate logging
    env_logger::init();

    let config = parse_args();

    let session = open(config.into()).wait().unwrap();

    // The resource to read the data from
    let reskey_ping = RId(session
        .declare_resource(&RName("/test/ping".to_string()))
        .wait()
        .unwrap());

    // The resource to echo the data back
    let reskey_pong = RId(session
        .declare_resource(&RName("/test/pong".to_string()))
        .wait()
        .unwrap());
    let _publ = session.declare_publisher(&reskey_pong).wait().unwrap();

    let sub_info = SubInfo {
        reliability: Reliability::Reliable,
        mode: SubMode::Push,
        period: None,
    };
    let mut sub = session
        .declare_subscriber(&reskey_ping, &sub_info)
        .wait()
        .unwrap();

    while let Ok(sample) = sub.receiver().recv() {
        session
            .write_ext(
                &reskey_pong,
                sample.payload,
                encoding::DEFAULT,
                data_kind::DEFAULT,
                CongestionControl::Block, // Make sure to not drop messages because of congestion control
            )
            .wait()
            .unwrap();
    }
}

fn parse_args() -> Properties {
    let args = App::new("janu-net delay sub example")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The janu session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --peer=[LOCATOR]...   'Peer locators used to initiate the janu session.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listener=[LOCATOR]...   'Locators to listen on.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .get_matches();

    let mut config = Properties::default();
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
