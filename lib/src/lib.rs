//! A lib to share declarations between server and app.

use core::fmt;

use serde::{Deserialize, Serialize};

/// Defines new types.
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

/// Authentication request body.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auth {
    /// Plain password.
    pub password: Plain,
    /// Username.
    pub username: Username,
}

impl Auth {
    /// Creates a new [`Auth`] from the given credentials.
    #[must_use]
    pub const fn new(username: Box<str>, password: Box<str>) -> Self {
        Self { password: Plain(password), username: Username(username) }
    }
}

/// Item store and returned by the server.
#[non_exhaustive]
#[expect(clippy::enum_variant_names, reason = "only one variant")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Item {
    /// Multiple choice question.
    MultipleChoice {
        /// Possible answers, the first is always the
        /// correct answer.
        answers: Vec<Box<str>>,
        /// Question.
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

/// Newtype for Plain.
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
#[expect(clippy::exhaustive_structs, reason = "newtype")]
pub struct Plain(pub Box<str>);

impl fmt::Debug for Plain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Plain").field(&"*****").finish()
    }
}
