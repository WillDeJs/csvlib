//! [crate]
//! A simple CSV Reader/Writer library created for personal projects.
//!
//! # Example (Writer):
//! ```no_run
//!
//!
//! // Write to file
//! let mut writer = csvlib::Writer::from_writer(std::fs::File::create("./test.txt").unwrap());
//!
//! // Create custom records
//! let header = csvlib::csv!["Header1", "Header 2", "Header,3"];
//! writer.write(&header).unwrap();
//! writer
//!     .write_all(&vec![
//!         csvlib::csv!["Header1", "Header 2", "Header,3"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!     ])
//!     .unwrap();
//!
//!```
//! # Example (Reader):
//! ```no_run
//!     // create custom records
//!     let record = csvlib::csv!["Intr,o", 34, "klk", "manito"];
//!
//!     // Parse record fields
//!     println!("Got: {}", record.get::<u32>(1).unwrap());
//!     println!("{}", record);
//!
//!     // Iterate through records
//!     let mut csv_reader = csvlib::Reader::from_path("./TSLA.csv")
//!         .unwrap();
//!
//!     println!("{}", csv_reader.headers().unwrap());
//!     for entry in csv_reader.entries() {
//!         println!("{}", entry);
//!     }
//!
//! ```

pub use std::ops::Index;
pub use std::str::FromStr;
use std::{
    any::type_name,
    borrow::BorrowMut,
    error::Error,
    fmt::{self, Display},
    io::{self},
};

pub mod reader;
pub mod writer;

pub use reader::Reader;
pub use writer::Writer;

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const QUOTE: u8 = b'"';
const DEFAULT_DELIM: char = ',';

/// Generic Error type for internal use.
pub type Result<T> = std::result::Result<T, CsvError<'static>>;

/// A simple CSV Field container
///
/// #Example
/// ```
/// # use csvlib::Field;
/// let field : Field = "This is a field".into();
/// assert_eq!(field, Field::from("This is a field"));
///
/// println!("This is a CSV Field: {}", field);
/// ```
#[derive(PartialEq, Debug, Clone)]
pub struct Field {
    inner: Vec<u8>,
}

impl Field {
    /// Creates a new CSV Field from a given vector of bytes
    ///
    /// # Arguments
    /// `inner` Vec<u8> bytes to be contained in the Field
    pub fn new(inner: &[u8]) -> Self {
        Self {
            inner: inner.to_vec(),
        }
    }

    /// Retrieves a reference the inner bytes from the Field
    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.inner
    }

    /// Convert the Field into a String.
    ///
    /// If parsing is possible a Result is returned which needs to be unwrapped
    /// in order to retrieve the inner string value.
    ///
    /// # Errors
    /// If the bytes inside of the field cannot be parsed into a valid UTF8 String
    pub fn to_string(&self) -> Result<String> {
        String::from_utf8(self.inner.clone()).map_err(|_| CsvError::InvalidString)
    }

    /// Cast field into a given type.
    ///
    /// If the parsing is not possible, a result with an error is returned.
    ///
    /// # Errors
    /// If the bytes inside the field cannot be parsed into valid UTF8 strings.
    /// If the resulting field cannot be parsed into the type specified for conversion
    pub fn cast<T: FromStr>(&self) -> Result<T> {
        self.to_string()?
            .parse::<T>()
            .map_err(|_| CsvError::FieldParseError(type_name::<T>()))
    }
}

impl FromStr for Field {
    type Err = CsvError<'static>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Field::new(s.as_bytes()))
    }
}
impl From<&str> for Field {
    fn from(value: &str) -> Self {
        Field::new(value.as_bytes())
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string().map_err(|_| std::fmt::Error)?)
    }
}

/// A CSV Record which may contain several CSV Fields
///
/// See [`Field`]
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    inner: Vec<u8>,
    ranges: Vec<(usize, usize)>,
    delim: char,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            ranges: Vec::new(),
            delim: DEFAULT_DELIM,
        }
    }
}
impl Record {
    /// Construct a simple empty CSV Record
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize a record with an allocated capacity of fields.
    ///
    /// Useful to avoid multiple allocations.
    ///
    /// # Arguments
    /// `size`  Number of headers in the record.
    pub fn with_capacity(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
            ranges: Vec::new(),
            delim: DEFAULT_DELIM,
        }
    }

    /// Set the record's delimiter
    ///
    /// Default delimiter is a comma '.'.
    pub fn delimiter(&mut self, delim: char) {
        self.delim = delim;
    }

    /// Returns an iterator over the inner fields
    ///
    ///  # Examples
    ///
    /// ```
    /// use csvlib::{Reader, FromStr};
    ///
    /// let data = r#"header1,header2,header3,header4
    ///"test,",12,13,"com,ma"
    ///"wow",22,23,24
    ///"b""d",32,33,34"#;
    ///
    /// let reader = Reader::from_str(data).unwrap();
    /// let mut header = reader.headers().unwrap();
    ///
    /// for field in header.iter() {
    ///     print!("{}\t", field.cast::<String>().unwrap());
    /// }
    /// ```
    ///
    pub fn iter(&self) -> FieldsIter {
        FieldsIter {
            record: self,
            index: 0,
        }
    }

    /// Adds a [`Field`] to the record.
    ///
    /// # Arguments:
    /// `field` A Field of any type that implements [`std::fmt::Display`]
    pub fn add(&mut self, field: &[u8]) {
        let start = self.inner.len();
        let length = field.len();
        self.ranges.push((start, start + length));
        self.inner.extend_from_slice(field)
    }

    /// Attempts to retrieve and cast a field to a given type.
    ///
    /// # Arguments
    /// `index` the index of the Field inside the record
    ///
    /// # Returns
    /// A result with either the casted field to type T or an error.
    ///
    /// # Examples:
    /// ```
    /// let record = csvlib::csv!["This is a record", 25, 56.2];
    ///
    /// assert_eq!(record.get::<String>(0).unwrap(), "This is a record".to_string());
    /// assert_eq!(record.get::<u32>(1).unwrap(), 25);
    /// assert_eq!(record.get::<f64>(2).unwrap(), 56.2);
    /// ```
    pub fn get<T: std::str::FromStr>(&self, index: usize) -> Result<T> {
        match self.ranges.get(index) {
            Some((start, end)) => Ok(String::from_utf8_lossy(&self.inner[*start..*end])
                .borrow_mut()
                .parse::<T>()
                .map_err(|_| CsvError::ConversionError(index, type_name::<T>()))?),
            _ => Err(CsvError::NotAField(index)),
        }
    }

    pub fn get_range(&self, index: usize) -> Option<&[u8]> {
        match self.ranges.get(index) {
            Some((start, end)) => Some(&self.inner[*start..*end]),
            None => None,
        }
    }

    /// Retrieves the number of [`Field`]s in the record
    pub fn count(&self) -> usize {
        self.ranges.len()
    }
}

impl Index<usize> for Record {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        self.get_range(index).unwrap()
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last_index = self.ranges.len().saturating_sub(1);
        for (index, field) in self.iter().enumerate() {
            let field_value = field.to_string().map_err(|_| std::fmt::Error)?;

            // escape every single quote. This assumes what's present in each field
            // is what the user wants in it, no need for the user to escape things for us
            let field_value = field_value.replace('\"', "\"\"");

            // If we have quotes or commas, then we need outer double quotes in this field
            if field_value.contains(self.delim) || field_value.contains(QUOTE as char) {
                write!(f, "\"{field_value}\"")?;
            }
            if index != last_index {
                write!(f, "{}{}", field_value, self.delim)?;
            } else {
                write!(f, "{}", &field_value)?;
            }
        }
        Ok(())
    }
}

pub struct FieldsIter<'a> {
    record: &'a Record,
    index: usize,
}

impl Iterator for FieldsIter<'_> {
    type Item = Field;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.record.get(self.index - 1).ok()
    }
}

/// Create a CSV [`Record`] from a several CSV [`Field`]s.
/// Defaults to separator comma (',').
///
/// # Examples:
/// ```
/// # #[macro_use]
///
/// let header = csvlib::csv!["Header 1", "Header 2", "Header 3"];
/// let entry1 = csvlib::csv!["This is text", 1.2, 5];
///
/// ```
#[macro_export]
macro_rules! csv {
    ($($e:expr),*) => {
        {
            let mut record = $crate::Record::new();
            $(record.add(&format!("{}",$e).as_bytes());)*
            record
        }
    };
}

#[derive(Debug)]
pub enum CsvError<'a> {
    RecordError,
    ReadError,
    ConversionError(usize, &'a str),
    InvalidString,
    FieldParseError(&'a str),
    NotAField(usize),
    FileError,
}

impl Display for CsvError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::RecordError => write!(f, "Error reading CSV record"),
            CsvError::ReadError => write!(f, "Error reading from source."),
            CsvError::ConversionError(index, type_name) => {
                write!(f, "Error converting field `{index}` to type `{type_name}`")
            }
            CsvError::InvalidString => write!(f, "Cannot convert field to a valid string."),
            CsvError::NotAField(index) => write!(f, "Not field at given index `{index}`."),
            CsvError::FieldParseError(type_name) => {
                write!(f, "Error parsing field to `{type_name}`.")
            }
            CsvError::FileError => write!(f, "Error accessing file."),
        }
    }
}

impl From<io::Error> for CsvError<'_> {
    fn from(_: io::Error) -> Self {
        CsvError::FileError
    }
}

impl Error for CsvError<'_> {}
