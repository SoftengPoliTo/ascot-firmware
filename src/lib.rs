#![no_std]

// REMINDERS:
//
// - The `heapless` dependency can consume a lot of stack. Reduce the number
//   of elements in the stack structures if some issues arises.

// Maximum number of elements on stack for a data structure.
const MAXIMUM_ELEMENTS: usize = 8;

pub mod actions;
pub mod device;
pub mod economy;
pub mod energy;
pub mod error;
pub mod hazards;
pub mod input;
pub mod payloads;
pub mod route;

mod utils;

pub use error::{Error, ErrorKind};
pub use utils::collections;
pub use utils::strings;
