# repos-fix

Automatically analyze and fix JIRA maintenance tickets using Cursor AI.

## Overview

The `repos-fix` plugin integrates JIRA issue tracking with the Cursor AI agent to automatically implement fixes for maintenance tickets. It operates as a plugin for the `repos` tool.

Key features:

1. **Fetches JIRA ticket details**: including description, priority, and attachments.
2. **Analyzes the codebase**: Detects platform (Java, iOS, Android, Angular), frameworks, and test structure.
3. **Generates comprehensive prompts**: Creates a "mission" for Cursor AI tailored to the specific project context.
4. **Runs cursor-agent**: Executes the fix in headless mode with auto-retries.
5. **Validates the implementation**: verifying build and tests pass.

## Prerequisites

- `repos` tool installed (this plugin is included with it).
- `cursor-agent` CLI installed and available in PATH.
- **JIRA Account**: with API token access.
- **Cursor API Key**: for the AI agent.

## Installation

1. **repos tool**: Ensure you have the `repos` tool installed. `repos-fix` is a built-in plugin.
2. **cursor-agent**: Install the Cursor Agent CLI:

    ```bash
    curl https://cursor.com/install -fsS | bash
    ```

    Verify installation with `cursor-agent --version`.

## Configuration

Set the following environment variables:

```bash
# JIRA Configuration
export JIRA_URL=https://your-company.atlassian.net
export JIRA_USERNAME=your-email@company.com
export JIRA_API_TOKEN=your-jira-api-token

# Cursor API Key
export CURSOR_API_KEY=your-cursor-api-key
```

- **JIRA API Token**: Generate at [id.atlassian.com](https://id.atlassian.com/manage-profile/security/api-tokens).
- **Cursor API Key**: Get it from Cursor Settings → General → API Keys.

### Template Overrides

You can customize the AI prompts and guidelines by placing files in your configuration directory:

- `${XDG_CONFIG_HOME}/repos/fix/` (usually `~/.config/repos/fix/`)

Supported files:

- `cursor_prompt.md`: The main instruction set for Cursor.
- `cursorrules.md`: Behavior rules for the agent.
- `agent_prompt.md`: The mission prompt passed to `cursor-agent`.
- Platform guidelines: `guidelines_ios.md`, `guidelines_android.md`, `guidelines_java.md`, `guidelines_angular.md`.

## Usage

### Basic Usage

Fix a JIRA ticket in a specific repository:

```bash
# Single repository (by name)
repos fix my-backend-service --ticket MAINT-1234

# Multiple repositories
repos fix backend-service frontend-app --ticket MAINT-1234
```

### Context-Aware Usage

If you are already in a `repos` context (e.g., using tag filters), you can omit the repository name:

```bash
# Fix a ticket in all repos matching 'production' tag
repos fix -t production --ticket MAINT-1234

# Auto-select if only one repo is in the current context
repos fix --ticket MAINT-1234
```

### Full JIRA URL

You can provide the full URL instead of just the ID:

```bash
repos fix mobile-app --ticket https://company.atlassian.net/browse/MAINT-1234
```

### Analysis Mode (Ask Mode)

Analyze the issue and propose a solution **without making code changes**:

```bash
repos fix my-service --ticket MAINT-1234 --ask
```

This generates a `SOLUTION_SUMMARY.md` with the proposed plan.

### Advanced Options

- `--workspace <DIR>`: Specify a custom directory for generated artifacts (default: `workspace/fix/<TICKET_ID>`).
- `--prompt "..."`: Append extra instructions to the AI agent (e.g., "Use Java 17 features").
- `--knowledge-dir <DIR>`: Copy markdown knowledge base files into the workspace and inline selected content into prompts.

## Workflow

When you run `repos fix`, the following steps occur:

1. **Fetch Ticket**: Downloads JIRA ticket information, description, and attachments.
2. **Setup Workspace**: Creates a working directory at `workspace/fix/<TICKET_ID>/`.
3. **Analyze Project**: detailed inspection of platform, languages, frameworks, dependencies, and test setup.
4. **Generate Context**: Creates `mission-context.json` with all analysis data.
5. **Generate Prompts**: Creates `.cursorrules` and `cursor_prompt.md` tailored to the specific project.
6. **Include Knowledge Base (optional)**: Copies markdown docs into `workspace/fix/<TICKET_ID>/knowledge/` and inlines selected docs into the prompt.
7. **Run Cursor Agent**:
    - Executes `cursor-agent` with `--force` and `--print` flags.
    - **Auto-Retry**: If the agent fails (e.g., build fails, tests fail), it automatically retries up to **3 times**, feeding the error message back to the AI.
    - **Workflow Switch**: CVE/security tickets use a safe upgrade protocol (no vulnerability reproduction); bug fixes require a repro-first flow.
8. **Validate**: The agent validates the fix by running build and test commands detected during analysis.
9. **Report**: Generates `SOLUTION_SUMMARY.md` with implementation details.

## Output

After execution, check the `workspace/fix/<TICKET_ID>/` directory:

```text
workspace/fix/MAINT-1234/
├── .cursorrules           # AI behavior rules
├── mission-context.json   # Complete project analysis & ticket data
├── cursor_prompt.md       # The "rulebook" for Cursor
├── agent_prompt.md        # The specific mission prompt
├── ANALYSIS.md            # Required pre-change analysis (root cause & plan)
├── SOLUTION_SUMMARY.md    # Final report of the implemented solution
```

Note: `ANALYSIS.md` is expected to be filled in by the agent before any changes.

## Supported Platforms

The plugin automatically detects and supports:

- **iOS**: Xcode projects (`.xcodeproj`, `.xcworkspace`), Swift/Obj-C, CocoaPods, SPM.
- **Android**: Gradle projects, Kotlin/Java, Android Manifests.
- **Java Backend**: Maven (`pom.xml`) or Gradle (`build.gradle`), Spring Boot, JUnit/Mockito.
- **Angular**: `angular.json` or `package.json` with Angular dependencies, TypeScript.

## Troubleshooting

### `cursor-agent` not found

Install it via `curl https://cursor.com/install -fsS | bash`. Ensure it is in your `$PATH`.

### JIRA authentication failed

Check your environment variables:

```bash
echo $JIRA_URL
echo $JIRA_USERNAME
echo $JIRA_API_TOKEN
```

Ensure `JIRA_API_TOKEN` is a valid API token, not your password.

### Repository not found

If using context filtering (e.g., `-t tag`), ensure the repository actually matches the filter. You can list matches with `repos list -t tag`.

### Agent keeps failing

Check the console output for the error message returned by `cursor-agent`. If it fails 3 times, check the generated prompts in the workspace directory to see if the AI instructions need manual adjustment (using `--prompt`).
