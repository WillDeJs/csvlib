use crate::{CsvError, Reader, Result, Row, Writer};
use std::{collections::HashMap, fmt::Display, fs::File, path::Path, slice::Iter, str::FromStr};

/// Simple document structure. This is merely an in-memory wrapper around a set of CSV Rows.
/// It offers functions to retrieve and write data as well as a way to serialize to a file.
///
/// As this structure lives in memory, beware not to overload RAM usage with large files.
/// For large files use regular lower level Writer/Reader structures.
///
/// # Example
/// ```rust
/// use csvlib::Document;
/// let mut doc = Document::with_headers(&["Name", "Age", "Email", "School"]);
/// doc.add_row(csvlib::csv![
///     "Mike",
///     15,
///     "kime@mail.com",
///     "Marktown High School"
/// ]);
/// doc.add_row(csvlib::csv![
///     "Jenny",
///     16,
///     "jeng@mail.com",
///     "Marktown High School"
/// ]);
/// doc.write_to_file("malist.csv")
///     .expect("Error writing to file");
/// ```
///
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Document {
    headers: Option<Row>,
    rows: Vec<Row>,
    header_indexes: HashMap<String, usize>,
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
    /// ```rust
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

    /// Create an empty document without headers
    pub fn empty() -> Self {
        Document::default()
    }

    /// Inserts a new row to the document.
    ///
    /// # Arguments
    /// `row` Row being inserted.
    pub fn add_row(&mut self, row: Row) {
        // TODO: Validate row length
        self.rows.push(row);
    }

    /// Inserts multiple rows to the document.
    ///
    /// # Arguments
    /// `row` Row being inserted.
    pub fn add_all(&mut self, rows: &[Row]) {
        self.rows.extend_from_slice(rows);
    }

    /// Remove anexisting row from a document.
    ///
    /// # Arguments
    /// `row` Row index being removed.
    pub fn remove_row(&mut self, row: usize) {
        if row < self.rows.len() {
            self.rows.remove(row);
        }
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
    /// ```rust
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
    pub fn get_value_at<T: std::str::FromStr>(&self, row: usize, column: usize) -> Result<T> {
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
    pub fn get_value<T: std::str::FromStr>(&self, row: usize, col_name: &str) -> Result<T> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.get_value_at::<T>(row, *col_index)
        } else {
            Err(CsvError::InvalidColumn(col_name.to_string()))
        }
    }

    ///  Get all the rows in the document that match the value passed in the selected column.
    ///
    /// # Arguments
    /// `col_name`  Name of the column to match.
    /// `value`     Value to match in each row.
    pub fn get_rows_where<T>(&self, col_name: &'static str, value: &T) -> Vec<&Row>
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
    pub fn get_rows_where_indexed<T>(&self, column: usize, value: &T) -> Vec<&Row>
    where
        T: Sized + Display + PartialEq + FromStr,
    {
        self.rows
            .iter()
            .filter(|row| row.get::<T>(column).as_ref() == Ok(value))
            .collect::<Vec<&Row>>()
    }

    /// Get the header row of the document.
    pub fn get_headers_row(&self) -> Row {
        if let Some(headers) = &self.headers {
            headers.clone()
        } else {
            Row::new()
        }
    }

    /// Get an iterator to all the rows in the document
    pub fn rows(&self) -> Iter<Row> {
        self.rows.iter()
    }

    /// Get the count of all rows in the document
    pub fn count(&self) -> usize {
        self.rows.len()
    }

    /// Check whether the given row evists in the document
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
    pub fn is_valid_column(&self, column: &'static str) -> bool {
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
    pub fn set_value_at<T>(&mut self, row: usize, column: usize, value: T)
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
    pub fn set_value<T: std::str::FromStr>(&mut self, row: usize, col_name: &'static str, value: T)
    where
        T: Sized + Display,
    {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.set_value_at::<T>(row, *col_index, value)
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
    /// ```rust
    /// use csvlib::Document;
    /// let mut doc = Document::with_headers(&["Name", "Age", "Email", "School"]);
    /// doc.add_row(csvlib::csv![
    ///     "Mike",
    ///     15,
    ///     "kime@mail.com",
    ///     "Marktown High School"
    /// ]);
    /// doc.add_row(csvlib::csv![
    ///     "Jenny",
    ///     16,
    ///     "jeng@mail.com",
    ///     "Marktown High School"
    /// ]);
    /// doc.write_to_file("malist.csv")
    ///     .expect("Error writing to file");
    /// ``````
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut writer = Writer::from_path(path)?;
        writer.write(&self.get_headers_row())?;
        for row in &self.rows {
            writer.write(row)?;
        }
        Ok(())
    }
}

impl TryFrom<Reader<File>> for Document {
    type Error = CsvError;
    fn try_from(reader: Reader<File>) -> Result<Self> {
        let headers = reader.headers();
        let rows = reader.entries().collect();
        let mut header_indexes = HashMap::new();
        if let Some(header) = &headers {
            for (index, value) in header.iter().enumerate() {
                let header_string_value = value.to_string().map_err(|_| {
                    CsvError::ConversionError(index, std::any::type_name::<String>().to_owned())
                })?;
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
