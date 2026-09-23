use csvlib::{Document, Result, Row};

pub struct Person {
    pub name: String,
    pub last_name: String,
    pub age: u32,
    pub email: String,
}

impl TryFrom<Row> for Person {
    type Error = csvlib::CsvError;
    fn try_from(value: Row) -> std::result::Result<Self, Self::Error> {
        Ok(Person {
            // Using column indices. Adjust indices to match your CSV header order.
            name: value.get::<String>(0)?,
            last_name: value.get::<String>(1)?,
            age: value.get::<u32>(2)?,
            email: value.get::<String>(3)?,
        })
    }
}

fn main() -> Result<()> {
    // Use the low-level Reader which yields `Row`s (the "Rows" iterator)
    let mut doc = Document::from_path("large_user_data.csv").expect("Could not open file");

    doc.remove_column("Age");
    doc.write_to_file("no_age_mail_list.csv")?;
    let mut value = 0;
    doc.add_column_with("sortno", move || {
        value += 1;
        value
    });

    doc.write_to_file("with_sortno.csv")?;
    Ok(())
}
