#!/usr/bin/env python3
"""
Master script to run all three exports and compare them.
This is the main entry point for the comprehensive validation test.
"""

import subprocess
import sys
from pathlib import Path
import time

def run_command(cmd, description):
    """Run a command and report results."""
    print("\n" + "=" * 80)
    print(f"STEP: {description}")
    print("=" * 80)
    print(f"Running: {' '.join(cmd)}")
    print()
    
    start_time = time.time()
    result = subprocess.run(cmd, shell=True)
    elapsed = time.time() - start_time
    
    print(f"\nCompleted in {elapsed:.2f} seconds")
    
    if result.returncode != 0:
        print(f"❌ ERROR: {description} failed with exit code {result.returncode}")
        return False
    else:
        print(f"✓ {description} completed successfully")
        return True

def main():
    print("=" * 80)
    print("COMPREHENSIVE ESE PARSER VALIDATION TEST")
    print("=" * 80)
    print()
    print("This script will:")
    print("  1. Export all 4 databases using ese.py (original Python)")
    print("  2. Export all 4 databases using Rust implementation")
    print("  3. Export all 4 databases using Python bindings")
    print("  4. Compare all three outputs for consistency")
    print()
    
    # Check if databases exist
    databases = ["SRUDB.dat", "WebCacheV01.dat", "Windows.edb", "Current.mdb"]
    missing = [db for db in databases if not Path(db).exists()]
    
    if missing:
        print("⚠️  Warning: Some databases are missing:")
        for db in missing:
            print(f"  - {db}")
        print()
        response = input("Continue anyway? (y/n): ")
        if response.lower() != 'y':
            return 1
    
    # Step 1: Export using ese.py
    if not run_command(
        [sys.executable, "scripts/export_ese_py.py"],
        "Export using ese.py (original Python)"
    ):
        return 1
    
    # Step 2: Export using Rust
    if not run_command(
        ["cargo", "run", "--release", "--example", "export_all_rust"],
        "Export using Rust implementation"
    ):
        return 1
    
    # Step 3: Check if Python bindings are built
    print("\n" + "=" * 80)
    print("Checking Python bindings...")
    print("=" * 80)
    
    try:
        sys.path.insert(0, str(Path("python")))
        import ese_parser
        print("✓ Python bindings are available")
    except ImportError:
        print("⚠️  Python bindings not found. Building them now...")
        if not run_command(
            ["maturin", "develop", "--release"],
            "Build Python bindings"
        ):
            print("\n❌ Failed to build Python bindings.")
            print("Please install maturin: pip install maturin")
            print("Then run: maturin develop --release")
            return 1
    
    # Step 4: Export using Python bindings
    if not run_command(
        [sys.executable, "scripts/export_python_bindings.py"],
        "Export using Python bindings"
    ):
        return 1
    
    # Step 5: Compare all three versions
    print("\n" + "=" * 80)
    print("FINAL STEP: Comparing all three versions")
    print("=" * 80)
    
    result = subprocess.run([sys.executable, "scripts/compare_all_three.py"])
    
    if result.returncode == 0:
        print("\n" + "=" * 80)
        print("🎉 ALL TESTS PASSED! 🎉")
        print("=" * 80)
        print("\nAll three parser implementations produce identical results:")
        print("  ✓ ese.py (original Python)")
        print("  ✓ Rust implementation")
        print("  ✓ Python bindings")
        print("\nYou are ready to publish to GitHub and PyPI!")
        return 0
    else:
        print("\n" + "=" * 80)
        print("❌ TESTS FAILED")
        print("=" * 80)
        print("\nThere are differences between the implementations.")
        print("Please review the comparison output above.")
        return 1

if __name__ == '__main__':
    sys.exit(main())
