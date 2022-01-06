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
use std::convert::{TryFrom, TryInto};
use janu::*;

#[async_std::main]
async fn main() {
    // initiate logging
    env_logger::init();

    let (config, path, value) = parse_args();

    println!("New janu...");
    let janu = Janu::new(config.into()).await.unwrap();

    println!("New workspace...");
    let workspace = janu.workspace(None).await.unwrap();

    println!("Put Data ('{}': '{}')...\n", path, value);
    workspace
        .put(&path.try_into().unwrap(), value.into())
        .await
        .unwrap();

    // --- Examples of put with other types:

    // - Integer
    // workspace.put(&"/demo/example/Integer".try_into().unwrap(), 3.into())
    //     .await.unwrap();

    // - Float
    // workspace.put(&"/demo/example/Float".try_into().unwrap(), 3.14.into())
    //     .await.unwrap();

    // - Properties (as a Dictionary with str only)
    // workspace.put(
    //         &"/demo/example/Properties".try_into().unwrap(),
    //         Properties::from("p1=v1;p2=v2").into()
    //     ).await.unwrap();

    // - Json (str format)
    // workspace.put(
    //         &"/demo/example/Json".try_into().unwrap(),
    //         Value::Json(r#"{"kind"="memory"}"#.to_string()),
    //     ).await.unwrap();

    // - Raw ('application/octet-stream' encoding by default)
    // workspace.put(
    //         &"/demo/example/Raw".try_into().unwrap(),
    //         vec![0x48u8, 0x69, 0x33].into(),
    //     ).await.unwrap();

    // - Custom
    // workspace.put(
    //         &"/demo/example/Custom".try_into().unwrap(),
    //         Value::Custom {
    //             encoding_descr: "my_encoding".to_string(),
    //             data: vec![0x48u8, 0x69, 0x33].into(),
    //     }).await.unwrap();

    janu.close().await.unwrap();
}

fn parse_args() -> (Properties, String, String) {
    let args = App::new("janu put example")
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
        .arg(
            Arg::from_usage("-p, --path=[PATH]        'The name of the resource to put.'")
                .default_value("/demo/example/janu-rs-put"),
        )
        .arg(
            Arg::from_usage("-v, --value=[VALUE]      'The value of the resource to put.'")
                .default_value("Put from Rust!"),
        )
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

    let path = args.value_of("path").unwrap().to_string();
    let value = args.value_of("value").unwrap().to_string();

    (config, path, value)
}
