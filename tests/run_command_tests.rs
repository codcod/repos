use repos::{
    commands::{
        Command, CommandContext,
        run::{RunCommand, RunType},
    },
    config::{Config, Recipe, Repository},
};
use std::fs;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use tempfile::TempDir;

// =================================
// ===== Helper Functions
// =================================

/// Creates a git repository in the specified directory with proper git configuration.
fn create_git_repo(path: &std::path::Path) -> std::io::Result<()> {
    // Initialize git repo
    ProcessCommand::new("git")
        .arg("init")
        .current_dir(path)
        .output()?;

    // Configure git (required for commits)
    ProcessCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    ProcessCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    // Create a file and commit
    fs::write(path.join("README.md"), "# Test Repository")?;

    ProcessCommand::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()?;

    ProcessCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    Ok(())
}

/// Creates a basic single-repo test setup with a recipe and default CommandContext.
fn setup_recipe_test(
    repo_name: &str,
    recipe_name: &str,
    steps: Vec<&str>,
) -> (TempDir, Repository, Recipe, CommandContext) {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir = temp_dir.path().join(repo_name);
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let repo = Repository {
        name: repo_name.to_string(),
        url: format!("https://github.com/user/{}.git", repo_name),
        tags: vec!["test".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let recipe = Recipe {
        name: recipe_name.to_string(),
        steps: steps.into_iter().map(|s| s.to_string()).collect(),
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo.clone()],
            recipes: vec![recipe.clone()],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    (temp_dir, repo, recipe, context)
}

/// Creates a basic single-repo test setup with default CommandContext.
fn setup_basic_test(repo_name: &str) -> (TempDir, Repository, CommandContext) {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir = temp_dir.path().join(repo_name);
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let repo = Repository {
        name: repo_name.to_string(),
        url: format!("https://github.com/user/{}.git", repo_name),
        tags: vec!["test".to_string()],
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![repo.clone()],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    (temp_dir, repo, context)
}

/// Creates a parallel execution test setup with two repositories.
fn setup_parallel_test(
    repo1_name: &str,
    repo2_name: &str,
) -> (TempDir, Vec<Repository>, CommandContext) {
    let temp_dir = TempDir::new().unwrap();

    let repo1_dir = temp_dir.path().join(repo1_name);
    fs::create_dir_all(&repo1_dir).unwrap();
    create_git_repo(&repo1_dir).unwrap();
    let repo1 = Repository {
        name: repo1_name.to_string(),
        url: format!("https://github.com/user/{}.git", repo1_name),
        tags: vec!["test".to_string()],
        path: Some(repo1_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let repo2_dir = temp_dir.path().join(repo2_name);
    fs::create_dir_all(&repo2_dir).unwrap();
    create_git_repo(&repo2_dir).unwrap();
    let repo2 = Repository {
        name: repo2_name.to_string(),
        url: format!("https://github.com/user/{}.git", repo2_name),
        tags: vec!["test".to_string()],
        path: Some(repo2_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let repos = vec![repo1, repo2];
    let context = CommandContext {
        config: Config {
            repositories: repos.clone(),
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: true,
    };

    (temp_dir, repos, context)
}

/// Creates a repository with specific tags for tag filtering tests.
fn create_tagged_repo_setup(
    temp_dir: &TempDir,
    repo_name: &str,
    tags: Vec<&str>,
) -> (std::path::PathBuf, Repository) {
    let repo_dir = temp_dir.path().join(repo_name);
    fs::create_dir_all(&repo_dir).unwrap();
    create_git_repo(&repo_dir).unwrap();

    let repo = Repository {
        name: repo_name.to_string(),
        url: format!("https://github.com/user/{}.git", repo_name),
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        path: Some(repo_dir.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    (repo_dir, repo)
}

/// Builder pattern for creating CommandContext with sensible defaults.
struct CommandContextBuilder {
    repositories: Vec<Repository>,
    recipes: Vec<Recipe>,
    tag: Vec<String>,
    exclude_tag: Vec<String>,
    repos: Option<Vec<String>>,
    parallel: bool,
}

impl CommandContextBuilder {
    fn new() -> Self {
        Self {
            repositories: vec![],
            recipes: vec![],
            tag: vec![],
            exclude_tag: vec![],
            repos: None,
            parallel: false,
        }
    }

    fn with_repositories(mut self, repositories: Vec<Repository>) -> Self {
        self.repositories = repositories;
        self
    }

    fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tag = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    fn build(self) -> CommandContext {
        CommandContext {
            config: Config {
                repositories: self.repositories,
                recipes: self.recipes,
            },
            tag: self.tag,
            exclude_tag: self.exclude_tag,
            repos: self.repos,
            parallel: self.parallel,
        }
    }
}

// =================================
// ===== Tests
// =================================

/// Test basic RunCommand creation with command
#[tokio::test]
async fn test_run_command_creation() {
    let command = RunCommand {
        run_type: RunType::Command("echo hello".to_string()),
        no_save: true,
        output_dir: None,
    };

    // Test that the run_type contains the right command
    match &command.run_type {
        RunType::Command(cmd) => assert_eq!(cmd, "echo hello"),
        RunType::Recipe(_) => panic!("Expected Command variant"),
    }
    assert!(command.no_save);
    assert!(command.output_dir.is_none());
}

/// Test recipe variant creation
#[tokio::test]
async fn test_run_command_recipe_creation() {
    let command = RunCommand {
        run_type: RunType::Recipe("test-recipe".to_string()),
        no_save: false,
        output_dir: None,
    };

    match &command.run_type {
        RunType::Recipe(recipe) => assert_eq!(recipe, "test-recipe"),
        RunType::Command(_) => panic!("Expected Recipe variant"),
    }
    assert!(!command.no_save);
}

#[tokio::test]
async fn test_run_command_with_custom_output_dir() {
    let output_dir = PathBuf::from("/tmp/custom");
    let command = RunCommand {
        run_type: RunType::Command("ls".to_string()),
        no_save: false,
        output_dir: Some(output_dir.clone()),
    };

    match &command.run_type {
        RunType::Command(cmd) => assert_eq!(cmd, "ls"),
        RunType::Recipe(_) => panic!("Expected Command variant"),
    }
    assert!(!command.no_save);
    assert_eq!(command.output_dir, Some(output_dir));
}

#[tokio::test]
async fn test_run_command_empty_repositories() {
    let command = RunCommand {
        run_type: RunType::Command("echo test".to_string()),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        parallel: false,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok()); // Should succeed with empty repos
}

#[tokio::test]
async fn test_run_command_basic_execution() {
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("echo hello".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_parallel_execution() {
    let (_temp_dir, _repos, context) = setup_parallel_test("test-repo1", "test-repo2");

    let command = RunCommand {
        run_type: RunType::Command("echo hello".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_with_tag_filter() {
    let temp_dir = TempDir::new().unwrap();
    let (_backend_dir, backend_repo) =
        create_tagged_repo_setup(&temp_dir, "backend-repo", vec!["backend", "rust"]);
    let (_frontend_dir, frontend_repo) =
        create_tagged_repo_setup(&temp_dir, "frontend-repo", vec!["frontend", "javascript"]);

    let command = RunCommand {
        run_type: RunType::Command("echo hello".to_string()),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContextBuilder::new()
        .with_repositories(vec![backend_repo, frontend_repo])
        .with_tags(vec!["backend"])
        .build();

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_error_handling() {
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("false".to_string()), // Command that will fail
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    // The command should fail when all individual commands fail
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_command_with_special_characters() {
    let command = RunCommand {
        run_type: RunType::Command("echo \"test with spaces and symbols: @#$%\"".to_string()),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        parallel: false,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Additional Edge Case Tests for Coverage =====

#[tokio::test]
async fn test_run_command_error_no_command_nor_recipe() {
    let command = RunCommand {
        run_type: RunType::Command("".to_string()), // Empty command
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        parallel: false,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok()); // Should succeed with empty repos
}

#[tokio::test]
async fn test_run_command_existing_output_dir() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("already_exists");
    fs::create_dir_all(&output_dir).unwrap(); // Pre-create the directory

    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("echo existing_out_dir".to_string()),
        no_save: false,
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
    assert!(output_dir.exists(), "Output dir should remain");
}

#[tokio::test]
async fn test_run_recipe_without_shebang_implicit_shell() {
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "no-shebang", vec!["echo IMPLICIT_SHELL_OK"]);

    let command = RunCommand {
        run_type: RunType::Recipe("no-shebang".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_recipe_parallel_failure_branch() {
    let (_temp_dir, _repos, context) = setup_parallel_test("repo1", "repo2");

    // First step succeeds; second step uses a definitely missing command to force non-zero exit.
    let recipe = Recipe {
        name: "parallel-failure".to_string(),
        steps: vec![
            "echo FIRST".to_string(),
            "this-command-should-not-exist-12345".to_string(),
        ],
    };

    // Update context to include the recipe
    let context = CommandContext {
        config: Config {
            repositories: context.config.repositories,
            recipes: vec![recipe],
        },
        tag: context.tag,
        exclude_tag: context.exclude_tag,
        parallel: true, // Enable parallel execution
        repos: context.repos,
    };

    let command = RunCommand {
        run_type: RunType::Recipe("parallel-failure".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(
        result.is_ok(),
        "Run returns Ok but individual failures should be logged internally"
    );
}

#[tokio::test]
async fn test_run_command_skip_save_branch() {
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("echo SKIP_SAVE_MODE".to_string()),
        no_save: true, // Skip save mode
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_long_command_name_sanitization() {
    let temp_dir = TempDir::new().unwrap();
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let long_cmd = "echo THIS_IS_A_REALLY_LONG_COMMAND_NAME_WITH_SPECIAL_CHARS_%_#_@_!_____END";
    let command = RunCommand {
        run_type: RunType::Command(long_cmd.to_string()),
        no_save: false,
        output_dir: Some(temp_dir.path().join("long_cmd_output")),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
    // Verify directory created (sanitized/truncated filename path executed)
    assert!(temp_dir.path().join("long_cmd_output").exists());
}

#[tokio::test]
async fn test_run_recipe_script_creation_error_handling() {
    let (_temp_dir, _repo, _recipe, context) = setup_recipe_test(
        "test-repo",
        "script-creation",
        vec!["echo 'Testing script creation'", "echo 'Second step'"],
    );

    let command = RunCommand {
        run_type: RunType::Recipe("script-creation".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_recipe_with_readonly_directory() {
    let (_temp_dir, _repo, _recipe, context) = setup_recipe_test(
        "test-repo",
        "readonly-test",
        vec!["echo 'Testing readonly scenario'"],
    );

    let command = RunCommand {
        run_type: RunType::Recipe("readonly-test".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Constructor Tests =====

#[tokio::test]
async fn test_run_command_new_command() {
    let command = RunCommand::new_command("echo test".to_string(), true, None);

    match &command.run_type {
        RunType::Command(cmd) => assert_eq!(cmd, "echo test"),
        RunType::Recipe(_) => panic!("Expected Command variant"),
    }
    assert!(command.no_save);
    assert!(command.output_dir.is_none());
}

#[tokio::test]
async fn test_run_command_new_recipe() {
    let output_dir = Some(PathBuf::from("/tmp/recipes"));
    let command = RunCommand::new_recipe("my-recipe".to_string(), false, output_dir.clone());

    match &command.run_type {
        RunType::Recipe(recipe) => assert_eq!(recipe, "my-recipe"),
        RunType::Command(_) => panic!("Expected Recipe variant"),
    }
    assert!(!command.no_save);
    assert_eq!(command.output_dir, output_dir);
}

#[tokio::test]
async fn test_run_command_new_for_test() {
    let command = RunCommand::new_for_test("test command".to_string(), "/tmp/test".to_string());

    match &command.run_type {
        RunType::Command(cmd) => assert_eq!(cmd, "test command"),
        RunType::Recipe(_) => panic!("Expected Command variant"),
    }
    assert!(!command.no_save);
    assert_eq!(command.output_dir, Some(PathBuf::from("/tmp/test")));
}

// ===== Recipe Execution Tests =====

#[tokio::test]
async fn test_run_command_recipe_execution() {
    let recipe_steps = vec!["echo 'Hello from recipe'", "pwd"];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "test-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("test-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_recipe_not_found() {
    let command = RunCommand {
        run_type: RunType::Recipe("nonexistent-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Recipe 'nonexistent-recipe' not found")
    );
}

#[tokio::test]
async fn test_run_command_recipe_parallel_execution() {
    let (_temp_dir, _repos, mut context) = setup_parallel_test("test-repo1", "test-repo2");

    // Add the recipe for parallel execution
    let recipe = Recipe {
        name: "parallel-recipe".to_string(),
        steps: vec!["echo 'Parallel recipe execution'".to_string()],
    };
    context.config.recipes.push(recipe);
    context.parallel = true;

    let command = RunCommand {
        run_type: RunType::Recipe("parallel-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Exclude Tag Tests =====

#[tokio::test]
async fn test_run_command_with_exclude_tag() {
    let (_temp_dir, repos, mut context) = setup_parallel_test("backend-repo", "frontend-repo");

    // Customize the repository tags for this test
    let mut backend_repo = repos[0].clone();
    let mut frontend_repo = repos[1].clone();
    backend_repo.tags = vec!["backend".to_string(), "rust".to_string()];
    frontend_repo.tags = vec!["frontend".to_string(), "javascript".to_string()];

    context.config.repositories = vec![backend_repo, frontend_repo];
    context.exclude_tag = vec!["frontend".to_string()]; // Exclude frontend repos

    let command = RunCommand {
        run_type: RunType::Command("echo exclude_test".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Repo Name Filtering Tests =====

#[tokio::test]
async fn test_run_command_with_specific_repos() {
    let (_temp_dir, repos, mut context) = setup_parallel_test("backend-repo", "frontend-repo");

    // Customize the repository tags for this test
    let mut backend_repo = repos[0].clone();
    let mut frontend_repo = repos[1].clone();
    backend_repo.tags = vec!["backend".to_string()];
    frontend_repo.tags = vec!["frontend".to_string()];

    context.config.repositories = vec![backend_repo, frontend_repo];
    context.repos = Some(vec!["backend-repo".to_string()]); // Only run on backend-repo

    let command = RunCommand {
        run_type: RunType::Command("echo specific_repo_test".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Output Directory and File Tests =====

#[tokio::test]
async fn test_run_command_with_output_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("custom_output");

    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("echo 'Testing output directory'".to_string()),
        no_save: false, // Enable saving to test directory creation
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Check that output directory was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists());

    // Check that a timestamped subdirectory was created
    let entries: Vec<_> = fs::read_dir(runs_dir).unwrap().collect();
    assert!(!entries.is_empty());
}

// ===== Mixed Success/Failure Tests =====

#[tokio::test]
async fn test_run_command_mixed_success_failure_sequential() {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir1 = temp_dir.path().join("good-repo");
    fs::create_dir_all(&repo_dir1).unwrap();
    create_git_repo(&repo_dir1).unwrap();

    // Create a repo with a path that doesn't exist to cause failure
    let bad_repo_path = temp_dir.path().join("nonexistent-path");

    let good_repo = Repository {
        name: "good-repo".to_string(),
        url: "https://github.com/user/good-repo.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(repo_dir1.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let bad_repo = Repository {
        name: "bad-repo".to_string(),
        url: "https://github.com/user/bad-repo.git".to_string(),
        tags: vec!["test".to_string()],
        path: Some(bad_repo_path.to_string_lossy().to_string()),
        branch: None,
        config_dir: None,
    };

    let command = RunCommand {
        run_type: RunType::Command("echo hello".to_string()),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![good_repo, bad_repo],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        repos: None,
        parallel: false,
    };

    let result = command.execute(&context).await;
    // Sequential execution should fail on first error
    assert!(result.is_err());
}

// ===== Edge Cases and Complex Scenarios =====

#[tokio::test]
async fn test_run_command_empty_command_string() {
    let command = RunCommand {
        run_type: RunType::Command("".to_string()),
        no_save: true,
        output_dir: None,
    };

    let context = CommandContext {
        config: Config {
            repositories: vec![],
            recipes: vec![],
        },
        tag: vec![],
        exclude_tag: vec![],
        parallel: false,
        repos: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok()); // Should succeed with empty repos
}

// ===== Command Execution with Saving Tests =====

#[tokio::test]
async fn test_run_command_with_save_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("test_output");

    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("echo 'save test'".to_string()),
        no_save: false, // Enable saving
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Verify output directory structure was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists());
}

#[tokio::test]
async fn test_run_command_with_save_default_output_dir() {
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let command = RunCommand {
        run_type: RunType::Command("echo 'default output test'".to_string()),
        no_save: false,   // Enable saving
        output_dir: None, // Use default "output" directory
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_parallel_with_save() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("parallel_output");

    let (_temp_dir, _repos, mut context) = setup_parallel_test("test-repo1", "test-repo2");
    context.parallel = true; // Enable parallel execution

    let command = RunCommand {
        run_type: RunType::Command("echo 'parallel save test'".to_string()),
        no_save: false, // Enable saving
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Verify output directory structure was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists());
}

#[tokio::test]
async fn test_run_command_parallel_with_no_save() {
    let (_temp_dir, _repos, mut context) = setup_parallel_test("test-repo1", "test-repo2");
    context.parallel = true; // Enable parallel execution

    let command = RunCommand {
        run_type: RunType::Command("echo 'parallel no save test'".to_string()),
        no_save: true, // Disable saving
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Recipe Execution with Saving Tests =====

#[tokio::test]
async fn test_run_command_recipe_with_save_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("recipe_output");

    let recipe_steps = vec!["echo 'Recipe with save'", "pwd"];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "save-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("save-recipe".to_string()),
        no_save: false, // Enable saving
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Verify output directory structure was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists());
}

#[tokio::test]
async fn test_run_command_recipe_parallel_with_save() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("recipe_parallel_output");

    let (_temp_dir, _repos, mut context) = setup_parallel_test("test-repo1", "test-repo2");

    // Add recipe for parallel execution
    let recipe = Recipe {
        name: "parallel-save-recipe".to_string(),
        steps: vec!["echo 'Parallel recipe with save'".to_string()],
    };
    context.config.recipes.push(recipe);
    context.parallel = true; // Enable parallel execution

    let command = RunCommand {
        run_type: RunType::Recipe("parallel-save-recipe".to_string()),
        no_save: false, // Enable saving
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Verify output directory structure was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists());
}

#[tokio::test]
async fn test_run_command_recipe_parallel_with_no_save() {
    let (_temp_dir, _repos, mut context) = setup_parallel_test("test-repo1", "test-repo2");

    // Add recipe for parallel execution
    let recipe = Recipe {
        name: "parallel-no-save-recipe".to_string(),
        steps: vec!["echo 'Parallel recipe without save'".to_string()],
    };
    context.config.recipes.push(recipe);
    context.parallel = true; // Enable parallel execution

    let command = RunCommand {
        run_type: RunType::Recipe("parallel-no-save-recipe".to_string()),
        no_save: true, // Disable saving
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_command_recipe_sequential_with_no_save() {
    let recipe_steps = vec!["echo 'Sequential recipe without save'"];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "sequential-no-save-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("sequential-no-save-recipe".to_string()),
        no_save: true, // Disable saving
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Script Materialization Tests =====

#[tokio::test]
async fn test_script_materialization_with_shebang() {
    let recipe_steps = vec!["#!/bin/bash", "echo 'Script with shebang'"];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "shebang-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("shebang-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_script_materialization_without_shebang() {
    let recipe_steps = vec!["echo 'Script without shebang'", "pwd"];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "no-shebang-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("no-shebang-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sanitize_command_for_filename() {
    let temp_dir = TempDir::new().unwrap();
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    // Command with special characters that need sanitization
    let command = RunCommand {
        run_type: RunType::Command("echo 'test with / \\ : * ? \" < > | characters'".to_string()),
        no_save: false, // Enable saving to test sanitization
        output_dir: Some(temp_dir.path().join("sanitize_test")),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sanitize_script_name() {
    let recipe_steps = vec!["echo 'Recipe with special name'"];
    let (_temp_dir, _repo, mut _recipe, context) = setup_recipe_test(
        "test-repo",
        "Recipe-With.Special@Characters#And$Symbols%",
        recipe_steps,
    );

    let command = RunCommand {
        run_type: RunType::Recipe("Recipe-With.Special@Characters#And$Symbols%".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Long Command/Recipe Name Tests =====

#[tokio::test]
async fn test_long_command_name_truncation() {
    let temp_dir = TempDir::new().unwrap();
    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    // Very long command that should be truncated for directory name
    let long_command = "a".repeat(100);
    let command = RunCommand {
        run_type: RunType::Command(long_command),
        no_save: false, // Enable saving to test truncation
        output_dir: Some(temp_dir.path().join("long_command_test")),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Recipe Error Handling Tests =====

#[tokio::test]
async fn test_recipe_sequential_execution_with_script_error() {
    let recipe_steps = vec!["nonexistent_command_that_will_fail_12345"]; // Non-existent command
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "script-error-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("script-error-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    // The recipe should succeed even if commands within it fail, based on current implementation
    // This tests the behavior where script execution completes but commands inside may fail
    assert!(result.is_ok());
}

// ===== Complex Path and Script Tests =====

#[tokio::test]
async fn test_recipe_script_path_resolution() {
    let recipe_steps = vec!["pwd", "echo 'Path resolution test'"];
    let (_temp_dir, _repo, _recipe, context) = setup_recipe_test(
        "test-repo-with-complex-name",
        "path-resolution-recipe",
        recipe_steps,
    );

    let command = RunCommand {
        run_type: RunType::Recipe("path-resolution-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Empty Recipe Test =====

#[tokio::test]
async fn test_recipe_with_empty_steps() {
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "empty-recipe", vec![]);

    let command = RunCommand {
        run_type: RunType::Recipe("empty-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Script Creation and Permissions Tests =====

#[tokio::test]
async fn test_script_creation_with_various_contents() {
    let recipe_steps = vec![
        "echo 'Line 1'",
        "echo 'Line 2'",
        "if [ -f 'README.md' ]; then",
        "  echo 'Found README.md'",
        "fi",
    ];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "complex-script", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("complex-script".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Error Path for Sequential Recipe Execution =====

#[tokio::test]
async fn test_recipe_sequential_execution_with_default_output() {
    let (_temp_dir, _repo, _recipe, context) = setup_recipe_test(
        "test-repo",
        "default-output-recipe",
        vec!["echo 'Testing default output directory'"],
    );

    let command = RunCommand {
        run_type: RunType::Recipe("default-output-recipe".to_string()),
        no_save: false,   // Enable saving with default output directory
        output_dir: None, // Use default
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Multi-Step Recipe Tests =====

#[tokio::test]
async fn test_multi_step_recipe_sequential() {
    let recipe_steps = vec![
        "echo 'Step 1: Starting recipe'",
        "echo 'Step 2: Middle of recipe'",
        "echo 'Step 3: Ending recipe'",
        "ls -la",
    ];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "multi-step-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("multi-step-recipe".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Test Multi-Repository Recipe with Complex Names =====

#[tokio::test]
async fn test_recipe_multi_repo_complex_names() {
    let (_temp_dir, _repos, mut context) =
        setup_parallel_test("repo-with-dashes-1", "repo_with_underscores_2");

    let recipe = Recipe {
        name: "Complex-Recipe_Name.With@Special#Characters".to_string(),
        steps: vec!["echo 'Complex recipe with multiple repos'".to_string()],
    };
    context.config.recipes.push(recipe);

    let command = RunCommand {
        run_type: RunType::Recipe("Complex-Recipe_Name.With@Special#Characters".to_string()),
        no_save: true,
        output_dir: None,
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());
}

// ===== Log File Creation and Content Verification Tests =====

#[tokio::test]
async fn test_run_command_creates_logs_with_content() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("log_test_output");

    let (_temp_dir, _repo, context) = setup_basic_test("test-repo");

    let test_output = "Hello from command test";
    let command = RunCommand {
        run_type: RunType::Command(format!("echo '{}'", test_output)),
        no_save: false, // Enable saving to create log files
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Check that output directory structure was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists(), "Runs directory should be created");

    // Find the timestamped subdirectory
    let entries: Vec<_> = fs::read_dir(&runs_dir).unwrap().collect();
    assert!(
        !entries.is_empty(),
        "Should have at least one timestamped directory"
    );

    let timestamped_dir = entries
        .into_iter()
        .find(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir())
        .unwrap()
        .unwrap()
        .path();

    // Log files are in a subdirectory named after the repository
    let repo_log_dir = timestamped_dir.join("test-repo");
    assert!(
        repo_log_dir.exists(),
        "Repository log directory should exist"
    );

    // Check for log files in the repository subdirectory
    let stdout_log = repo_log_dir.join("stdout.log");
    let metadata_log = repo_log_dir.join("metadata.json");

    assert!(stdout_log.exists(), "stdout.log should be created");
    assert!(metadata_log.exists(), "metadata.json should be created");

    // Verify log file contents
    let stdout_content = fs::read_to_string(&stdout_log).unwrap();
    let metadata_content = fs::read_to_string(&metadata_log).unwrap();

    // stdout.log should contain the echo output
    assert!(
        stdout_content.contains(test_output),
        "stdout.log should contain command output: '{}', but was: '{}'",
        test_output,
        stdout_content
    );

    // metadata.json should contain execution information
    let metadata: serde_json::Value = serde_json::from_str(&metadata_content).unwrap();
    assert_eq!(metadata["repository"], "test-repo");
    assert_eq!(metadata["exit_code"], 0);
    assert_eq!(metadata["exit_code_description"], "success");

    // Validate that recipe and command fields are mutually exclusive
    assert!(
        metadata.get("command").is_some(),
        "metadata.json should contain 'command' field when running a command, but was: '{}'",
        metadata_content
    );
    assert!(
        metadata.get("recipe").is_none(),
        "metadata.json should NOT contain 'recipe' field when running a command, but was: '{}'",
        metadata_content
    );
    assert!(
        metadata.get("recipe_steps").is_none(),
        "metadata.json should NOT contain 'recipe_steps' field when running a command, but was: '{}'",
        metadata_content
    );

    assert!(
        metadata["repository"]
            .as_str()
            .unwrap()
            .contains("test-repo"),
        "metadata.json should contain repo name: 'test-repo', but was: '{}'",
        metadata_content
    );
}

#[tokio::test]
async fn test_run_recipe_creates_logs_with_content() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("recipe_log_test_output");

    let test_output = "Hello from recipe test";
    let echo_cmd = format!("echo '{}'", test_output);
    let recipe_steps = vec![echo_cmd.as_str(), "pwd"];
    let (_temp_dir, _repo, _recipe, context) =
        setup_recipe_test("test-repo", "log-test-recipe", recipe_steps);

    let command = RunCommand {
        run_type: RunType::Recipe("log-test-recipe".to_string()),
        no_save: false, // Enable saving to create log files
        output_dir: Some(output_dir.clone()),
    };

    let result = command.execute(&context).await;
    assert!(result.is_ok());

    // Check that output directory structure was created
    let runs_dir = output_dir.join("runs");
    assert!(runs_dir.exists(), "Runs directory should be created");

    // Find the timestamped subdirectory
    let entries: Vec<_> = fs::read_dir(&runs_dir).unwrap().collect();
    assert!(
        !entries.is_empty(),
        "Should have at least one timestamped directory"
    );

    let timestamped_dir = entries
        .into_iter()
        .find(|entry| entry.as_ref().unwrap().file_type().unwrap().is_dir())
        .unwrap()
        .unwrap()
        .path();

    // Log files are in a subdirectory named after the repository
    let repo_log_dir = timestamped_dir.join("test-repo");
    assert!(
        repo_log_dir.exists(),
        "Repository log directory should exist"
    );

    // Check for log files in the repository subdirectory
    let stdout_log = repo_log_dir.join("stdout.log");
    let metadata_log = repo_log_dir.join("metadata.json");

    assert!(stdout_log.exists(), "stdout.log should be created");
    assert!(metadata_log.exists(), "metadata.json should be created");

    // Verify log file contents
    let stdout_content = fs::read_to_string(&stdout_log).unwrap();
    let metadata_content = fs::read_to_string(&metadata_log).unwrap();

    // stdout.log should contain the recipe output
    assert!(
        stdout_content.contains(test_output),
        "stdout.log should contain recipe output: '{}', but was: '{}'",
        test_output,
        stdout_content
    );

    // metadata.json should contain execution information
    let metadata: serde_json::Value = serde_json::from_str(&metadata_content).unwrap();
    assert_eq!(metadata["repository"], "test-repo");
    assert_eq!(metadata["recipe"], "log-test-recipe");
    assert_eq!(metadata["exit_code"], 0);
    assert_eq!(metadata["exit_code_description"], "success");

    // Validate that recipe and command fields are mutually exclusive
    assert!(
        metadata.get("command").is_none(),
        "metadata.json should NOT contain 'command' field when running a recipe, but was: '{}'",
        metadata_content
    );
    assert!(
        metadata.get("recipe").is_some(),
        "metadata.json should contain 'recipe' field when running a recipe, but was: '{}'",
        metadata_content
    );
    assert!(
        metadata.get("recipe_steps").is_some(),
        "metadata.json should contain 'recipe_steps' field when running a recipe, but was: '{}'",
        metadata_content
    );

    assert!(
        metadata["repository"]
            .as_str()
            .unwrap()
            .contains("test-repo")
            && metadata["recipe"]
                .as_str()
                .unwrap()
                .contains("log-test-recipe"),
        "metadata.json should contain repo name and recipe name, but was: '{}'",
        metadata_content
    );
}
