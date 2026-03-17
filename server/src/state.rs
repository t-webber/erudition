use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::item::Item;
use crate::storage::StoredData;

/// Unlocks the mutex, even if poisened, as data can't really be corrupted
macro_rules! lock {
    ($data:expr) => {
        match $data.lock() {
            Ok(ref mut data) => data,
            Err(ref mut data) => data.get_mut(),
        }
    };
}

/// State of the server, accessible from all route handlers
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerState {
    /// Data that is saved and reloaded
    data: Mutex<StoredData>,
    /// Path of the file where the items are stored
    data_path: PathBuf,
    /// Path of the file where to write logs
    log_path: PathBuf,
}

impl ServerState {
    /// Save a new feedback
    #[must_use]
    pub fn add_feedback(&self, feedback: String) -> bool {
        lock!(self.data).add_feedback(feedback);
        self.store()
    }

    /// Adds a new item to the server
    #[must_use]
    pub fn add_item(&self, item: Item) -> bool {
        lock!(self.data).add_item(item);
        self.store()
    }

    /// Edit an existant item
    #[must_use]
    pub fn edit_item(&self, index: usize, item: Item) -> Option<bool> {
        lock!(self.data).edit_item(index, item).map(|old| {
            self.log(&format!("Replaced item {index}: was {old:?}"));
            self.store()
        })
    }

    /// Returns the list of feedback
    pub fn get_feedback(&self) -> Vec<String> {
        lock!(self.data).get_feedback().to_owned()
    }

    /// Returns the list of items currently on the server
    pub fn get_items(&self) -> Vec<Item> {
        lock!(self.data).get_items().to_owned()
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
        let data = if data_exists {
            postcard::from_bytes(
                fs::read_to_string(&data_path)
                    .with_context(|| {
                        format!("Failed to read {}", data_path.display())
                    })?
                    .as_bytes(),
            )
            .with_context(|| {
                format!("File {} has invalid data", data_path.display())
            })?
        } else {
            StoredData::default()
        }
        .into();
        Ok(Self { data, data_path, log_path })
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
        postcard::to_allocvec(&self.data)
            .map_err(|err| {
                format!(
                    "Failed to serialise data to \
                     disk:\nData:\n{:?}\n\nError:\n{err}",
                    self.data
                )
            })
            .and_then(|data| {
                fs::write(&self.data_path, data).map_err(|err| {
                    format!("Failed to save data to disk: {err}")
                })
            })
            .map_err(|msg| self.log(&msg))
            .is_ok()
    }
}
