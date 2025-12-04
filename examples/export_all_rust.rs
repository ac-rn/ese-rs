use ese_parser::{ColumnValue, Database};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

enum SerializedValue {
    RawU64(u64),
    Json(serde_json::Value),
}

fn serialize_value(value: &ColumnValue) -> SerializedValue {
    match value {
        ColumnValue::Null => SerializedValue::Json(serde_json::Value::Null),
        ColumnValue::Boolean(b) => SerializedValue::Json(serde_json::Value::Bool(*b)),
        ColumnValue::I8(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::U8(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::I16(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::U16(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::I32(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::U32(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::I64(n) => SerializedValue::Json(serde_json::Value::Number((*n).into())),
        ColumnValue::U64(n) => SerializedValue::RawU64(*n),
        ColumnValue::F32(f) => SerializedValue::Json(serde_json::Number::from_f64(*f as f64).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)),
        ColumnValue::F64(f) => SerializedValue::Json(serde_json::Number::from_f64(*f).map(serde_json::Value::Number).unwrap_or(serde_json::Value::Null)),
        ColumnValue::DateTime(dt) => SerializedValue::RawU64(*dt),
        ColumnValue::Currency(c) => SerializedValue::RawU64(*c),
        ColumnValue::Text(s) => {
            let trimmed = s.trim_end_matches('\0');
            SerializedValue::Json(serde_json::Value::String(trimmed.to_string()))
        },
        ColumnValue::Binary(b) | ColumnValue::LongValue(b) => {
            SerializedValue::Json(serde_json::Value::String(hex::encode(b)))
        },
        ColumnValue::Guid(g) => {
            SerializedValue::Json(serde_json::Value::String(format!(
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                g[3], g[2], g[1], g[0], g[5], g[4], g[7], g[6],
                g[8], g[9], g[10], g[11], g[12], g[13], g[14], g[15]
            )))
        },
        ColumnValue::MultiValue(v) => {
            SerializedValue::Json(serde_json::Value::String(hex::encode(v)))
        },
    }
}

fn export_table(
    db: &Database,
    table_name: &[u8],
    db_name: &str,
    output_dir: &Path,
) -> Result<usize, Box<dyn std::error::Error>> {
    let safe_table_name = String::from_utf8_lossy(table_name).replace(['/', '\\', '{', '}'], "_");

    fs::create_dir_all(output_dir)?;

    let output_file = output_dir.join(format!("{}_{}.jsonl", db_name, safe_table_name));
    let file = File::create(&output_file)?;
    let mut writer = BufWriter::new(file);

    let mut cursor = db.open_table(table_name)?;
    let mut rows_exported = 0;

    while let Some(row) = cursor.next_row()? {
        let mut serialized: Vec<(String, SerializedValue)> = Vec::new();

        for (col_name, value) in row {
            let col_str = String::from_utf8_lossy(&col_name).to_string();
            serialized.push((col_str, serialize_value(&value)));
        }

        serialized.sort_by(|a, b| a.0.cmp(&b.0));

        let mut parts: Vec<String> = Vec::new();
        for (key, value) in serialized {
            let value_str = match value {
                SerializedValue::RawU64(n) => n.to_string(),
                SerializedValue::Json(v) => serde_json::to_string(&v)?,
            };
            parts.push(format!("\"{}\":{}", key, value_str));
        }

        writeln!(writer, "{{{}}}", parts.join(","))?;
        rows_exported += 1;
    }

    writer.flush()?;

    println!("  {}/{}: {} rows", db_name, safe_table_name, rows_exported);
    Ok(rows_exported)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let databases = vec![
        ("SRUDB.dat", "SRUDB"),
        ("WebCacheV01.dat", "WebCacheV01"),
        ("Windows.edb", "Windows"),
        ("Current.mdb", "Current"),
    ];

    let output_dir = Path::new("rust_exports");
    fs::create_dir_all(output_dir)?;

    let mut total_tables = 0;
    let mut total_rows = 0;

    for (db_file, db_name) in &databases {
        if !Path::new(db_file).exists() {
            println!("Skipping {} (not found)", db_file);
            continue;
        }

        println!("\nProcessing {}...", db_file);
        let db = Database::open(db_file)?;

        let mut user_tables = Vec::new();
        for (table_name, _table_info) in db.tables() {
            if !table_name.starts_with(b"MSys") {
                user_tables.push(table_name.clone());
            }
        }

        println!(
            "Found {} total tables, {} user tables",
            db.tables().len(),
            user_tables.len()
        );

        for table_name in &user_tables {
            match export_table(&db, table_name, db_name, output_dir) {
                Ok(count) => {
                    total_rows += count;
                    total_tables += 1;
                }
                Err(e) => {
                    eprintln!(
                        "  Error exporting {}: {}",
                        String::from_utf8_lossy(table_name),
                        e
                    );
                }
            }
        }
    }

    println!("\n=== EXPORT COMPLETE ===");
    println!("Total tables exported: {}", total_tables);
    println!("Total rows exported: {}", total_rows);
    println!("Output directory: rust_exports/");

    Ok(())
}
