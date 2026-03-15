use std::sync::Mutex;
use std::{fs, io};

use color_eyre::eyre::Context as _;
use serde::{Deserialize, Serialize};

use crate::item::Item;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ServerState {
    pub items: Mutex<Vec<Item>>,
    pub file: String,
}

pub enum ItemError {
    Io(io::Error),
    PostCard(postcard::Error),
}

impl ServerState {
    pub fn store(&self) -> Result<(), ItemError> {
        let data = postcard::to_allocvec(&self).map_err(ItemError::PostCard)?;
        fs::write(&self.file, data).map_err(ItemError::Io)?;
        Ok(())
    }

    pub fn load(file: String) -> color_eyre::Result<Self> {
        Ok(Self {
            items: postcard::from_bytes(
                fs::read_to_string(&file)
                    .with_context(|| format!("Failed to read {file}"))?
                    .as_bytes(),
            )
            .with_context(|| format!("File {file} has invalid data"))?,
            file,
        })
    }
}
