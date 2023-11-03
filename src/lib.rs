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
    fmt::{self, Debug, Display},
    io::{self, BufReader, BufWriter, Write},
    path::Path,
};

const CR: u8 = b'\r';
const LF: u8 = b'\n';
const QUOTE: u8 = b'"';
const DEFAULT_DELIM: char = ',';

#[doc(hidden)]
/// Generic Error type for internal use.
pub type Result<T> = std::result::Result<T, CsvError<'static>>;

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
                    }

                    if current_char == QUOTE {
                        if quote_count == 1 {
                            escaping = true;
                            continue;
                        } else if quote_count > 1 && quote_count % 2 == 0 {
                            escaping = false;
                            continue;
                        }
                    } else if current_char == separator as u8 {
                        if !escaping {
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

/// A CSV Writer implementation. Write to files or standard output.
pub struct Writer<R: io::Write> {
    writer: BufWriter<R>,
    delimiter: Option<char>,
    // record: Vec<u8>,
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
            // record: Vec::new(),
        }
    }

    /// Set a delimiter for a writer
    /// # Arguments:
    /// `delim` delimiter for CSV records being written.
    pub fn with_delimiter(mut self, delim: char) -> Self {
        self.delimiter = Some(delim);
        self
    }

    /// Writes a single CSV [`Record`]
    ///
    /// # Arguments:
    /// `record` CSV record to be written.
    pub fn write(&mut self, record: &Record) -> Result<()> {
        let delimiter = match self.delimiter {
            Some(delim) => delim as u8,
            _ => record.delim as u8,
        };

        // Since we now write behind a buffered writer, we can write single characters without much penalty
        // May not be pretty but it helps a lot in performance
        for (index, (start, end)) in record.ranges.iter().enumerate() {
            // To avoid slow allocation and string formatting, we escape fields manually
            let field = &record.inner[*start..*end];

            if field.contains(&QUOTE) {
                println!(
                    "We got some quotes in here: {}",
                    String::from_utf8_lossy(field)
                );
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

            if index != record.ranges.len() - 1 {
                // We only add the delimiter at the end of the each field except for the last
                self.writer.write_all(&[delimiter])?;
            }
        }
        self.writer.write_all(&[CR, LF])?;

        Ok(())
    }

    /// Convenient method to write several [`Record`]s at once.
    ///
    /// # Arguments
    /// `records`  vector of records to be written.
    pub fn write_all(&mut self, records: &[Record]) -> Result<()> {
        for record in records {
            self.write(record)?;
        }
        Ok(())
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
