//! An android app.
#![allow(
    clippy::pub_use,
    clippy::wildcard_imports,
    reason = "needed by dioxus"
)]

/// Page that displays the / page.
mod home;
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
