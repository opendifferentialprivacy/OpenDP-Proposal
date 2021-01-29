//! FFI bindings for OpenDP.
//!
//! This crate contains FFI bindings for [OpenDP](opendp).

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod dispatch;

mod core;
mod data;
mod meas;
mod trans;
mod util;
