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
#[derive(Serialize, Deserialize, Debug, Clone)]
#[expect(clippy::exhaustive_structs, reason = "no fields should be added")]
pub struct Item {
    /// Possible answers, the first is always the
    /// correct answer.
    pub answer: Box<str>,
    /// Question.
    pub question: Box<str>,
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
