A simple CSV Reader/Writer library created for personal projects. 
 # Example (Writer):
 ``` rs
fn main() { 

  // Write to file
    let mut writer = csvlib::Writer::from_path("./test.txt").unwrap();

    // Create custom rows
    let header = csvlib::csv!["Header1", "Header2", "Header3"];
    writer.write(&header).unwrap();
    writer
        .write_all(&[
            csvlib::csv!["entry11", "entry12", "entry13"],
            csvlib::csv!["entry21", "entry22", "entry23"],
            csvlib::csv!["entry31", "entry32", "entry33"],
            csvlib::csv!["entry41", "entry42", "entry43"],
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
