# Creating Releases

This guide explains how to create downloadable executables for Shadow Swap.

## How It Works

GitHub Actions automatically builds executables for all platforms when you create a release tag.

## Creating a Release

### Method 1: Using Git Tags (Recommended)

1. **Update version in Cargo.toml** (if needed):
   ```toml
   version = "0.1.0"
   ```

2. **Create and push a version tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. **GitHub Actions will automatically**:
   - Build executables for all platforms
   - Create a GitHub Release
   - Upload all binaries as release assets

4. **Go to GitHub Releases**:
   - Visit: https://github.com/DmarshalTU/shadow-swap/releases
   - Edit the release to add release notes
   - The binaries will be attached automatically

### Method 2: Using GitHub UI

1. Go to your repository on GitHub
2. Click "Releases" → "Create a new release"
3. Create a new tag (e.g., `v0.1.0`)
4. Add release notes
5. Click "Publish release"
6. GitHub Actions will build and attach binaries automatically

## What Gets Built

The workflow builds executables for:
- ✅ Windows (x86_64) - `.exe` in `.tar.gz`
- ✅ Linux (x86_64) - binary in `.tar.gz`
- ✅ macOS Intel (x86_64) - binary in `.tar.gz`
- ✅ macOS Apple Silicon (ARM64) - binary in `.tar.gz`

## Download Links

Once a release is created, users can download from:
- **Releases page**: https://github.com/DmarshalTU/shadow-swap/releases
- Direct download links will be available for each platform

## Manual Build (For Testing)

If you want to test builds locally:

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows (on Windows)
cargo build --release --target x86_64-pc-windows-msvc

# macOS
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

## Troubleshooting

- **Builds failing?** Check the Actions tab on GitHub
- **Missing binaries?** Make sure the tag starts with `v` (e.g., `v0.1.0`)
- **Need to rebuild?** Delete the release and tag, then recreate

