#!/usr/bin/env python3
"""
Compare Python and Rust exports to verify data consistency
"""

import json
from pathlib import Path
from collections import defaultdict

def compare_table(py_file, rust_file):
    """Compare two JSONL files"""
    # Read Python export
    py_rows = []
    with open(py_file, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                py_rows.append(json.loads(line))
    
    # Read Rust export
    rust_rows = []
    with open(rust_file, 'r', encoding='utf-8') as f:
        for line in f:
            if line.strip():
                rust_rows.append(json.loads(line))
    
    # Compare counts
    if len(py_rows) != len(rust_rows):
        return {
            'status': 'count_mismatch',
            'py_count': len(py_rows),
            'rust_count': len(rust_rows)
        }
    
    # Sort rows for comparison to handle different ordering
    def sort_key(row):
        if 'EntryId' in row:
            return row['EntryId']
        # Sort by JSON string representation for consistent ordering
        return json.dumps(row, sort_keys=True)
    
    py_sorted = sorted(py_rows, key=sort_key)
    rust_sorted = sorted(rust_rows, key=sort_key)
    
    # Compare data
    for i in range(len(py_sorted)):
        if py_sorted[i] != rust_sorted[i]:
            return {
                'status': 'data_mismatch',
                'py_count': len(py_rows),
                'rust_count': len(rust_rows)
            }
    
    return {
        'status': 'perfect',
        'py_count': len(py_rows),
        'rust_count': len(rust_rows)
    }

def main():
    py_dir = Path("python_exports")
    rust_dir = Path("rust_exports")
    
    if not py_dir.exists() or not rust_dir.exists():
        print("Error: Export directories not found. Run exports first.")
        return
    
    py_files = {f.name: f for f in py_dir.glob("*.jsonl")}
    rust_files = {f.name: f for f in rust_dir.glob("*.jsonl")}
    
    perfect_matches = []
    data_mismatches = []
    count_mismatches = []
    
    # Group by database
    by_database = defaultdict(lambda: {'perfect': [], 'data_diff': [], 'count_diff': []})
    
    for filename in sorted(py_files.keys()):
        if filename not in rust_files:
            print(f"Missing in Rust: {filename}")
            continue
        
        result = compare_table(py_files[filename], rust_files[filename])
        
        # Extract database name
        db_name = filename.split('_')[0]
        
        if result['status'] == 'perfect':
            perfect_matches.append((filename, result['py_count']))
            by_database[db_name]['perfect'].append((filename, result['py_count']))
        elif result['status'] == 'data_mismatch':
            data_mismatches.append((filename, result['py_count']))
            by_database[db_name]['data_diff'].append((filename, result['py_count']))
        else:
            count_mismatches.append((filename, result['py_count'], result['rust_count']))
            by_database[db_name]['count_diff'].append((filename, result['py_count'], result['rust_count']))
    
    # Print results by database
    print("=" * 80)
    print("RESULTS BY DATABASE")
    print("=" * 80)
    
    for db_name in sorted(by_database.keys()):
        db_results = by_database[db_name]
        total = len(db_results['perfect']) + len(db_results['data_diff']) + len(db_results['count_diff'])
        perfect_pct = 100 * len(db_results['perfect']) / total if total > 0 else 0
        
        print(f"\n{db_name}:")
        print(f"  Perfect: {len(db_results['perfect'])}/{total} ({perfect_pct:.1f}%)")
        
        if db_results['data_diff']:
            print(f"  Data differences: {len(db_results['data_diff'])}")
            for name, count in db_results['data_diff'][:5]:
                print(f"    - {name}: {count} rows")
        
        if db_results['count_diff']:
            print(f"  Count mismatches: {len(db_results['count_diff'])}")
            for name, py_count, rust_count in db_results['count_diff'][:5]:
                print(f"    - {name}: Python={py_count}, Rust={rust_count}")
    
    # Overall summary
    total = len(py_files)
    print("\n" + "=" * 80)
    print("OVERALL SUMMARY")
    print("=" * 80)
    print(f"Total tables: {total}")
    print(f"Perfect matches: {len(perfect_matches)} ({100*len(perfect_matches)/total:.1f}%)")
    print(f"Data mismatches: {len(data_mismatches)} ({100*len(data_mismatches)/total:.1f}%)")
    print(f"Count mismatches: {len(count_mismatches)} ({100*len(count_mismatches)/total:.1f}%)")
    
    if data_mismatches:
        print(f"\nTables with data differences ({len(data_mismatches)}):")
        for name, count in data_mismatches:
            print(f"  - {name}: {count} rows")
    
    if count_mismatches:
        print(f"\nTables with count mismatches ({len(count_mismatches)}):")
        for name, py_count, rust_count in count_mismatches:
            print(f"  - {name}: Python={py_count}, Rust={rust_count}")
    
    # Success message
    if len(perfect_matches) == total:
        print("\n" + "=" * 80)
        print("SUCCESS! ALL TABLES MATCH PERFECTLY!")
        print("=" * 80)

if __name__ == '__main__':
    main()
