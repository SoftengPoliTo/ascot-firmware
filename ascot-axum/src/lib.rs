extern crate alloc;

pub mod devices;

pub mod actions;
pub mod device;
pub mod error;
pub mod server;
pub mod service;

pub mod extract {
    pub use axum::extract::{Extension, Json, Path};
}

mod services;
