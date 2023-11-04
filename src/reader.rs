//! Simple CSV Reader. Offers the ability to parse CSV records from files.
//! This implementation is done for educational purposes to be used in personal
//! projects.
//!
//!  
//!  # Example (Reader):
//! ``` rs
//! fn main() {
//!    // Read from a file
//!    let csv_reader = csvlib::Reader::from_path("./AAPL.csv").unwrap();
//!
//!    // Iterate through rows
//!    println!("{}", csv_reader.headers().unwrap());
//!    for entry in csv_reader.entries() {
//!        println!("{}", entry);
//!    }
//! }
//! ```

use std::{io::BufReader, path::Path};

use crate::*;

/// A CSV Reader struct to allow reading from files and other streams
#[allow(dead_code)]
#[derive(Debug)]
pub struct Reader<R> {
    reader: BufReader<R>,
    header: Option<Record>,
    has_header: bool,
    delimiter: Option<char>,
}

impl<R: io::Read> Reader<R> {
    pub fn entries(self) -> Entries<R> {
        Entries::new(self)
    }
}

impl<R> Reader<R>
where
    R: io::Read,
{
    /// Creates a [`ReaderBuilder`] to construct a CSV Reader
    pub fn builder() -> ReaderBuilder<R> {
        ReaderBuilder::new()
    }

    /// Retrieves the headers for this reader
    pub fn headers(&self) -> Option<Record> {
        self.header.clone()
    }
}

impl Reader<std::fs::File> {
    /// Create a reader from a file path.
    ///
    /// Comma `,` is assumed as delimiter and headers to be present.
    /// If an alternative delimiter or header is required please see
    /// `
    /// csvlib::Reader::builder().with_delimiter(';').with_header(true);
    /// `
    ///
    ///
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path).map_err(|_| CsvError::FileError)?;
        let mut reader = BufReader::new(file);
        let header = read_fields(
            &mut reader,
            DEFAULT_DELIM,
            &mut Vec::with_capacity(100),
            &mut Vec::with_capacity(100),
        )?;

        Ok(Reader {
            reader,
            header: Some(header),
            has_header: true,
            delimiter: Some(DEFAULT_DELIM),
        })
    }
}

impl FromStr for Reader<std::io::Cursor<String>> {
    type Err = CsvError<'static>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let cursor = std::io::Cursor::new(s.to_owned());
        let mut reader = BufReader::new(cursor);
        let header = read_fields(
            &mut reader,
            DEFAULT_DELIM,
            &mut Vec::with_capacity(100),
            &mut Vec::with_capacity(100),
        )?;

        Ok(Reader {
            reader,
            header: Some(header),
            has_header: true,
            delimiter: Some(DEFAULT_DELIM),
        })
    }
}

/// A CSV Reader builder that allows to read CSV data from files and other steams.
pub struct ReaderBuilder<R> {
    reader: Option<R>,
    header: Option<Record>,
    has_header: bool,
    delimiter: Option<char>,
}

impl<R> ReaderBuilder<R> {
    /// Create a new empty ReaderBuilder from an empty implementation
    pub fn new() -> Self {
        Self::default()
    }
}

impl<R> Default for ReaderBuilder<R> {
    fn default() -> Self {
        Self {
            reader: None,
            header: None,
            has_header: false,
            delimiter: None,
        }
    }
}

impl<R> ReaderBuilder<R>
where
    R: io::Read,
{
    /// Constructs a CSV Reader from a builder.
    ///
    /// Compiles all options and required fields from what's fed to the ReaderBuilder.
    ///
    /// # Returns
    /// A Result with either a Reader or an Error in case the reader returns errors upon creation.
    ///
    /// # Examples:
    /// ```no_run
    /// # use csvlib::Reader;
    ///
    /// let mut csv_reader = csvlib::Reader::from_path("name.csv")
    ///     .unwrap();
    /// println!("{}", csv_reader.headers().unwrap());
    /// ```
    pub fn build(mut self) -> Result<Reader<R>> {
        match self.reader {
            Some(reader) => {
                let mut reader = BufReader::new(reader);
                let delimiter = match self.delimiter {
                    Some(delim) => delim,
                    _ => DEFAULT_DELIM,
                };
                if self.has_header {
                    self.header = Some(read_fields(
                        &mut reader,
                        delimiter,
                        &mut Vec::with_capacity(100),
                        &mut Vec::with_capacity(100),
                    )?);
                }

                Ok(Reader {
                    reader,
                    header: self.header,
                    has_header: self.has_header,
                    delimiter: self.delimiter,
                })
            }
            _ => Err(CsvError::ReadError),
        }
    }

    /// Build Reader with a custom delimiter. If not given, defaults to comma (',') as delimiter.
    /// # Arguments:
    /// `delim` character delimiter to be used.
    pub fn with_delimiter(mut self, delim: char) -> Self {
        self.delimiter = Some(delim);
        self
    }

    /// Sets whether the given reader contains a header line.
    ///
    /// # Arguments:
    /// `has_header` boolean whether the current reader contains headers
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Sets the reader interface for this Reader.
    ///
    /// # Arguments:
    /// `reader` std::io::Read implementation used to get CSV data
    pub fn with_reader(mut self, reader: R) -> Self {
        self.reader = Some(reader);
        self
    }
}

/// Iterator of Reader entries ([`Record`]s).
///
/// # Examples:
/// ```no_run
///  let file = std::fs::File::open("./TSLA.csv").unwrap();
///  let mut csv_reader = csvlib::Reader::builder()
///        .with_delimiter(',')
///        .with_reader(file)
///        .with_header(true)
///        .build()
///        .unwrap();
///  println!("{}", csv_reader.headers().unwrap());
///  for entry in csv_reader.entries() {
///  println!("{}", entry);
///  }
/// ```
pub struct Entries<R>
where
    R: io::Read,
{
    owner: Reader<R>,

    line_buffer: Vec<u8>,

    field_buffer: Vec<u8>,
}
impl<R: io::Read> Entries<R> {
    fn new(owner: Reader<R>) -> Self {
        Self {
            owner,
            line_buffer: Vec::with_capacity(100),
            field_buffer: Vec::with_capacity(100),
        }
    }
}

impl<R: io::Read> Iterator for Entries<R> {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        let delimiter = match self.owner.delimiter {
            Some(delim) => delim,
            _ => DEFAULT_DELIM,
        };
        read_fields(
            &mut self.owner.reader,
            delimiter,
            &mut self.field_buffer,
            &mut self.line_buffer,
        )
        .ok()
    }
}

#[doc(hidden)]
/// Internal function this is where the parsing happens.
///
/// # Arguments:
/// `reader` std::io::Read to get data from
/// `separator' character delimiter for CSV files
fn read_fields(
    reader: &mut impl io::BufRead,
    separator: char,
    field_buffer: &mut Vec<u8>,
    line_buffer: &mut Vec<u8>,
) -> Result<Record> {
    let mut record = Record::with_capacity(line_buffer.capacity());
    let mut multi_line = true;
    let mut quote_first_char = false;

    while multi_line {
        multi_line = false;
        line_buffer.clear();
        match reader.read_until(b'\n', line_buffer) {
            Ok(0) => return Err(CsvError::RecordError),
            Ok(_n) => {
                let mut escaping = false;

                field_buffer.clear();
                let mut quote_count = 0;
                for current_char in line_buffer.iter() {
                    let current_char = *current_char;
                    if current_char == QUOTE {
                        quote_count += 1;
                        if field_buffer.is_empty() {
                            quote_first_char = true;
                        }
                    }

                    if current_char == QUOTE && quote_first_char {
                        if quote_count == 1 {
                            escaping = true;
                            continue;
                        } else if quote_count > 1 && quote_count % 2 == 0 {
                            escaping = false;
                            continue;
                        }
                    } else if current_char == separator as u8 {
                        if !escaping {
                            quote_first_char = false;
                            record.add(field_buffer);
                            field_buffer.clear();
                            quote_count = 0;
                            continue;
                        }
                    } else if current_char == LF {
                        if !escaping {
                            continue;
                        }
                    } else if current_char == CR {
                        if !escaping {
                            record.add(field_buffer);
                            field_buffer.clear();
                            return Ok(record);
                        } else {
                            multi_line = true;
                        }
                    }

                    field_buffer.push(current_char);
                }

                // got to the end and but did not find  a carriage return
                if !field_buffer.is_empty() {
                    record.add(field_buffer);
                    field_buffer.clear();
                }
            }
            Err(_) => return Err(CsvError::ReadError),
        }
    }

    Ok(record)
}
