use std::sync::Mutex;
use std::{fs, io};

use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};

use crate::item::Item;

/// State of the server, accessible from all route handlers
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerState {
    /// Path of the file where the items are stored
    pub data_path: String,
    /// List of current items
    pub items: Mutex<Vec<Item>>,
    /// Path of the file where to write logs
    pub log_path: String,
}

/// Error that may happen when handling [`Item`]s
pub enum ItemError {
    /// Io error
    Io(io::Error),
    /// Serde error through postcard
    PostCard(postcard::Error),
}

impl ServerState {
    /// Loads the state from the given file path
    pub fn load(data_path: String, log_path: String) -> color_eyre::Result<Self> {
        let data_exists = fs::exists(&data_path).with_context(|| {
            format!("Failed to check existance of {data_path}, do I have access?")
        })?;
        let items = if data_exists {
            Some(
                postcard::from_bytes(
                    fs::read_to_string(&data_path)
                        .with_context(|| format!("Failed to read {data_path}"))?
                        .as_bytes(),
                )
                .with_context(|| format!("File {data_path} has invalid data"))?,
            )
        } else {
            None
        };
        Ok(Self {
            items: items.unwrap_or_default(),
            data_path,
            log_path,
        })
    }

    /// Writes some log to the log file
    #[expect(clippy::print_stderr, reason = "goal of function")]
    pub fn log(&self, msg: &str) {
        eprintln!("{msg}");
        if let Err(err) = fs::write(&self.log_path, msg) {
            eprintln!("Failed to log error to {}: {err}", self.log_path);
        }
    }

    /// Store the current state of the server at the given file path
    pub fn store(&self) -> Result<(), ItemError> {
        let data = postcard::to_allocvec(&self).map_err(ItemError::PostCard)?;
        fs::write(&self.data_path, data).map_err(ItemError::Io)?;
        Ok(())
    }
}
