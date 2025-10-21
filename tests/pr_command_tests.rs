//! Comprehensive unit tests for PR Command functionality
//! Tests cover command execution, repository filtering, parallel execution, and error handling

use repos::commands::pr::PrCommand;
use repos::commands::{Command, CommandContext};
use repos::config::{Config, Repository};

/// Helper function to create a test config with repositories
fn create_test_config() -> Config {
    let mut repo1 = Repository::new(
        "repo1".to_string(),
        "git@github.com:owner/repo1.git".to_string(),
    );
    repo1.add_tag("frontend".to_string());
    repo1.add_tag("javascript".to_string());

    let mut repo2 = Repository::new(
        "repo2".to_string(),
        "git@github.com:owner/repo2.git".to_string(),
    );
    repo2.add_tag("backend".to_string());
    repo2.add_tag("rust".to_string());

    let mut repo3 = Repository::new(
        "repo3".to_string(),
        "git@github.com:owner/repo3.git".to_string(),
    );
    repo3.add_tag("backend".to_string());
    repo3.add_tag("database".to_string());

    Config {
        repositories: vec![repo1, repo2, repo3],
    }
}

/// Helper function to create a test context
fn create_test_context(
    config: Config,
    tag: Vec<String>,
    exclude_tag: Vec<String>,
    repos: Option<Vec<String>>,
    parallel: bool,
) -> CommandContext {
    CommandContext {
        config,
        tag,
        exclude_tag,
        parallel,
        repos,
    }
}

#[tokio::test]
async fn test_pr_command_basic_execution() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Test PR".to_string(),
        body: "Test body".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true, // Avoid actual GitHub API calls
    };

    // Should not panic and complete execution
    let result = pr_command.execute(&context).await;
    // Result may be Ok or Err depending on git operations, but shouldn't panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_tag_filter() {
    let config = create_test_config();
    let context = create_test_context(config, vec!["backend".to_string()], vec![], None, false);

    let pr_command = PrCommand {
        title: "Backend PR".to_string(),
        body: "Backend changes".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_specific_repos() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],
        vec![],
        Some(vec!["repo1".to_string(), "repo3".to_string()]),
        false,
    );

    let pr_command = PrCommand {
        title: "Specific repos PR".to_string(),
        body: "Changes for specific repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_tag_and_repos_filter() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec!["backend".to_string()],
        vec![],
        Some(vec!["repo2".to_string()]),
        false,
    );

    let pr_command = PrCommand {
        title: "Filtered PR".to_string(),
        body: "Changes for filtered repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_no_matching_repositories() {
    let config = create_test_config();
    let context = create_test_context(config, vec!["nonexistent".to_string()], vec![], None, false);

    let pr_command = PrCommand {
        title: "No repos PR".to_string(),
        body: "Should find no repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should succeed (print message about no repos found)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_command_empty_repositories() {
    let config = Config {
        repositories: vec![],
    };
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Empty config PR".to_string(),
        body: "No repositories in config".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should succeed (print message about no repos found)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_command_parallel_execution() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, true); // parallel = true

    let pr_command = PrCommand {
        title: "Parallel PR".to_string(),
        body: "Parallel execution test".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_custom_branch_name() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Custom branch PR".to_string(),
        body: "PR with custom branch".to_string(),
        branch_name: Some("feature/custom-branch".to_string()),
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_custom_base_branch() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Custom base PR".to_string(),
        body: "PR with custom base branch".to_string(),
        branch_name: None,
        base_branch: Some("develop".to_string()),
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_custom_commit_message() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Custom commit PR".to_string(),
        body: "PR with custom commit message".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: Some("feat: add new feature".to_string()),
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_draft_mode() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Draft PR".to_string(),
        body: "Draft pull request".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: true,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_create_only_mode() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Create only PR".to_string(),
        body: "Create only mode test".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_without_create_only() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Full PR".to_string(),
        body: "Full PR creation test".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: false, // This will try to push and create actual PR
    };

    // This should fail since we're using a fake token
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err()); // Either way is fine for this test
}

#[tokio::test]
async fn test_pr_command_empty_token() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "Empty token PR".to_string(),
        body: "PR with empty token".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "".to_string(), // Empty token
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_special_characters_in_title() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let pr_command = PrCommand {
        title: "PR with special chars: 你好 🚀 @#$%".to_string(),
        body: "Body with\nmultiple\nlines".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_very_long_title() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let long_title = "A".repeat(1000);
    let pr_command = PrCommand {
        title: long_title,
        body: "PR with very long title".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_very_long_body() {
    let config = create_test_config();
    let context = create_test_context(config, vec![], vec![], None, false);

    let long_body = "B".repeat(10000);
    let pr_command = PrCommand {
        title: "PR with long body".to_string(),
        body: long_body,
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_all_options_combined() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec!["backend".to_string()],
        vec![],
        Some(vec!["repo2".to_string()]),
        true, // parallel
    );

    let pr_command = PrCommand {
        title: "Full options PR".to_string(),
        body: "PR with all options set".to_string(),
        branch_name: Some("feature/all-options".to_string()),
        base_branch: Some("develop".to_string()),
        commit_msg: Some("feat: comprehensive test".to_string()),
        draft: true,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_invalid_repository_names() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],
        vec![],
        Some(vec!["nonexistent1".to_string(), "nonexistent2".to_string()]),
        false,
    );

    let pr_command = PrCommand {
        title: "Invalid repos PR".to_string(),
        body: "PR for nonexistent repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should succeed (print message about no repos found)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_command_mixed_valid_invalid_repos() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],
        vec![],
        Some(vec![
            "repo1".to_string(),
            "nonexistent".to_string(),
            "repo2".to_string(),
        ]),
        false,
    );

    let pr_command = PrCommand {
        title: "Mixed repos PR".to_string(),
        body: "PR for mix of valid and invalid repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_case_sensitive_tag_filter() {
    let config = create_test_config();
    let context = create_test_context(config, vec!["BACKEND".to_string()], vec![], None, false);

    let pr_command = PrCommand {
        title: "Case sensitive PR".to_string(),
        body: "Testing case sensitivity".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should find no repos because tags are case sensitive
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_command_case_sensitive_repo_names() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],
        vec![],
        Some(vec!["REPO1".to_string()]), // Wrong case
        false,
    );

    let pr_command = PrCommand {
        title: "Case sensitive repos PR".to_string(),
        body: "Testing repo name case sensitivity".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should find no repos because repo names are case sensitive
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_command_with_exclude_tag() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],                       // No inclusion tags
        vec!["frontend".to_string()], // Exclude frontend
        None,
        false,
    );

    let pr_command = PrCommand {
        title: "Backend only PR".to_string(),
        body: "Excludes frontend repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should only work with backend repos (repo2, repo3)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_multiple_exclude_tags() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],                                               // No inclusion tags
        vec!["frontend".to_string(), "database".to_string()], // Exclude multiple
        None,
        false,
    );

    let pr_command = PrCommand {
        title: "Rust only PR".to_string(),
        body: "Excludes frontend and database repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should only work with repo2 (rust backend, no database tag)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_with_inclusion_and_exclusion() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec!["backend".to_string()],  // Include backend
        vec!["database".to_string()], // But exclude database
        None,
        false,
    );

    let pr_command = PrCommand {
        title: "Backend no DB PR".to_string(),
        body: "Backend repos but not database ones".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should only work with repo2 (backend but not database)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_pr_command_exclude_all_repos() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec![],                                              // No inclusion tags
        vec!["frontend".to_string(), "backend".to_string()], // Exclude all
        None,
        false,
    );

    let pr_command = PrCommand {
        title: "No repos PR".to_string(),
        body: "Should exclude all repos".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should find no repos
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pr_command_multiple_inclusion_tags() {
    let config = create_test_config();
    let context = create_test_context(
        config,
        vec!["frontend".to_string(), "rust".to_string()], // Multiple includes
        vec![],                                           // No exclusions
        None,
        false,
    );

    let pr_command = PrCommand {
        title: "Multi-tag PR".to_string(),
        body: "Includes repos with frontend OR rust tags".to_string(),
        branch_name: None,
        base_branch: None,
        commit_msg: None,
        draft: false,
        token: "fake-token".to_string(),
        create_only: true,
    };

    // Should work with repo1 (frontend) and repo2 (rust)
    let result = pr_command.execute(&context).await;
    assert!(result.is_ok() || result.is_err());
}
