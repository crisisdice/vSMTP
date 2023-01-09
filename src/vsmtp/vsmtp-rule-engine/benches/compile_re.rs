use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vsmtp_rule_engine::{sub_domain_hierarchy::Builder, RuleEngine};

macro_rules! compile_re {
    ($rules:expr) => {{
        let config = black_box(vsmtp_test::config::local_test());

        let mut re = RuleEngine::new_rhai_engine();

        RuleEngine::build_static_modules(&mut re, &config).unwrap();
        RuleEngine::build_global_modules(&mut re).unwrap();

        Builder::new(&re)
            .unwrap()
            .add_root_filter_rules(black_box($rules))
            .unwrap();
    }};
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compile");

    group.bench_function("empty map", |b| b.iter(|| compile_re!("#{}")));

    for nbr_rules in [0, 1, 10, 100] {
        group.bench_function(format!("{nbr_rules} rules"), |b| {
            b.iter(|| {
                compile_re!(&format!(
                    "#{{ connect: [ {} ] }}",
                    (0..=nbr_rules)
                        .map(|i| format!("action \"action {i}\" || {{}},"))
                        .collect::<String>()
                ))
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
