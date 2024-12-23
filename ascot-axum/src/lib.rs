//! A library to develop firmware which might be run on an any operating system.
//!
//! This crate has been conceived for those devices which require a great amount
//! of resources for their execution, in terms of computing times, memory size,
//! and connected components.
//!
//! As of now, only `x86_64` and `ARM` devices have been tested as hardware
//! architectures.
//!
//! `ascot-axum` models devices firmware on real use cases. Each device is
//! identified by a description and a set of actions, each with their own
//! hazards associated.
//!
//! A description is defined as a fixed structure subdivided in fields, such as
//! the device name, kind, and other information about the security of
//! the communication channel. However, the set of actions are the ones which
//! mainly characterize a device.
//!
//! An **action** represents the sequence of operations which are going to be
//! executed on a device whenever an external device invokes a determined
//! server route. Each action is **always** coupled with a route and it can
//! have zero or more hazards.
//!
//! If no hazards have been defined for an action, that action might have
//! unknown dangers.
//!
//! This crate is not able to check at compile time all possible effects which
//! might occur in a determined environment during the execution of an action.
//! Indeed, hazards represent only informative data usable by another device,
//! such as a controller, to block or allow the invocation of an action
//! depending on some privacy policies.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// All device kinds implementable in a firmware.
pub mod devices;

/// All action kinds with their payloads.
pub mod actions;
/// Methods to define a device and its actions.
pub mod device;
/// Error handling.
pub mod error;
/// Methods to define and run the server which represents the firwmare.
pub mod server;
/// Methods to define and run the discovery service necessary to detect a
/// device in a network.
pub mod service;

/// Methods to parse route inputs.
pub mod extract {
    pub use axum::extract::{FromRef, Json, Path, State};
}

pub use axum::http::header;

mod services;
