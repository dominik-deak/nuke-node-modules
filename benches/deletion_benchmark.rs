//! Benchmarks for directory deletion performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nuke_node_modules::{cleanup_node_modules, Config};
use std::fs;
use tempfile::TempDir;

fn create_benchmark_structure(temp_dir: &TempDir, num_dirs: usize) -> anyhow::Result<()> {
    for i in 0..num_dirs {
        let node_modules_path = temp_dir.path().join(format!("project_{}/node_modules", i));
        fs::create_dir_all(&node_modules_path)?;

        // Add some files to make it more realistic
        for j in 0..5 {
            let file_path = node_modules_path.join(format!("file_{}.js", j));
            fs::write(&file_path, format!("// File {} in directory {}", j, i))?;
        }

        // Add a subdirectory
        let sub_dir = node_modules_path.join("subdir");
        fs::create_dir_all(&sub_dir)?;
        fs::write(sub_dir.join("nested.js"), "// nested file")?;
    }

    Ok(())
}

fn benchmark_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cleanup_performance");

    for num_dirs in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel_cleanup", num_dirs),
            num_dirs,
            |b, &num_dirs| {
                b.iter_batched(
                    // Setup: Create fresh directory structure for each iteration
                    || {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        create_benchmark_structure(&temp_dir, num_dirs).expect("Failed to create structure");
                        temp_dir
                    },
                    // Benchmark: Perform the cleanup
                    |temp_dir| {
                        let config = Config {
                            quiet: true,
                            no_confirm: true,
                            ..Default::default()
                        };
                        cleanup_node_modules(black_box(temp_dir.path()), black_box(&config))
                            .expect("Cleanup failed")
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );

        // Benchmark single-threaded performance for comparison
        group.bench_with_input(
            BenchmarkId::new("single_threaded_cleanup", num_dirs),
            num_dirs,
            |b, &num_dirs| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().expect("Failed to create temp dir");
                        create_benchmark_structure(&temp_dir, num_dirs).expect("Failed to create structure");
                        temp_dir
                    },
                    |temp_dir| {
                        let config = Config {
                            threads: Some(1),
                            quiet: true,
                            no_confirm: true,
                            ..Default::default()
                        };
                        cleanup_node_modules(black_box(temp_dir.path()), black_box(&config))
                            .expect("Cleanup failed")
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

fn benchmark_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("scanning_performance");

    for num_dirs in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("directory_scanning", num_dirs),
            num_dirs,
            |b, &num_dirs| {
                // Setup once for all iterations
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                create_benchmark_structure(&temp_dir, num_dirs).expect("Failed to create structure");

                b.iter(|| {
                    let scanner = nuke_node_modules::scanner::Scanner::new(temp_dir.path(), &[]);
                    scanner.find_node_modules_dirs().expect("Scanning failed")
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_cleanup, benchmark_scanning);
criterion_main!(benches);