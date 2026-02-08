# repos clone

The `clone` command clones repositories specified in your `repos.yaml` file
into your local workspace.

## Usage

```bash
repos clone [OPTIONS] [REPOS]...
```

## Description

This command is used to bring remote repositories to your local machine based on
the configurations you've set. You can clone all repositories, or filter them by
name or by tags.

## Arguments

- `[REPOS]...`: A space-separated list of specific repository names to clone. If
not provided, `repos` will fall back to filtering by tags or cloning all
repositories defined in the config.

## Options

- `-c, --config <CONFIG>`: Specifies the path to the configuration file.
Defaults to `repos.yaml`.
- `-t, --tag <TAG>`: Filters repositories to clone only those that have the
specified tag. This option can be used multiple times to include repositories
with *any* of the specified tags (OR logic).
- `-e, --exclude-tag <EXCLUDE_TAG>`: Excludes repositories that have the
specified tag. This can be used to filter out repositories from a selection.
This option can be used multiple times.
- `-p, --parallel`: Executes the clone operations in parallel for faster
performance.
- `-h, --help`: Prints help information.

## Examples

### Clone all repositories

```bash
repos clone
```

### Clone specific repositories by name

```bash
repos clone repo-one repo-two
```

### Clone repositories with a specific tag

```bash
repos clone --tag backend
```

### Clone repositories with multiple tags

This will clone repositories that have *either* the `frontend` or the `rust`
tag.

```bash
repos clone -t frontend -t rust
```

### Exclude repositories with a specific tag

This will clone all repositories *except* those with the `java` tag.

```bash
repos clone --exclude-tag java
```

### Combine inclusion and exclusion

This will clone all repositories with the `backend` tag but exclude those that
also have the `deprecated` tag.

```bash
repos clone -t backend -e deprecated
```

### Clone in parallel

For large numbers of repositories, using the parallel flag can significantly
speed up the process.

```bash
repos clone --parallel
```
