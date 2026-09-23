use csvlib::{reader::Reader, Document, Row, Writer};

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
    assert_eq!(doc.get::<i32>(0, "header1"), Ok(11));
    assert_eq!(doc.get::<i32>(1, "header1"), Ok(31));
    assert_eq!(doc.count(), 2);
}

#[test]
fn test_csv_row_display_serializes_fields_with_commas_and_quotes() {
    let row = Row::from(&["alpha,beta", "say \"hi\"", "zeta"][..]);
    let output = row.to_string();
    assert_eq!(output, r#""alpha,beta","say ""hi""",zeta"#);
}

#[test]
fn test_csv_row_display_writes_delimiter_only_once() {
    let row = Row::from(&["one", "two,three"][..]);
    assert_eq!(row.to_string(), "one,\"two,three\"");
}

#[test]
fn test_writer_quotes_line_breaks_and_escaped_quotes() {
    let row = Row::from(&["first line\nsecond line", "say \"hi\""][..]);
    let mut writer = Writer::from_writer(Vec::new());

    writer.write(&row).unwrap();

    assert_eq!(
        writer.into_inner().unwrap(),
        b"\"first line\nsecond line\",\"say \"\"hi\"\"\"\r\n"
    );
}

#[test]
fn test_writer_preserves_unicode_delimiter() {
    let row = Row::from(&["alpha", "beta¦gamma"][..]);
    let mut writer = Writer::from_writer(Vec::new()).with_delimiter('¦');

    writer.write(&row).unwrap();

    assert_eq!(
        writer.into_inner().unwrap(),
        "alpha¦\"beta¦gamma\"\r\n".as_bytes()
    );
}

#[test]
fn test_document_try_insert_rejects_mismatched_row_width() {
    let mut doc = Document::with_headers(&["name", "age"]);

    assert!(doc.try_insert(Row::from(&["alice"][..])).is_err());
    assert_eq!(doc.count(), 0);

    assert!(doc.try_insert(Row::from(&["alice", "30"][..])).is_ok());
    assert_eq!(doc.count(), 1);
}

#[test]
fn test_document_try_insert_all_rejects_mismatched_rows() {
    let mut doc = Document::with_headers(&["name", "age"]);
    let rows = vec![
        Row::from(&["alice", "30"][..]),
        Row::from(&["bob"][..]),
        Row::from(&["charlie", "40"][..]),
    ];

    assert!(doc.try_insert_all(&rows).is_err());
    assert_eq!(doc.count(), 0);
}

#[test]
fn test_reader_parses_quoted_fields_with_commas_and_escaped_quotes() {
    let data = "name,notes,city\n\"alpha,beta\",\"say \"\"hi\"\"\",zeta\n";
    let reader = Reader::builder()
        .with_reader(std::io::Cursor::new(data))
        .with_header(true)
        .build()
        .expect("reader should parse quoted CSV");

    let rows: Vec<_> = reader.entries().collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].count(), 3);
    assert_eq!(rows[0].get::<String>(0).unwrap(), "alpha,beta");
    assert_eq!(rows[0].get::<String>(1).unwrap(), "say \"hi\"");
    assert_eq!(rows[0].get::<String>(2).unwrap(), "zeta");
}

#[test]
fn test_reader_parses_multiline_quoted_fields() {
    let data = "name,notes\n\"alice\",\"first line\nsecond line\"\n";
    let reader = Reader::builder()
        .with_reader(std::io::Cursor::new(data))
        .with_header(true)
        .build()
        .expect("reader should parse multiline quoted fields");

    let rows: Vec<_> = reader.entries().collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].count(), 2);
    assert_eq!(rows[0].get::<String>(1).unwrap(), "first line\nsecond line");
}

#[test]
fn test_reader_rejects_unterminated_quoted_field() {
    let data = "name,notes\n\"alice\",\"unterminated\n";
    let reader = Reader::builder()
        .with_reader(std::io::Cursor::new(data))
        .with_header(true)
        .build()
        .expect("reader should allow creation even with bad input");

    let rows: Vec<_> = reader.entries().collect();
    assert!(rows.is_empty());
}
