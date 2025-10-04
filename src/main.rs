use csvlib::Document;

fn main() {
    let start_time = std::time::Instant::now();
    let document = Document::from_path(
        r#"C:\Users\Will DeJs\Downloads\5m-Sales-Records\5m Sales Records.csv"#,
    )
    .unwrap();
    let mut high_sellers = 0;
    for row in document.rows() {
        let sold_units = row.get::<i32>("Units Sold").unwrap();
        if sold_units > 9000 {
            high_sellers += 1;
            // println!("{row}")
        }
    }
    println!("Number of high selling products: {}", high_sellers);
    let end_time = std::time::Instant::now();
    let duration = end_time.duration_since(start_time);
    println!("Time taken: {:?} ms", duration.as_millis());
}
