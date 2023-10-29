use criterion::{criterion_group, criterion_main, Criterion};

fn mine() {
    let reader = csvlib::Reader::builder()
        .with_reader(std::fs::File::open("AAPL.csv").unwrap())
        .with_header(true)
        .build()
        .unwrap();
    // Date,Open,High,Low,Close,Adj Close,Volume
    let mut total: f64 = 0.0;
    let mut count = 0;
    for record in reader.entries() {
        total += record
            .get::<f64>(1)
            .map_err(|error| {
                println!("Got an error converting to f64: {error}");
                0.0
            })
            .unwrap();
        count += 1;
    }

    println!(
        "Total: {}, Avg: {}, count: {count}",
        total,
        total / (count as f64)
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("CSV Performance");
    group.significance_level(0.1).sample_size(10);
    group.bench_function("mine", |b| b.iter(mine));
    group.finish();
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
