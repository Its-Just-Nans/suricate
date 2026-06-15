//! Suricate is a gedcom viewer made with [bladvak](https://github.com/Its-Just-Nans/bladvak) (egui)
//!
//! ```sh
//! cargo install suricate --locked
//!
//! suricate path/to/file.ged
//! ```

#![warn(clippy::all, rust_2018_idioms)]
#![deny(
    missing_docs,
    clippy::all,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::pedantic,
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf
)]
#![allow(clippy::multiple_crate_versions)]

mod app;
mod central_panel;
mod panels;
mod windows;

pub use app::SuricateApp;
