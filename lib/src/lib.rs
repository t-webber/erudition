//! A lib to share declarations between server and app.
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
    clippy::missing_inline_in_public_items,
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::pub_with_shorthand,
    clippy::unseparated_literal_suffix,
    clippy::else_if_without_else,
    clippy::doc_paragraphs_missing_punctuation,
    reason = "bad lints"
)]

use core::fmt;

use serde::{Deserialize, Serialize};

/// Defines new types
macro_rules! newtype {
    ($($name:ident),*) => {
        $(
            #[doc = stringify!(concat_idents!(Newtype, for, $name))]
            #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
            #[expect(clippy::exhaustive_structs, reason = "newtype")]
            pub struct $name(pub Box<str>);
        )*
    };
}

/// Authentication request body
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth {
    /// Plain password
    pub password: Plain,
    /// Username
    pub username: Username,
}

impl Auth {
    /// Creates a new [`Auth`] from the given credentials
    #[must_use]
    pub const fn new(username: Box<str>, password: Box<str>) -> Self {
        Self { username: Username(username), password: Plain(password) }
    }
}

/// Item store and returned by the server
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Item {
    /// Multiple choice question
    MultipleChoice {
        /// Possible answers, the first is always the
        /// correct answer
        answers: Vec<Box<str>>,
        /// Question
        question: Box<str>,
    },
}

impl Item {
    /// Returns the question that corresponds to the this item.
    #[must_use]
    pub fn question(self) -> Box<str> {
        match self {
            Self::MultipleChoice { question, .. } => question,
        }
    }
}

newtype!(SessionId, Username, Hashed);

/// Newtype for Plain
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
#[expect(clippy::exhaustive_structs, reason = "newtype")]
pub struct Plain(pub Box<str>);

impl fmt::Debug for Plain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plain").field(&"*****").finish()
    }
}
