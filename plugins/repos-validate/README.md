# repos-validate Plugin

The `repos-validate` plugin checks your `config.yaml` file for correctness,
verifies repository accessibility, and can synchronize GitHub topics with local
tags.

## Features

- **Configuration Syntax Validation**: Confirms that `config.yaml` is properly
formatted and parseable.
- **Repository Connectivity Check**: Verifies that each repository exists and is
accessible.
- **Topic Synchronization**: Synchronizes GitHub topics with config tags, adding
missing topics from GitHub to your local config and prefixing them with `gh:`.
- **Automatic Backup**: Creates timestamped backups before modifying `config.yaml`.

## Installation

The plugin is installed as part of the `repos` workspace. You can build it from
the root of the project:

```bash
cargo build --release
```

The binary will be located at `target/release/repos-validate`. Ensure this
location is in your `PATH`, or move the binary to a directory like
`/usr/local/bin`.

## Usage

```bash
repos validate [OPTIONS]
```

## Description

This command performs several levels of validation:

1. **Syntax Validation (Default)**: By default, `repos validate` only parses the
`config.yaml` file to ensure it is well-formed. No network calls are made.
2. **Connectivity Check**: When the `--connect` flag is added, it performs the
syntax check and also attempts to connect to the Git remote URL for each
repository. This verifies that the repository exists and that you have the
necessary access permissions.
3. **GitHub Topic Synchronization**: The `--sync-topics` option (which requires
`--connect`) compares the tags in your `config.yaml` with the repository topics
on GitHub. It will suggest adding missing topics as tags (with a `gh:` prefix)
to your configuration. By itself, this option only shows a diff of the suggested
changes.

This command is essential for ensuring your configuration is correct before
running bulk operations like `clone` or `run`.

## Options

- `-c, --config <CONFIG>`: Specifies the path to the configuration file.
Defaults to `config.yaml`.
- `--connect`: Checks network connectivity by attempting to connect to each
repository's remote URL.
- `--sync-topics`: Must be used with `--connect`. Compares repository topics on
GitHub with local tags in `config.yaml` and suggests changes. It will prefix
tags sourced from GitHub with `gh:`. This option only prints a diff of the
suggested changes; it does not modify the file.
- `--apply`: Must be used with `--sync-topics`. Applies the suggested topic
synchronization changes directly to your `config.yaml` file. A backup of the
original `config.yaml` will be created before changes are written.
- `-h, --help`: Prints help information.

## Examples

### Validate config syntax only

This is the default behavior. It runs quickly and performs no network
operations.

```bash
repos validate
```

Example output:

```console
✅ config.yaml syntax is valid.
```

### Validate syntax and check repository connectivity

This will check the config file and also verify that every repository URL is
accessible.

```bash
repos validate --connect
```

Example output:

```console
✅ config.yaml syntax is valid.

Validating repository connectivity...
✅ codcod/repos: Accessible.
✅ another/project: Accessible.

Validation finished successfully.
```

### Preview topic synchronization changes

This will check connectivity and show you which topics are on GitHub but are
missing from the tags in your `config.yaml`. No files will be changed.

```bash
repos validate --connect --sync-topics
```

Example output:

```console
✅ config.yaml syntax is valid.

Validating repository connectivity...
✅ codcod/repos: Accessible.
    - Would add: ["gh:cli", "gh:rust", "gh:automation"]
✅ another/project: Accessible.
    - Topics already synchronized

Validation finished successfully.
```

### Apply topic synchronization changes

This will check connectivity, find missing topics, and automatically add them as
tags (with a `gh:` prefix) to your `config.yaml`.

```bash
repos validate --connect --sync-topics --apply
```

Example output:

```console
✅ config.yaml syntax is valid.

Validating repository connectivity...
✅ codcod/repos: Accessible.
    - Topics to add: ["gh:cli", "gh:rust", "gh:automation"]
✅ another/project: Accessible.
    - Topics already synchronized

Validation finished successfully.

Applying topic synchronization to config.yaml...
✅ Created backup: "config.yaml.backup.20251111_143022"
✅ Successfully updated config.yaml
   1 repositories were synchronized
```

## Backup Files

When using `--apply`, the plugin automatically creates a backup before modifying
your configuration. Backup files are named with a timestamp pattern:

```console
config.yaml.backup.YYYYMMDD_HHMMSS
```

You can restore from a backup at any time:

```bash
cp config.yaml.backup.20251111_143022 config.yaml
```

## Authentication

For private repositories or to avoid rate limiting, set your GitHub token:

```bash
export GITHUB_TOKEN=your_github_personal_access_token
repos validate --connect
```

## Exit Codes

- `0`: All checks passed successfully.
- `1`: One or more validation checks failed.

## Supported Repository URL Formats

- SSH: `git@github.com:owner/repo.git`
- HTTPS: `https://github.com/owner/repo.git`
