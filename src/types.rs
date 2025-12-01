pub use std::ops::Index;
pub use std::str::FromStr;
use std::{
    any::type_name,
    error::Error,
    fmt::{self, Display},
    io::{self},
};

pub(crate) const CR: char = '\r';
pub(crate) const LF: char = '\n';
pub(crate) const QUOTE: char = '"';
pub(crate) const QUOTE_BYTE: u8 = b'"';
pub(crate) const NEW_LINE: [u8; 2] = [b'\r', b'\n'];
pub(crate) const DEFAULT_DELIM: char = ',';

/// Generic Error type for internal use.
pub type Result<T> = std::result::Result<T, CsvError>;

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
    pub(crate) inner: Vec<u8>,
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

    /// Cast field into a given type.
    ///
    /// If the parsing is not possible, a result with an error is returned.
    ///
    /// # Errors
    /// If the bytes inside the field cannot be parsed into valid UTF8 strings.
    /// If the resulting field cannot be parsed into the type specified for conversion
    pub fn cast<T: FromStr>(&self) -> Result<T> {
        let string_val = self.to_string();
        string_val
            .parse::<T>()
            .map_err(|_| CsvError::FieldParseError(string_val, type_name::<T>().to_string()))
    }
}

impl FromStr for Field {
    type Err = CsvError;

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
        write!(f, "{}", String::from_utf8_lossy(&self.inner))
    }
}

/// A CSV row which may contain several CSV Fields
///
/// See [`Field`]
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub(crate) inner: Vec<u8>,
    pub(crate) ranges: Vec<(usize, usize)>,
    pub(crate) delim: char,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            ranges: Vec::new(),
            delim: DEFAULT_DELIM,
        }
    }
}
impl Row {
    /// Construct a simple empty CSV row
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize a row with an allocated capacity of fields.
    ///
    /// Useful to avoid multiple allocations.
    ///
    /// # Arguments
    /// `size`  Number of headers in the row.
    pub fn with_capacity(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
            ranges: Vec::new(),
            delim: DEFAULT_DELIM,
        }
    }

    /// Set the row's delimiter
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
    pub fn iter(&self) -> FieldsIter<'_> {
        FieldsIter {
            row: self,
            index: 0,
        }
    }

    /// Adds a  from a byte slice to the row.
    ///
    /// # Arguments:
    /// `field` A slice of bytes that can be turned into a string
    pub fn add_bytes(&mut self, field: &[u8]) {
        let start = self.inner.len();
        let length = field.len();
        self.ranges.push((start, start + length));
        self.inner.extend_from_slice(field)
    }

    /// Adds a [`Field`] to the  to the row.
    /// Type conversion is done behind the scines to turn the field into a string.
    /// Thus the field is required to impl the [`Display``] trait.
    ///
    /// # Arguments
    /// `field` value being added to the row.
    pub fn add<T>(&mut self, field: T)
    where
        T: Sized + Display,
    {
        let string_value = format!("{field}");
        self.add_bytes(string_value.as_bytes());
    }

    /// Remove a [`Field`] from the row.
    ///
    /// # Arguments:
    /// `index` index of the field within the row
    pub fn remove(&mut self, index: usize) {
        if index >= self.ranges.len() {
            return;
        }
        // can unwrap because we checked length
        let (start, end) = self.ranges.get(index).unwrap();
        let start = *start;
        let end = *end;

        // Move the ranges back by the same amount of bytes that are being removed
        for (i, range) in self.ranges.iter_mut().enumerate() {
            if i > index {
                range.0 -= end - start;
                range.1 -= end - start;
            }
        }

        // Now remove the bytes and the range
        self.inner.drain(start..end);
        self.ranges.remove(index);
    }

    pub fn replace<T>(&mut self, index: usize, new_field: T)
    where
        T: Sized + Display,
    {
        if index >= self.ranges.len() {
            return;
        }

        let mut row = Row::new();
        for (i, field) in self.iter().enumerate() {
            if i == index {
                row.add(&new_field);
                continue;
            }
            row.add_bytes(field.as_bytes());
        }
        std::mem::swap(self, &mut row)
    }

    /// Attempts to retrieve and cast a field to a given type.
    ///
    /// # Arguments
    /// `index` the index of the Field inside the row
    ///
    /// # Returns
    /// A result with either the casted field to type T or an error.
    ///
    /// # Examples:
    /// ```
    /// let row = csvlib::csv!["This is a row", 25, 56.2];
    ///
    /// assert_eq!(row.get::<String>(0).unwrap(), "This is a row".to_string());
    /// assert_eq!(row.get::<u32>(1).unwrap(), 25);
    /// assert_eq!(row.get::<f64>(2).unwrap(), 56.2);
    /// ```
    pub fn get<T: std::str::FromStr>(&self, index: usize) -> Result<T> {
        match self.ranges.get(index) {
            Some((start, end)) => {
                let field = &self.inner[*start..*end];
                let field_str = String::from_utf8_lossy(field).to_string();
                let parsed = field_str.parse::<T>().map_err(|_| {
                    CsvError::ConversionError(index, field_str, type_name::<T>().to_string())
                })?;
                Ok(parsed)
            }
            _ => Err(CsvError::NotAField(index)),
        }
    }

    pub fn get_range(&self, index: usize) -> Option<&[u8]> {
        match self.ranges.get(index) {
            Some((start, end)) => Some(&self.inner[*start..*end]),
            None => None,
        }
    }
    /// Retrieves the raw string value of a field at the given index.
    /// Returns an empty string if the index is out of bounds or the field is not valid UTF-8.
    pub fn get_value(&self, index: usize) -> Option<String> {
        match self.ranges.get(index) {
            Some((start, end)) => {
                Some(String::from_utf8_lossy(&self.inner[*start..*end]).to_string())
            }
            None => None,
        }
    }
    /// Retrieves the number of [`Field`]s in the row
    pub fn count(&self) -> usize {
        self.ranges.len()
    }
}
impl From<&[&str]> for Row {
    fn from(fields: &[&str]) -> Self {
        let mut row = Row::new();
        for field in fields {
            row.add_bytes(field.as_bytes());
        }
        row
    }
}
impl From<&[&String]> for Row {
    fn from(fields: &[&String]) -> Self {
        let mut row = Row::new();
        for field in fields {
            row.add_bytes(field.as_bytes());
        }
        row
    }
}
impl From<&Row> for Vec<String> {
    fn from(value: &Row) -> Self {
        value.iter().map(|f| f.to_string()).collect::<Vec<String>>()
    }
}

impl Index<usize> for Row {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        let (start, end) = self.ranges.get(index).expect("Index out of bounds in Row");
        std::str::from_utf8(&self.inner[*start..*end]).expect("Invalid UTF-8 in Row field")
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let last_index = self.ranges.len().saturating_sub(1);
        for (index, field) in self.iter().enumerate() {
            let field_value = field.to_string();

            // escape every single quote. This assumes what's present in each field
            // is what the user wants in it, no need for the user to escape things for us
            let field_value = field_value.replace('\"', "\"\"");

            // If we have quotes or commas, then we need outer double quotes in this field
            if field_value.contains(self.delim) || field_value.contains(QUOTE) {
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
    row: &'a Row,
    index: usize,
}

impl Iterator for FieldsIter<'_> {
    type Item = Field;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.row.get(self.index - 1).ok()
    }
}

/// Create a CSV [`row`] from a several CSV [`Field`]s.
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
            let mut row = $crate::Row::new();
            $(row.add_bytes(&format!("{}",$e).as_bytes());)*
            row
        }
    };
}

#[derive(Debug, PartialEq)]
pub enum CsvError {
    RecordError(String),
    ReadError(String),
    ConversionError(usize, String, String),
    InvalidString,
    FieldParseError(String, String),
    NotAField(usize),
    IOError(String),
    FileAccessError(String, String),
    InvalidColumn(String),
    InvalidRow(usize),
    InvalidColumnIndex(usize),
    Generic(String),
}

impl Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::RecordError(line) => write!(f, "Error parsing CSV row\n\t`{line}`."),
            CsvError::ReadError(error) => write!(f, "Error reading from source `{error}`."),
            CsvError::InvalidString => write!(f, "Cannot convert field to a valid string."),
            CsvError::NotAField(index) => write!(f, "Not field at given index `{index}`."),
            CsvError::ConversionError(field, value, type_name) => {
                write!(
                    f,
                    "Error parsing field  `{field}` with value `{value}` into `{type_name}`."
                )
            }
            CsvError::IOError(error) => write!(f, "Error accessing resource `{error}`."),
            CsvError::FieldParseError(value, type_name) => {
                write!(f, "Error parsing field `{value}` to `{type_name}`.")
            }
            CsvError::FileAccessError(file, reason) => {
                write!(f, "Error accessing file `{file}`: {reason}.")
            }
            CsvError::InvalidColumn(column) => {
                write!(f, "Invalid Column: `{column}`. Not found in document.")
            }
            CsvError::InvalidColumnIndex(column) => {
                write!(f, "Invalid Column: `{column}`. Not found in document.")
            }
            CsvError::InvalidRow(row) => {
                write!(f, "Invalid Row: `{row}`. Not found in document.")
            }
            CsvError::Generic(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<io::Error> for CsvError {
    fn from(e: io::Error) -> Self {
        CsvError::IOError(e.to_string())
    }
}

impl From<CsvError> for String {
    fn from(value: CsvError) -> Self {
        value.to_string()
    }
}

impl Error for CsvError {}

/// Trait for types that can be created from a CSV [`Row`].
pub trait FromRow {
    fn from(row: &Row) -> Result<Self>
    where
        Self: Sized;
}
