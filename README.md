A simple CSV Reader/Writer library created for personal projects. Written in rust. 
 # Example (Writer):
 ``` rs
fn main() { 

  // Write to file
    let mut writer = csvlib::Writer::from_path("./test.txt").unwrap();

    // Create custom records
    let header = csvlib::csv!["Header1", "Header 2", "Header,3"];
    writer.write(&header).unwrap();
    writer
        .write_all(&[
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
    // Read from a file
    let csv_reader = csvlib::Reader::from_path("./AAPL.csv").unwrap();

    // Iterate through rows
    println!("{}", csv_reader.headers().unwrap());
    for entry in csv_reader.entries() {
        println!("{}", entry);
    }
}
 ```
