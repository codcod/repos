# Repos: manage multiple GitHub repositories

**Repos** is a CLI tool designed to streamline the management of multiple Git
repositories. Whether you need to clone, update, or create pull requests across
a handful of projects or thousands, `repos` provides the tools to do it
efficiently. With features like tag-based filtering, parallel execution, and
comprehensive logging, it's the ultimate utility for developers and DevOps
engineers working with complex microservice architectures.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Configuration](#configuration)
- [Docker Image](#docker-image)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Centralized Repository Management**: Define all your repositories in a
single, easy-to-manage `config.yaml` file.
- **Tag-Based Filtering**: Assign tags to your repositories and use them to run
commands on specific subsets of your projects (e.g., `backend`, `frontend`,
`java`, `rust`).
- **Inclusion and Exclusion**: Fine-tune your repository selection with both
include (`--tag`) and exclude (`--exclude-tag`) filters.
- **Parallel Execution**: Speed up your workflows by running commands across
multiple repositories simultaneously with the `--parallel` flag.
- **Pull Request Automation**: Create pull requests across dozens of
repositories with a single command.
- **Comprehensive Logging**: Every command run is logged, with detailed,
per-repository output files for easy debugging.
- **Built in Rust**: Fast, memory-safe, and reliable.

## Installation

### Homebrew (macOS)

```bash
brew tap codcod/taps
brew install repos
```

### Using Cargo

If you have the Rust toolchain installed, you can install `repos` directly from the source:

```bash
cargo install --path .
```

### From Source

```bash
git clone https://github.com/codcod/repos.git
cd repos
cargo build --release
sudo cp target/release/repos /usr/local/bin/
```

## Quick Start

The easiest way to get started is to let `repos` generate a configuration file
for you.

1. **Clone your repositories** into a single directory:

   ```bash
   mkdir my-projects && cd my-projects
   git clone https://github.com/my-org/project-one.git
   git clone https://github.com/my-org/project-two.git
   ```

2. **Generate the config file**:
   Run `repos init` in the `my-projects` directory. It will scan for Git
   repositories and create a `config.yaml` file.

   ```bash
   repos init
   ```

3. **Start managing your repos!**
   Now you can run commands across your projects. For example, to see the Git
   status for all of them:

   ```bash
   repos run "git status"
   ```

## Commands

`repos` offers a suite of commands to manage your repositories. Here is a brief
overview. Click on a command to see its detailed documentation.

| Command | Description |
|---|---|
| [**`clone`**](./docs/commands/clone.md) | Clones repositories from your config file. |
| [**`run`**](./docs/commands/run.md) | Runs a shell command in each repository. |
| [**`pr`**](./docs/commands/pr.md) | Creates pull requests for repositories with changes. |
| [**`rm`**](./docs/commands/rm.md) | Removes cloned repositories from your local disk. |
| [**`init`**](./docs/commands/init.md) | Generates a `config.yaml` file from local Git repositories. |

For a full list of options for any command, run `repos <COMMAND> --help`.

## Configuration

The `config.yaml` file is the heart of `repos`. It defines your repositories and
their metadata.

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

## Docker Image

You can use `repos` within a Docker container, which is great for CI/CD
pipelines.

### Build the Image

From the root of the `repos` source directory:

```bash
docker build -t repos:latest .
```

### Run Commands

To run a command, you can pass it to the container. Remember to pass your
`GITHUB_TOKEN` if you're creating pull requests.

```bash
# View help menu
docker run --rm repos:latest --help

# Create a pull request
docker run --rm \
  -e GITHUB_TOKEN="$GITHUB_TOKEN" \
  repos:latest pr --title "fix: update config" --body "Detailed description"
```

To operate on local files, mount your current working directory into the
container:

```bash
docker run --rm -v "$(pwd):/work" -w /work repos:latest run "ls -la"
```

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/my-new-feature`).
3. Make your changes.
4. Add tests if applicable.
5. Run `cargo test` and `cargo fmt` to ensure everything is in order.
6. Submit a pull request.

### Documentation

- [Release Guide](./docs/release.md)
- [Semantic Versioning Guide](./docs/semantic.md)
- [Release Strategy](./docs/release-strategy.md)

## Alternatives

If `repos` isn't quite what you're looking for, check out these other great
tools:

- [gita](https://github.com/nosarthur/gita)
- [gr](http://mixu.net/gr)
- [meta](https://github.com/mateodelnorte/meta)
- [mu-repo](https://fabioz.github.io/mu-repo)
- [myrepos](https://myrepos.branchable.com)
- [repo](https://android.googlesource.com/tools/repo)

## License

This project is licensed under the MIT License.
