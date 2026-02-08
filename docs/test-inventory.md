# Test Case Inventory

Comprehensive test case catalog for the `repos` CLI. Each item lists purpose,
coverage dimensions (happy / negative / edge), and expected outcomes. This
inventory complements existing automated tests and surfaces gaps.

---

## 1. Configuration

### 1.1 Load valid config file with repositories and recipes

- Description: Verify a well-formed YAML with repositories + recipes parses into internal model.
- Preconditions: `repos.yaml` exists, contains at least one repository and one recipe.
- Steps: Run any command requiring config load (e.g. `repos clone --config repos.yaml`).
- Expected:
  - Happy: Config loads; repositories & recipes accessible; no warnings.
  - Negative: N/A (covered in malformed / missing cases below).
  - Edge: Minimal config (single repo, single recipe) still loads; extra unknown keys ignored gracefully.

### 1.2 Fail on missing config file

- Description: Tool should error if specified config path does not exist.
- Steps: `repos clone --config missing.yaml`.
- Expected:
  - Happy: Error message references missing file; non-zero exit.
  - Negative: Using directory path instead of file also errors clearly.
  - Edge: Path with special characters -> error still readable; no panic.

### 1.3 Fail on malformed YAML

- Description: Syntax errors in YAML produce clear failure.
- Steps: Provide broken indentation or invalid YAML tokens.
- Expected:
  - Happy: Parse error surfaced with line/column if available.
  - Negative: Empty file -> error indicating missing root key.
  - Edge: File with BOM, trailing spaces still parsed (if syntactically valid).

### 1.4 Load repositories with different path forms (absolute, relative)

- Description: Resolve repo target directories correctly.
- Expected:
  - Happy: Relative paths resolved against config directory; absolute paths untouched.
  - Negative: Invalid path (pointing to file, not dir) flagged when used.
  - Edge: Symlinks followed; tilde expansion unsupported (documented) or handled if implemented.

### 1.5 Handle empty repositories list

- Description: Zero repositories should lead to graceful no-op for repo-based commands.
- Expected:
  - Happy: Command exits success with message like "No repositories".
  - Negative: Attempt actions requiring repos (run/clone) yields no panic.
  - Edge: Config with only `recipes:` still valid.

### 1.6 Handle empty recipes list

- Description: Absence of recipes does not break command mode.
- Expected:
  - Happy: `run` with direct command works; recipe invocation fails with clear not-found.
  - Negative: `--recipe` specified returns error strictly about missing recipe.
  - Edge: Empty `recipes: []` instead of omission still OK.

### 1.7 Resolve recipe names uniquely

- Description: Name lookup is case-sensitive (based on existing tests) and returns correct recipe.
- Expected:
  - Happy: Exact match succeeds.
  - Negative: Wrong case fails with not found.
  - Edge: Names with spaces / dashes sanitized only for script file, not lookup.

---

## 2. Repository Management

### 2.1 Clone single repository

- Expected: Creates target directory; initializes git remote; success exit.

### 2.2 Clone multiple repositories sequentially

- Expected: Each repo processed in order; failures reported individually; process continues.

### 2.3 Clone multiple repositories in parallel

- Expected: All clone jobs spawned; race-safe logging; partial failures do not abort others.

### 2.4 Skip cloning if directory exists

- Expected: Existing directory detected; skip message emitted; status success overall.

### 2.5 Handle invalid repo URL

- Expected: Git clone returns non-zero; error captured and surfaced without panic.

### 2.6 Respect branch override when provided

- Expected: After clone, HEAD matches specified branch; missing branch triggers error (negative).

### 2.7 Tag filtering include-only

- Expected: Only repos containing tag(s) selected; empty result leads to graceful no-op.

### 2.8 Tag exclusion logic

- Expected: Repos with excluded tags removed from candidate set.

### 2.9 Explicit repos selection overrides tag filters

- Expected: Provided names used verbatim even if tags would exclude them.

### 2.10 Mixed tags with include and exclude

- Expected: Inclusion performed first, then exclusion prunes; documented precedence.

Edge Cases (Repos): Duplicate repo names prevented at config load; names with special characters still log cleanly; parallel cloning handles network errors.

---

## 3. Run Command (Command Mode)

### 3.1 Run simple echo command across one repo

- Expected: Command executes; metadata produced (save mode); exit_code 0.

### 3.2 Run command across multiple repos (sequential)

- Expected: Ordered execution; each metadata.json written.

### 3.3 Run command across multiple repos (parallel)

- Expected: All metadata files present; no interleaved stdout in individual files.

### 3.4 Run long command name (sanitization for metadata directory)

- Expected: Output directory suffix truncated to <=50 chars; remains unique/logical.

### 3.5 Handle command containing special characters and spaces

- Expected: Proper quoting; command stored verbatim in metadata.json.

### 3.6 Fail on empty command string

- Expected: Immediate validation error (currently runner treats empty as no-op success; test should assert defined expectation—potential gap).

### 3.7 No-save mode skips metadata/stdout file creation

- Expected: No run directory created; overall success exit.

### 3.8 Save mode creates `output/runs/<timestamp>_<sanitized>` structure

- Expected: Directory exists; per-repo subdirectories created.

### 3.9 Existing output directory reuse

- Expected: If base exists, new timestamped run directory created; no collision.

### 3.10 Proper exit code recording (0,1,2,126,127,130,>128)

- Expected: `exit_code` matches process status; description aligns mapping.

### 3.11 Exit code description mapping correctness

- Expected: Each known code string correct; unknown >128 -> "terminated by signal".

### 3.12 Metadata.json structure for command (no recipe fields)

- Expected: Contains: command, exit_code, exit_code_description, repository, timestamp ONLY.

Edge Cases (Command Mode): Large stdout handled; mixed stdout/stderr captured; failing command still writes logs; nonexistent directory errors early.

---

## 4. Run Command (Recipe Mode)

### 4.1 Run single-step recipe

- Expected: Script created in repo root; executes; removed; exit_code 0.

### 4.2 Run multi-step recipe sequential

- Expected: All steps run in order; combined output captured.

### 4.3 Run recipe across multiple repositories parallel

- Expected: Each repo gets its own script; isolated execution; cleanup.

### 4.4 Recipe script created in repo root, then removed

- Expected: After run, `*.script` absent; no `.repos/recipes` directory created.

### 4.5 Metadata.json contains recipe, recipe_steps, exit info (no command field)

- Expected: Fields: recipe, recipe_steps, exit_code, exit_code_description, repository, timestamp.

### 4.6 Fail when recipe name not found

- Expected: Error "Recipe 'name' not found"; no script created.

### 4.7 Fail when recipe has zero steps

- Expected: Early error or empty script logic defined (if allowed) — inventory marks gap if not enforced.

### 4.8 Shebang-less recipe executes under default shell

- Expected: Auto prepend `#!/bin/sh`; success.

### 4.9 Script materialization permissions (executable)

- Expected: Mode 750 (unix); execution works; negative: permission change failure -> error.

### 4.10 Cleanup always occurs even on failure

- Expected: Failing step causes non-zero exit; script file still deleted.

### 4.11 Exit codes propagate from failing step

- Expected: Metadata exit_code == failing command code.

### 4.12 Mixed success/failure steps (failure halts remaining steps)

- Expected: Steps after failing one NOT executed; output stops there.

Edge Cases (Recipe): Steps with environment variables preserved; multi-line heredoc processed; scripts with Unicode names sanitized.

---

## 5. Logging & Output

### 5.1 metadata.json created per repo in run (save mode)

- Expected: Exists with correct schema.

### 5.2 stdout.log contains command or aggregated recipe output

- Expected: Content matches actual stdout lines; order preserved.

### 5.3 metadata.json absent in no-save mode

- Expected: Directory not present; test asserts absence.

### 5.4 Proper timestamp format in metadata.json

- Expected: `YYYY-MM-DD HH:MM:SS` local time.

### 5.5 Directory naming pattern

- Expected: Matches `output/runs/<YYYYMMDD-HHMMSS>_<sanitized>`.

### 5.6 Sanitization truncation for extremely long names

- Expected: Suffix length <=50; no broken UTF-8.

Edge Cases: Simultaneous runs produce distinct timestamps; invalid characters replaced by `_`.

---

## 6. Parallel vs Sequential Behavior

### 6.1 Parallel execution does not interleave stdout logs improperly

- Expected: Each repo's stdout confined to its own file.

### 6.2 Sequential preserves order of execution

- Expected: Time ordering in logs roughly matches repo iteration order.

### 6.3 Parallel mode still generates per-repo metadata.json

- Expected: Count of metadata files == repo count.

Edge: Large number of repos (stress) still stable; resource exhaustion handled gracefully (potential future test).

---

## 7. Tag & Repo Selection

### 7.1 Include tag selects only matching repos

- Expected: Only repos with tag appear in run/clone output.

### 7.2 Exclude tag removes only excluded

- Expected: Repos with exclude tags omitted.

### 7.3 Combine include and exclude correctly

- Expected: Intersection minus excludes.

### 7.4 Explicit repos overrides tag filtering entirely

- Expected: Provided names used even if exclude tags present.

### 7.5 No overlap results in zero execution (graceful)

- Expected: Success exit + message; no errors.

Edge: Multiple include tags requiring all vs any (verify implemented semantics).

---

## 8. Error Handling

### 8.1 Missing command and recipe (CLI validation)

- Expected: Error message; non-zero exit; no run directory.

### 8.2 Nonexistent binary invocation in plugin tests

- Expected: Failure logged; test asserts graceful handling.

### 8.3 Missing recipe metadata absence of command field enforced

- Expected: For recipe run: "command" key not present; schema mutual exclusivity.

### 8.4 Command not found exit code (127) recorded with description

- Expected: exit_code 127; description "command not found".

### 8.5 Script cannot execute (126) recorded

- Expected: exit_code 126; description "command invoked cannot execute".

### 8.6 Interrupted execution (130) recorded

- Expected: Simulated Ctrl-C surfaces description "script terminated by Control-C".

Edge: >128 signals map to "terminated by signal".

---

## 9. Plugins

### 9.1 Built-in commands still work when plugins enabled

- Expected: Core behaviors unaffected.

### 9.2 Plugin discovery ignores invalid plugin paths

- Expected: Invalid entries skipped silently or with warning.

### 9.3 Plugin environment isolation (PATH override)

- Expected: External plugin execution does not mutate global state.

### 9.4 Fallback when no plugins present

- Expected: Listing or invoking plugin features yields empty set, no error.

### 9.5 Help text still accessible with plugins

- Expected: `--help` output unchanged.

### 9.6 Plugin does not interfere with core logging

- Expected: Standard metadata + log lines unaffected.

Edge: Multiple plugins simultaneously (future test).

---

## 10. Pull Requests

### 10.1 Create PR for single repo

- Expected: Branch creation + PR initiation (mock) success.

### 10.2 Create PR for multiple repos

- Expected: Each repo processed; failures isolated.

### 10.3 Fail on missing remote

- Expected: Error explaining remote not configured.

### 10.4 Handle authentication failure (mock/skip)

- Expected: Clear auth error; does not panic.

### 10.5 Title and body formatting correctness

- Expected: Special characters preserved; long body accepted.

Edge: Extremely long title truncated or handled (as implemented).

---

## 11. Init Command

### 11.1 Creates initial config file if absent

- Expected: File created with sample content; success exit.

### 11.2 Does not overwrite existing config

- Expected: Existing file retained; no destructive write.

### 11.3 Generates sample repositories section

- Expected: Minimal scaffold present.

### 11.4 Generates sample recipes section

- Expected: At least one example recipe included.

Edge: Run in non-writable directory returns error.

---

## 12. Git Operations

### 12.1 Fetch updates without modifying working tree

- Expected: Current branch unchanged; remote data updated.

### 12.2 Handle detached head properly

- Expected: Operations that require branch fail gracefully or adapt.

### 12.3 Branch checkout when branch provided

- Expected: HEAD matches requested branch.

### 12.4 Error on invalid branch

- Expected: Clear git error surfaced.

### 12.5 Local changes do not block read-only operations

- Expected: Status queries succeed with uncommitted changes.

Edge: Commit with empty message prevented.

---

## 13. Script / Filename Handling

### 13.1 Sanitization removes unsafe filesystem chars

- Expected: All problematic chars replaced by `_`.

### 13.2 Truncation preserves suffix uniqueness

- Expected: Long names shortened deterministically; collisions avoidable.

### 13.3 Collision avoidance for same command different repos

- Expected: Per-run root unique by timestamp; repo subdirs separate.

Edge: Unicode characters converted safely (non-panic).

---

## 14. Cleanup & Ephemeral Artifacts

### 14.1 Recipe script removed after success

- Expected: No lingering `*.script` files.

### 14.2 Recipe script removed after failure

- Expected: Same cleanup guarantee.

### 14.3 Temporary directories not left behind

- Expected: No stray temp paths after runs.

### 14.4 No residual `.repos` directory creation after change

- Expected: Directory absent unless purposefully added in future.

---

## 15. Metadata.json Integrity

### 15.1 Command run schema

- Expected: Keys exactly: command, exit_code, exit_code_description, repository, timestamp.

### 15.2 Recipe run schema

- Expected: Keys exactly: recipe, recipe_steps, exit_code, exit_code_description, repository, timestamp.

### 15.3 Mutual exclusivity

- Expected: Never both command and recipe present.

### 15.4 JSON valid (parsable)

- Expected: Deserialize succeeds under serde.

### 15.5 Exit code description matches numeric value

- Expected: Mapping correct per defined lookup.

Edge: Unknown negative code -> fallback description "error".

---

## 16. Additional Edge Cases

### 16.1 Empty tags list behaves like no filter

- Expected: All repositories included.

### 16.2 Large number of repositories performance (smoke)

- Expected: Operation completes within acceptable time; no timeouts.

### 16.3 Parallel execution does not exceed system limits

- Expected: No thread / FD exhaustion; graceful degradation if limits.

### 16.4 Unicode in repository names handled

- Expected: Logging intact; filesystem safe.

### 16.5 Unicode in recipe steps handled

- Expected: Output preserved; no encoding errors.

---

## 17. Suggested Missing Tests / Gaps

### 17.1 Interrupted (Ctrl-C simulation) propagation

- Expected: Child process terminates; exit_code 130 captured; cleanup occurs.

### 17.2 Signal >128 exit description mapping

- Expected: Forced signal termination mapped to "terminated by signal".

### 17.3 Concurrent runs creating distinct output directories

- Expected: Parallel invocations produce separate timestamp directories (different seconds or fallback randomization).

### 17.4 Invalid permission on repo directory (read-only) handling

- Expected: Script creation fails with permission error; graceful message.

### 17.5 metadata.json absent in failure before creation (early abort)

- Expected: If repo directory missing: no metadata file written; clear error.

---
*End of inventory.*

---

## 18. Test Classification (Unit vs Integration vs E2E)

Classification criteria:

- Unit: Pure logic or small function scope; no external processes, network, real git, or filesystem side-effects beyond trivial in-memory or temp file usage. Deterministic, fast (<50ms), isolated.
- Integration: Combines multiple subsystems (filesystem, git repos, command runner, process spawning). Uses real temp directories or invokes `git` / shell. Validates interactions and produced artifacts.
- E2E: Invokes the compiled CLI (`cargo run` / built binary) exercising full argument parsing, configuration loading, execution path, logging/metadata generation and cleanup across multiple repositories or plugins. Can be slower; closest to real user workflow.

### 18.1 Configuration

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|1.1 Load valid config| Integration | Parses real file from disk and produces model used by other commands| ✅ Automated |
|1.2 Missing config file| Integration | Depends on filesystem error handling| ✅ Automated |
|1.3 Malformed YAML| Unit | Focus on parse failure surfaced by loader logic| ❌ Gap |
|1.4 Path forms (abs/rel)| Unit | Pure path resolution logic; can be isolated| ✅ Automated |
|1.5 Empty repositories list| Unit | Behavior is early-return logic| ✅ Automated |
|1.6 Empty recipes list| Unit | Lookup logic and conditional absence handling| ✅ Automated |
|1.7 Resolve recipe names uniquely| Unit | Name lookup & matching only| ✅ Automated |
|Symlink repository path resolution| Integration | FS symlink target resolution & safety| ❌ Gap |

### 18.2 Repository Management

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|2.1 Clone single repository| Integration | Invokes `git clone` and filesystem| ⚠️ Partial |
|2.2 Clone multiple sequential| Integration | Iterative multi repo interaction| ❌ Gap |
|2.3 Clone multiple parallel| Integration | Concurrency + multiple git operations| ❌ Gap |
|2.4 Skip cloning if directory exists| Integration | Relies on FS presence checks| ✅ Automated |
|2.5 Invalid repo URL| Integration | Captures external command failure| ✅ Automated |
|2.6 Branch override| Integration | Uses git branch checkout| ❌ Gap |
|2.7 Tag filtering include-only| Unit | Pure filtering logic| ✅ Automated |
|2.8 Tag exclusion| Unit | Pure filtering logic| ✅ Automated |
|2.9 Explicit repos override| Unit | Selection precedence logic| ✅ Automated |
|2.10 Mixed include/exclude| Unit | Logical combination test| ✅ Automated |

### 18.3 Run Command (Command Mode)

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|3.1 Single echo| Integration | Executes shell in repo context| ✅ Automated |
|3.2 Multiple sequential| Integration | Iterative multi-repo execution| ✅ Automated |
|3.3 Multiple parallel| Integration | Concurrency execution path| ✅ Automated |
|3.4 Long command name sanitization| Unit | String transformation only| ✅ Automated |
|3.5 Special characters command| Integration | Actual shell invocation & metadata capture| ✅ Automated |
|3.6 Empty command string| Unit | Validation logic (candidate for stricter behavior) | ✅ Automated |
|3.7 No-save mode behavior| Integration | Affects artifact creation side-effects| ✅ Automated |
|3.8 Save mode directory creation| Integration | Filesystem structure| ✅ Automated |
|3.9 Existing output directory reuse| Integration | FS existence + new path logic| ✅ Automated |
|3.10 Exit code recording| Unit | Mapping + extraction (can isolate via fake status) | ✅ Automated |
|3.11 Exit code description mapping| Unit | Pure match function| ✅ Automated |
|3.12 Metadata.json structure (command)| Integration | Requires file writing & JSON content| ✅ Automated |

### 18.4 Run Command (Recipe Mode)

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|4.1 Single-step recipe| Integration | Script materialization + execution| ✅ Automated |
|4.2 Multi-step sequential| Integration | Aggregate execution order| ✅ Automated |
|4.3 Multi-repo parallel recipe| Integration | Concurrency + FS per repo| ✅ Automated |
|4.4 Script created & removed| Integration | FS artifact lifecycle| ✅ Automated |
|4.5 Metadata.json recipe fields| Integration | Generated file content| ✅ Automated |
|4.6 Recipe name not found| Unit | Lookup error path| ✅ Automated |
|4.7 Zero steps recipe| Unit | Validation / precondition logic| ✅ Automated |
|4.8 Implicit shebang| Unit | Script content transformation| ✅ Automated |
|4.9 Permissions set| Integration | Actual FS permissions required| ⚠️ Partial |
|4.10 Cleanup on failure| Integration | Execution + post-failure cleanup| ✅ Automated |
|4.11 Exit codes propagate| Integration | Real failing script status| ✅ Automated |
|4.12 Mixed success/failure halts| Integration | Execution control flow| ✅ Automated |
|Unicode script name sanitization| Unit | Ensures generated script filename handles Unicode safely| ❌ Gap |

### 18.5 Logging & Output

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|5.1 metadata.json per repo| Integration | File existence created by runner| ✅ Automated |
|5.2 stdout.log correctness| Integration | Captured process output| ✅ Automated |
|5.3 Absence in no-save mode| Integration | Side-effect suppression| ✅ Automated |
|5.4 Timestamp format| Unit | Formatting function| ⚠️ Partial |
|5.5 Directory naming pattern| Unit | String assembly + sanitization| ✅ Automated |
|5.6 Truncation behavior| Unit | String length logic| ✅ Automated |
|Simultaneous runs distinct timestamps| Integration | Parallel invocations produce non-colliding directories| ❌ Gap |

### 18.6 Parallel vs Sequential Behavior

All considered Integration (multi-repo orchestration). Stress/performance variants treated as E2E if full CLI invoked under load.

### 18.7 Tag & Repo Selection

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|7.1 Include tag match| Unit | Filtering logic| ✅ Automated |
|7.2 Exclude tag| Unit | Filtering logic| ✅ Automated |
|7.3 Combine include/exclude| Unit | Logical composition| ✅ Automated |
|7.4 Explicit repos override| Unit | Precedence resolution| ✅ Automated |
|7.5 No overlap graceful| Unit | Early-return logic| ✅ Automated |

### 18.8 Error Handling

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|8.1 Missing command & recipe| E2E | Full CLI argument validation path| ✅ Automated |
|8.2 Nonexistent binary (plugin)| Integration | External process failure under plugin harness| ✅ Automated |
|8.3 Mutual exclusivity metadata| Integration | Generated file schema| ✅ Automated |
|8.4 Command not found 127| Integration | Real process exit| ✅ Automated |
|8.5 Script cannot execute 126| Integration | Permission/exec failure| ❌ Gap |
|8.6 Interrupted 130| Integration | Signal handling from process| ❌ Gap |
|Signal >128 mapping (edge)| Unit | Mapping function correctness| ✅ Automated |

### 18.9 Plugins

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|9.1 Built-in commands still work when plugins enabled| Integration | Ensures plugin layer doesn't break core CLI | ✅ Automated |
|9.2 Plugin discovery ignores invalid plugin paths| Integration | Scans filesystem / executable bits | ✅ Automated |
|9.3 Plugin environment isolation (PATH override)| Integration | Requires process env isolation validation | ❌ Gap |
|9.4 Fallback when no plugins present| Integration | Graceful empty state | ✅ Automated |
|9.5 Help text still accessible with plugins| E2E | Full CLI parsing with dynamic plugin context | ❌ Gap |
|9.6 Plugin does not interfere with core logging| Integration | Compare logs with/without plugins | ⚠️ Partial |
|Multiple plugins simultaneously| Integration | Validates isolation & non-interference with more than one plugin | ❌ Gap |

### 18.10 Pull Requests

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|10.1 Create PR for single repo| Integration | Git ops + mock PR creation | ✅ Automated |
|10.2 Create PR for multiple repos| Integration | Iterates over multiple repos | ✅ Automated |
|10.3 Fail on missing remote| Integration | Git remote validation | ❌ Gap |
|10.4 Handle authentication failure (mock/skip)| Integration | Simulated auth failure path | ✅ Automated |
|10.5 Title and body formatting correctness| Integration | Content handling & escaping | ✅ Automated |
|Edge: Extremely long title truncated| Integration | Boundary handling | ✅ Automated |

### 18.11 Init Command

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|11.1 Initial file creation| Integration | Filesystem write | ✅ Automated |
|11.2 No overwrite existing| Integration | FS state check | ✅ Automated |
|11.3 Sample repositories scaffold| Unit | Content template generation | ✅ Automated |
|11.4 Sample recipes scaffold| Unit | Template generation | ✅ Automated |
|Edge non-writable dir| Integration | Permission failure on FS | ❌ Gap |

\n### 18.12 Git Operations
All Git operations Integration (depend on real git behavior). Detached head handling remains Integration. Pure parsing of branch names could be Unit if factored out (future refactor candidate).

### 18.13 Script / Filename Handling

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|13.1 Sanitization unsafe chars| Unit | String transformation | ✅ Automated |
|13.2 Truncation uniqueness| Unit | Deterministic length rule | ✅ Automated |
|13.3 Collision avoidance timestamp| Integration | Relies on FS and time source | ❌ Gap |
|Unicode safety| Unit | String handling | ❌ Gap |

\n### 18.14 Cleanup & Ephemeral Artifacts
All cleanup behaviors Integration (require actual run + artifact lifecycle). Missing .repos directory check Integration.

### 18.15 Metadata.json Integrity

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|15.1 Command run schema| Unit / Integration | Logic + persisted artifact | ✅ Automated |
|15.2 Recipe run schema| Unit / Integration | Logic + persisted artifact | ✅ Automated |
|15.3 Mutual exclusivity| Unit | Construction logic enforced | ✅ Automated |
|15.4 JSON valid parsable| Unit | Round-trip serde test | ✅ Automated |
|15.5 Exit code description mapping| Unit | Pure function mapping | ✅ Automated |
|Negative unknown code fallback| Unit | Edge mapping test | ✅ Automated |

### 18.16 Additional Edge Cases

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|16.1 Empty tags list behavior| Unit | Filtering logic default | ✅ Automated |
|16.2 Large number performance| E2E | Full-system scalability smoke | ❌ Gap |
|16.3 Parallel resource limits| E2E | System-level concurrency behavior | ❌ Gap |
|16.4 Unicode repo names| Integration | FS + logging interplay | ❌ Gap |
|16.5 Unicode recipe steps| Integration | Script materialization & output | ❌ Gap |

### 18.17 Suggested Missing Tests / Gaps

| Case | Type | Rationale | Status |
|------|------|-----------|---------|
|17.1 Ctrl-C propagation| Integration | Signal during process execution | ❌ Gap |
|17.2 Signal >128 mapping| Unit | Mapping function only | ✅ Automated |
|17.3 Concurrent runs distinct dirs| Integration | Timestamp & FS isolation | ❌ Gap |
|17.4 Read-only repo dir| Integration | Permission failure on write | ❌ Gap |
|17.5 Early abort no metadata| Integration | FS absence on precondition failure | ❌ Gap |

### 18.18 Summary Counts

Approximate classification (one test may have both Unit & Integration facets if split):

- Primarily Unit candidates: sanitization, filtering, mapping, schema exclusivity (~30%)
- Integration: git, command execution, logging, filesystem lifecycle (~55%)
- E2E: holistic CLI argument validation, performance/stress, plugin-enabled flows (~15%)

### 18.19 Recommendations

1. Refactor logic-heavy areas (exit code mapping, sanitization, filtering) into dedicated modules to keep unit tests fast and focused.
2. Separate mixed Unit/Integration tests (e.g., metadata schema) into two layers: construct JSON (unit) then file emission (integration).
3. Introduce tagging in test framework (feature: cargo nextest or custom) to selectively run Unit vs Integration vs E2E in CI stages.
4. Add E2E smoke suite executing representative scenarios (parallel recipe run, PR creation mock, plugin discovery) nightly.
5. Track execution time per test to detect regressions (baseline now while codebase stable).

---

*End of classification section.*
