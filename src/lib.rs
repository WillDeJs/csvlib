//! [crate]
//! A simple CSV Reader/Writer library created for personal projects.
//!
//! # Example (Writer):
//! ```no_run
//!
//!
//! // Write to file
//! let mut writer = csvlib::Writer::from_writer(std::fs::File::create("./test.txt").unwrap());
//!
//! // Create custom rows
//! let header = csvlib::csv!["Header1", "Header 2", "Header,3"];
//! writer.write(&header).unwrap();
//! writer
//!     .write_all(&vec![
//!         csvlib::csv!["Header1", "Header 2", "Header,3"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!         csvlib::csv!["entry", "entry", "entry"],
//!     ])
//!     .unwrap();
//!
//!```
//! # Example (Reader):
//! ```no_run
//!     // create custom rows
//!     let row = csvlib::csv!["Intr,o", 34, "klk", "manito"];
//!
//!     // Parse row fields
//!     println!("Got: {}", row.get::<u32>(1).unwrap());
//!     println!("{}", row);
//!
//!     // Iterate through rows
//!     let mut csv_reader = csvlib::Reader::from_path("./TSLA.csv")
//!         .unwrap();
//!
//!     println!("{}", csv_reader.headers().unwrap());
//!     for entry in csv_reader.entries() {
//!         println!("{}", entry);
//!     }
//!
//! ```

pub mod doc;
pub mod reader;
pub mod types;
pub mod writer;

pub use doc::*;
pub use reader::*;
pub use types::*;
pub use writer::*;
