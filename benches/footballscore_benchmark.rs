use criterion::{criterion_group, criterion_main, Criterion};

use footballscore::football_data::FootballData;

pub fn criterion_benchmark(c: &mut Criterion) {
    let buf = include_str!("../tests/resource/fixtures.json");
    let data: FootballData = serde_json::from_str(buf).unwrap();

    c.bench_function("footballscore_data", |b| {
        b.iter(|| data.get_current_fixtures())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
