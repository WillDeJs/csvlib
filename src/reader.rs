//! Simple CSV Reader. Offers the ability to parse CSV rows from files.
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

use std::{
    io::{self, BufReader},
    path::Path,
};

use crate::*;

/// A CSV Reader struct to allow reading from files and other streams
#[allow(dead_code)]
#[derive(Debug)]
pub struct Reader<R> {
    reader: BufReader<R>,
    header: Option<Row>,
    has_header: bool,
    delimiter: Option<char>,
}

impl<R: io::Read> Reader<R> {
    pub fn entries(self) -> Entries<R> {
        Entries::new(self)
    }

    ///
    /// Iterate over all rows, decoding them into the given type T.
    /// An iterator of Results is returned.
    /// # Example
    /// ```no_run
    /// use csvlib::{Reader, Result, Row};
    ///
    /// pub struct Person {
    ///     pub name: String,
    ///     pub last_name: String,
    ///     pub age: u32,
    ///     pub email: String,
    /// }
    ///
    /// impl TryFrom<Row> for Person {
    ///    type Error = csvlib::CsvError;
    ///     fn try_from(row: Row) -> Result<Self> {
    ///         Ok(Person {
    ///             // Using column indices. Adjust indices to match your CSV header order.
    ///             name: row.get::<String>(0)?,
    ///             last_name: row.get::<String>(1)?,
    ///             age: row.get::<u32>(2)?,
    ///             email: row.get::<String>(3)?,
    ///         })
    ///     }
    /// }
    ///
    /// fn main() -> Result<()> {
    ///     // Use the low-level Reader which yields `Row`s (the "Rows" iterator)
    ///     let reader = Reader::from_path("people.csv")?;
    ///
    ///     let mut total_age: u32 = 0;
    ///     let mut count: u32 = 0;
    ///
    ///     for person_res in reader.entries_decoded::<Person>() {
    ///         let person = person_res?;
    ///         total_age += person.age;
    ///         count += 1;
    ///     }
    ///
    ///     if count == 0 {
    ///         println!("No people found");
    ///         return Ok(());
    ///     }
    ///
    ///     let average_age = total_age as f32 / count as f32;
    ///     println!("Average age: {}", average_age);
    ///
    ///     Ok(())
    /// }
    ///```
    pub fn entries_decoded<T>(self) -> impl Iterator<Item = Result<T>>
    where
        T: TryFrom<Row, Error = CsvError>,
    {
        self.entries().map(|row| T::try_from(row))
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
    pub fn headers(&self) -> Option<Row> {
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
        let file_name = path.as_ref().display().to_string();
        let file = std::fs::File::open(path)
            .map_err(|e| CsvError::FileAccessError(file_name, e.to_string()))?;
        let mut reader = BufReader::new(file);
        let header = read_fields(&mut reader, DEFAULT_DELIM, &mut String::with_capacity(100))?;

        Ok(Reader {
            reader,
            header: Some(header),
            has_header: true,
            delimiter: Some(DEFAULT_DELIM),
        })
    }
}

impl FromStr for Reader<std::io::Cursor<String>> {
    type Err = CsvError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let cursor = std::io::Cursor::new(s.to_owned());
        let mut reader = BufReader::new(cursor);
        let header = read_fields(&mut reader, DEFAULT_DELIM, &mut String::with_capacity(100))?;

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
    header: Option<Row>,
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
                        &mut String::with_capacity(100),
                    )?);
                }

                Ok(Reader {
                    reader,
                    header: self.header,
                    has_header: self.has_header,
                    delimiter: self.delimiter,
                })
            }
            _ => Err(CsvError::ReadError("No reader given".to_string())),
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

/// Iterator of Reader entries ([`row`]s).
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

    line_buffer: String,
}
impl<R: io::Read> Entries<R> {
    fn new(owner: Reader<R>) -> Self {
        Self {
            owner,
            line_buffer: String::with_capacity(100),
        }
    }
}

impl<R: io::Read> Iterator for Entries<R> {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        let delimiter = match self.owner.delimiter {
            Some(delim) => delim,
            _ => DEFAULT_DELIM,
        };
        read_fields(&mut self.owner.reader, delimiter, &mut self.line_buffer).ok()
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
    line_buffer: &mut String,
) -> Result<Row> {
    let mut row = Row::with_capacity(line_buffer.capacity());
    let mut field = String::new();
    let mut in_quotes = false;

    loop {
        line_buffer.clear();
        match reader.read_line(line_buffer) {
            Ok(0) => {
                if in_quotes {
                    return Err(CsvError::RecordError(
                        "Unterminated quoted field".to_string(),
                    ));
                }

                if !field.is_empty() || row.count() > 0 {
                    row.add_bytes(field.as_bytes());
                    field.clear();
                    if row.count() > 0 {
                        return Ok(row);
                    }
                }

                return Err(CsvError::RecordError(line_buffer.to_string()));
            }
            Ok(_) => {
                let mut chars = line_buffer.chars().peekable();
                while let Some(ch) = chars.next() {
                    match ch {
                        '"' if in_quotes => {
                            if chars.peek() == Some(&'"') {
                                field.push('"');
                                chars.next();
                            } else {
                                in_quotes = false;
                            }
                        }
                        '"' => {
                            if field.is_empty() {
                                in_quotes = true;
                            } else {
                                field.push(ch);
                            }
                        }
                        c if c == separator && !in_quotes => {
                            row.add_bytes(field.as_bytes());
                            field.clear();
                        }
                        CR if !in_quotes => {
                            if chars.peek() == Some(&LF) {
                                chars.next();
                            }
                            row.add_bytes(field.as_bytes());
                            field.clear();
                            return Ok(row);
                        }
                        LF if !in_quotes => {
                            row.add_bytes(field.as_bytes());
                            field.clear();
                            return Ok(row);
                        }
                        _ => field.push(ch),
                    }
                }

                if in_quotes {
                    continue;
                }
            }
            Err(e) => return Err(CsvError::ReadError(e.to_string())),
        }
    }
}
