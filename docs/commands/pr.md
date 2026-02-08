# repos pr

The `pr` command creates pull requests in repositories that have local changes.

## Usage

```bash
repos pr [OPTIONS] [REPOS]...
```

## Description

This command automates the process of creating pull requests. It will:

1. Identify repositories with uncommitted changes.
2. Create a new branch.
3. Add and commit all changes.
4. Push the branch to the remote.
5. Create a pull request on GitHub.

A `GITHUB_TOKEN` environment variable is required for authentication.

## Arguments

- `[REPOS]...`: A space-separated list of repository names to create PRs for. If
omitted, filters by tags.

## Options

- `--title <TITLE>`: The title of the pull request. Default: "Automated
changes".
- `--body <BODY>`: The body text of the pull request. Default: "This PR was
created automatically".
- `--branch <BRANCH>`: The name of the new branch to create. If not provided, a
name will be generated automatically.
- `--base <BASE>`: The base branch for the pull request (e.g., `main`,
`develop`). If not provided, the repository's default branch is used.
- `--message <MESSAGE>`: The commit message. If not provided, it defaults to the
PR title.
- `--draft`: Creates the pull request as a draft.
- `--token <TOKEN>`: Your GitHub personal access token. Can also be provided via
the `GITHUB_TOKEN` environment variable.
- `--create-only`: A "dry-run" mode. It prepares the PR but does not create it
on GitHub.
- `-c, --config <CONFIG>`: Path to the configuration file. Defaults to
`repos.yaml`.
- `-t, --tag <TAG>`: Filter repositories by tag. Can be specified multiple
times.
- `-e, --exclude-tag <EXCLUDE_TAG>`: Exclude repositories with a specific tag.
- `-p, --parallel`: Execute PR creation in parallel.
- `-h, --help`: Prints help information.

## Examples

### Create a basic pull request

This will create a PR in all repositories with changes, using default values for
title, body, and branch name.

```bash
export GITHUB_TOKEN=your_github_token
repos pr --title "Apply latest security patches"
```

### Create a PR with a specific branch and base

```bash
repos pr --branch feature/new-api --base develop --title "Feature: New API"
```

### Create a draft pull request

```bash
repos pr --title "WIP: Experimental changes" --draft
```

### Create PRs only for backend repositories

```bash
repos pr -t backend --title "Backend-specific updates"
```

### Exclude certain repositories

This creates PRs for all repositories *except* those tagged as `legacy`.

```bash
repos pr -e legacy --title "Modernization updates"
```
