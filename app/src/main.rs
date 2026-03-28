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

use dioxus::core::Element;
use dioxus::dioxus_core;
use dioxus::hooks::use_signal;
use dioxus::prelude::*;
use erudition_lib::Item;

fn main() {
    dioxus::launch(App);
}

/// Fetches the items from the server.
async fn fetch_items() -> Option<Vec<Item>> {
    reqwest::get("http://10.0.2.2:3000/items").await.ok()?.json().await.ok()
}

#[component]
fn App() -> Element {
    let default_item =
        Item::MultipleChoice { answers: vec![], question: "a question".into() };

    let mut refetch = use_signal(|| true);
    let items = use_resource(move || async move {
        refetch();
        fetch_items().await
    });
    let mut index = use_signal(|| 0usize);

    let question = items
        .read()
        .clone()
        .unwrap_or_default()
        .unwrap_or_default()
        .get(index())
        .unwrap_or(&default_item)
        .clone()
        .question();
    let len =
        items.read().clone().unwrap_or_default().unwrap_or_default().len();

    rsx! {
        button {
            onclick: move |_| if len != 0 {*index.write() = (index() + 1usize) % len},
            "next"
        }
        button {
            onclick: move |_| *refetch.write()= true,
            "refetch"
        }
        div {
            class: "bg-black",
            "{question}"

        }
    }
}
