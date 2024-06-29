//! Simple CSV Writer which can be used to write CSV row to a file or other
//! sources.
//!
//! Aim to be simple to use and a simple implementation for educational purposes.
//!
//!  # Example (Writer):
//! ``` rs
//! fn main() {
//!
//!   // Write to file
//!     let mut writer = csvlib::Writer::from_path("./test.txt").unwrap();
//!
//!     // Create custom rows
//!     let header = csvlib::csv!["Header1", "Header 2", "Header,3"];
//!     writer.write(&header).unwrap();
//!     writer
//!         .write_all(&[
//!             csvlib::csv!["Header1", "Header 2", "Header,3"],
//!             csvlib::csv!["entry", "entry", "entry"],
//!             csvlib::csv!["entry", "entry", "entry"],
//!             csvlib::csv!["entry", "entry", "entry"],
//!             csvlib::csv!["entry", "entry", "entry"],
//!         ])
//!         .unwrap();
//! }
//!
//!

use std::{
    io::{self, BufWriter, Write},
    path::Path,
};

use crate::*;

/// A CSV Writer implementation. Write to files or standard output.
pub struct Writer<R: io::Write> {
    writer: BufWriter<R>,
    delimiter: Option<char>,
    // row: Vec<u8>,
}

impl Writer<std::fs::File> {
    /// Creates a CSV Writer using a path given by the user.
    ///
    /// A default delimiter of comma "," is assumed. If an alternative separator
    /// is desired, please see `csvlib::Writer::from_writer(...).with_delimiter(...)`.
    ///
    /// # Arguments
    /// `path` the path to the file to be used to write CSV
    /// # Returns
    /// A result with the given writer, or an error if an error accessing the file.
    ///
    /// # Error
    /// If the underlying file behind path is not accessible for any reason.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let writer = BufWriter::new(std::fs::File::create(path).map_err(|_| CsvError::FileError)?);
        Ok(Self {
            writer,
            delimiter: None,
        })
    }
}

impl<R: io::Write + Sized> Writer<R> {
    /// Initialize a CSV Writer from a std::io::Write implementation
    ///
    /// # Arguments:
    /// `writer` std::io::Write implementation to write to
    pub fn from_writer(writer: R) -> Self {
        Self {
            writer: BufWriter::new(writer),
            delimiter: None,
            // row: Vec::new(),
        }
    }

    /// Set a delimiter for a writer
    /// # Arguments:
    /// `delim` delimiter for CSV rows being written.
    pub fn with_delimiter(mut self, delim: char) -> Self {
        self.delimiter = Some(delim);
        self
    }

    /// Writes a single CSV [`row`]
    ///
    /// # Arguments:
    /// `row` CSV row to be written.
    pub fn write(&mut self, row: &Row) -> Result<()> {
        let delimiter = match self.delimiter {
            Some(delim) => delim as u8,
            _ => row.delim as u8,
        };

        // Since we now write behind a buffered writer, we can write single characters without much penalty
        // May not be pretty but it helps a lot in performance
        for (index, (start, end)) in row.ranges.iter().enumerate() {
            // To avoid slow allocation and string formatting, we escape fields manually
            let field = &row.inner[*start..*end];

            if field.contains(&QUOTE) {
                // When we have quotes, we escape each quote and put quotes around the field itself
                self.writer.write_all(&[QUOTE])?;
                for byte in field {
                    if byte == &QUOTE {
                        // escape the quote!
                        self.writer.write_all(&[*byte, QUOTE])?;
                    } else {
                        self.writer.write_all(&[*byte])?;
                    }
                }
                self.writer.write_all(&[QUOTE])?;
            } else if field.contains(&delimiter) {
                // If the delimiter is part of the field, then let's escape the field
                self.writer.write_all(&[QUOTE])?;
                self.writer.write_all(field)?;
                self.writer.write_all(&[QUOTE])?;
            } else {
                self.writer.write_all(field)?;
            }

            if index != row.ranges.len() - 1 {
                // We only add the delimiter at the end of the each field except for the last
                self.writer.write_all(&[delimiter])?;
            }
        }
        self.writer.write_all(&[CR, LF])?;

        Ok(())
    }

    /// Convenient method to write several [`row`]s at once.
    ///
    /// # Arguments
    /// `rows`  vector of rows to be written.
    pub fn write_all(&mut self, rows: &[Row]) -> Result<()> {
        for row in rows {
            self.write(row)?;
        }
        Ok(())
    }
}
