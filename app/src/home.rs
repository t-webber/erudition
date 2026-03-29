use dioxus::prelude::*;

use crate::router::Route;

#[component]
pub fn Home() -> Element {
    rsx! {

        p { "Welcome" }
        Link {
            to: Route::Questions,
            class: "p-2 border border-red-500",
            "Go to question"
        }

    }
}
