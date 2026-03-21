//! Server
#![deny(
    missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
//     clippy::cargo
)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::pattern_type_mismatch,
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_trait_methods,
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::pub_with_shorthand,
    clippy::unseparated_literal_suffix,
    clippy::else_if_without_else,
    clippy::doc_paragraphs_missing_punctuation,
    reason = "bad lint"
)]

/// Server routes and their handler
mod routes;
/// Server cli and runner
mod server;
/// Server state, shared across route handlers
mod state;
/// State that is stored to the file system to be persistent after the server is
/// restarted
mod storage;
#[cfg(test)]
mod tests;

use clap::Parser as _;

use crate::server::Server;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    Server::parse().run()
}
