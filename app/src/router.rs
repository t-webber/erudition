use dioxus::prelude::*;
use dioxus::router::Routable;

use crate::questions::Questions;

/// Different routes and the components that need to be rendered.
#[derive(Routable, Clone, PartialEq, Eq)]
pub enum Route {
    /// Page to display the question.
    #[route("/")]
    Questions,
}
