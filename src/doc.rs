use crate::{CsvError, Reader, Result, Row, Writer};
use std::ops::Index;
use std::{
    collections::{hash_map::Keys, HashMap},
    fmt::{Debug, Display},
    path::Path,
    slice::{Iter, IterMut},
    str::FromStr,
};

/// Simple document structure. This is merely an in-memory wrapper around a set of CSV Rows.
/// It offers functions to retrieve and write data as well as a way to serialize to a file.
///
/// As this structure lives in memory, beware not to overload RAM usage with large files.
/// For large files use regular lower level Writer/Reader structures.
///
/// # Example
/// ```no_run
/// use csvlib::Document;
/// let mut doc = Document::with_headers(&["Name", "Age", "Email", "School"]);
/// doc.insert(csvlib::csv![
///     "Mike",
///     15,
///     "kime@mail.com",
///     "Marktown High School"
/// ]);
/// doc.insert(csvlib::csv![
///     "Jenny",
///     16,
///     "jeng@mail.com",
///     "Marktown High School"
/// ]);
/// doc.write_to_file("mail_list.csv")
///     .expect("Error writing to file");
/// ```
///
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    pub(crate) headers: Option<Row>,
    pub(crate) rows: Vec<Row>,
    pub(crate) header_indexes: HashMap<String, usize>,
}

impl Document {
    /// Create a new document. with the given headers.
    ///
    /// # Arguments
    /// `headers`   a slice of string literals containing the headers for this document.
    pub fn with_headers(headers: &[&str]) -> Self {
        let mut header_indexes = HashMap::new();
        for (index, value) in headers.iter().enumerate() {
            header_indexes.insert(value.to_string(), index);
        }

        Document {
            headers: Some(Row::from(headers)),
            rows: Vec::new(),
            header_indexes,
        }
    }

    /// Create a document for a given path.
    ///
    /// # Arguments
    /// `path` path/string to file to be read.
    ///
    /// # Errors
    /// If file cannot be accessible or does not exist.
    /// If file is not valid CSV and cannot be parsed.
    ///
    /// # Example:
    /// ```no_run
    /// use csvlib::Document;
    /// let doc = Document::from_path("filename.csv").expect("Could not open file");
    ///
    /// // Filter some column values
    /// let ages = doc.get_column::<i32>("Age").unwrap();
    /// let emails = doc.get_column::<String>(&String::from("Email")).unwrap();
    /// let schools = doc.get_column::<String>("School").unwrap();
    ///
    /// ```
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let reader = Reader::from_path(path)?;
        Document::try_from(reader)
    }

    /// Create a document for a given path but only takes rows
    /// according to the given filter.
    ///
    /// # Arguments
    /// `path` path/string to file to be read.
    /// `filter` filtering function to be applied to each [DocEntry]
    ///
    /// # Errors
    /// If file cannot be accessible or does not exist.
    /// If file is not valid CSV and cannot be parsed.
    pub fn from_path_filtered<F>(path: impl AsRef<Path>, filter: F) -> Result<Self>
    where
        F: Fn(&DocEntry) -> bool,
    {
        let reader = Reader::from_path(path)?;
        let headers = reader.headers();
        let mut header_indexes = HashMap::new();
        if let Some(header) = &headers {
            for (index, value) in header.iter().enumerate() {
                let header_string_value = value.to_string();
                header_indexes.insert(header_string_value, index);
            }
        }
        let rows = reader
            .entries()
            .filter(|row| {
                let doc_entry = DocEntry {
                    headers: &headers,
                    row,
                    header_indexes: &header_indexes,
                };
                filter(&doc_entry)
            })
            .collect();
        Ok(Document {
            headers,
            rows,
            header_indexes,
        })
    }

    /// Create an empty document without headers
    pub fn empty() -> Self {
        Document::default()
    }

    /// Inserts a new row to the document.
    ///
    /// # Arguments
    /// `row` Row being inserted.
    pub fn insert<T>(&mut self, row: T)
    where
        T: Into<Row>,
    {
        // TODO: Validate row length
        self.rows.push(row.into());
    }

    /// Inserts multiple rows to the document.
    ///
    /// # Arguments
    /// `row` Row being inserted.
    pub fn insert_all(&mut self, rows: &[Row]) {
        self.rows.extend_from_slice(rows);
    }

    /// Append another document.
    ///
    /// # Arguments
    /// `doc` Document being appended.
    pub fn append(&mut self, doc: &Document) {
        self.insert_all(&doc.rows);
    }

    /// Remove an existing row from a document.
    ///
    /// # Arguments
    /// `row` Row index being removed.
    pub fn remove_row(&mut self, row: usize) {
        if row < self.rows.len() {
            self.rows.remove(row);
        }
    }

    ///  Remove all the rows in the document that match the value passed in the selected column.
    ///
    /// # Arguments
    /// `col_name`  Name of the column to match.
    /// `value`     Value to match in each row.
    pub fn remove_where<T>(&mut self, col_name: &str, value: &T)
    where
        T: Display + PartialEq,
    {
        if let Some(column) = self.header_indexes.get(col_name) {
            self.rows
                .retain(|row| row.get::<String>(*column) != Ok(value.to_string()));
        }
    }

    ///  Retain all the rows in the document that match the predicate function called.
    ///
    /// # Arguments
    /// `predicate`  Predicate function to test each `DocEntry` against. Used to determine match.
    ///
    /// # Examples:
    ///
    /// ```no_run
    /// use csvlib::Document;
    ///
    /// // Open document
    /// let mut document = Document::from_path(r#"students.csv"#).unwrap();
    ///
    /// // Filter students from the school
    /// document.retain(|entry| {
    ///     entry.get::<String>("School") == Ok(String::from("Springfield High School"))
    /// });
    ///
    /// // Save filtered student list
    /// document.write_to_file("springfield_list.csv").unwrap();
    ///
    /// ```
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&DocEntry) -> bool,
    {
        self.rows.retain(|row| {
            // Wrap Row into DocEntry, to allow use of headers in querying
            let entry = DocEntry {
                headers: &self.headers,
                row,
                header_indexes: &self.header_indexes,
            };
            predicate(&entry)
        })
    }

    /// Get the given column for every row in the document.
    ///
    /// # Arguments
    /// `col_name` name of the column being searched.
    ///
    /// # Errors
    /// If the given column name does not exist in the document
    /// or if the data cannot properly be parsed into the type T.
    ///
    /// # Example:
    /// ```no_run
    /// use csvlib::Document;
    /// let doc = Document::from_path("filename.csv").expect("Could not open file");
    ///
    /// // Filter some column values
    /// let ages = doc.get_column::<i32>("Age").unwrap();
    /// let emails = doc.get_column::<String>(&String::from("Email")).unwrap();
    /// let schools = doc.get_column::<String>("School").unwrap();
    ///
    /// ```
    pub fn get_column<T: std::str::FromStr>(&self, col_name: &str) -> Result<Vec<T>> {
        if let Some(index) = self.header_indexes.get(col_name) {
            self.get_column_by_index(*index)
        } else {
            Err(CsvError::InvalidColumn(col_name.to_string()))
        }
    }

    /// Get the given column for every row in the document by using the column index.
    ///
    /// # Arguments
    /// `column` index of the column being searched.
    ///
    /// # Errors
    /// If the given column index does not exist in the row
    /// or if the data cannot properly be parsed into the type T.
    pub fn get_column_by_index<T: std::str::FromStr>(&self, column: usize) -> Result<Vec<T>> {
        let mut result_vec = Vec::new();
        for row in &self.rows {
            result_vec.push(row.get(column)?);
        }
        Ok(result_vec)
    }

    /// Get the value at the given row-column intersection.
    ///
    /// # Arguments
    /// `row`   index of the row being searched.
    /// `column` index of the column being searched.
    ///
    /// # Errors
    /// If the given column or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn get_indexed<T: std::str::FromStr>(&self, row: usize, column: usize) -> Result<T> {
        if let Some(row) = self.rows.get(row) {
            row.get::<T>(column)
        } else {
            Err(CsvError::InvalidRow(row))
        }
    }

    /// Get the value at the given row-column intersection. This time the column is given as a string.
    ///
    /// # Arguments
    /// `row`   index of the row being searched.
    /// `col_name` name of the column being searched.
    ///
    /// # Errors
    /// If the given column name or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn get<T: std::str::FromStr>(&self, row: usize, col_name: &str) -> Result<T> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.get_indexed::<T>(row, *col_index)
        } else {
            Err(CsvError::InvalidColumn(col_name.to_string()))
        }
    }

    ///  Get all the rows in the document that match the value passed in the selected column.
    ///
    /// # Arguments
    /// `col_name`  Name of the column to match.
    /// `value`     Value to match in each row.
    pub fn get_rows_where<T>(&self, col_name: &str, value: &T) -> Vec<DocEntry<'_>>
    where
        T: Sized + Display + PartialEq + FromStr,
    {
        if let Some(column) = self.header_indexes.get(col_name) {
            self.get_rows_where_indexed(*column, value)
        } else {
            Vec::new()
        }
    }

    ///  Get all the rows in the document that match the value passed in the selected column.
    /// This does the same as [`get_rows_where`]` but instead of column name being a string,
    /// it must be an integer.
    ///
    /// # Arguments
    /// `column`    Index of the column to match.
    /// `value`     Value to match in each row.
    pub fn get_rows_where_indexed<T>(&self, column: usize, value: &T) -> Vec<DocEntry<'_>>
    where
        T: Sized + Display + PartialEq + FromStr,
    {
        self.rows
            .iter()
            .filter(|row| row.get::<T>(column).as_ref() == Ok(value))
            .map(|row| DocEntry {
                headers: &self.headers,
                row,
                header_indexes: &self.header_indexes,
            })
            .collect::<Vec<DocEntry>>()
    }

    /// Get the header row of the document.
    pub fn get_header_row(&self) -> &Option<Row> {
        &self.headers
    }

    /// Get the header row of the document.
    pub fn get_header_names(&self) -> Option<Vec<String>> {
        if let Some(headers) = &self.headers {
            Some(headers.into())
        } else {
            None
        }
    }

    /// Get an iterator to all the rows in the document
    pub fn rows(&self) -> DocIter<'_> {
        DocIter {
            headers: &self.headers,
            header_indexes: &self.header_indexes,
            iter: self.rows.iter(),
        }
    }
    /// Get a mutable iterator to all the rows in the document
    pub fn rows_mut(&mut self) -> DocIterMut<'_> {
        DocIterMut {
            headers: &self.headers,
            header_indexes: &self.header_indexes,
            iter: self.rows.iter_mut(),
        }
    }

    /// Iterate over all rows, decoding them into the given type T.
    /// An iterator of Results is returned.
    ///
    /// # Example
    /// ```no_run
    /// use csvlib::{CsvError, DocEntry, Document};
    /// pub struct Person {
    ///     pub name: String,
    ///     pub last_name: String,
    ///     pub age: u32,
    ///     pub email: String,
    /// }
    ///
    /// impl TryFrom<DocEntry<'_>> for Person {
    ///     type Error = CsvError;
    ///     fn try_from(entry: DocEntry) -> Result<Self, CsvError> {
    ///         Ok(Person {
    ///             name: entry.get::<String>("name")?,
    ///             age: entry.get::<u32>("age")?,
    ///             last_name: entry.get::<String>("last_name")?,
    ///             email: entry.get::<String>("email")?,
    ///         })
    ///     }
    /// }
    /// fn main() {
    ///     let document = Document::from_path("people.csv").unwrap();
    ///     let mut total_age = 0;
    ///     let mut count = 0;
    ///     // Use the rows() iterator and decode each DocEntry manually
    ///     for person_res in document.rows_decoded::<Person>() {
    ///         let person = person_res.unwrap();
    ///         total_age += person.age;
    ///         count += 1;
    ///     }
    ///     let average_age = total_age as f32 / count as f32;
    ///     println!("Average age: {}", average_age);
    /// }
    /// ```
    pub fn rows_decoded<T>(&self) -> impl Iterator<Item = Result<T>> + use<'_, T>
    where
        for<'a> T: TryFrom<DocEntry<'a>, Error = CsvError>,
    {
        self.rows().map(move |entry| T::try_from(entry))
    }

    /// Get the count of all rows in the document
    pub fn count(&self) -> usize {
        self.rows.len()
    }

    /// Check whether the given row exists in the document
    ///
    /// # Arguments
    /// `row`    row index to search
    pub fn is_valid_row_index(&self, row: usize) -> bool {
        self.rows.len() > row
    }

    /// Check whether the given column exists in the document.
    ///
    /// # Arguments
    /// `column`    column index to search
    pub fn is_valid_column_index(&self, column: usize) -> bool {
        self.header_indexes.values().any(|col| col == &column)
    }

    /// Check whether the given column name exists in the document.
    ///
    /// # Arguments
    /// `column`    column name to search
    pub fn is_valid_column(&self, column: &str) -> bool {
        self.header_indexes.contains_key(column)
    }

    // Set the value at the given row-column intersection.
    ///
    /// # Arguments
    /// `row`   index of the row being searched.
    /// `column` index of the column being searched.
    /// `value` Value to set on the row.
    ///
    /// # Errors
    /// If the given column or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn set_indexed<T>(&mut self, row: usize, column: usize, value: T)
    where
        T: Sized + Display,
    {
        if let Some(row) = self.rows.get_mut(row) {
            row.replace(column, value);
        }
    }

    /// Get the value at the given row-column intersection. This time the column is given as a string.
    ///
    /// # Arguments
    /// `row`   index of the row being searched.
    /// `col_name` name of the column being searched.
    ///
    /// # Errors
    /// If the given column name or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn set<T>(&mut self, row: usize, col_name: &str, value: T)
    where
        T: Sized + Display + std::str::FromStr,
    {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.set_indexed::<T>(row, *col_index, value)
        }
    }

    /// Write the contents of this header to the given file.
    ///
    ///  # Arguments path
    /// `path`  File path for the file to write the document.
    ///
    /// # Errors
    /// If writing to file fails for IO related reasons.
    ///
    /// # Example
    /// ```no_run
    /// use csvlib::Document;
    /// let mut doc = Document::with_headers(&["Name", "Age", "Email", "School"]);
    /// doc.insert(csvlib::csv![
    ///     "Mike",
    ///     15,
    ///     "kime@mail.com",
    ///     "Marktown High School"
    /// ]);
    /// doc.insert(csvlib::csv![
    ///     "Jenny",
    ///     16,
    ///     "jeng@mail.com",
    ///     "Marktown High School"
    /// ]);
    /// doc.write_to_file("mail_list.csv")
    ///     .expect("Error writing to file");
    /// ``````
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut writer = Writer::from_path(path)?;
        if let Some(header) = self.headers.as_ref() {
            writer.write(header)?;
        }
        for row in &self.rows {
            writer.write(row)?;
        }
        Ok(())
    }
}

impl<T> TryFrom<Reader<T>> for Document
where
    T: std::io::Read,
{
    type Error = CsvError;
    fn try_from(reader: Reader<T>) -> Result<Self> {
        let headers = reader.headers();
        let rows = reader.entries().collect();
        let mut header_indexes = HashMap::new();
        if let Some(header) = &headers {
            for (index, value) in header.iter().enumerate() {
                let header_string_value = value.to_string();
                header_indexes.insert(header_string_value, index);
            }
        }
        Ok(Document {
            headers,
            rows,
            header_indexes,
        })
    }
}

pub struct DocEntry<'a> {
    pub(crate) headers: &'a Option<Row>,
    pub(crate) row: &'a Row,
    pub(crate) header_indexes: &'a HashMap<String, usize>,
}

impl DocEntry<'_> {
    /// Get the value at the current row-column intersection. This time the column is given as a string.
    ///
    /// # Arguments
    /// `col_name` name of the column being searched.
    ///
    /// # Errors
    /// If the given column name or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn get<T: std::str::FromStr>(&self, col_name: &str) -> Result<T> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.row.get::<T>(*col_index)
        } else {
            Err(CsvError::InvalidColumn(col_name.to_string()))
        }
    }
    /// Get the raw string value at the current row-column intersection.
    /// # Arguments
    /// `col_name`
    /// # Returns
    /// An optional string if the column exists
    pub fn get_value(&self, col_name: &str) -> Option<String> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            // Assuming Row implements Index<usize, Output = String>
            self.row.get_value(*col_index)
        } else {
            None
        }
    }
    /// Get a an iterator of all the columns in this row.
    pub fn columns(&self) -> Keys<'_, String, usize> {
        self.header_indexes.keys()
    }
}
impl Debug for DocEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last = self.header_indexes.len() - 1;
        write!(f, "{{")?;
        for (item, (column, index)) in self.header_indexes.iter().enumerate() {
            if item < last {
                write!(
                    f,
                    "{column}: {}, ",
                    self.row.get::<String>(*index).unwrap_or_default()
                )?;
            } else {
                write!(
                    f,
                    "{column}: {}",
                    self.row.get::<String>(*index).unwrap_or_default()
                )?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}
impl Display for DocEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.row, f)
    }
}

pub struct DocIter<'a> {
    pub(crate) headers: &'a Option<Row>,
    pub(crate) iter: Iter<'a, Row>,
    pub(crate) header_indexes: &'a HashMap<String, usize>,
}

impl<'a> Iterator for DocIter<'a> {
    type Item = DocEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row) = self.iter.next() {
            Some(DocEntry {
                headers: self.headers,
                row,
                header_indexes: self.header_indexes,
            })
        } else {
            None
        }
    }
}
pub struct DocEntryMut<'a> {
    headers: &'a Option<Row>,
    pub(crate) row: &'a mut Row,
    pub(crate) header_indexes: &'a HashMap<String, usize>,
}

impl DocEntryMut<'_> {
    /// Get the value at the current row-column intersection.
    ///
    /// # Arguments
    /// `col_name` name of the column being searched.
    ///
    /// # Errors
    /// If the given column name or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn get<T: std::str::FromStr>(&self, col_name: &str) -> Result<T> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.row.get::<T>(*col_index)
        } else {
            Err(CsvError::InvalidColumn(col_name.to_string()))
        }
    }

    /// Get the raw string value at the current row-column intersection.
    /// # Arguments
    /// `col_name`
    /// # Returns
    /// An optional string if the column exists
    pub fn get_raw(&self, col_name: &str) -> Option<String> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            // Assuming Row implements Index<usize, Output = String>
            self.row.get_value(*col_index)
        } else {
            None
        }
    }

    /// Get the value at the current row-column intersection. .
    ///
    /// # Arguments
    /// `col_name` name of the column being searched.
    ///
    /// # Errors
    /// If the given column name or row index does not exist.
    /// or if the data cannot properly be parsed into the type T.
    pub fn set<T>(&mut self, col_name: &str, value: T)
    where
        T: Sized + Display,
    {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.row.replace(*col_index, value);
        }
    }

    /// Get a an iterator of all the columns in this row.
    pub fn columns(&mut self) -> Keys<'_, String, usize> {
        self.header_indexes.keys()
    }
}

impl Debug for DocEntryMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let last = self.header_indexes.len() - 1;
        write!(f, "{{")?;
        for (item, (column, index)) in self.header_indexes.iter().enumerate() {
            if item < last {
                write!(
                    f,
                    "{column}: {}, ",
                    self.row.get::<String>(*index).unwrap_or_default()
                )?;
            } else {
                write!(
                    f,
                    "{column}: {}",
                    self.row.get::<String>(*index).unwrap_or_default()
                )?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl Display for DocEntryMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.row, f)
    }
}

impl<'a> Index<&str> for DocEntryMut<'a> {
    type Output = str;

    fn index(&self, col_name: &str) -> &Self::Output {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            // Assuming Row implements Index<usize, Output = String>
            // and we want to return &str
            &self.row.index(*col_index)
        } else {
            panic!("Invalid column name: {}", col_name);
        }
    }
}
impl<'a> Index<&str> for DocEntry<'a> {
    type Output = str;

    fn index(&self, col_name: &str) -> &Self::Output {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            // Assuming Row implements Index<usize, Output = String>
            // and we want to return &str
            &self.row.index(*col_index)
        } else {
            panic!("Invalid column name: {}", col_name);
        }
    }
}

pub struct DocIterMut<'a> {
    headers: &'a Option<Row>,
    iter: IterMut<'a, Row>,
    pub(crate) header_indexes: &'a HashMap<String, usize>,
}

impl<'a> Iterator for DocIterMut<'a> {
    type Item = DocEntryMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row) = self.iter.next() {
            Some(DocEntryMut {
                headers: self.headers,
                row,
                header_indexes: self.header_indexes,
            })
        } else {
            None
        }
    }
}

impl<'a> FromIterator<DocEntry<'a>> for Document {
    fn from_iter<T: IntoIterator<Item = DocEntry<'a>>>(iter: T) -> Self {
        let mut doc = Document::default();
        for entry in iter {
            // Copy headers and header_indexes from the first entry if not set
            if doc.headers.is_none() {
                if let Some(headers) = entry.headers.as_ref() {
                    doc.headers = Some(headers.clone());
                }
                doc.header_indexes = entry.header_indexes.clone();
            }
            doc.rows.push(entry.row.clone());
        }
        doc
    }
}

impl<'a> FromIterator<DocEntryMut<'a>> for Document {
    fn from_iter<T: IntoIterator<Item = DocEntryMut<'a>>>(iter: T) -> Self {
        let mut doc = Document::default();
        for entry in iter {
            // Copy headers and header_indexes from the first entry if not set
            if doc.headers.is_none() {
                if let Some(headers) = entry.headers.as_ref() {
                    doc.headers = Some(headers.clone());
                }
                doc.header_indexes = entry.header_indexes.clone();
            }
            doc.rows.push(entry.row.clone());
        }
        doc
    }
}
