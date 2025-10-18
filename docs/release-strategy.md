# Release Branch Strategy

This document outlines the release branch strategy for semantic versioning and feature aggregation in the repos project.

## Overview

Instead of creating individual releases for each feature or fix, we use release branches to aggregate multiple changes into a single semantic version release. This provides better control over releases and allows for more meaningful version increments.

## Process

### 1. Feature Development

- Develop features and fixes on feature branches as usual
- Merge approved changes to `main` branch
- The workflow runs tests on `main` branch commits for validation
- **No releases are automatically created from `main` branch commits**
- Releases are only created from `release/**` branches

### 2. Release Preparation

When ready to create a new release:

1. **Create a release branch** from `main`:

   ```bash
   git checkout main
   git pull origin main
   git checkout -b release/1.2.0
   ```

2. **Push the release branch**:

   ```bash
   git push -u origin release/1.2.0
   ```

3. **Automatic release creation**: The GitHub Actions workflow will:
   - Analyze all commits since the last release
   - Determine the appropriate semantic version increment
   - Create a new release with aggregated changelog
   - Build and upload release artifacts

### 3. Manual Release Trigger

You can also manually trigger a release:

1. Go to the **Actions** tab in GitHub
2. Select the **Release** workflow
3. Click **Run workflow**
4. Optionally specify a release branch (defaults to `main`)

## Semantic Versioning Rules

The workflow analyzes commit messages to determine version increments:

- **Major version** (1.0.0 → 2.0.0): Commits containing `BREAKING CHANGE:` in the body
- **Minor version** (1.0.0 → 1.1.0): Commits with `feat:` prefix
- **Patch version** (1.0.0 → 1.0.1): All other changes (fixes, docs, etc.)

## Conventional Commits

To ensure proper semantic versioning, use conventional commit format:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Examples

- **Feature**: `feat: add support for organization-wide repository listing`
- **Fix**: `fix: handle rate limiting in GitHub API calls`
- **Breaking change**:

  ```text
  feat: redesign configuration file format

  BREAKING CHANGE: Configuration file format has changed from YAML to JSON.
  Users must migrate their existing config files.
  ```

## Branch Management

- **`main`**: Latest development code, always deployable
- **`release/*`**: Release preparation branches (e.g., `release/1.2.0`)
- **Feature branches**: Individual feature development (merged to `main`)

## Workflow Configuration

The project uses separate workflows for continuous integration and deployment:

### CI Workflow (`/.github/workflows/ci.yml`)

- **Triggers**: All pushes and pull requests to `main`
- **Purpose**: Continuous integration testing and validation
- **Actions**: Run tests, clippy, and formatting checks on multiple Rust versions

### Release Workflow (`/.github/workflows/release.yml`)

- **Triggers**: Pushes to `release/**` branches and manual dispatch
- **Purpose**: Continuous deployment and release creation
- **Configuration**:
  - **`bump_each_commit: false`**: Only increment version once per release
  - **`search_commit_body: true`**: Look for breaking changes in commit bodies
  - **Release branch triggers**: Automatically trigger on `release/*` branches
  - **Manual dispatch**: Allow manual release creation

## Benefits

1. **Controlled releases**: Choose when to release and what to include
2. **Meaningful versions**: Aggregate related changes into single releases
3. **Better changelogs**: Comprehensive release notes for multiple features
4. **Flexibility**: Manual and automatic release options
5. **Semantic accuracy**: Proper version increments based on actual impact

## Example Workflow

```bash
# Develop features
git checkout -b feature/new-command
# ... develop and test
git checkout main
git merge feature/new-command

git checkout -b feature/ui-improvements
# ... develop and test
git checkout main
git merge feature/ui-improvements

# Ready to release
git checkout -b release/1.3.0
git push -u origin release/1.3.0
# → GitHub Actions creates v1.3.0 release automatically
```

This strategy ensures that releases are intentional, well-tested, and contain meaningful collections of changes rather than incremental updates for every small modification.
