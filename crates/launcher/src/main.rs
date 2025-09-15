use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use bevy::prelude::{App, Plugin};

#[cfg(target_family = "wasm")]
mod wasm;

#[cfg(not(target_family = "wasm"))]
mod native;

fn main() {
    #[cfg(target_family = "wasm")]
    wasm::run();

    #[cfg(not(target_family = "wasm"))]
    native::run();
}
