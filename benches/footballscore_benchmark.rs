use criterion::{criterion_group, criterion_main, Criterion};

use footballscore::{
    football_fixtures_data::FootballFixturesData, football_teams_data::FootballTeamsData,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    let buf = include_str!("../tests/resource/fixtures.json");
    let data: FootballFixturesData = serde_json::from_str(buf).unwrap();

    c.bench_function("footballscore_fixtures_data", |b| {
        b.iter(|| data.get_current_fixtures())
    });

    let buf = include_str!("../tests/resource/teams.json");
    let data: FootballTeamsData = serde_json::from_str(buf).unwrap();

    c.bench_function("footballscore_teams_data", |b| {
        b.iter(|| data.get_teams_information())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
