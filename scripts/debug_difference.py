#!/usr/bin/env python3
"""Debug script to show specific differences between exports."""

import json
import sys
from pathlib import Path

def load_first_row(file_path):
    """Load first row from JSONL file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                return json.loads(line)
    return None

def compare_rows(row1, row2, name1="Row1", name2="Row2"):
    """Compare two rows and show differences."""
    all_keys = set(row1.keys()) | set(row2.keys())
    
    differences = []
    for key in sorted(all_keys):
        val1 = row1.get(key)
        val2 = row2.get(key)
        
        if val1 != val2:
            differences.append({
                'key': key,
                name1: val1,
                name2: val2
            })
    
    return differences

def main():
    if len(sys.argv) < 2:
        print("Usage: python debug_difference.py <table_name>")
        print("Example: python debug_difference.py Current_CLIENTS")
        return
    
    table_name = sys.argv[1]
    if not table_name.endswith('.jsonl'):
        table_name += '.jsonl'
    
    ese_py_file = Path("ese_py_exports") / table_name
    rust_file = Path("rust_exports") / table_name
    python_bindings_file = Path("python_bindings_exports") / table_name
    
    print(f"Comparing: {table_name}")
    print("=" * 80)
    
    # Load first row from each
    ese_py_row = load_first_row(ese_py_file)
    rust_row = load_first_row(rust_file)
    python_bindings_row = load_first_row(python_bindings_file)
    
    if not ese_py_row or not rust_row or not python_bindings_row:
        print("Error: Could not load rows from all files")
        return
    
    print(f"\nRow counts:")
    print(f"  ese.py columns: {len(ese_py_row)}")
    print(f"  rust columns: {len(rust_row)}")
    print(f"  python_bindings columns: {len(python_bindings_row)}")
    
    # Compare ese.py vs rust
    print("\n" + "=" * 80)
    print("ESE.PY vs RUST")
    print("=" * 80)
    diffs = compare_rows(ese_py_row, rust_row, "ese.py", "rust")
    if diffs:
        for diff in diffs[:10]:  # Show first 10 differences
            print(f"\nColumn: {diff['key']}")
            print(f"  ese.py: {diff['ese.py']}")
            print(f"  rust:   {diff['rust']}")
        if len(diffs) > 10:
            print(f"\n... and {len(diffs) - 10} more differences")
    else:
        print("No differences!")
    
    # Compare ese.py vs python_bindings
    print("\n" + "=" * 80)
    print("ESE.PY vs PYTHON_BINDINGS")
    print("=" * 80)
    diffs = compare_rows(ese_py_row, python_bindings_row, "ese.py", "python_bindings")
    if diffs:
        for diff in diffs[:10]:
            print(f"\nColumn: {diff['key']}")
            print(f"  ese.py:           {diff['ese.py']}")
            print(f"  python_bindings:  {diff['python_bindings']}")
        if len(diffs) > 10:
            print(f"\n... and {len(diffs) - 10} more differences")
    else:
        print("No differences!")
    
    # Compare rust vs python_bindings
    print("\n" + "=" * 80)
    print("RUST vs PYTHON_BINDINGS")
    print("=" * 80)
    diffs = compare_rows(rust_row, python_bindings_row, "rust", "python_bindings")
    if diffs:
        for diff in diffs[:10]:
            print(f"\nColumn: {diff['key']}")
            print(f"  rust:             {diff['rust']}")
            print(f"  python_bindings:  {diff['python_bindings']}")
        if len(diffs) > 10:
            print(f"\n... and {len(diffs) - 10} more differences")
    else:
        print("No differences!")

if __name__ == '__main__':
    main()
