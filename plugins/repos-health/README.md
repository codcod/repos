# repos-health

A health check plugin for the repos tool that:

- Scans each repository for `package.json` files
- Checks for outdated npm dependencies using `npm outdated`
- Updates dependencies using `npm update`
- Creates git branches and commits changes
- Opens pull requests for dependency updates

## Requirements

- Node.js and npm installed
- Git repository with push permissions
- GitHub token configured for PR creation

## Usage

```bash
repos health
```

The plugin will:

1. Load the default repos configuration
2. Process each repository that contains a `package.json`
3. Check for outdated dependencies
4. Update dependencies if found
5. Create a branch and commit changes
6. Push the branch and open a PR

## Output

The plugin reports:

- Repositories processed
- Outdated packages found
- Successful dependency updates
- PR creation status
