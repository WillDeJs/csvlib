//! [crate]
//! A simple CSV Reader/Writer library created for personal projects.
//!
//! # Example (Writer):
//! ```no_run
//! fn main() {
//!
//! // Write to file
//! let mut writer = csvlib::Writer::from_writer(std::fs::File::create("./test.txt").unwrap());
//!
//! // Create custom records
//! let header = csvlib::make_record!["Header1", "Header 2", "Header,3"];
//! writer.write_record(header).unwrap();
//! writer
//!     .write_all_records(vec![
//!         csvlib::make_record!["Header1", "Header 2", "Header,3"],
//!         csvlib::make_record!["entry", "entry", "entry"],
//!         csvlib::make_record!["entry", "entry", "entry"],
//!         csvlib::make_record!["entry", "entry", "entry"],
//!         csvlib::make_record!["entry", "entry", "entry"],
//!     ])
//!     .unwrap();
//! }
//!```
//! # Example (Reader):
//! ```no_run
//! fn main() {
//!
//!     // Read from files
//!     let file = std::fs::File::open("./TSLA.csv").unwrap();
//!
//!     // create custom records
//!     let record = csvlib::make_record!["Intr,o", 34, "klk", "manito"];
//!
//!     // Parse record fields
//!     println!("Got: {}", record.get_casted::<u32>(1).unwrap());
//!     println!("{}", record);
//!
//!     // Iterate through records
//!     let mut csv_reader = csvlib::Reader::builder()
//!         .with_delim(',')
//!         .with_reader(file)
//!         .with_header(true)
//!         .build()
//!         .unwrap();
//!     println!("{}", csv_reader.header().unwrap());
//!     for entry in csv_reader.entries() {
//!         println!("{}", entry);
//!     }
//! }
//! ```

pub use std::ops::Index;
use std::{
    any::type_name,
    fmt::{self, Debug},
};

const CR: u8 = '\r' as u8;
const LF: u8 = '\n' as u8;
const QUOTE: u8 = '"' as u8;
const DEFAULT_DELIM: char = ',' as char;

#[doc(hidden)]
/// Generic Error type for internal use.
pub type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;

/// A simple CSV Field container
///
/// #Example
/// ```
/// # use csvlib::Field;
/// let field : Field = String::from("This is a field").into();
/// assert_eq!(field, Field::from_str("This is a field".into()));
///
/// let number_field : Field = 1.into();
/// assert_eq!(number_field, Field::from_str("1"));
/// assert_ne!(number_field, Field::from_str("2"));
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
    pub fn new(inner: Vec<u8>) -> Self {
        Self { inner }
    }

    /// Creates a CSV Field from a string
    ///
    /// # Arguments
    /// `string` A string slice to be contained  in the Field
    pub fn from_str(string: &str) -> Self {
        format!("{}", string).into()
    }

    /// Retrieves a reference the inner bytes from the Field
    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.inner
    }

    /// Attempts to convert the Field into a String.
    ///
    /// If parsing is possible a Result is returned which needs to be unwrapped
    /// in order to retrieve the inner string value.
    ///
    /// # Errors
    /// If the bytes inside of the field cannot be parsed into a valid UTF8 String
    pub fn to_string(&self) -> Result<String> {
        String::from_utf8(self.inner.clone()).map_err(|_err| "Error parsing string field".into())
    }
}

impl<T: std::str::FromStr + std::fmt::Display> From<T> for Field {
    fn from(entry: T) -> Self {
        let string_val = format!("{}", entry);
        Field::new(string_val.into_bytes())
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string().unwrap())
    }
}

/// A CSV Record which may contain several CSV Fields
///
/// See [`Field`]
#[derive(Debug, Clone)]
pub struct Record {
    inner: Vec<Field>,
    delim: char,
}

impl Record {
    /// Constrcut a simple empty CSV Record
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            delim: DEFAULT_DELIM,
        }
    }

    /// Set the record's delimiter
    ///
    /// Default delimiter is a comma '.'.
    pub fn delimiter(&mut self, delim: char) {
        self.delim = delim;
    }

    /// Adds a [`Field`] to the record.
    ///
    /// # Arguments:
    /// `field` A Field of any type that implements [`std::fmt::Display`]
    pub fn add<T>(&mut self, field: T)
    where
        Field: From<T>,
    {
        self.inner.push(field.into())
    }

    /// Remove a [`Field`] from a record if it exists at a given index.
    ///
    /// # Arguments:
    /// `index` the location of the Field in the record.
    ///
    /// # Panics
    /// Panics if the index is not a valid entry in the record.
    ///
    /// If record index is not known consider using remove_item
    pub fn remove<T>(&mut self, index: usize) -> Field
    where
        Field: From<T>,
        T: Clone,
    {
        self.inner.remove(index)
    }

    /// Remove a [`Field`] from a record if it exists.
    ///
    /// # Arguments:
    /// `field` a reference to the field bering deleted
    pub fn remove_item<T>(&mut self, field: &T)
    where
        Field: From<T>,
        T: Clone,
    {
        for (index, inner_field) in self.inner.iter().enumerate() {
            if inner_field == &field.clone().into() {
                self.inner.remove(index);

                break;
            }
        }
    }

    /// Retrieves a [`Field`] from the record if it has been added.
    pub fn get(&self, index: usize) -> Option<&Field> {
        self.inner.get(index)
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
    /// let record = csvlib::make_record!["This is a record", 25, 56.2];
    ///
    /// assert_eq!(record.get_casted::<String>(0).unwrap(), "This is a record".to_string());
    /// assert_eq!(record.get_casted::<u32>(1).unwrap(), 25);
    /// assert_eq!(record.get_casted::<f64>(2).unwrap(), 56.2);
    /// ```
    pub fn get_casted<T: std::str::FromStr>(&self, index: usize) -> Result<T> {
        match self.get(index) {
            Some(field) => field.to_string()?.parse::<T>().map_err(|_| {
                format!(
                    "Error: Could parse field at index: {} to {}",
                    index,
                    type_name::<T>(),
                )
                .into()
            }),
            _ => Err(format!(
                "Error: Failed conversion to {} no valid field at index {}.",
                type_name::<T>(),
                index
            )
            .into()),
        }
    }

    /// Retrieves the number of [`Field`]s in the record
    pub fn count(&self) -> usize {
        self.inner.len()
    }
}

impl Index<usize> for Record {
    type Output = Field;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut record = String::new();
        let last_index = self.inner.len().saturating_sub(1);
        for (index, field) in self.inner.iter().enumerate() {
            if field.inner.contains(&(self.delim as u8))
                && !(field.inner.starts_with(&[QUOTE]) && field.inner.ends_with(&[QUOTE]))
            {
                if index != last_index {
                    record.push_str(&format!(
                        "{}{}{}{}",
                        QUOTE as char, field, QUOTE as char, self.delim
                    ));
                } else {
                    record.push_str(&format!("{}{}{}", QUOTE as char, field, QUOTE as char));
                }
            } else {
                if index != last_index {
                    record.push_str(&format!("{}{}", field, self.delim));
                } else {
                    record.push_str(&format!("{}", field));
                }
            }
        }
        write!(f, "{}", record)
    }
}

/// A CSV Reader struct to allow reading from files and other streams
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Reader<R> {
    reader: R,
    header: Option<Record>,
    has_header: bool,
    delimeter: Option<char>,
}

impl<R: std::io::Read> Reader<R> {
    pub fn entries(&mut self) -> Entries<R> {
        Entries::new(self)
    }
}

impl<R> Reader<R>
where
    R: std::io::Read,
{
    /// Creates a [`ReaderBuilder`] to construct a CSV Reader
    pub fn builder() -> ReaderBuilder<R> {
        ReaderBuilder::new()
    }

    /// Retreives the headers for this reader
    pub fn header(&self) -> Option<Record> {
        self.header.clone()
    }
}

/// A CSV Reader builder that allows to read CSV data from files and other steams.
pub struct ReaderBuilder<R> {
    reader: Option<R>,
    header: Option<Record>,
    has_header: bool,
    delimeter: Option<char>,
}

impl<R> ReaderBuilder<R> {
    /// Create a new empty ReaderBuilder from an empty implementation
    pub fn new() -> Self {
        Self {
            reader: None,
            header: None,
            has_header: false,
            delimeter: None,
        }
    }
}

impl<R> ReaderBuilder<R>
where
    R: std::io::Read,
{
    /// Constrcuts a CSV Reader from a builder.
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
    /// let mut csv_reader = csvlib::Reader::builder()
    ///     .with_delim(',')
    ///     .with_reader(std::io::stdin())
    ///     .with_header(true)
    ///     .build()
    ///     .unwrap();
    /// println!("{}", csv_reader.header().unwrap());
    /// ```
    pub fn build(mut self) -> Result<Reader<R>> {
        match self.reader {
            Some(mut reader) => {
                let delimiter = match self.delimeter {
                    Some(delim) => delim,
                    _ => DEFAULT_DELIM,
                };
                if self.has_header {
                    self.header = Some(read_fields(&mut reader, delimiter)?);
                }

                Ok(Reader {
                    reader,
                    header: self.header,
                    has_header: self.has_header,
                    delimeter: self.delimeter,
                })
            }
            _ => Err("Error: Cannot build a CSV reader without initailizing a reader/file".into()),
        }
    }

    /// Build Reader with a custom delimiter. If not given, defaults to comma (',') as delimiter.
    /// # Arguments:
    /// `delim` character delimiter to be used.
    pub fn with_delim(mut self, delim: char) -> Self {
        self.delimeter = Some(delim);
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
///        .with_delim(',')
///        .with_reader(file)
///        .with_header(true)
///        .build()
///        .unwrap();
///  println!("{}", csv_reader.header().unwrap());
///  for entry in csv_reader.entries() {
///  println!("{}", entry);
///  }
/// ```
pub struct Entries<'a, R>
where
    R: std::io::Read,
{
    owner: &'a mut Reader<R>,
}
impl<'a, R: std::io::Read> Entries<'a, R> {
    fn new(owner: &'a mut Reader<R>) -> Self {
        Self { owner }
    }
}

impl<'a, R: std::io::Read> Iterator for Entries<'a, R> {
    type Item = Record;

    fn next(&mut self) -> Option<Self::Item> {
        let delimiter = match self.owner.delimeter {
            Some(delim) => delim,
            _ => DEFAULT_DELIM,
        };
        read_fields(&mut self.owner.reader, delimiter).ok()
    }
}

#[doc(hidden)]
/// Internal function this is where the parsing happens.
///
/// # Arguments:
/// `reader` std::io::Read to get data from
/// `separator' character delimiter for CSV files
fn read_fields(reader: &mut impl std::io::Read, separator: char) -> Result<Record> {
    let mut field = Vec::<u8>::new();
    let mut one_char: [u8; 1] = [0; 1];
    let mut record = Record::new();
    let mut escaped = false;

    while let Ok(count) = reader.read(&mut one_char) {
        if count > 0 {
            let current_char = one_char[0];
            if current_char == LF {
                if field.ends_with(&[CR]) {
                    field.pop();
                }
                if field.ends_with(&[QUOTE]) && field.starts_with(&[QUOTE]) && field.len() > 1 {
                    field.pop();
                    field.remove(0);
                    field.clear();
                }
                record.add(Field::new(field.clone()));
                break;
            }

            if current_char == separator as u8 && !escaped {
                if field.ends_with(&[QUOTE]) && field.starts_with(&[QUOTE]) {
                    field.pop();
                    field.remove(0);
                    field.clear();
                }
                record.add(Field::new(field.clone()));
                field.clear();
                escaped = false;
                continue;
            }
            if escaped && current_char == QUOTE {
                escaped = true;
            }
            field.push(current_char);
        } else {
            if !field.is_empty() {
                record.add(Field::new(field.clone()));
                break;
            } else {
                return Err("Error: Could not parse entry from reader".into());
            }
        }
    }
    Ok(record)
}

/// A CSV Writer implementation. Write to files or standard output.
pub struct Writer<R> {
    writer: R,
}

impl<R: std::io::Write + Sized> Writer<R> {
    /// Initialize a CSV Writer from a std::io::Write implementation
    ///
    /// # Arguments:
    /// `writer` std::io::Write implemetnation to write to
    pub fn from_writer(writer: R) -> Self {
        Self { writer }
    }

    /// Writes a single CSV [`Record`]
    ///
    /// # Arguments:
    /// `record` CSV record to be written.
    pub fn write_record(&mut self, record: Record) -> Result<()> {
        write!(self.writer, "{}\n", record).map_err(|op| op.to_string().into())
    }

    /// Convenient method to write several [`Record`]s at once.
    ///
    /// # Arguments
    /// `records`  vector of records to be written.
    pub fn write_all_records(&mut self, records: Vec<Record>) -> Result<()> {
        for record in records {
            self.write_record(record)?;
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
/// # fn main() {
/// let header = csvlib::make_record!["Header 1", "Header 2", "Header 3"];
/// let entry1 = csvlib::make_record!["This is text", 1.2, 5];
/// # }
/// ```
#[macro_export]
macro_rules! make_record {
    ($($e:expr),*) => {
        {
            let mut record = $crate::Record::new();
            $(record.add(format!("{}",$e));)*
            record
        }
    };
}
