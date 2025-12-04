#!/usr/bin/env python
"""Export all tables from ESE databases using the original ese.py parser."""

import sys
import os
import json
from pathlib import Path

# Add parent directory to path to import ese.py
sys.path.insert(0, str(Path(__file__).parent.parent))

import ese
from binascii import hexlify

def serialize_value(value):
    """Serialize a value to JSON-compatible format."""
    if value is None:
        return None
    elif isinstance(value, bytes):
        return hexlify(value).decode('ascii')
    elif isinstance(value, str):
        # Remove null terminators
        return value.rstrip('\x00')
    elif isinstance(value, (int, float, bool)):
        return value
    else:
        # Fallback for unknown types
        return str(value)

def export_table(db, table_name, db_name, output_dir):
    """Export a single table to JSONL format."""
    safe_table_name = table_name.decode('utf-8', errors='replace').replace('/', '_').replace('\\', '_').replace('{', '_').replace('}', '_')
    
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f"{db_name}_{safe_table_name}.jsonl"
    
    cursor = db.openTable(table_name)
    if cursor is None:
        print(f"  Could not open table: {table_name}")
        return 0
    
    rows_exported = 0
    
    with open(output_file, 'w', encoding='utf-8') as f:
        while True:
            try:
                record = db.getNextRow(cursor)
                if record is None:
                    break
                
                # Convert record to serializable format
                serialized = {}
                for col_name, value in record.items():
                    col_str = col_name.decode('utf-8', errors='replace')
                    serialized[col_str] = serialize_value(value)
                
                # Sort keys for consistent output
                sorted_record = {k: serialized[k] for k in sorted(serialized.keys())}
                
                # Write as JSONL
                f.write(json.dumps(sorted_record, ensure_ascii=False) + '\n')
                rows_exported += 1
                
            except Exception as e:
                print(f"  Error reading row: {e}")
                continue
    
    print(f"  {db_name}/{safe_table_name}: {rows_exported} rows")
    return rows_exported

def main():
    databases = [
        ("SRUDB.dat", "SRUDB"),
        ("WebCacheV01.dat", "WebCacheV01"),
        ("Windows.edb", "Windows"),
        ("Current.mdb", "Current"),
    ]
    
    output_dir = Path("ese_py_exports")
    output_dir.mkdir(parents=True, exist_ok=True)
    
    total_tables = 0
    total_rows = 0
    
    for db_file, db_name in databases:
        if not Path(db_file).exists():
            print(f"Skipping {db_file} (not found)")
            continue
        
        print(f"\nProcessing {db_file}...")
        try:
            db = ese.ESENT_DB(db_file)
            
            # Get all tables (excluding MSys tables)
            user_tables = []
            for table_name in db._ESENT_DB__tables.keys():
                if not table_name.startswith(b'MSys'):
                    user_tables.append(table_name)
            
            print(f"Found {len(db._ESENT_DB__tables)} total tables, {len(user_tables)} user tables")
            
            for table_name in user_tables:
                try:
                    count = export_table(db, table_name, db_name, output_dir)
                    total_rows += count
                    total_tables += 1
                except Exception as e:
                    print(f"  Error exporting {table_name.decode('utf-8', errors='replace')}: {e}")
            
            db.close()
            
        except Exception as e:
            print(f"Error processing {db_file}: {e}")
            continue
    
    print("\n=== EXPORT COMPLETE ===")
    print(f"Total tables exported: {total_tables}")
    print(f"Total rows exported: {total_rows}")
    print(f"Output directory: {output_dir}/")

if __name__ == "__main__":
    main()
