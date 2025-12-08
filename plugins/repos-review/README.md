# repos-review

Interactive repository review plugin for the `repos` CLI tool.

## Overview

`repos-review` allows you to interactively review changes made in repositories before creating a pull request. It uses `fzf` for repository selection with a live preview of `git status`, then displays both `git status` and `git diff` for detailed review.

## Requirements

- `fzf` - Fuzzy finder for interactive repository selection
  - Install on macOS: `brew install fzf`
  - Install on Linux: Use your package manager (e.g., `apt install fzf`, `yum install fzf`)

## Usage

```bash
repos review
```

The plugin will:

1. Display a list of all repositories with an `fzf` interface
2. Show a preview of `git status` for each repository
3. After selection, display full `git status` and `git diff`
4. Wait for user input to either:
   - Press **Enter** to go back to the repository list
   - Press **Escape** or **Q** to exit

## Features

- **Interactive Selection**: Uses `fzf` with live preview of repository status
- **Color Output**: Syntax highlighting for better readability
- **Loop Mode**: Review multiple repositories in a single session
- **Simple Navigation**: Easy keyboard controls for efficient workflow

## Example Workflow

```bash
# Review changes across all repositories
repos review

# Use with tag filters to review specific repositories
repos review --tags backend

# Review repositories matching a pattern
repos review --pattern "api-*"
```

## Key Bindings

- **↑/↓** or **Ctrl-N/Ctrl-P**: Navigate repository list
- **Enter**: Select repository for review
- **Escape** or **Q**: Exit after reviewing a repository
- **Enter** (in review): Return to repository list

## Notes

- The plugin respects the same filters as other `repos` commands (`--tags`, `--pattern`, etc.)
- Only repositories with a configured path are shown
- If `fzf` is not installed, the plugin will exit with an error message
