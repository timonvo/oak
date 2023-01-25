//
// Copyright 2022 The Project Oak Authors
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
//

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::{boxed::Box, sync::Arc};
use core::panic::PanicInfo;
use log::info;
use oak_remote_attestation_amd::PlaceholderAmdAttestationGenerator;
use oak_restricted_kernel_api::{FileDescriptorChannel, StderrLogger};

static LOGGER: StderrLogger = StderrLogger {};

#[no_mangle]
fn _start() -> ! {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
    oak_enclave_runtime_support::init();
    main();
}

fn main() -> ! {
    info!("In main!");
    let service = oak_functions_service::OakFunctionsService::new(Arc::new(
        PlaceholderAmdAttestationGenerator,
    ));
    let server = oak_functions_service::schema::OakFunctionsServer::new(service);
    oak_channel::server::start_blocking_server(Box::<FileDescriptorChannel>::default(), server)
        .expect("server encountered an unrecoverable error");
}

#[alloc_error_handler]
fn out_of_memory(layout: ::core::alloc::Layout) -> ! {
    panic!("error allocating memory: {:#?}", layout);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("PANIC: {}", info);
    oak_restricted_kernel_api::syscall::exit(-1);
}