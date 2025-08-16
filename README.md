# CSVLIB #
A simple Rust CSV Reader/Writer library with a simple API. Implements `Reader`, `Writer` and `Document` structures.

## Reader ##
Read and iterate throw CSV files easily. See CSV Reader example below.
### Example (Reader): ###
 ``` rs
// Open a CSV file to read.
let csv_reader = csvlib::Reader::from_path("./AAPL.csv").unwrap();

// Iterate through rows
println!("{}", csv_reader.headers().unwrap());
for entry in csv_reader.entries() {
    println!("{}", entry);
}
 ```

## Writer ##
Create CSV files and write rows easily. See CSV Writer example.
### Example (Writer): ###
 ``` rs
// Create a writer from a file path
let mut writer = csvlib::Writer::from_path("./test.csv").unwrap();

// Write rows to file
writer.write_all(&[
        csvlib::csv!["Header1", "Header2", "Header3"]
        csvlib::csv!["entry11", "entry12", "entry13"],
        csvlib::csv!["entry21", "entry22", "entry23"],
        csvlib::csv!["entry31", "entry32", "entry33"],
        csvlib::csv!["entry41", "entry42", "entry43"],
    ])
    .unwrap();
```

## Document ##
Easily open a document and search through it.
```rs
 use csvlib::Document;
 let doc = Document::from_path("students.csv").expect("Could not open file");

 // Get some field values
 let ages = doc.get_column::<i32>("Age").unwrap();
 let emails = doc.get_column::<String>(&String::from("Email")).unwrap();
 let schools = doc.get_column::<String>("School").unwrap();
```
Additionally modify, filter  and save values inside of the document and make copies if needed.


```rs
use csvlib::{CsvError, Document};

fn main() -> Result<(), CsvError> {
    // Open document
    let mut students_document = Document::from_path(r#"students.csv"#).unwrap();

    // Filter the document based on the desired criteria
    students_document.retain(|entry| {
        // Keep students only from Springfield high
        match entry.get::<String>("school") {
            Ok(school_name) => school_name == "Springfield High School",
            _ => false,
        }
    });

    // Iterate through the rows in the document
    for mut student in students_document.rows_mut() {
        let name = student.get::<String>("name")?;
        let last_name = student.get::<String>("lastname")?;

        // do intended work
        ...
       
    }

    // Save filtered results
    students_document.write_to_file("Springfield Students.csv")
}
```