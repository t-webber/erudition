use std::path::PathBuf;

use color_eyre::eyre::{Context as _, ContextCompat as _, ensure};
use erudition_lib::Item;
use polars::frame::DataFrame;
use polars::io::SerReader as _;
use polars::prelude::{CsvParseOptions, CsvReadOptions};

use crate::dir::DataDir;
use crate::state::ServerState;

/// Initialises data from some `csv` files.
#[derive(Debug)]
pub struct Initialise(ServerState);

impl Initialise {
    /// Add geography items.
    ///
    /// # Errors
    ///
    /// Fails if the data supposed to exist for geography information doesn't
    /// exist, or is in the wrong format.
    fn geography(&self) -> color_eyre::Result<()> {
        let df = Self::geography_data()?;
        for values in df
            .column("Country")?
            .str()?
            .iter()
            .zip(df.column("Capital")?.str()?.iter())
        {
            let name = values.0.context("Missing name")?;
            let Some(capital) = values.1 else {
                continue;
            };

            ensure!(
                self.0.add_item(Item {
                    question: format!("What is the capital of {name}?").into(),
                    answer: format!("The capital of {name} is {capital}")
                        .into(),
                }),
                "Failed to add item {values:?}"
            );
        }
        Ok(())
    }

    /// Load geography data.
    ///
    /// # Errors
    ///
    /// Fails if the data supposed to exist for geography information doesn't
    /// exist, or is in the wrong format.
    fn geography_data() -> color_eyre::Result<DataFrame> {
        const PATH: &str = "data/countryInfo.tsv";
        let parse_options = CsvParseOptions::default().with_separator(b'\t');
        Ok(CsvReadOptions::default()
            .with_parse_options(parse_options)
            .try_into_reader_with_file_path(Some(PathBuf::from(PATH)))
            .with_context(|| format!("Failed to read {PATH}"))?
            .finish()
            .with_context(|| format!("Failed to parse {PATH}"))?
            .select([
                "#ISO",
                "Country",
                "Capital",
                "Area(in sq km)",
                "Population",
                "tld",
                "CurrencyName",
                "Phone",
                "neighbours",
            ])?)
    }

    /// Initialises data from some `csv` files.
    ///
    /// # Errors
    ///
    /// Returns an error if some data is already present.
    pub fn initialise(dir: DataDir) -> color_eyre::Result<()> {
        let this = Self(ServerState::load(dir)?);
        this.geography()?;
        Ok(())
    }
}
