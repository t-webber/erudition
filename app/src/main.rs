//! An android app.
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
#![expect(clippy::pub_use, reason = "needed by dioxus")]
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
    clippy::integer_division_remainder_used,
    reason = "bad lints"
)]

/// Page that displays the questions.
mod questions;
/// Handles the routes and displays the page corresponding to the current route.
mod router;

use dioxus::core::Element;
use dioxus::dioxus_core;
use dioxus::prelude::*;

use crate::router::Route;

/// Predefined Tailwind CSS classes and styles.
#[expect(clippy::volatile_composites, reason = "foreign macro")]
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        main {
            class: "bg-black h-dvh text-gray-200",

            Router::<Route> {}
        }

    }
}
