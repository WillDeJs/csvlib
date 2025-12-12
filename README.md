# CSVLIB #
A simple Rust CSV Reader/Writer library with a simple API. Implements `Reader`, `Writer` and `Document` structures.

## Example 1: Reading a file:
 ```rust
     // create custom rows
     let row = csvlib::csv!["Intr,o", 34, "klk", "manito"];

     // Parse row fields
     println!("Got: {}", row.get::<u32>(1).unwrap());
     println!("{}", row);

     // Iterate through rows
     let mut csv_reader = csvlib::Reader::from_path("./TSLA.csv")
         .unwrap();

     println!("{}", csv_reader.headers().unwrap());
     for entry in csv_reader.entries() {
         println!("{}", entry);
     }

 ```
 ## Example 2: Writing to a file:
 ```rust
 // Write to file
 let mut writer = csvlib::Writer::from_writer(std::fs::File::create("./test.txt").unwrap());

 // Create custom rows
 let header = csvlib::csv!["Header1", "Header 2", "Header,3"];
 writer.write(&header).unwrap();
 writer
     .write_all(&vec![
         csvlib::csv!["Header1", "Header 2", "Header,3"],
         csvlib::csv!["entry", "entry", "entry"],
         csvlib::csv!["entry", "entry", "entry"],
         csvlib::csv!["entry", "entry", "entry"],
         csvlib::csv!["entry", "entry", "entry"],
     ])
     .unwrap();

```
 ## Example 3: Writing to a Document:
 ```rust
 use csvlib::Document;
 let mut doc = Document::with_headers(&["Name", "Age", "Email", "School"]);
 doc.insert(csvlib::csv![
     "Mike",
     15,
     "kime@mail.com",
     "Marktown High School"
 ]);
 doc.insert(csvlib::csv![
     "Jenny",
     16,
     "jeng@mail.com",
     "Marktown High School"
 ]);
 doc.write_to_file("malist.csv")
     .expect("Error writing to file");
 ```

 ##  Example 4: Reading decoded rows from a document:
 ```rust
 use csvlib::{CsvError, DocEntry, Document, FromDocEntry};
 pub struct Person {
     pub name: String,
     pub last_name: String,
     pub age: u32,
     pub email: String,
 }

 impl FromDocEntry for Person {
     fn from(entry: &DocEntry) -> Result<Self, CsvError> {
         Ok(Person {
             name: entry.get::<String>("name")?,
             age: entry.get::<u32>("age")?,
             last_name: entry.get::<String>("last_name")?,
             email: entry.get::<String>("email")?,
         })
     }
 }
 fn main() {
     let document = Document::from_path("people.csv").unwrap();
     let mut total_age = 0;
     let mut count = 0;
     for person in document.rows_decoded::<Person>() {
         let person = person.unwrap();
         total_age += person.age;
         count += 1;
     }
     let average_age = total_age as f32 / count as f32;
     println!("Average age: {}", average_age);
 }
```