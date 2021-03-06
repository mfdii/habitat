// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate habitat_butterfly;

use std::env;
use std::thread;
use std::time::Duration;
use std::net::SocketAddr;

use habitat_butterfly::{server, member, trace};

fn main() {
    env_logger::init().unwrap();
    let mut args = env::args();
    let _ = args.next();

    let bind_to = args.next().unwrap();
    println!("Binding to {}", bind_to);

    let bind_to_addr = bind_to.parse::<SocketAddr>().unwrap();
    let bind_port = bind_to_addr.port();
    let mut gossip_bind_addr = bind_to_addr.clone();
    let gport = bind_port + 1;
    gossip_bind_addr.set_port(gport);

    let mut member = member::Member::new();
    member.set_swim_port(bind_port as i32);
    member.set_gossip_port(gport as i32);

    let server = server::Server::new(bind_to_addr,
                                     gossip_bind_addr,
                                     member,
                                     trace::Trace::default(),
                                     None,
                                     None)
        .unwrap();
    println!("Server ID: {}", server.member_id);

    let targets: Vec<String> = args.collect();
    for target in &targets {
        let addr: SocketAddr = target.parse().unwrap();
        let mut member = member::Member::new();
        member.set_address(format!("{}", addr.ip()));
        member.set_swim_port(addr.port() as i32);
        member.set_gossip_port(addr.port() as i32);
        server.member_list.add_initial_member(member);
    }

    server.start(server::timing::Timing::default()).expect("Cannot start server");
    loop {
        println!("{:#?}", server.member_list);
        thread::sleep(Duration::from_millis(1000));
    }
}
