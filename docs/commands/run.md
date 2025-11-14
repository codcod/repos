# repos run

The `run` command executes a shell command or a named recipe in each of the
specified repositories.

## Usage

To run a command:

```bash
repos run [OPTIONS] [COMMAND] [REPOS]...
```

To run a recipe:

```bash
repos run [OPTIONS] --recipe <RECIPE_NAME> [REPOS]...
```

## Description

This is one of the most powerful commands in `repos`, allowing you to automate
tasks across hundreds or thousands of repositories at once. You can run any
shell command, from simple `ls` to complex `docker build` scripts.

Additionally, you can define **recipes**—multi-step scripts—in your
`config.yaml` and execute them by name using the `--recipe` option. This is
perfect for standardizing complex workflows like dependency updates, code
generation, or release preparation.

By default, the output of each command is logged to a file in the `output/runs/`
directory, but this can be disabled.

## Arguments

- `[COMMAND]`: The shell command to execute. This is a positional argument. It
should be enclosed in quotes if it contains spaces or special characters.
- `[REPOS]...`: A space-separated list of specific repository names to run the
command in. If not provided, filtering will be based on tags.

## Options

- `-c, --config <CONFIG>`: Path to the configuration file. Defaults to
`config.yaml`.
- `-r, --recipe <RECIPE_NAME>`: The name of the recipe to run. This option is
mutually exclusive with the `COMMAND` argument.
- `-t, --tag <TAG>`: Filter repositories by tag. Can be specified multiple times
(OR logic).
- `-e, --exclude-tag <EXCLUDE_TAG>`: Exclude repositories with a specific tag.
Can be specified multiple times.
- `-p, --parallel`: Execute the command or recipe in parallel across all
selected repositories.
- `--no-save`: Disables saving the command output to log files.
- `--output-dir <OUTPUT_DIR>`: Specifies a custom directory for log files
instead of the default `output/runs`.
- `-h, --help`: Prints help information.

## Recipes

Recipes are named, multi-step scripts defined in your `config.yaml`. They allow
you to codify and reuse common workflows.

### Defining a Recipe

Add a `recipes` section to your `config.yaml`:

```yaml
recipes:
  - name: update-deps
    steps:
      - git checkout main
      - git pull
      - cargo update
      - cargo build --release

  - name: test
    steps:
      - cargo test --all-features
      - cargo clippy
```

Each recipe has a `name` and a list of `steps`. Each step is a shell command
executed sequentially.

### Running a Recipe

To run a recipe, use its name with the `--recipe` option.

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

### Run a command without saving output

Useful for quick, simple commands where you don't need a record of the output.

```bash
repos run --no-save "ls -la"
```

### Run the 'update-deps' recipe on all repositories

```bash
repos run --recipe update-deps
```

### Run the 'test' recipe on backend repositories in parallel

```bash
repos run -t backend -p --recipe test
```
