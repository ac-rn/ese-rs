# Release Process

This document describes how to release a new version of `ese-parser` to PyPI.

## Automated Release (Recommended)

The project uses GitHub Actions to automatically build wheels for all platforms and Python versions (3.8-3.13).

### Steps:

1. **Update version numbers** in both:
   - `Cargo.toml` (line 3)
   - `python/pyproject.toml` (line 7)

2. **Commit and push changes:**
   ```bash
   git add Cargo.toml python/pyproject.toml
   git commit -m "Bump version to X.Y.Z"
   git push
   ```

3. **Create and push a git tag:**
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

4. **Monitor the release:**
   - Go to GitHub Actions tab
   - Watch the "Release" workflow
   - Wheels will be automatically built for:
     - Linux (x86_64, aarch64)
     - Windows (x64, x86)
     - macOS (x86_64, aarch64)
     - Python versions: 3.8, 3.9, 3.10, 3.11, 3.12, 3.13
   - Automatically uploaded to PyPI

### Requirements:

- `PYPI_API_TOKEN` secret must be set in GitHub repository settings
- Get token from: https://pypi.org/manage/account/token/

## Manual Release (Local)

If you need to build and upload manually:

### Windows:
```powershell
.\build_wheels.ps1
maturin upload dist/*
```

### Linux/macOS:
```bash
chmod +x build_wheels.sh
./build_wheels.sh
maturin upload dist/*
```

**Note:** Manual builds only create wheels for your current platform. For full cross-platform support, use the automated GitHub Actions workflow.

## Verification

After release, verify the package:

```bash
pip install --upgrade ese-parser
python -c "import ese_parser; print(ese_parser.__version__)"
```

## Version History

- **0.1.2** - Added Python 3.13 support
- **0.1.1** - Bug fixes and improvements
- **0.1.0** - Initial release
