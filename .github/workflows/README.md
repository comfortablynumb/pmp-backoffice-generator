# GitHub Workflows

This directory contains GitHub Actions workflows for continuous integration, security, and release automation.

## Workflows

### 🔧 CI (ci.yml)

Runs on every push to `main`/`master` and on pull requests.

**Jobs:**
- **Format Check** - Validates code formatting with `cargo fmt`
- **Clippy Lint** - Runs static analysis with `cargo clippy`
- **Test** - Executes all tests with `cargo test`
- **Build** - Builds the project on stable and beta Rust versions

### 🔒 Security Audit (security.yml)

Runs on every push, pull request, and daily at midnight UTC.

**Jobs:**
- **Security Audit** - Scans dependencies for known vulnerabilities using `cargo-audit`
- **Dependency Review** - Reviews dependency changes in pull requests

### 📊 Code Coverage (coverage.yml)

Runs on every push to `main`/`master` and on pull requests.

**Jobs:**
- **Coverage** - Generates code coverage reports using `cargo-llvm-cov`
- Uploads coverage to Codecov (requires `CODECOV_TOKEN` secret)

### 🚀 Release (release.yml)

Triggers automatically when a version tag (e.g., `v1.0.0`) is pushed.

**Jobs:**
- **Create Release** - Creates a GitHub release
- **Build Release** - Builds release binaries for multiple platforms:
  - Linux (glibc and musl)
  - macOS (Intel and ARM)
  - Windows

## Setup

### Required Secrets

For full functionality, add these secrets to your repository settings:

- `CODECOV_TOKEN` - Token for uploading coverage reports to Codecov (optional)

### Creating a Release

To create a new release:

```bash
# Tag the release
git tag v1.0.0

# Push the tag
git push origin v1.0.0
```

The release workflow will automatically build binaries for all platforms and attach them to the GitHub release.

## Badge Examples

Add these badges to your README.md:

```markdown
[![CI](https://github.com/YOUR_USERNAME/pmp-backoffice-generator/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/pmp-backoffice-generator/actions/workflows/ci.yml)
[![Security Audit](https://github.com/YOUR_USERNAME/pmp-backoffice-generator/workflows/Security%20Audit/badge.svg)](https://github.com/YOUR_USERNAME/pmp-backoffice-generator/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/YOUR_USERNAME/pmp-backoffice-generator/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/pmp-backoffice-generator)
```

## Local Testing

Before pushing, you can run these checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test --all-features --verbose

# Coverage (requires cargo-llvm-cov)
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --html
```
