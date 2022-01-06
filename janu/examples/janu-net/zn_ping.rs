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

    let (config, size, n) = parse_args();
    let session: Session = open(config.into()).wait().unwrap();

    // The resource to publish data on
    let reskey_ping = RId(session
        .declare_resource(&RName("/test/ping".to_string()))
        .wait()
        .unwrap());

    // The resource to wait the response back
    let reskey_pong = RId(session
        .declare_resource(&RName("/test/pong".to_string()))
        .wait()
        .unwrap());

    let sub_info = SubInfo {
        reliability: Reliability::Reliable,
        mode: SubMode::Push,
        period: None,
    };

    let mut sub: Subscriber = session
        .declare_subscriber(&reskey_pong, &sub_info)
        .wait()
        .unwrap();

    let data: ZBuf = (0usize..size)
        .map(|i| (i % 10) as u8)
        .collect::<Vec<u8>>()
        .into();

    let mut samples = Vec::with_capacity(n);

    // -- warmup --
    let wun = 1000;
    let stream = sub.receiver();
    for _ in 0..wun {
        let data = data.clone();
        session
            .write_ext(
                &reskey_ping,
                data,
                encoding::DEFAULT,
                data_kind::DEFAULT,
                CongestionControl::Block, // Make sure to not drop messages because of congestion control
            )
            .wait()
            .unwrap();

        let _ = stream.recv();
    }

    for _ in 0..n {
        let data = data.clone();
        let write_time = Instant::now();
        session
            .write_ext(
                &reskey_ping,
                data,
                encoding::DEFAULT,
                data_kind::DEFAULT,
                CongestionControl::Block, // Make sure to not drop messages because of congestion control
            )
            .wait()
            .unwrap();

        let _ = stream.recv();
        let ts = write_time.elapsed().as_micros();
        samples.push(ts);
    }

    for (i, rtt) in samples.iter().enumerate().take(n) {
        println!("{} bytes: seq={} time={:?}µs", size, i, rtt);
    }
}

fn parse_args() -> (Properties, usize, usize) {
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
            Arg::from_usage("-n, --samples=[N]          'The number of round-trips to measure'")
                .default_value("100"),
        )
        .arg(Arg::from_usage(
            "--no-multicast-scouting 'Disable the multicast-based scouting mechanism.'",
        ))
        .arg(Arg::from_usage(
            "<PAYLOAD_SIZE>          'Sets the size of the payload to publish'",
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
    let n: usize = args.value_of("samples").unwrap().parse().unwrap();
    let size: usize = args.value_of("PAYLOAD_SIZE").unwrap().parse().unwrap();

    (config, size, n)
}
