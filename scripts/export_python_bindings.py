#!/usr/bin/env python
"""Export all tables from ESE databases using the Python bindings for Rust parser."""

import sys
import json
from pathlib import Path

try:
    from ese_parser import EseDatabase
except ImportError:
    print("Error: Could not import ese_parser. Make sure the Python bindings are built.")
    print("Run: python -m maturin build --release")
    print("Then: pip install target/wheels/ese_parser-*.whl")
    sys.exit(1)

def serialize_value(value):
    """Serialize a value to JSON-compatible format."""
    if value is None:
        return None
    elif isinstance(value, bytes):
        return value.hex()
    elif isinstance(value, str):
        # Remove null terminators
        return value.rstrip('\x00')
    elif isinstance(value, (int, float, bool)):
        return value
    elif isinstance(value, dict):
        # Handle nested structures
        return {k: serialize_value(v) for k, v in value.items()}
    elif isinstance(value, (list, tuple)):
        return [serialize_value(v) for v in value]
    else:
        # Fallback for unknown types
        return str(value)

def export_table(db, table_name, db_name, output_dir):
    """Export a single table to JSONL format."""
    safe_table_name = table_name.replace('/', '_').replace('\\', '_').replace('{', '_').replace('}', '_')
    
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f"{db_name}_{safe_table_name}.jsonl"
    
    rows_exported = 0
    
    # Read all records from the table
    records = db.read_table(table_name)
    
    with open(output_file, 'w', encoding='utf-8') as f:
        for row in records:
            # Convert record to serializable format
            serialized = {}
            for col_name, value in row.items():
                serialized[col_name] = serialize_value(value)
            
            # Sort keys for consistent output
            sorted_record = {k: serialized[k] for k in sorted(serialized.keys())}
            
            # Write as JSONL
            f.write(json.dumps(sorted_record, ensure_ascii=False) + '\n')
            rows_exported += 1
    
    print(f"  {db_name}/{safe_table_name}: {rows_exported} rows")
    return rows_exported

def main():
    databases = [
        ("SRUDB.dat", "SRUDB"),
        ("WebCacheV01.dat", "WebCacheV01"),
        ("Windows.edb", "Windows"),
        ("Current.mdb", "Current"),
    ]
    
    output_dir = Path("python_bindings_exports")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    total_tables = 0
    total_rows = 0
    
    for db_file, db_name in databases:
        if not Path(db_file).exists():
            print(f"Skipping {db_file} (not found)")
            continue
        
        print(f"\nProcessing {db_file}...")
        try:
            db = EseDatabase(db_file)
            
            # Get all tables (excluding MSys tables)
            all_tables = db.get_tables()
            user_tables = [t for t in all_tables if not t.startswith('MSys')]
            
            print(f"Found {len(all_tables)} total tables, {len(user_tables)} user tables")
            
            for table_name in user_tables:
                try:
                    count = export_table(db, table_name, db_name, output_dir)
                    total_rows += count
                    total_tables += 1
                except Exception as e:
                    print(f"  Error exporting {table_name}: {e}")
            
        except Exception as e:
            print(f"Error processing {db_file}: {e}")
            import traceback
            traceback.print_exc()
            continue
    
    print("\n=== EXPORT COMPLETE ===")
    print(f"Total tables exported: {total_tables}")
    print(f"Total rows exported: {total_rows}")
    print(f"Output directory: {output_dir}/")

if __name__ == "__main__":
    main()
