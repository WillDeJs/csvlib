use criterion::{criterion_group, criterion_main, Criterion};

fn million_records() {
    let reader = csvlib::reader::Reader::from_path("AAPL.csv").unwrap();

    // Date,Open,High,Low,Close,Adj Close,Volume
    let mut total: f64 = 0.0;
    let mut count = 0;
    for row in reader.entries() {
        total += row.get::<f64>(1).unwrap();
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
    group.bench_function("1 Million rows", |b| b.iter(million_records));
    group.finish();
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
