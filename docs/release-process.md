# Release Process Guide

This guide outlines the complete release process for the `repos` project, from
development and pull requests to creating a new versioned release. Our strategy
is built on semantic versioning, conventional commits, and a release branch
workflow to ensure releases are intentional and well-documented.

## Overview

- **Main Branch (`main`)**: The `main` branch is the primary development branch.
All new features and fixes are merged here. No releases are created directly
from `main`.
- **Release Branches (`release/**`)**: Releases are created by pushing a branch
with the `release/` prefix (e.g., `release/2025.09`). This triggers the release
workflow.
- **Semantic Versioning**: The release workflow automatically determines the new
version number based on the commit messages since the last release.
- **Squash Merges**: We use squash merges for pull requests to maintain a clean,
meaningful commit history on the `main` branch, which is essential for accurate
semantic versioning.

## The Release Workflow: Step-by-Step

The process is designed to be simple and automated, with clear quality gates.

### Step 1: Development & Pull Requests

All development happens on feature branches. When a feature or fix is ready,
open a Pull Request (PR) against the `main` branch.

#### PR Best Practices: Squash Merging

When merging a PR, **always use the "Squash and merge"** option.

**Why?**

Squash merging combines all of a PR's commits into a single commit on the `main`
branch. This is critical for our release process because:

1. **Clean History**: It keeps the `main` branch history clean and readable.
2. **Accurate Versioning**: It allows us to craft a single, precise conventional
commit message that the semantic versioning tool can parse to determine the
correct version bump. Merge commits often hide the original `feat:` or `fix:`
prefixes.

**How to Squash Merge:**

1. **From the GitHub UI**:
    - In the PR, select **"Squash and merge"** from the merge dropdown.
    - **Crucially, edit the commit title** to follow the
    [Conventional Commits](#conventional-commits) format (e.g.,
    `feat: add new command`).
    - Confirm the merge.

2. **From the GitHub CLI**:

    ```bash
    # Example for PR #42
    gh pr merge 42 \
      --squash \
      --subject "feat: add a new feature" \
      --body "Detailed description of the feature."
    ```

### Step 2: Creating a Release

When you are ready to release the features and fixes that have been merged into
`main`, you create a release branch.

1. **Ensure your `main` branch is up-to-date**:

    ```bash
    git checkout main
    git pull origin main
    ```

2. **Create and push a release branch**:
    The branch name can be anything, but it **must** start with `release/`. A
    good practice is to name it after the expected version or date.

    ```bash
    # Create a release branch (e.g., release/2025.10)
    git checkout -b release/prepare-release

    # Push the branch to GitHub
    git push -u origin release/prepare-release
    ```

### Step 3: Automated Release Workflow

Pushing the `release/` branch automatically triggers the `release.yml` GitHub
Actions workflow, which performs the following steps:

1. **Compute Version**: Analyzes commit messages on `main` since the last tag
and determines the new semantic version (e.g., `v1.3.0`).
2. **Run Quality Gates**: Runs the full test suite, linter (`clippy`), and
format checks. The release fails if any of these checks do not pass.
3. **Update `Cargo.toml`**: Bumps the `version` in `Cargo.toml` and pushes the
change to the release branch.
4. **Create GitHub Release**:
    - Creates a new Git tag (e.g., `v1.3.0`).
    - Generates a new GitHub Release with a "What's Changed" section populated
    from the conventional commit messages.
5. **Build Release Assets**: Compiles the application and creates binaries for
Linux and macOS (x86_64, aarch64, and universal). These are attached to the
GitHub Release.
6. **Sync `main` Branch**: After the release is successfully created, the
version bump in `Cargo.toml` is automatically merged back into the `main` branch
to keep it in sync.

## Conventional Commits

To ensure the automation works correctly, all commits merged into `main` must
follow the Conventional Commits specification.

### Commit Message Structure

```text
<type>[optional scope]: <description>
```

- **`<type>`**: Must be one of the following.
- **`<description>`**: A short, imperative-tense description of the change.

### Commit Types and Version Bumps

- **`feat`**: A new feature. Triggers a **minor** version bump (e.g., `1.2.3` →
`1.3.0`).
  - `feat: add a new 'run' command`
- **`fix`**: A bug fix. Triggers a **patch** version bump (e.g., `1.2.3` →
`1.2.4`).
  - `fix: correct an issue with path resolution`
- **`BREAKING CHANGE`**: A commit that introduces a breaking API change. This
can be added to the footer of any commit type and triggers a **major** version
bump (e.g., `1.2.3` → `2.0.0`).

    ```text
    feat: change CLI argument structure

    BREAKING CHANGE: The '--repo' argument has been renamed to '--repository'.
    ```

- **Other Types**: These trigger a **patch** version bump and are great for
organizing your work.
  - `docs:`: Documentation changes.
  - `style:`: Code style changes (formatting, etc.).
  - `refactor:`: Code changes that neither fix a bug nor add a feature.
  - `perf:`: A code change that improves performance.
  - `test:`: Adding or correcting tests.
  - `ci:`: Changes to CI configuration files and scripts.
  - `chore:`: Other changes that don't modify `src` or `test` files.
