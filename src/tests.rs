use csvlib::{reader::Reader, Document, Row};

#[test]
fn test_well_formed_csv_no_commas_no_quotes() {
    let data = r#"header1,header2,header3,header4
r1c1,r1c2,r1c3,r1c4
r2c1,r2c2,r2c3,r2c4
r3c1,r3c2,r3c3,r3c4"#;
    let input = std::io::Cursor::new(data);
    let reader = Reader::builder()
        .with_header(true)
        .with_reader(input)
        .build()
        .expect("could not create reader.");
    let header = reader.headers();
    let rows: Vec<_> = reader.entries().collect();

    assert_eq!(header.unwrap().count(), 4);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].count(), 4);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "r1c1".to_owned());
    assert_eq!(rows[1].get::<String>(1).unwrap(), "r2c2".to_owned());
}

#[test]
fn test_well_formed_csv_with_number_fields() {
    let data = r#"header1,header2,header3,header4
11,12,13,14
21,22,23,24
31,32,33,34"#;
    let input = std::io::Cursor::new(data);
    let reader = Reader::builder()
        .with_header(true)
        .with_reader(input)
        .build()
        .expect("could not create reader.");

    let rows: Vec<_> = reader.entries().collect();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].count(), 4);
    assert_eq!(rows[0].get::<i32>(0).unwrap(), 11);
    assert_eq!(rows[1].count(), 4);
    assert_eq!(rows[1].get::<i32>(1).unwrap(), 22);
    assert_eq!(rows[2].get::<i32>(2).unwrap(), 33);
}

#[test]
fn test_well_formed_csv_with_quoted_strings() {
    let data = r#"header1,header2,header3,header4
"test,",12,13,"com,ma"
"""wow""",22,23,24
"b""d",32,33,34"#;
    let input = std::io::Cursor::new(data);
    let reader = Reader::builder()
        .with_header(true)
        .with_reader(input)
        .build()
        .expect("could not create reader.");

    let rows: Vec<_> = reader.entries().collect();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].count(), 4);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "test,".to_owned());
    assert_eq!(rows[0].get::<String>(3).unwrap(), "com,ma".to_owned());
    assert_eq!(rows[1].count(), 4);
    assert_eq!(rows[1].get::<String>(0).unwrap(), "\"wow\"".to_owned());
    assert_eq!(rows[2].count(), 4);
    assert_eq!(rows[2].get::<String>(0).unwrap(), "b\"d".to_owned());
}

#[test]
fn test_empty_fields() {
    let data = r#"header1,header2,header3,header4
,,,
,,,
,,,"#;
    let input = std::io::Cursor::new(data);
    let reader = Reader::builder()
        .with_header(true)
        .with_reader(input)
        .build()
        .expect("could not create reader.");

    let rows: Vec<_> = reader.entries().collect();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].count(), 4);
    assert!(rows[0].get::<String>(0).unwrap().is_empty());
    assert!(rows[0].get::<String>(3).unwrap().is_empty());
    assert_eq!(rows[1].count(), 4);
    assert!(rows[1].get::<String>(0).unwrap().is_empty());
    assert_eq!(rows[2].count(), 4);
    assert!(rows[2].get::<String>(0).unwrap().is_empty());
}

#[test]
fn test_csv_row_remove() {
    let mut row = Row::from(&["Hi", "there", "partner."][..]);
    assert_eq!(row.get::<String>(0).unwrap(), "Hi");
    assert_eq!(row.get::<String>(1).unwrap(), "there");
    assert_eq!(row.get::<String>(2).unwrap(), "partner.");

    // Now remove item
    row.remove(1);
    assert_eq!(row.get::<String>(0).unwrap(), "Hi");
    assert_eq!(row.get::<String>(1).unwrap(), "partner.");
}

#[test]
fn test_csv_row_replace() {
    let mut row = Row::from(&["Hi", "there", "partner."][..]);
    assert_eq!(row.get::<String>(0).unwrap(), "Hi");
    assert_eq!(row.get::<String>(1).unwrap(), "there");
    assert_eq!(row.get::<String>(2).unwrap(), "partner.");

    // Now remove item
    row.replace(1, "nameless person");
    assert_eq!(row.get::<String>(0).unwrap(), "Hi");
    assert_eq!(row.get::<String>(1).unwrap(), "nameless person");
    assert_eq!(row.get::<String>(2).unwrap(), "partner.");
}
#[test]
fn test_csv_doc_remove_row() {
    let data = r#"header1,header2,header3,header4
11,12,13,14
21,22,23,24
31,32,33,34"#;
    let input = std::io::Cursor::new(data);
    let csv_reader = Reader::builder()
        .with_reader(input)
        .with_header(true)
        .build()
        .expect("Creating document reader");
    let mut doc = Document::try_from(csv_reader).expect("Converting reader into document");
    doc.remove_where("header1", &21);
    assert_eq!(doc.get_value::<i32>(0, "header1"), Ok(11));
    assert_eq!(doc.get_value::<i32>(1, "header1"), Ok(31));
    assert_eq!(doc.count(), 2);
}
