#!/usr/bin/env python3
"""
Clean up all test artifacts created during validation testing.
"""

import shutil
from pathlib import Path

def remove_directory(path):
    """Remove a directory and all its contents."""
    if path.exists():
        print(f"Removing {path}/")
        shutil.rmtree(path)
        print(f"  ✓ Removed")
    else:
        print(f"  - {path}/ does not exist (already clean)")

def main():
    print("=" * 80)
    print("CLEANING UP TEST ARTIFACTS")
    print("=" * 80)
    print()
    
    # List of directories to remove
    test_dirs = [
        Path("ese_py_exports"),
        Path("rust_exports"),
        Path("python_bindings_exports"),
        Path("python_exports"),  # Old name if it exists
    ]
    
    print("This will remove the following directories:")
    for d in test_dirs:
        if d.exists():
            print(f"  - {d}/")
    
    print()
    response = input("Are you sure you want to delete these directories? (y/n): ")
    
    if response.lower() != 'y':
        print("Cleanup cancelled.")
        return 0
    
    print()
    for directory in test_dirs:
        remove_directory(directory)
    
    print()
    print("=" * 80)
    print("✓ CLEANUP COMPLETE")
    print("=" * 80)
    
    return 0

if __name__ == '__main__':
    import sys
    sys.exit(main())
