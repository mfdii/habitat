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

//! Starts a service from an installed Habitat package.
//!
//! Services run by the Supervisor support one or more *topologies*, which are state machines that
//! handle the lifecycle of a service; they are members of a *group*, which is a namespace for
//! their configuration and state.
//!
//! # Examples
//!
//! ```bash
//! $ hab-sup start acme/redis
//! ```
//!
//! Will start the `redis` service in the `default` group, using the `standalone` topology.
//!
//! ```bash
//! $ hab-sup start acme/redis -g production
//! ```
//!
//! Will do the same, but in the `production` group.
//!
//! ```bash
//! $ hab-sup start acme/redis -t leader
//! ```
//!
//! Will start the `redis` service using the `leader` topology.
//!
//! ```bash
//! $ hab-sup start acme/redis -t leader -g production
//! ```
//!
//! Will start the `redis` service using the `leader` topology in the `production` group.
//!
//! See the [documentation on topologies](../topology) for a deeper discussion of how they function.
//!

use std::path::Path;

use ansi_term::Colour::Yellow;
use common::command::package::install;
use common::ui::UI;
use depot_client::Client;
use hcore::fs::{am_i_root, cache_artifact_path, FS_ROOT_PATH};
use hcore::package::{PackageIdent, PackageInstall};

use {PRODUCT, VERSION};
use error::{Error, Result};
use manager::{Manager, ManagerConfig};
use manager::{Service, ServiceSpec, UpdateStrategy};

static LOGKEY: &'static str = "CS";

pub fn package(cfg: ManagerConfig, spec: ServiceSpec, local_artifact: Option<&str>) -> Result<()> {
    let mut ui = UI::default();
    if !am_i_root() {
        try!(ui.warn("Running the Habitat Supervisor requires root or administrator privileges. \
                      Please retry this command as a super user or use a privilege-granting \
                      facility such as sudo."));
        try!(ui.br());
        return Err(sup_error!(Error::RootRequired));
    }

    if let Some(artifact) = local_artifact {
        outputln!("Installing local artifact {}",
                  Yellow.bold().paint(artifact));
        try!(install::start(&mut ui,
                            &spec.depot_url,
                            artifact,
                            PRODUCT,
                            VERSION,
                            Path::new(&*FS_ROOT_PATH),
                            &cache_artifact_path(None),
                            false));
    }

    start_package(cfg, spec)
}

fn start_package(cfg: ManagerConfig, spec: ServiceSpec) -> Result<()> {
    let service = try!(Service::load(spec, &cfg));
    let mut manager = try!(Manager::new(cfg));
    try!(manager.add_service(service));
    manager.run()
}

pub fn install_package(ui: &mut UI,
                       depot_url: &str,
                       ident: &PackageIdent)
                       -> Result<PackageInstall> {
    let fs_root_path = Path::new(&*FS_ROOT_PATH);
    let installed_ident = try!(install::start(ui,
                                              depot_url,
                                              &ident.to_string(),
                                              PRODUCT,
                                              VERSION,
                                              fs_root_path,
                                              &cache_artifact_path(None),
                                              false));
    Ok(try!(PackageInstall::load(&installed_ident, Some(&fs_root_path))))
}

pub fn maybe_install_newer_package(ui: &mut UI,
                                   spec: &ServiceSpec,
                                   current: PackageInstall)
                                   -> Result<PackageInstall> {
    let latest_ident: PackageIdent = {
        let depot_client = try!(Client::new(&spec.depot_url, PRODUCT, VERSION, None));
        try!(depot_client.show_package(&spec.ident)).get_ident().clone().into()
    };

    if &latest_ident > current.ident() {
        outputln!("Newer version of {} detected. Installing {} from {}",
                  spec.ident,
                  latest_ident,
                  spec.depot_url);
        install_package(ui, &spec.depot_url, &latest_ident)
    } else {
        outputln!("Confirmed latest version of {} is {}",
                  spec.ident,
                  current.ident());
        Ok(current)
    }
}
