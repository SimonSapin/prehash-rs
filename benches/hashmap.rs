use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Bencher;
use criterion::Criterion;
use prehash::PreHashMap;
use prehash::WithHash;

fn bench_prehash_get(bencher: &mut Bencher<'_>) {
    let mut map = PreHashMap::new();
    for i in 0..100 {
        map.insert(WithHash::from(format!("value # {i}")), i);
    }
    let key: Box<WithHash<str>> = "value # 12".into();
    bencher.iter(|| black_box(&map).get(black_box(&key)))
}

fn bench_hashbrown_get(bencher: &mut Bencher<'_>) {
    let mut map = hashbrown::HashMap::new();
    for i in 0..100 {
        map.insert(format!("value # {i}"), i);
    }
    let key = "value # 12".to_owned();
    bencher.iter(|| black_box(&map).get(black_box(&key)))
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap get");
    group.bench_function("prehash", bench_prehash_get);
    group.bench_function("hashbrown", bench_hashbrown_get);
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
