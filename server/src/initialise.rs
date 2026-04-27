use color_eyre::eyre::bail;

use crate::dir::DataDir;

/// Initialises data from some `csv` files.
pub struct Initialise;

impl Initialise {
    /// Initialises data from some `csv` files.
    ///
    /// # Errors
    ///
    /// Returns an error if some data is already present.
    pub fn initialise(dir: &DataDir) -> color_eyre::Result<()> {
        let data = dir.data();
        if data.exists() {
            bail!("{} exists, remove it to initialise again", data.display())
        }
        Ok(())
    }
}
