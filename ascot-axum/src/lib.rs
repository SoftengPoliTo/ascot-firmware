extern crate alloc;

pub mod devices;

pub mod device;
pub mod error;
pub mod server;
pub mod service;

pub mod extract {
    pub use axum::extract::{Extension, Json, Path};
}

mod output_type;
mod services;

// Maximum stack elements.
const MAXIMUM_ELEMENTS: usize = 16;
