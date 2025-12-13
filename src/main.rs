use csvlib::{Reader, Result, Row};

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
    let reader = Reader::from_path("people.csv")?;

    let mut total_age: u32 = 0;
    let mut count: u32 = 0;

    for person_res in reader.entries_decoded::<Person>() {
        let person = person_res?;
        total_age += person.age;
        count += 1;
    }

    if count == 0 {
        println!("No people found");
        return Ok(());
    }

    let average_age = total_age as f32 / count as f32;
    println!("Average age: {}", average_age);

    Ok(())
}
