use core::mem::replace;
use std::collections::HashMap;

use erudition_lib::{Hashed, Item, Username};
use serde::{Deserialize, Serialize};

/// State data that is stored on the disk to be still available after the server
/// is restarted
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct StoredData {
    /// List of feedback
    feedback: Vec<String>,
    /// List of current items
    items: Vec<Item>,
    /// Maps username to password
    users: HashMap<Username, Hashed>,
}

impl StoredData {
    /// Adds a new feedback entry
    pub fn add_feedback(&mut self, feedback: String) {
        self.feedback.push(feedback);
    }

    /// Adds a new item entry
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Edits an existing item.
    pub fn edit_item(&mut self, index: usize, item: Item) -> Option<Item> {
        self.items.get_mut(index).map(|old| replace(old, item))
    }

    /// Returns all the feedback
    #[must_use]
    pub fn get_feedback(&self) -> &[String] {
        &self.feedback
    }

    /// Returns every item
    #[must_use]
    pub fn get_items(&self) -> &[Item] {
        &self.items
    }

    /// Checks that those credentials are valid
    pub fn get_user(&self, username: &Username) -> Option<&Hashed> {
        self.users.get(username)
    }
}
