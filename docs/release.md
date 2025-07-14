# Release Workflow Summary

## What Was Created

1. **Updated `.github/workflows/release.yml`** with semantic versioning
2. **Created `SEMANTIC_VERSIONING.md`** documentation

## Key Features

### Automatic Semantic Versioning

- Uses `paulhatch/semantic-version@v5.4.0` action (most popular for Rust projects)
- Determines version from conventional commit messages
- Supports major, minor, and patch version bumps

### Conventional Commits Support

- `feat:` → minor version bump
- `fix:` → patch version bump
- `feat!:` or `(MAJOR)` → major version bump
- Other types (`docs:`, `chore:`, etc.) → patch version bump

### Release Process

- **Triggers**: Only on pushes to main/master branch with actual changes
- **Testing**: Runs full test suite before any release
- **Versioning**: Auto-updates Cargo.toml version
- **Release**: Creates GitHub release with auto-generated changelog
- **Binaries**: Builds for multiple platforms (Linux, macOS Intel/ARM)
- **Publishing**: Auto-publishes to crates.io (stable releases only)

### Quality Gates

- All tests must pass
- Clippy linting must pass
- Code formatting must be correct
- Only releases if there are actual changes

## Usage

Simply commit with conventional commit messages and push to main:

```bash
git commit -m "feat: add repository filtering by name"
git push origin main
# → Will trigger minor version release (e.g., 1.0.0 → 1.1.0)
```

The workflow will handle everything else automatically!
