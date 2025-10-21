# repos run

The `run` command executes a shell command in each of the specified
repositories.

## Usage

```bash
repos run [OPTIONS] <COMMAND> [REPOS]...
```

## Description

This is one of the most powerful commands in `repos`, allowing you to automate
tasks across hundreds or thousands of repositories at once. You can run any
shell command, from simple `ls` to complex `docker build` or `terraform apply`
scripts.

By default, the output of each command is logged to a file in the `logs/runs/`
directory, but this can be disabled.

## Arguments

- `<COMMAND>`: The shell command to execute in each repository. It should be
enclosed in quotes if it contains spaces or special characters.
- `[REPOS]...`: A space-separated list of specific repository names to run the
command in. If not provided, filtering will be based on tags.

## Options

- `-c, --config <CONFIG>`: Path to the configuration file. Defaults to
`config.yaml`.
- `-t, --tag <TAG>`: Filter repositories by tag. Can be specified multiple times
(OR logic).
- `-e, --exclude-tag <EXCLUDE_TAG>`: Exclude repositories with a specific tag.
Can be specified multiple times.
- `-p, --parallel`: Execute the command in parallel across all selected
repositories.
- `--no-save`: Disables saving the command output to log files.
- `--output-dir <OUTPUT_DIR>`: Specifies a custom directory for log files
instead of the default `logs/runs`.
- `-h, --help`: Prints help information.

## Examples

### Run a command on all repositories

```bash
repos run "git status"
```

### Run a command on repositories with a specific tag

```bash
repos run -t backend "mvn clean install"
```

### Run a command on repositories matching multiple tags

This will run `npm install` on repositories tagged with either `frontend` or
`javascript`.

```bash
repos run -t frontend -t javascript "npm install"
```

### Exclude repositories from a run

This will run `cargo check` on all repositories *except* for those tagged as
`archived`.

```bash
repos run -e archived "cargo check"
```

### Run a command in parallel

This is highly recommended for long-running commands to save time.

```bash
repos run -p "docker build ."
```

### Run a command without saving logs

Useful for quick, simple commands where you don't need a record of the output.

```bash
repos run --no-save "ls -la"
```
