A simple CSV Reader/Writer library created for personal projects. Written in rust. 
 # Example (Writer):
 ``` rs
fn main() { 

// Write to file
let mut writer = csvlib::Writer::from_writer(std::fs::File::create("./test.txt").unwrap());

// Create custom records
let header = csvlib::csv!["Header1", "Header 2", "Header,3"];
writer.write(header).unwrap();
writer
    .write_all(vec![
        csvlib::csv!["Header1", "Header 2", "Header,3"],
        csvlib::csv!["entry", "entry", "entry"],
        csvlib::csv!["entry", "entry", "entry"],
        csvlib::csv!["entry", "entry", "entry"],
        csvlib::csv!["entry", "entry", "entry"],
    ])
    .unwrap();
}
```
 # Example (Reader):
 ``` rs
fn main() {
    // Read from files
    let file = std::fs::File::open("./TSLA.csv").unwrap();
    // create custom records
    let record = csvlib::csv!["Intr,o", 34, "klk", "manito"];
    // Parse record fields
    println!("Got: {}", record.get_casted::<u32>(1).unwrap());
    println!("{}", record);
    // Iterate through records
    let mut csv_reader = csvlib::Reader::builder()
        .with_delimiter(',')
        .with_reader(file)
        .with_header(true)
        .build()
        .unwrap();
    println!("{}", csv_reader.header().unwrap());
    for entry in csv_reader.entries() {
        println!("{}", entry);
    }
}
 ```
