#![warn(clippy::cargo,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
    clippy::todo,
    // clippy::pedantic,
    // clippy::unwrap_used,
    // clippy::expect_used,
)]
#![allow(
    clippy::cargo_common_metadata,
    // clippy::cast_sign_loss,
    // clippy::cast_possible_truncation,
    // clippy::cast_precision_loss,
    // clippy::cast_possible_wrap,
    clippy::missing_panics_doc,
    clippy::multiple_crate_versions,
    clippy::missing_const_for_fn,
    clippy::must_use_candidate
)]
#![warn(missing_docs)]
#![feature(string_remove_matches)]
#![feature(stmt_expr_attributes)]
#![doc = ::embed_doc_image::embed_image!("layoutmodel", "LayoutModel.png")]
#![doc = include_str!("../README.md")]

// #[cfg(all(feature = "multi_thread", feature = "single_thread"))]
// compile_error!("Cant activate both treading options");

use std::path::Path;
use std::time::Instant;

use crate::model::{Model, ModelParameters};
use crate::util::{init, ModelError};

pub mod agent;
pub mod attack;
pub mod grid;
pub mod model;
mod norayon;
pub mod util;

/// Run using [`ModelParameters`] from a configuration file
pub fn run_from_config(configfile: impl AsRef<Path>) -> Result<(), ModelError> {
    let param = ModelParameters::from_config(configfile)?;
    run_model(param)?;
    Ok(())
}

/// Run the model providing the [`ModelParameters`]
pub fn run_model(param: ModelParameters) -> Result<(), ModelError> {
    let start_time = Instant::now();
    init(&param).expect("Failed to init the logger");
    let mut model = Model::new(param)?;
    let res = model.run();
    log::info!("Running took {:?}", start_time.elapsed());
    println!("Running took {:?}", start_time.elapsed());
    fast_log::flush()?;
    fast_log::exit()?;
    res
}
