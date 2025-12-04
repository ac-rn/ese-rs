#!/usr/bin/env python3
"""
Export all user tables from all ESE databases to JSONL format
"""

import json
import sys
from pathlib import Path
from binascii import hexlify

sys.path.insert(0, str(Path(__file__).parent.parent))
from ese import ESENT_DB

def serialize_value(value):
    """Convert a value to JSON-serializable format"""
    if value is None:
        return None
    elif isinstance(value, bool):
        return value
    elif isinstance(value, int):
        return value
    elif isinstance(value, float):
        return value
    elif isinstance(value, str):
        # Strip null terminators for consistency with Rust
        return value.rstrip('\x00')
    elif isinstance(value, bytes):
        # Check if already hex-encoded ASCII (from hexlify)
        try:
            value.decode('ascii')
            # If it decodes as ASCII and looks like hex, it's already encoded
            if all(c in b'0123456789abcdefABCDEF' for c in value):
                return value.decode('ascii')
        except:
            pass
        # Otherwise, hex encode it
        return hexlify(value).decode('ascii')
    else:
        return str(value)

def export_table(db_path, table_name, db_name, output_dir):
    """Export a single table to JSONL"""
    db = ESENT_DB(db_path)
    
    try:
        cursor = db.openTable(table_name)
    except Exception as e:
        print(f"  Error opening table {table_name.decode('utf-8', errors='replace')}: {e}")
        db.close()
        return 0
    
    safe_table_name = table_name.decode('utf-8', errors='replace').replace('/', '_').replace('\\', '_').replace('{', '_').replace('}', '_')
    output_file = output_dir / f"{db_name}_{safe_table_name}.jsonl"
    
    rows_exported = 0
    with open(output_file, 'w', encoding='utf-8') as f:
        row = db.getNextRow(cursor)
        while row:
            # Convert row to JSON
            json_row = {}
            for col_name, value in row.items():
                col_str = col_name.decode('utf-8', errors='replace')
                json_row[col_str] = serialize_value(value)
            
            f.write(json.dumps(json_row, sort_keys=True) + '\n')
            rows_exported += 1
            row = db.getNextRow(cursor)
    
    db.close()
    print(f"  {db_name}/{safe_table_name}: {rows_exported} rows")
    return rows_exported

def main():
    databases = [
        (b"SRUDB.dat", "SRUDB"),
        (b"WebCacheV01.dat", "WebCacheV01"),
        (b"Windows.edb", "Windows"),
        (b"Current.mdb", "Current"),
    ]
    
    output_dir = Path("python_exports")
    output_dir.mkdir(exist_ok=True)
    
    total_tables = 0
    total_rows = 0
    
    for db_file, db_name in databases:
        db_path = Path(db_file.decode())
        if not db_path.exists():
            print(f"Skipping {db_file.decode()} (not found)")
            continue
        
        print(f"\nProcessing {db_file.decode()}...")
        db = ESENT_DB(db_file)
        
        # Get all tables - access private __tables dict
        all_tables = []
        table_dict = db._ESENT_DB__tables
        for table_name in table_dict.keys():
            if not table_name.startswith(b'MSys'):
                all_tables.append(table_name)
        
        print(f"Found {len(table_dict)} total tables, {len(all_tables)} user tables")
        db.close()
        
        for table_name in all_tables:
            try:
                rows = export_table(db_file, table_name, db_name, output_dir)
                total_rows += rows
                total_tables += 1
            except Exception as e:
                print(f"  Error exporting {table_name.decode('utf-8', errors='replace')}: {e}")
    
    print(f"\n=== EXPORT COMPLETE ===")
    print(f"Total tables exported: {total_tables}")
    print(f"Total rows exported: {total_rows}")
    print(f"Output directory: python_exports/")

if __name__ == '__main__':
    main()
