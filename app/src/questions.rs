use dioxus::core::Element;
use dioxus::hooks::{use_resource, use_signal};
use dioxus::prelude::*;
use erudition_lib::Item;

/// Cross-platform HOST ip.
const HOST: &str =
    if cfg!(target_os = "android") { "10.0.2.2" } else { "127.0.0.1" };

#[component]
pub fn Questions() -> Element {
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
            class: "bg-blue-600 p-1 m-1",
            onclick: move |_| *refetch.write()= true,
            "refetch"
        }
        div {
            class: "bg-red-600 p-2 w-fit",
            p { "{question}" }
        }

    }
}

/// Fetches the items from the server.
async fn fetch_items() -> Option<Vec<Item>> {
    reqwest::get(format!("http://{HOST}:3000/items"))
        .await
        .ok()?
        .json()
        .await
        .ok()
}
