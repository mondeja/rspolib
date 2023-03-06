use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use rspolib::{mofile, pofile, MOFile, POFile};

fn pofile_to_string(file: &POFile) {
    file.to_string();
}

fn mofile_to_string(file: &MOFile) {
    file.to_string();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "POFile('django-complete.po').to_string()",
        |b| {
            b.iter(|| {
                pofile_to_string(black_box(
                    &pofile("tests-data/django-complete.po").unwrap(),
                ))
            })
        },
    );
    c.bench_function("MOFile('all.mo').to_string()", |b| {
        b.iter(|| {
            mofile_to_string(black_box(
                &mofile("tests-data/all.mo").unwrap(),
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
