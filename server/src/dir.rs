use std::fs;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

use color_eyre::eyre::{Context as _, ContextCompat as _};

/// Holds the location at which the logs and data should be stored.
#[derive(Debug)]
pub struct DataDir(PathBuf);

#[cfg(test)]
impl Clone for DataDir {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl DataDir {
    #[cfg(test)]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Returns the path to the data file.
    pub fn data(&self) -> PathBuf {
        self.0.join("data")
    }

    /// Returns the path to the temporary data file, used to secure writing
    /// (write then move).
    pub fn data_temp(&self) -> PathBuf {
        self.0.join("data.temp")
    }

    /// Returns the path to the logs file.
    pub fn logs(&self) -> PathBuf {
        self.0.join("logs")
    }

    /// Resolves a path in case it is not provided as a CLI
    /// argument.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment is missing a crucial variable, like
    /// `HOME`.
    pub fn new(path_opt: Option<PathBuf>) -> color_eyre::Result<Self> {
        let path = if let Some(path) = path_opt {
            path
        } else {
            dirs::data_dir()
                .context(if cfg!(target_os = "windows") {
                    "Your environment seems to be broken: \
                     FOLDERID_RoamingAppData variable does not exist"
                } else {
                    "Your environment seems to be broken: HOME variable does \
                     not exist"
                })
                .map(|path| path.join("erudition"))?
        };

        fs::create_dir_all(&path)
            .with_context(|| format!("Failed to mkdir {}", path.display()))?;

        Ok(path.into())
    }
}

impl From<PathBuf> for DataDir {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}
