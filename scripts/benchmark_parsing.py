#!/usr/bin/env python3
"""
Benchmark parsing performance: Python vs Rust
"""

import time
import subprocess
import sys
from pathlib import Path

def benchmark_python():
    """Benchmark Python ESE parser"""
    print("=" * 80)
    print("BENCHMARKING PYTHON PARSER")
    print("=" * 80)
    
    start_time = time.time()
    
    # Run Python parser
    result = subprocess.run(
        [sys.executable, "scripts/export_all_python.py"],
        capture_output=True,
        text=True
    )
    
    end_time = time.time()
    elapsed = end_time - start_time
    
    if result.returncode != 0:
        print(f"Error running Python parser: {result.stderr}")
        return None
    
    print(f"Python parsing completed in: {elapsed:.2f} seconds")
    return elapsed

def benchmark_rust():
    """Benchmark Rust ESE parser"""
    print("\n" + "=" * 80)
    print("BENCHMARKING RUST PARSER")
    print("=" * 80)
    
    # First, ensure it's compiled in release mode
    print("Building Rust parser in release mode...")
    build_result = subprocess.run(
        ["cargo", "build", "--release", "--example", "export_all_rust"],
        capture_output=True,
        text=True
    )
    
    if build_result.returncode != 0:
        print(f"Error building Rust parser: {build_result.stderr}")
        return None
    
    print("Running Rust parser...")
    start_time = time.time()
    
    # Run Rust parser (it processes all databases)
    result = subprocess.run(
        ["target/release/examples/export_all_rust.exe"],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"Error running Rust parser: {result.stderr}")
        return None
    
    end_time = time.time()
    elapsed = end_time - start_time
    
    print(f"Rust parsing completed in: {elapsed:.2f} seconds")
    return elapsed

def main():
    """Run benchmarks and compare"""
    print("\n" + "=" * 80)
    print("ESE PARSER PERFORMANCE BENCHMARK")
    print("=" * 80)
    print("\nParsing all tables from:")
    print("  - Current.mdb")
    print("  - SRUDB.dat")
    print("  - WebCacheV01.dat")
    print("\n")
    
    # Benchmark Python
    python_time = benchmark_python()
    
    # Benchmark Rust
    rust_time = benchmark_rust()
    
    # Compare results
    if python_time and rust_time:
        print("\n" + "=" * 80)
        print("PERFORMANCE COMPARISON")
        print("=" * 80)
        print(f"Python:  {python_time:8.2f} seconds")
        print(f"Rust:    {rust_time:8.2f} seconds")
        print("-" * 80)
        
        speedup = python_time / rust_time
        time_saved = python_time - rust_time
        percent_faster = ((python_time - rust_time) / python_time) * 100
        
        if rust_time < python_time:
            print(f"Rust is {speedup:.2f}x FASTER than Python")
            print(f"Time saved: {time_saved:.2f} seconds ({percent_faster:.1f}% faster)")
        else:
            slowdown = rust_time / python_time
            print(f"Python is {slowdown:.2f}x faster than Rust")
        
        print("=" * 80)
    else:
        print("\n[ERROR] Benchmark failed - could not complete timing")

if __name__ == '__main__':
    main()
