//! Benchmarks for ESE database parsing operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Mock data for testing when no real database is available
fn create_mock_database() -> Vec<u8> {
    let mut data = vec![0u8; 16384]; // 2 pages minimum

    // Set database header signature
    data[4..8].copy_from_slice(&[0xef, 0xcd, 0xab, 0x89]);

    // Set version and page size
    data[8..12].copy_from_slice(&0x620u32.to_le_bytes());
    data[216..220].copy_from_slice(&8192u32.to_le_bytes());

    data
}

fn bench_header_parsing(c: &mut Criterion) {
    let data = create_mock_database();

    c.bench_function("parse_db_header", |b| {
        b.iter(|| {
            use ese_parser::header::DbHeader;
            let header = DbHeader::from_bytes(black_box(&data[0..1024]));
            black_box(header)
        });
    });
}

fn bench_page_parsing(c: &mut Criterion) {
    let data = create_mock_database();
    let page_data = &data[8192..16384]; // Second page

    c.bench_function("parse_page_header", |b| {
        b.iter(|| {
            use ese_parser::page::PageHeader;
            let result = PageHeader::parse(
                black_box(page_data),
                black_box(0x620),
                black_box(0x11),
                black_box(8192),
            );
            black_box(result)
        });
    });
}

// Benchmark with real database (commented out by default)
/*
fn bench_real_database(c: &mut Criterion) {
    // Update this path to point to a real ESE database for benchmarking
    const DB_PATH: &str = "path/to/database.edb";

    if std::path::Path::new(DB_PATH).exists() {
        c.bench_function("open_database", |b| {
            b.iter(|| {
                let db = Database::open(black_box(DB_PATH));
                black_box(db)
            });
        });

        let db = Database::open(DB_PATH).unwrap();

        c.bench_function("iterate_table", |b| {
            b.iter(|| {
                if let Some((table_name, _)) = db.tables().iter().next() {
                    if let Ok(mut cursor) = db.open_table(table_name) {
                        let mut count = 0;
                        while let Ok(Some(_record)) = cursor.next_row() {
                            count += 1;
                            if count >= 100 { break; } // Limit to 100 rows
                        }
                        black_box(count);
                    }
                }
            });
        });
    }
}
*/

criterion_group!(
    benches,
    bench_header_parsing,
    bench_page_parsing,
    // bench_real_database, // Uncomment when you have a real database
);

criterion_main!(benches);
