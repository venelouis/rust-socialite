use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_socialite::providers::apple::AppleProvider;

fn bench_apple_provider(c: &mut Criterion) {
    c.bench_function("apple_provider_new", |b| {
        b.iter(|| {
            AppleProvider::new(
                black_box("client_id".to_string()),
                black_box("team_id".to_string()),
                black_box("key_id".to_string()),
                black_box("private_key_pem".to_string()),
                black_box("redirect_url".to_string()),
            )
        })
    });
}

criterion_group!(benches, bench_apple_provider);
criterion_main!(benches);
