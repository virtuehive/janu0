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
extern crate criterion;

use criterion::Criterion;
use janu::net::protocol::core::{Channel, CongestionControl, Priority, Reliability, ResKey};
use janu::net::protocol::io::{WBuf, ZBuf};
use janu::net::protocol::proto::defaults::BATCH_SIZE;
use janu::net::protocol::proto::JanuMessage;

fn criterion_benchmark(c: &mut Criterion) {
    let batch_size = BATCH_SIZE as usize;
    let mut current = 8;
    let mut pld = vec![];
    while current < batch_size {
        pld.push(current);
        current *= 2;
    }

    let res_key_set = [
        ResKey::RId(1),
        ResKey::RName("/frame/bench".to_string()),
        ResKey::RIdWithSuffix(1, "/frame/bench".to_string()),
    ];

    for p in &pld {
        for r in &res_key_set {
            let res_key = r.clone();
            let payload = ZBuf::from(vec![0; *p]);
            let channel = Channel {
                priority: Priority::default(),
                reliability: Reliability::Reliable,
            };
            let congestion_control = CongestionControl::default();
            let info = None;

            let mut msg = JanuMessage::make_data(
                res_key,
                payload,
                channel,
                congestion_control,
                info,
                None,
                None,
                None,
            );

            let mut wbuf = WBuf::new(batch_size, true);
            let mut num = 0;
            while wbuf.write_janu_message(&mut msg) {
                num += 1;
            }
            drop(wbuf);

            c.bench_function(
                format!("frame_creation {} {} {} {}", batch_size, p, num, r).as_str(),
                |b| {
                    let mut wbuf = WBuf::new(batch_size, true);
                    b.iter(|| {
                        wbuf.write_frame_header(
                            channel.priority,
                            channel.reliability,
                            1,
                            None,
                            None,
                        );
                        for _ in 0..num {
                            let res_key = r.clone();
                            let payload = ZBuf::from(vec![0; *p]);
                            let channel = Channel {
                                priority: Priority::default(),
                                reliability: Reliability::Reliable,
                            };
                            let congestion_control = CongestionControl::default();
                            let info = None;

                            let mut msg = JanuMessage::make_data(
                                res_key,
                                payload,
                                channel,
                                congestion_control,
                                info,
                                None,
                                None,
                                None,
                            );
                            wbuf.write_janu_message(&mut msg);
                            drop(msg);
                        }
                    })
                },
            );

            c.bench_function(
                format!(
                    "frame_encoding_yes_contiguous {} {} {} {}",
                    batch_size, p, num, r
                )
                .as_str(),
                |b| {
                    let mut wbuf = WBuf::new(batch_size, true);
                    b.iter(|| {
                        wbuf.write_frame_header(
                            channel.priority,
                            channel.reliability,
                            1,
                            None,
                            None,
                        );
                        for _ in 0..num {
                            wbuf.write_janu_message(&mut msg);
                        }
                    })
                },
            );

            c.bench_function(
                format!(
                    "frame_encoding_no_contiguous {} {} {} {}",
                    batch_size, p, num, r
                )
                .as_str(),
                |b| {
                    let mut wbuf = WBuf::new(*p, false);
                    b.iter(|| {
                        wbuf.write_frame_header(
                            channel.priority,
                            channel.reliability,
                            1,
                            None,
                            None,
                        );
                        for _ in 0..num {
                            wbuf.write_janu_message(&mut msg);
                        }
                    })
                },
            );

            c.bench_function(
                format!(
                    "frame_decoding_yes_contiguous {} {} {} {}",
                    batch_size, p, num, r
                )
                .as_str(),
                |b| {
                    let mut wbuf = WBuf::new(batch_size, true);
                    wbuf.write_frame_header(channel.priority, channel.reliability, 1, None, None);

                    for _ in 0..num {
                        wbuf.write_janu_message(&mut msg);
                    }

                    let mut zbuf = ZBuf::from(&wbuf);
                    b.iter(|| {
                        zbuf.reset();
                        let _ = zbuf.read_transport_message().unwrap();
                    })
                },
            );

            c.bench_function(
                format!(
                    "frame_decoding_no_contiguous {} {} {} {}",
                    batch_size, p, num, r
                )
                .as_str(),
                |b| {
                    let mut wbuf = WBuf::new(*p, false);
                    wbuf.write_frame_header(channel.priority, channel.reliability, 1, None, None);

                    for _ in 0..num {
                        wbuf.write_janu_message(&mut msg);
                    }

                    let mut zbuf = ZBuf::from(&wbuf);
                    b.iter(|| {
                        zbuf.reset();
                        let _ = zbuf.read_transport_message().unwrap();
                    })
                },
            );
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
