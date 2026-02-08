# repos init

The `init` command scans your local filesystem for Git repositories and
generates a `repos.yaml` file from them.

## Usage

```bash
repos init [OPTIONS]
```

## Description

This command is the easiest way to get started with `repos`. Instead of writing
a `repos.yaml` file by hand, you can clone your repositories into a directory
and then run `repos init` to automatically generate the configuration. It will
discover all Git repositories in the current directory and its subdirectories.

## Options

- `-o, --output <OUTPUT>`: Specifies the name of the output configuration file.
Defaults to `repos.yaml`.
- `--overwrite`: If a configuration file already exists at the output path, this
flag allows `repos` to overwrite it.
- `--supplement`: If a configuration file already exists, this flag will add
newly discovered repositories to the existing file without removing the ones
that are already there.
- `-h, --help`: Prints help information.

## Examples

### Generate a new config file

First, clone some repositories, then run `init`.

```bash
mkdir my-projects
cd my-projects
git clone https://github.com/owner/project-one.git
git clone https://github.com/owner/project-two.git

repos init
```

This will create a `repos.yaml` file in the `my-projects` directory.

### Generate a config with a custom name

```bash
repos init --output my-company-repos.yaml
```

### Overwrite an existing config

```bash
repos init --overwrite
```

### Add new repositories to an existing config

If you have an existing `repos.yaml` and have cloned new repositories, you can
add them without losing your existing configuration (including tags, custom
paths, etc.).

```bash
# repos.yaml already exists
git clone https://github.com/owner/new-project.git
repos init --supplement
```
