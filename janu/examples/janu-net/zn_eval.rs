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
use futures::select;
use janu::net::queryable::EVAL;
use janu::net::*;
use janu::Properties;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let (config, path, value) = parse_args();

    println!("Opening session...");
    let session = open(config.into()).await.unwrap();

    println!("Declaring Queryable on {}", path);
    let mut queryable = session
        .declare_queryable(&path.clone().into(), EVAL)
        .await
        .unwrap();

    let mut stdin = async_std::io::stdin();
    let mut input = [0u8];
    loop {
        select!(
            query = queryable.receiver().next().fuse() => {
                let query = query.unwrap();
                println!(">> [Query handler] Handling '{}{}'", query.res_name, query.predicate);
                query.reply(Sample{
                    res_name: path.clone(),
                    payload: value.as_bytes().into(),
                    data_info: None,
                });
            },

            _ = stdin.read_exact(&mut input).fuse() => {
                if input[0] == b'q' {break}
            }
        );
    }
}

fn parse_args() -> (Properties, String, String) {
    let args = App::new("janu-net eval example")
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
        .arg(
            Arg::from_usage("-p, --path=[PATH]        'The name of the resource to evaluate.'")
                .default_value("/demo/example/janu-rs-eval"),
        )
        .arg(
            Arg::from_usage("-v, --value=[VALUE]      'The value to reply to queries.'")
                .default_value("Eval from Rust!"),
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

    let path = args.value_of("path").unwrap().to_string();
    let value = args.value_of("value").unwrap().to_string();

    (config, path, value)
}
