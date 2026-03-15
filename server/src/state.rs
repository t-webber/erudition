use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};

use crate::item::Item;

/// State of the server, accessible from all route handlers
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerState {
    /// Path of the file where the items are stored
    data_path: PathBuf,
    /// List of current items
    items: Mutex<Vec<Item>>,
    /// Path of the file where to write logs
    log_path: PathBuf,
}

impl ServerState {
    /// Adds a new item to the server
    pub fn add_item(&self, item: Item) -> bool {
        match self.items.lock() {
            Ok(ref mut items) => items.push(item),
            Err(ref mut err) => {
                err.get_mut().push(item);
            }
        }
        self.store()
    }

    /// Edit an existant item
    pub fn edit_item(&self, index: usize, item: Item) -> Option<bool> {
        let edit = |maybe_old: Option<&mut Item>| {
            maybe_old.map(|old| {
                *old = item;
            })
        };

        match self.items.lock() {
            Ok(mut items) => edit(items.get_mut(index)),
            Err(mut err) => edit(err.get_mut().get_mut(index)),
        }
        .map(|()| self.store())
    }

    /// Returns the list of items currently on the server
    pub const fn items(&self) -> &Mutex<Vec<Item>> {
        &self.items
    }

    /// Loads the state from the given file path
    pub fn load(
        data_path: PathBuf,
        log_path: PathBuf,
    ) -> color_eyre::Result<Self> {
        let data_exists = fs::exists(&data_path).with_context(|| {
            format!(
                "Failed to check existance of {}, do I have access?",
                data_path.display()
            )
        })?;
        let items = if data_exists {
            Some(
                postcard::from_bytes(
                    fs::read_to_string(&data_path)
                        .with_context(|| {
                            format!("Failed to read {}", data_path.display())
                        })?
                        .as_bytes(),
                )
                .with_context(|| {
                    format!("File {} has invalid data", data_path.display())
                })?,
            )
        } else {
            None
        };
        Ok(Self { items: items.unwrap_or_default(), data_path, log_path })
    }

    /// Writes some log to the log file
    #[expect(clippy::print_stderr, reason = "goal of function")]
    pub fn log(&self, msg: &str) {
        eprintln!("{msg}");
        if let Err(err) = fs::write(&self.log_path, msg) {
            eprintln!(
                "Failed to log error to {}: {err}",
                self.log_path.display()
            );
        }
    }

    /// Store the current state of the server at the given
    /// file path
    pub fn store(&self) -> bool {
        self.log("Storing data");
        postcard::to_allocvec(&self.items)
            .map_err(|err| {
                format!(
                    "Failed to serialise items to \
                     disk:\nItems:\n{:?}\n\nError:\n{err}",
                    self.items()
                )
            })
            .and_then(|data| {
                fs::write(&self.data_path, data).map_err(|err| {
                    format!("Failed to save items to disk: {err}")
                })
            })
            .map_err(|msg| self.log(&msg))
            .is_ok()
    }
}
