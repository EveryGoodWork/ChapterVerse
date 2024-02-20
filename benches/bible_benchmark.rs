use bible::csv_import::bible_import;
use criterion::{criterion_group, criterion_main, Criterion};
use std::error::Error;
use std::hint::black_box;
use std::{env, fs};

fn bible_import_benchmark(c: &mut Criterion) {
    let bibles_directory = env::current_dir()
        .expect("Failed to get current directory")
        .join("bibles");
    let files = fs::read_dir(&bibles_directory).expect("Failed to read directory");

    println!("{:?}", bibles_directory);
    for file in files {
        let file = file.expect("Failed to read file");
        let path = file.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) == Some("csv") {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            c.bench_function(&format!("import {}", file_name), |b| {
                b.iter(|| bible_import(black_box(path.to_str().unwrap())))
            });
        }
    }
}

fn bible_get_scripture_benchmark(c: &mut Criterion) {
    let bibles_directory = env::current_dir()
        .expect("Failed to get current directory")
        .join("bibles");
    let files = fs::read_dir(&bibles_directory).expect("Failed to read directory");

    println!("{:?}", bibles_directory);
    for file in files {
        let file = file.expect("Failed to read file");
        let path = file.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) == Some("csv") {
            let file_name = path.file_name().unwrap().to_str().unwrap();

            let bible = bible_import(path.to_str().unwrap()).expect("Failed to import bible");

            c.bench_function("get_scripture 17:8:9", |b| {
                b.iter(|| {
                    let scripture = "17:8:9"; // Longest verse in the bible.
                    bible.get_scripture(black_box(scripture))
                })
            });
        }
    }
}

criterion_group!(
    benches,
    bible_import_benchmark,
    bible_get_scripture_benchmark
);
criterion_main!(benches);