# repos-validate

A plugin for the `repos` CLI tool that validates your `config.yaml` file and checks connectivity to all configured repositories.

## Features

- **Configuration Syntax Validation**: Confirms that `config.yaml` is properly formatted and parseable
- **Repository Connectivity Check**: Optionally verifies that each repository exists and is accessible via the GitHub API (with `--connect`)
- **Topic Synchronization**: Synchronizes GitHub topics with config tags - adds missing topics and removes outdated gh: tags (with `--sync-topics`)
- **Automatic Backup**: Creates timestamped backups before modifying `config.yaml`

## Installation

Build and install the plugin as part of the `repos` workspace:

```bash
cargo build --release
sudo cp target/release/repos-validate /usr/local/bin/
```

Or install it directly:

```bash
cd plugins/repos-validate
cargo install --path .
```

## Usage

### Basic Validation

Validate your configuration syntax:

```bash
repos validate
```

Example output:

```console
✅ config.yaml syntax is valid.

Validation finished successfully.
```

### Check Repository Connectivity

Validate configuration and check that all repositories are accessible:

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

### Synchronize GitHub Topics

Preview which GitHub topics would be synchronized (added/removed) with config tags:

```bash
repos validate --connect --sync-topics
```

Example output:

```console
✅ config.yaml syntax is valid.

Validating repository connectivity...
✅ codcod/repos: Accessible.
    - Would add: ["gh:cli", "gh:rust", "gh:automation"]
    - Would remove: ["gh:deprecated-topic"]
✅ another/project: Accessible.
    - Topics already synchronized

Validation finished successfully.
```

### Apply Topic Synchronization

To actually update your `config.yaml` with synchronized GitHub topics, use the `--apply` flag:

```bash
repos validate --connect --sync-topics --apply
```

This will:

1. Create a timestamped backup of your `config.yaml` (e.g., `config.yaml.backup.20251111_143022`)
2. Fetch topics from GitHub for each repository
3. Add missing topics as tags (prefixed with `gh:`)
4. Remove outdated `gh:` tags that no longer exist in GitHub topics

Example output:

```console
✅ config.yaml syntax is valid.

Validating repository connectivity...
✅ codcod/repos: Accessible.
    - Topics to add: ["gh:cli", "gh:rust", "gh:automation"]
    - Topics to remove: ["gh:deprecated-topic"]
✅ another/project: Accessible.
    - Topics already synchronized

Validation finished successfully.

Applying topic synchronization to config.yaml...
✅ Created backup: "config.yaml.backup.20251111_143022"
✅ Successfully updated config.yaml
   1 repositories were synchronized
```

## Backup Files

When using `--apply`, the plugin automatically creates a backup before modifying your configuration. Backup files are named with a timestamp pattern:

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
repos validate
```

## Exit Codes

- `0`: All repositories are accessible
- `1`: One or more repositories failed connectivity check

## Supported Repository URL Formats

- SSH: `git@github.com:owner/repo.git`
- HTTPS: `https://github.com/owner/repo.git`
