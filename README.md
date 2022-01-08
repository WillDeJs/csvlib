A simple CSV Reader/Writer library created for personal projects. Written in rust. 
 # Example (Writer):
 ```
fn main() { 

// Write to file
let mut writer = csvlib::Writer::from_writer(std::fs::File::create("./test.txt").unwrap());

// Create custom records
let header = csvlib::make_record!["Header1", "Header 2", "Header,3"];
writer.write_record(header).unwrap();
writer
    .write_all_records(vec![
        csvlib::make_record!["Header1", "Header 2", "Header,3"],
        csvlib::make_record!["entry", "entry", "entry"],
        csvlib::make_record!["entry", "entry", "entry"],
        csvlib::make_record!["entry", "entry", "entry"],
        csvlib::make_record!["entry", "entry", "entry"],
    ])
    .unwrap();
}
```
 # Example (Reader):
 ```
fn main() {
    // Read from files
    let file = std::fs::File::open("./TSLA.csv").unwrap();
    // create custom records
    let record = csvlib::make_record!["Intr,o", 34, "klk", "manito"];
    // Parse record fields
    println!("Got: {}", record.get_casted::<u32>(1).unwrap());
    println!("{}", record);
    // Iterate through records
    let mut csv_reader = csvlib::Reader::builder()
        .with_delim(',')
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