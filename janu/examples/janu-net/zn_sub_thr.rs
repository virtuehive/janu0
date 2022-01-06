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
use std::time::Instant;
use janu::net::ResKey::*;
use janu::net::*;
use janu::Properties;

fn main() {
    // initiate logging
    env_logger::init();

    let (config, m, n) = parse_args();

    let session = open(config.into()).wait().unwrap();

    let reskey = RId(session
        .declare_resource(&RName("/test/thr".to_string()))
        .wait()
        .unwrap());

    let mut count = 0u128;
    let mut start = Instant::now();

    let sub_info = SubInfo {
        reliability: Reliability::Reliable,
        mode: SubMode::Push,
        period: None,
    };
    let mut nm = 0;
    let _sub = session
        .declare_callback_subscriber(&reskey, &sub_info, move |_sample| {
            if count == 0 {
                start = Instant::now();
                count += 1;
            } else if count < n {
                count += 1;
            } else {
                print_stats(start, n);
                nm += 1;
                count = 0;
                if nm >= m {
                    std::process::exit(0)
                }
            }
        })
        .wait()
        .unwrap();

    // Stop forever
    std::thread::park();
}

fn print_stats(start: Instant, n: u128) {
    let elapsed = start.elapsed().as_secs_f64();
    let thpt = (n as f64) / elapsed;
    println!("{} msg/s", thpt);
}

fn parse_args() -> (Properties, u32, u128) {
    let args = App::new("janu-net throughput sub example")
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
        .arg(
            Arg::from_usage("-s, --samples=[number] 'Number of throughput measurements.'")
                .default_value("10"),
        )
        .arg(
            Arg::from_usage(
                "-n, --number=[number] 'Number of messages in each throughput measurements.'",
            )
            .default_value("100000"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Properties::from(std::fs::read_to_string(conf_file).unwrap())
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

    let samples: u32 = args.value_of("samples").unwrap().parse().unwrap();
    let number: u128 = args.value_of("number").unwrap().parse().unwrap();

    (config, samples, number)
}
