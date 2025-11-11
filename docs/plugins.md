# Plugin System

The `repos` tool supports an extensible plugin system that allows you to add new functionality without modifying the core codebase. This is implemented using Phase 1 of the plugin architecture: external command plugins.

## How It Works

The plugin system follows the same pattern as Git's external subcommands:

- Any executable named `repos-<plugin>` in your `PATH` becomes a plugin
- When you run `repos <plugin> <args>`, the tool automatically finds and executes `repos-<plugin>` with the provided arguments
- **NEW**: The core `repos` CLI automatically handles common options (`--config`, `--tag`, `--exclude-tag`, `--debug`) and passes filtered context to plugins via environment variables
- This provides complete isolation, crash safety, and the ability to write plugins in any language

## Context Injection (Simplified Plugin Development)

As of version 0.2.0, plugins can opt into receiving pre-processed context from the core `repos` CLI. This means plugins don't need to:

- Parse `--config`, `--tag`, `--exclude-tag`, `--debug` options themselves
- Load and parse the YAML configuration file
- Apply tag filtering logic

### How Context Injection Works

When you run:

```bash
repos health --config custom.yaml --tag flow --exclude-tag deprecated prs
```

The core CLI:

1. Parses `--config`, `--tag`, `--exclude-tag` options
2. Loads the config file
3. Applies tag filtering (28 repos → 5 repos matching criteria)
4. Serializes filtered repositories to a temp JSON file
5. Sets environment variables:
   - `REPOS_PLUGIN_PROTOCOL=1` (indicates context injection is available)
   - `REPOS_FILTERED_REPOS_FILE=/tmp/repos-xxx.json` (path to filtered repos)
   - `REPOS_DEBUG=1` (if --debug flag was passed)
   - `REPOS_TOTAL_REPOS=28` (total repos in config)
   - `REPOS_FILTERED_COUNT=5` (repos after filtering)
   - `REPOS_CONFIG_FILE=/path/to/your/config.yaml` (path to config file)
6. Executes `repos-health prs` with only plugin-specific args

### Using Context Injection in Your Plugin

**Rust Example:**

```rust
use anyhow::Result;
use repos::{Repository, load_plugin_context, is_debug_mode};

#[tokio::main]
async fn main() -> Result<()> {
    // Try to load injected context
    let repos = if let Some(repos) = load_plugin_context()? {
        // New protocol: use pre-filtered repos from core CLI
        let debug = is_debug_mode();
        if debug {
            eprintln!("Using injected context with {} repos", repos.len());
        }
        repos
    } else {
        // Legacy fallback: parse args and load config manually
        // (for backwards compatibility when run directly)
        load_config_manually()?
    };

    // Now just implement your plugin logic
    for repo in repos {
        println!("Processing: {}", repo.name);
        // Your plugin functionality here
    }

    Ok(())
}
```

**Python Example:**

```python
#!/usr/bin/env python3
import os
import json
import sys

def main():
    # Check if context injection is available
    if os.environ.get('REPOS_PLUGIN_PROTOCOL') == '1':
        # Load pre-filtered repositories
        repos_file = os.environ.get('REPOS_FILTERED_REPOS_FILE')
        with open(repos_file, 'r') as f:
            repos = json.load(f)

        debug = os.environ.get('REPOS_DEBUG') == '1'
        if debug:
            total = os.environ.get('REPOS_TOTAL_REPOS', '?')
            print(f"Using injected context: {len(repos)}/{total} repos", file=sys.stderr)
    else:
        # Legacy fallback: parse args and load config manually
        repos = load_config_manually()

    # Implement plugin logic with filtered repos
    for repo in repos:
        print(f"Processing: {repo['name']}")
        # Your plugin functionality here

if __name__ == '__main__':
    main()
```

**Bash Example:**

```bash
#!/bin/bash

# Check if context injection is available
if [ "$REPOS_PLUGIN_PROTOCOL" = "1" ]; then
    # Load pre-filtered repositories
    REPOS=$(cat "$REPOS_FILTERED_REPOS_FILE")

    if [ "$REPOS_DEBUG" = "1" ]; then
        echo "Using injected context: $REPOS_FILTERED_COUNT/$REPOS_TOTAL_REPOS repos" >&2
    fi

    # Process filtered repos using jq
    echo "$REPOS" | jq -r '.[] | .name' | while read -r repo_name; do
        echo "Processing: $repo_name"
        # Your plugin functionality here
    done
else
    # Legacy fallback: parse args and load config manually
    # (for backwards compatibility when run directly)
    echo "Loading config manually..." >&2
    # ... manual config loading logic ...
fi
```

### Benefits of Context Injection

1. **Less boilerplate**: No need to parse common CLI options
2. **Consistent behavior**: Filtering works the same across all plugins
3. **Better performance**: Config loaded once, not per plugin
4. **Backwards compatible**: Plugins still work when run directly
5. **Language agnostic**: Available via environment variables

### Supported Common Options

When invoking plugins through `repos <plugin>`, these options are automatically handled:

- `--config <path>` or `-c <path>`: Custom config file
- `--tag <tag>` or `-t <tag>`: Filter repos by tag (can be repeated)
- `--exclude-tag <tag>` or `-e <tag>`: Exclude repos by tag (can be repeated)
- `--debug` or `-d`: Enable debug output

All other arguments are passed to the plugin as-is.

## Creating a Plugin

To create a plugin:

1. **Create an executable** named `repos-<name>` where `<name>` is your plugin name
2. **Make it executable** (`chmod +x repos-<name>`)
3. **Add it to your PATH** so the `repos` tool can find it

### Example: Health Plugin

Here's a simple example of a health check plugin written in bash:

```bash
#!/bin/bash
# Save as: repos-health

echo "=== Repository Health Check ==="

# Parse arguments
CONFIG_FILE="config.yaml"
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -c|--config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Using config: $CONFIG_FILE"
echo "Verbose mode: $VERBOSE"

# Add your health check logic here
# You can parse the YAML config file and iterate over repositories
# Example checks:
# - Check for outdated dependencies (cargo outdated, npm outdated, etc.)
# - Analyze cognitive complexity (radon, lizard, etc.)
# - Security audits (cargo audit, npm audit, etc.)
# - Code coverage statistics
# - Git repository health (uncommitted changes, etc.)

echo "Health check completed!"
```

### Example: Security Plugin in Python

```python
#!/usr/bin/env python3
# Save as: repos-security

import argparse
import yaml
import subprocess
import sys

def main():
    parser = argparse.ArgumentParser(description='Security audit for repositories')
    parser.add_argument('-c', '--config', default='config.yaml', help='Config file path')
    parser.add_argument('--fix', action='store_true', help='Attempt to fix issues automatically')
    args = parser.parse_args()

    # Load configuration
    try:
        with open(args.config, 'r') as f:
            config = yaml.safe_load(f)
    except FileNotFoundError:
        print(f"Error: Config file '{args.config}' not found", file=sys.stderr)
        sys.exit(1)

    repositories = config.get('repositories', [])

    print("=== Security Audit ===")

    for repo in repositories:
        name = repo['name']
        path = repo.get('path', f"./{name}")

        print(f"\n🔍 Auditing {name}...")

        # Example security checks
        if check_rust_security(path):
            print(f"  ✅ {name}: No security issues found")
        else:
            print(f"  ⚠️  {name}: Security issues detected")

def check_rust_security(repo_path):
    """Run cargo audit for Rust projects"""
    try:
        result = subprocess.run(
            ['cargo', 'audit'],
            cwd=repo_path,
            capture_output=True,
            text=True
        )
        return result.returncode == 0
    except FileNotFoundError:
        # cargo not available, skip Rust checks
        return True

if __name__ == '__main__':
    main()
```

## Using Plugins

### List Available Plugins

```bash
repos --list-plugins
```

This command scans your `PATH` for any executables matching the `repos-*` pattern and displays them.

### Execute a Plugin

```bash
repos <plugin-name> [arguments...]
```

Examples:

```bash
# Run health check with default config
repos health

# Run health check with custom config and verbose output
repos health -c my-config.yaml -v

# Run security audit
repos security --config production.yaml

# Run security audit with auto-fix
repos security --fix
```

## Plugin Guidelines

### Naming

- Plugin executables must be named `repos-<name>`
- Choose descriptive, lowercase names
- Use hyphens for multi-word names (e.g., `repos-code-quality`)

### Arguments

- Follow Unix conventions for command-line arguments
- Support `-h` or `--help` for usage information
- Consider supporting `-c` or `--config` for custom config files
- Use long options with double dashes (`--verbose`) for clarity

### Output

- Use clear, structured output
- Consider using emoji or symbols for visual feedback (✅ ❌ ⚠️)
- Write errors to stderr, normal output to stdout
- Use appropriate exit codes (0 for success, non-zero for errors)

### Integration

- Plugins should work with the standard `config.yaml` format
- Parse the YAML configuration to access repository information
- Consider the repository structure (name, path, tags, etc.)

## Plugin Development Tips

### Configuration Access

Most plugins will need to read the repos configuration file. Here's how to parse it in different languages:

**Bash (using yq):**

```bash
# Install yq: brew install yq (macOS) or similar
repos=$(yq eval '.repositories[].name' config.yaml)
```

**Python:**

```python
import yaml
with open('config.yaml', 'r') as f:
    config = yaml.safe_load(f)
    repositories = config.get('repositories', [])
```

**Rust:**

```rust
use serde_yaml;
use std::fs;

let content = fs::read_to_string("config.yaml")?;
let config: serde_yaml::Value = serde_yaml::from_str(&content)?;
```

### Error Handling

- Always validate input arguments
- Check if required tools are available before using them
- Provide helpful error messages
- Use appropriate exit codes

### Testing

- Create test repositories for development
- Test with different repository structures
- Verify behavior with missing or invalid configurations

## Limitations

This Phase 1 implementation has some limitations that future phases may address:

- No built-in dependency management for plugins
- No plugin metadata or versioning system
- No automatic plugin updates
- Limited inter-plugin communication

## Future Phases

The plugin system is designed for gradual expansion:

- **Phase 2**: Plugin registry and installation system
- **Phase 3**: Plugin API for deeper integration
- **Phase 4**: Plugin dependency management and sandboxing

For now, Phase 1 provides a solid foundation for extending the repos tool with external functionality while maintaining simplicity and safety.
