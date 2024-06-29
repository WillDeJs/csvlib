use crate::{CsvError, Reader, Result, Row, Writer};
use std::{collections::HashMap, fs::File, path::Path, slice::Iter};

/// Simple document structure. This is merely an in-memory wrapper around a set of CSV Rows.
/// It offers functions to retrieve and write data as well as a way to serialize to a file.
///
/// As this structure lives in memory, beware not to overload RAM usage with large files.
/// For large files use regular lower level Writer/Reader structures.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    headers: Option<Row>,
    rows: Vec<Row>,
    header_indexes: HashMap<String, usize>,
}

impl TryFrom<Reader<File>> for Document {
    type Error = CsvError<'static>;
    fn try_from(reader: Reader<File>) -> Result<Self> {
        let headers = reader.headers();
        let rows = reader.entries().collect();
        let mut header_indexes = HashMap::new();
        if let Some(header) = &headers {
            for (index, value) in header.iter().enumerate() {
                let header_string_value = value
                    .to_string()
                    .map_err(|_| CsvError::ConversionError(index, "String"))?;
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

impl Document {
    /// Create a new document.
    ///
    /// # Arguments
    /// `headers`   a slice of string literals containing the headers for this document.
    pub fn new(headers: &[&str]) -> Self {
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

    /// Inserts a new row to the document.
    ///
    /// # Arguments
    /// `row` Row being inserted.
    pub fn add_row(&mut self, row: Row) {
        // TODO: Validate row length
        self.rows.push(row);
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
    pub fn get_column<T: std::str::FromStr>(&self, col_name: &'static str) -> Result<Vec<T>> {
        if let Some(index) = self.header_indexes.get(col_name) {
            self.get_column_by_index(*index)
        } else {
            Err(CsvError::InvalidColumn(col_name))
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
    pub fn get_value<T: std::str::FromStr>(&self, row: usize, col_name: &'static str) -> Result<T> {
        if let Some(col_index) = self.header_indexes.get(col_name) {
            self.get_value_at::<T>(row, *col_index)
        } else {
            Err(CsvError::InvalidColumn(col_name))
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

    /// Get the header row of the document.
    pub fn get_headers_row(&self) -> Row {
        if let Some(headers) = &self.headers {
            headers.clone()
        } else {
            Row::new()
        }
    }

    /// Write the contents of this header to the given file.
    ///
    ///  # Arguments path
    /// `path`  File path for the file to write the document.
    ///
    /// # Errors
    /// If writing to file fails for IO related reasons.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut writer = Writer::from_path(path)?;
        writer.write(&self.get_headers_row())?;
        for row in &self.rows {
            writer.write(row)?;
        }
        Ok(())
    }
}
