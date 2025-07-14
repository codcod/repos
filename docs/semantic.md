# Semantic Versioning Guide

This project uses automated semantic versioning with GitHub Actions. The version is automatically determined based on commit message conventions.

## Commit Message Format

Use conventional commit messages to trigger appropriate version bumps:

### Patch Version (0.0.X)

```text
fix: fix a bug
docs: update documentation
style: fix code formatting
refactor: internal code changes
test: add or update tests
chore: maintenance tasks
```

### Minor Version (0.X.0)

```text
feat: add new feature
feat: add repository filtering by name
```

### Major Version (X.0.0)

```text
feat!: breaking change to API
fix!: breaking fix
feat(MAJOR): major feature overhaul
```

You can also use explicit markers in commit messages:

- `(MAJOR)` - triggers major version bump
- `(MINOR)` - triggers minor version bump

## Release Process

1. **Development**: Work on feature branches and create PRs
2. **Merge to main**: When PR is merged to main branch, the workflow:
   - Runs all tests and quality checks
   - Determines new version based on commits since last release
   - Updates `Cargo.toml` with new version
   - Creates a GitHub release with auto-generated changelog
   - Builds and uploads release binaries for multiple platforms
   - Publishes to crates.io (stable releases only)

## Version Determination

The semantic versioning action looks at:

- Commit messages since the last release
- Changes in the `src/` directory
- Conventional commit patterns

## Manual Releases

If you need to create a manual release:

1. Tag the commit: `git tag v1.2.3`
2. Push the tag: `git push origin v1.2.3`

## Examples

```bash
# Will trigger patch version (e.g., 1.0.0 -> 1.0.1)
git commit -m "fix: resolve path resolution issue"

# Will trigger minor version (e.g., 1.0.1 -> 1.1.0)
git commit -m "feat: add support for repository name filtering"

# Will trigger major version (e.g., 1.1.0 -> 2.0.0)
git commit -m "feat!: change CLI argument structure (breaking change)"
```

## Release Assets

Each release automatically includes:

- Pre-built binaries for:
  - Linux (x86_64-unknown-linux-gnu)
  - Linux musl (x86_64-unknown-linux-musl)
  - macOS Intel (x86_64-apple-darwin)
  - macOS Apple Silicon (aarch64-apple-darwin)
- Source code archives
- Auto-generated changelog

## Cargo.toml

The version in `Cargo.toml` is automatically updated during the release process. Don't manually update it - let the automation handle versioning.
