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
        let file_name = path.as_ref().display().to_string();
        let writer = BufWriter::new(
            std::fs::File::create(path)
                .map_err(|e| CsvError::FileAccessError(file_name, e.to_string()))?,
        );
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

    /// Consume the writer and return the underlying output stream.
    pub fn into_inner(self) -> Result<R> {
        self.writer
            .into_inner()
            .map_err(|error| CsvError::IOError(error.to_string()))
    }

    /// Writes a single CSV [`row`]
    ///
    /// # Arguments:
    /// `row` CSV row to be written.
    pub fn write(&mut self, row: &Row) -> Result<()> {
        let delimiter = match self.delimiter {
            Some(delim) => delim,
            _ => row.delim,
        };
        let mut delimiter_bytes = [0; 4];
        let delimiter_bytes = delimiter.encode_utf8(&mut delimiter_bytes).as_bytes();

        for (index, (start, end)) in row.ranges.iter().enumerate() {
            let field = &row.inner[*start..*end];
            let needs_quotes = field.contains(&QUOTE_BYTE)
                || field
                    .windows(delimiter_bytes.len())
                    .any(|window| window == delimiter_bytes)
                || field.contains(&b'\r')
                || field.contains(&b'\n');

            if needs_quotes {
                self.writer.write_all(&[QUOTE_BYTE])?;
                for byte in field {
                    if *byte == QUOTE_BYTE {
                        self.writer.write_all(&[QUOTE_BYTE, QUOTE_BYTE])?;
                    } else {
                        self.writer.write_all(std::slice::from_ref(byte))?;
                    }
                }
                self.writer.write_all(&[QUOTE_BYTE])?;
            } else {
                self.writer.write_all(field)?;
            }

            if index != row.ranges.len() - 1 {
                self.writer.write_all(delimiter_bytes)?;
            }
        }
        self.writer.write_all(&NEW_LINE)?;

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
