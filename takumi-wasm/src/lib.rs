//! WebAssembly bindings for Takumi.

#![deny(clippy::unwrap_used, clippy::expect_used)]
#![deny(missing_docs)]
#![allow(
  clippy::module_name_repetitions,
  clippy::missing_errors_doc,
  clippy::missing_panics_doc,
  clippy::must_use_candidate
)]

mod helper;
mod model;
mod renderer;

pub use helper::*;
pub use model::*;
pub use renderer::*;
