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
use futures::prelude::*;
use std::convert::TryFrom;
use std::time::Instant;
use janu::*;

const N: u64 = 100000;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let config = parse_args();

    println!("New janu...");
    let janu = Janu::new(config.into()).await.unwrap();

    println!("New workspace...");
    let workspace = janu.workspace(None).await.unwrap();

    let selector = Selector::try_from("/test/thr").unwrap();

    let mut count = 0u64;
    let mut start = Instant::now();

    let subscriber = workspace
        .subscribe_with_callback(&selector, move |_change| {
            if count == 0 {
                start = Instant::now();
                count += 1;
            } else if count < N {
                count += 1;
            } else {
                print_stats(start);
                count = 0;
            }
        })
        .await
        .unwrap();

    // Stop forever
    future::pending::<()>().await;

    subscriber.close().await.unwrap();
    janu.close().await.unwrap();
}

fn print_stats(start: Instant) {
    let elapsed = start.elapsed().as_secs_f64();
    let thpt = (N as f64) / elapsed;
    println!("{} msg/s", thpt);
}

fn parse_args() -> Properties {
    let args = App::new("janu throughput sub example")
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
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
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
