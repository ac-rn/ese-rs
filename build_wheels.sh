#!/bin/bash
# Build wheels for all Python versions locally
# This script builds wheels for the current platform only

set -e

echo "Building wheels for all Python versions..."

# Ensure maturin is installed
pip install maturin

# Build wheels for all available Python interpreters
python -m maturin build --release --out dist --find-interpreter --manifest-path Cargo.toml --features python

echo ""
echo "Wheels built successfully in ./dist/"
echo "To upload to PyPI, run: maturin upload dist/*"
