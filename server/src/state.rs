use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::item::Item;

/// State of the server, accessible from all route handlers
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerState {
    /// Path of the file where the items are stored
    data_path: PathBuf,
    /// List of feedback
    feedback: Mutex<Vec<String>>,
    /// List of current items
    items: Mutex<Vec<Item>>,
    /// Path of the file where to write logs
    log_path: PathBuf,
}

impl ServerState {
    /// Save a new feedback
    pub fn add_feedback(&self, feedback: String) {
        match self.feedback.lock() {
            Ok(ref mut list) => list.push(feedback),
            Err(ref mut err) => {
                err.get_mut().push(feedback);
            }
        }
    }

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
        Ok(Self {
            items: items.unwrap_or_default(),
            data_path,
            log_path,
            feedback: Mutex::new(vec![]),
        })
    }

    /// Writes some timestamped log to the log file and to the terminal.
    pub fn log(&self, msg: &str) {
        match OffsetDateTime::now_utc().format(&Rfc3339) {
            Ok(time) => self.log_no_date(&format!("[{time}] {msg}")),
            Err(err) => {
                self.log_no_date(msg);
                self.log_no_date(&format!("Failed to get date: {err}"));
            }
        }
    }

    /// Writes some log to the log file and to stdout, without a date or
    /// anything
    #[expect(clippy::print_stderr, reason = "goal of function")]
    fn log_no_date(&self, msg: &str) {
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
