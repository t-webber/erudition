use dioxus::core::Element;
use dioxus::dioxus_core;
use dioxus::prelude::{component, rsx};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! { "Hello, world!" }
}
