use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use rougers::{RougeScorer, Tokenizer};

const TARGET: &str = "Officials said the new bridge, which connects the two largest districts \
of the city, will open to traffic next spring after three years of construction.\n\
The project cost about 450 million dollars, nearly twice the original estimate.\n\
Residents have complained about noise and closed roads during the building work.";

const PREDICTION: &str = "The new bridge connecting the city's two biggest districts is \
expected to open next spring, officials said.\n\
Construction took three years and cost roughly 450 million dollars, almost double \
the planned budget.\n\
Many residents complained about the noise and road closures.";

fn tokenizer(c: &mut Criterion) {
    let plain = Tokenizer::new(false);
    let stemming = Tokenizer::new(true);
    c.bench_function("tokenize", |b| b.iter(|| plain.tokenize(black_box(TARGET))));
    c.bench_function("tokenize_stem", |b| {
        b.iter(|| stemming.tokenize(black_box(TARGET)))
    });
}

fn scorer(c: &mut Criterion) {
    for use_stemmer in [false, true] {
        let scorer = RougeScorer::new(["rouge1", "rouge2", "rougeL", "rougeLsum"], use_stemmer)
            .expect("valid rouge types");
        let name = if use_stemmer { "score_stem" } else { "score" };
        c.bench_function(name, |b| {
            b.iter(|| scorer.score(black_box(TARGET), black_box(PREDICTION)))
        });
    }
}

criterion_group!(benches, tokenizer, scorer);
criterion_main!(benches);
