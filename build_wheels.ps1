# Build wheels for all Python versions locally
# This script builds wheels for the current platform only

Write-Host "Building wheels for all Python versions..." -ForegroundColor Green

# Ensure maturin is installed
pip install maturin

# Build wheels for all available Python interpreters
python -m maturin build --release --out dist --find-interpreter --manifest-path Cargo.toml --features python

Write-Host "`nWheels built successfully in ./dist/" -ForegroundColor Green
Write-Host "To upload to PyPI, run: maturin upload dist/*" -ForegroundColor Yellow
