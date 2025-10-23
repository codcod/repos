# Plugin System

The `repos` tool supports an extensible plugin system that allows you to add new functionality without modifying the core codebase. This is implemented using Phase 1 of the plugin architecture: external command plugins.

## How It Works

The plugin system follows the same pattern as Git's external subcommands:

- Any executable named `repos-<plugin>` in your `PATH` becomes a plugin
- When you run `repos <plugin> <args>`, the tool automatically finds and executes `repos-<plugin>` with the provided arguments
- This provides complete isolation, crash safety, and the ability to write plugins in any language

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
