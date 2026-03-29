use dioxus::prelude::*;
use dioxus::router::Routable;

use crate::home::Home;
use crate::questions::Questions;

/// Different routes and the components that need to be rendered.
#[derive(Routable, Clone, PartialEq, Eq)]
pub enum Route {
    /// Landing page.
    #[route("/")]
    Home,
    /// Page to display the question.
    #[route("/questions")]
    Questions,
}
