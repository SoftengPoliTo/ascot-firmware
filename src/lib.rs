//! The `Ascot` crate provides a series of interfaces to interact with
//! different IoT devices firmwares.
//! Every interface provides a series of methods to manage the possible errors
//! which might occur in setting up and configure firmwares.
//!
//! The crate also provides:
//! - A way to build the `mDNS` service so that the firmware can be discovered
//! in the network by other devices
//! - A way to build the various REST APIs exposed by the firmware

pub mod action;
pub mod device;
pub mod server;
pub mod service;
