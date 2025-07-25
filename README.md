# Repos: multi-repository management tool

Repos is a CLI tool to manage multiple GitHub repositories - clone them, run
commands across all repositories, create pull requests, and more—all with
colored output and comprehensive logging.

## Features

- **Multi-repository management**: Clone and manage multiple repositories from a
single config file
- **Tag-based filtering**: Run commands on specific repository groups using tags
- **Parallel execution**: Execute commands across repositories simultaneously
for faster operations
- **Colorized output**: Real-time colored logs with repository identification
- **Comprehensive logging**: Per-repository log files for detailed command
history
- **Pull request automation**: Create and manage pull requests across multiple
repositories
- **Built in Rust**: Memory-safe, fast, and reliable implementation

## Installation

### From Source

```bash
git clone https://github.com/codcod/repos.git
cd repos
cargo build --release
cp target/release/repos /usr/local/bin/
```

### Using Cargo

```bash
cargo install --path .
```

### Homebrew

```bash
brew tap codcod/taps
brew install repos
```

## Configuration

The `config.yaml` file defines which repositories to manage and how to organize
them. Repos supports various Git URL formats including GitHub Enterprise
instances.

```yaml
repositories:
  - name: loan-pricing
    url: git@github.com:yourorg/loan-pricing.git
    tags: [java, backend]
    branch: develop # Optional: Branch to clone
    path: cloned_repos/loan-pricing # Optional: Directory to place cloned repo

  - name: web-ui
    url: git@github.com:yourorg/web-ui.git
    tags: [frontend, react]
    # When branch is not specified, the default branch will be cloned
    # When path is not specified, the current directory will be used

  - name: enterprise-repo
    url: git@github-enterprise:company/project.git
    tags: [enterprise, backend]
    # GitHub Enterprise and custom SSH configurations are supported
```

**Tip:** You can clone repositories first and use these to generate your `config.yaml`:

```bash
mkdir cloned_repos && cd "$_"
git clone http://github.com/example/project1.git
git clone http://github.com/example/project2.git
repos init
```

## Typical Session

Once you have a configuration file in place, an example session can look like the following:

```bash
# Remove existing repositories
repos rm

# Clone rust-based repositories in parallel
repos clone -t rust -p

# Run command to update dependencies in all repos
repos run "cargo update"

# Validate changes to see if updates were applied properly
find . -name "Cargo.lock" -exec ls -la {} \;

# Create pull requests for all changes
repos pr --title "Update dependencies" --body "Update Cargo.lock files"
```

## Usage

### Repository Management

To configure, clone and remove repositories:

```bash
# Scan current directory for git repositories
repos init

# Create a different output file
repos init -o my-repos-config.yaml

# Overwrite existing config file
repos init --overwrite

# Clone all repositories
repos clone

# Clone only repositories with tag "rust"
repos clone -t rust

# Clone in parallel
repos clone -p

# Use a custom config file
repos clone -c custom-config.yaml

# Remove cloned repositories
repos rm

# Remove only repositories with tag "rust"
repos rm -t rust

# Remove in parallel
repos rm -p
```

### Running Commands

To run arbitrary commands in repositories:

```bash
# Run a command in all repositories
repos run "cargo check"

# Run a command only in repositories with tag "rust"
repos run -t rust "cargo build"

# Run in parallel
repos run -p "cargo test"

# Specify a custom log directory
repos run -l custom/logs "make build"
```

#### Example Commands

Example commands to run with `repos run ""`:

```bash
# Count the number of lines
find . -type f | wc -l

# Build Rust projects (consider using --parallel flag)
cargo build

# Update dependencies
cargo update

# Format code
cargo fmt

# Run tests
cargo test

# Create a report of the changes made in the previous month
git log --all --author='$(id -un)' --since='1 month ago' --pretty=format:'%h %an %ad %s' --date=short
```

### Creating Pull Requests

To submit changes made in the cloned repositories:

```bash
export GITHUB_TOKEN=your_github_token

# Create PRs for repositories with changes
repos pr --title "My changes" --body "Description of changes"

# Create PRs with specific branch name
repos pr --branch feature/my-changes --title "My changes"

# Create draft pull requests
repos pr --draft

# Create PRs for specific repositories
repos pr -t backend
```

## Command Reference

```text
A tool to manage multiple GitHub repositories

Usage: repos [OPTIONS] <COMMAND>

Commands:
  clone  Clone repositories specified in config
  run    Run a command in each repository
  pr     Create pull requests for repositories with changes
  rm     Remove cloned repositories
  init   Create a config.yaml file from discovered Git repositories
  help   Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Configuration file path [default: config.yaml]
  -t, --tag <TAG>        Filter repositories by tag
  -p, --parallel         Execute operations in parallel
  -h, --help             Print help
  -V, --version          Print version
```

## Dependencies

- `clap` - Command line argument parsing
- `serde` & `serde_yaml` - Configuration file parsing
- `tokio` - Async runtime
- `reqwest` - HTTP client for GitHub API
- `colored` - Terminal colors
- `anyhow` - Error handling
- `chrono` - Date/time operations
- `walkdir` - Directory traversal
- `uuid` - Unique ID generation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo fmt`
6. Submit a pull request

### Documentation

- [Release Guide](docs/release.md) - Release conventions
- [Semantic Versioning Guide](docs/semantic.md) - How to use semantic versioning

## Alternatives

The following are the alternatives to `repos`:

- [gita](https://github.com/nosarthur/gita): A tool to manage multiple Git
repositories.
- [gr](http://mixu.net/gr): Another multi-repo management tool.
- [meta](https://github.com/mateodelnorte/meta): Helps in managing multiple
repositories.
- [mu-repo](https://fabioz.github.io/mu-repo): For managing many repositories.
- [myrepos](https://myrepos.branchable.com): A tool to manage multiple
repositories.
- [repo](https://android.googlesource.com/tools/repo): A repository management
tool often used for Android source code.

## License

MIT
