# repos rm

The `rm` command removes cloned repositories from your local filesystem.

## Usage

```bash
repos rm [OPTIONS] [REPOS]...
```

## Description

This command is the counterpart to `clone`. It deletes the directories of the
specified repositories. This is useful for cleaning up your workspace or before
re-cloning repositories to get a fresh start.

## Arguments

- `[REPOS]...`: A space-separated list of specific repository names to remove.
If not provided, filtering will be based on tags.

## Options

- `-c, --config <CONFIG>`: Path to the configuration file. Defaults to
`config.yaml`.
- `-t, --tag <TAG>`: Filter repositories to remove only those with the specified
tag. Can be used multiple times.
- `-e, --exclude-tag <EXCLUDE_TAG>`: Exclude repositories with a specific tag
from being removed.
- `-p, --parallel`: Executes the removal operations in parallel.
- `-h, --help`: Prints help information.

## Examples

### Remove all repositories

This will remove all repositories defined in your config file from your local
disk.

```bash
repos rm
```

### Remove specific repositories

```bash
repos rm repo-one repo-three
```

### Remove repositories with a specific tag

```bash
repos rm --tag frontend
```

### Remove all but certain repositories

This will remove all repositories *except* for those tagged as `production`.

```bash
repos rm -e production
```

### Remove repositories in parallel

```bash
repos rm -p
```
