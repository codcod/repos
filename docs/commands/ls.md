# repos ls

The `ls` command lists repositories specified in your `repos.yaml` file with
optional filtering capabilities.

## Usage

```bash
repos ls [OPTIONS] [REPOS]...
```

## Description

This command is used to display information about repositories defined in your
configuration. It's particularly useful for reviewing which repositories will be
included when using specific tag filters, helping you preview the scope of
operations before running commands like `clone`, `run`, or `pr`.

The output includes repository names, URLs, tags, configured paths, and branches
for each repository.

## Arguments

- `[REPOS]...`: A space-separated list of specific repository names to list. If
not provided, `repos` will fall back to filtering by tags or listing all
repositories defined in the config.

## Options

- `-c, --config <CONFIG>`: Specifies the path to the configuration file.
Defaults to `repos.yaml`.
- `-t, --tag <TAG>`: Filters repositories to list only those that have the
specified tag. This option can be used multiple times to include repositories
with *any* of the specified tags (OR logic).
- `-e, --exclude-tag <EXCLUDE_TAG>`: Excludes repositories that have the
specified tag. This can be used to filter out repositories from the listing.
This option can be used multiple times.
- `-h, --help`: Prints help information.

## Output Format

For each repository, the command displays:

- **Name**: The repository identifier
- **URL**: The Git remote URL
- **Tags**: Associated tags (if any)
- **Path**: Configured local path (if specified)
- **Branch**: Configured branch (if specified)

The output also includes a summary showing the total count of repositories found.

## Examples

### List all repositories

```bash
repos ls
```

### List specific repositories by name

```bash
repos ls repo-one repo-two
```

### List repositories with a specific tag

This is particularly useful to see which repositories will be affected when
running commands with the same tag filter.

```bash
repos ls --tag backend
```

### List repositories with multiple tags

This will list repositories that have *either* the `frontend` or the `rust`
tag.

```bash
repos ls -t frontend -t rust
```

### Exclude repositories with a specific tag

This will list all repositories *except* those with the `deprecated` tag.

```bash
repos ls --exclude-tag deprecated
```

### Combine inclusion and exclusion

This will list all repositories with the `backend` tag but exclude those that
also have the `deprecated` tag.

```bash
repos ls -t backend -e deprecated
```

### Preview before cloning

Before cloning repositories with a specific tag, you can preview which ones will
be affected:

```bash
# Preview which repositories have the 'flow' tag
repos ls --tag flow

# Then clone them
repos clone --tag flow
```

### Use with custom config

```bash
repos ls --config path/to/custom-repos.yaml
```

## Use Cases

1. **Preview Tag Filters**: Check which repositories will be included in
   operations that use the same tag filters.

2. **Explore Configuration**: Quickly view all repositories defined in your
   config without needing to open the file.

3. **Verify Tags**: Ensure repositories are properly tagged before running bulk
   operations.

4. **Review Paths**: Check configured paths and branches for repositories.

5. **Filter Testing**: Experiment with different tag combinations to understand
   how filters work before applying them to operations like `clone` or `run`.
