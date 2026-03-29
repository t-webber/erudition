use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Mutex;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{
    Argon2, PasswordHash, PasswordHasher as _, PasswordVerifier as _
};
use color_eyre::eyre::Context as _;
use erudition_lib::{Auth, Hashed, Item, SessionId, Username};
use getrandom::fill;
use time::OffsetDateTime;
use time::macros::format_description;

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

/// Name of the file within the state folder where the persistent data of the
/// state is stored.
const DATA: &str = "data";
/// Name of the file within the state folder where writes occur, to prevent
/// corrupting the file.
const TEMP_DATA: &str = "data.temp";
/// Name of the file within the state folder where the logs are written.
const LOGS: &str = "logs";

/// State of the server, accessible from all route handlers
#[derive(Default, Debug)]
pub struct ServerState {
    /// Data that is saved and reloaded
    data: Mutex<StoredData>,
    /// Path of the folder where data is written (persistent state, logs,
    /// temporary files, etc.)
    path: PathBuf,
    /// Session ids of logged in users
    session_ids: Mutex<HashMap<SessionId, Username>>,
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

    /// Edit an existing item
    #[must_use]
    pub fn edit_item(&self, index: usize, item: Item) -> Option<bool> {
        { lock!(self.data).edit_item(index, item) }.map(|old| {
            self.log(&format!("Replaced item {index}: was {old:?}"));
            self.store()
        })
    }

    /// Returns the path to a data file
    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
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
    ///
    /// # Errors
    ///
    /// Returns an error if an state was stored in the data file, but it can't
    /// read it or it is invalid.
    pub fn load(path: PathBuf) -> color_eyre::Result<Self> {
        let data_path = path.join(DATA);
        let data_exists = fs::exists(&data_path).with_context(|| {
            format!(
                "Failed to check existence of {}, do I have access?",
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
        fs::create_dir_all(&path).with_context(|| {
            format!("Failed to create parent of {}", path.display())
        })?;
        Ok(Self { data, path, session_ids: Mutex::default() })
    }

    /// Writes some timestamped log to the log file and to the terminal.
    pub fn log(&self, msg: &str) {
        let timed_msg = Self::log_no_write(msg);
        let log_path = self.file(LOGS);
        if let Err(err) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .and_then(|mut log_file| writeln!(log_file, "{timed_msg}"))
        {
            Self::log_no_write(&format!(
                "\x1b[31mFailed to write logs to {}: {err}",
                log_path.display()
            ));
        }
    }

    /// Creates some timestamped log and prints it on the terminal
    #[expect(clippy::print_stderr, reason = "goal of function")]
    pub fn log_no_write(msg: &str) -> String {
        let format = format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        );
        let timestamp = OffsetDateTime::now_utc()
            .format(&format)
            .unwrap_or_else(|_| "0000-00-00 00:00:00".to_owned());
        let timed_msg = format!("\x1b[33m[{timestamp}]\x1b[35m {msg}\x1b[0m");
        eprintln!("{timed_msg}");
        timed_msg
    }

    /// Login a user
    ///
    /// Checks that username and password are valid, and returns a session id.
    #[must_use]
    pub fn login(&self, auth: Auth) -> Option<SessionId> {
        Argon2::default()
            .verify_password(
                auth.password.0.as_bytes(),
                &PasswordHash::new(
                    &lock!(self.data).get_user(&auth.username)?.0,
                )
                .ok()?,
            )
            .ok()?;
        self.make_session_id(auth.username)
    }

    /// Makes a new session id for the given user
    fn make_session_id(&self, username: Username) -> Option<SessionId> {
        let mut bytes = [0; 64];
        fill(&mut bytes).ok()?;
        let ascii_bytes = bytes.map(random_to_ascii);
        let id = String::from_utf8_lossy(&ascii_bytes);
        let session_id = SessionId(id.into());
        lock!(self.session_ids).insert(session_id.clone(), username);
        Some(session_id)
    }

    /// Signin a user
    ///
    /// Checks that username and password are valid, and returns a session id.
    #[must_use]
    pub fn signin(&self, auth: Auth) -> Option<SessionId> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(auth.password.0.as_bytes(), &salt)
            .ok()?;
        if lock!(self.data).add_user(
            auth.username.clone(),
            Hashed(hash.to_string().into_boxed_str()),
        ) {
            self.make_session_id(auth.username)
        } else {
            None
        }
    }

    /// Store the current state of the server at the given
    /// file path
    pub fn store(&self) -> bool {
        self.log("Storing data");
        let tmp_file = self.file(TEMP_DATA);
        let data_file = self.file(DATA);
        postcard::to_allocvec(&self.data)
            .map_err(|err| {
                format!(
                    "Failed to serialise data to \
                     disk:\nData:\n{:?}\n\nError:\n{err}",
                    self.data
                )
            })
            .and_then(|data| {
                fs::write(&tmp_file, data).map_err(|err| {
                    format!(
                        "Failed to save data to {}: {err}",
                        tmp_file.display()
                    )
                })
            })
            .and_then(|()| {
                fs::rename(&tmp_file, &data_file).map_err(|err| {
                    format!(
                        "Failed to move {} to {}: {err}",
                        tmp_file.display(),
                        data_file.display()
                    )
                })
            })
            .map_err(|msg| self.log(&msg))
            .is_ok()
    }
}

/// Casts and transforms a byte to make it valid ASCII for cookies
#[expect(
    clippy::arithmetic_side_effects,
    clippy::integer_division_remainder_used,
    reason = "in bounds and valid"
)]
const fn random_to_ascii(byte: u8) -> u8 {
    33 + byte % (127 - 33)
}
