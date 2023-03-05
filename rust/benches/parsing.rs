use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use rspolib::{mofile, pofile};

fn pofile_parse(basename: &str) {
    pofile(format!("tests-data/{}", basename).as_str()).ok();
}

fn mofile_parse(basename: &str) {
    mofile(format!("tests-data/{}", basename).as_str()).ok();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pofile('django-complete.po')", |b| {
        b.iter(|| pofile_parse(black_box("django-complete.po")))
    });
    c.bench_function("mofile('all.mo')", |b| {
        b.iter(|| mofile_parse(black_box("all.mo")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
