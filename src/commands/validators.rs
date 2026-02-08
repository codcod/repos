//! Command argument validation utilities
//!
//! This module provides centralized validation logic for command arguments
//! after clap parsing. It handles domain-specific validation rules that
//! go beyond basic argument parsing.

use anyhow::{Result, anyhow};

/// Validation errors for command arguments
#[derive(Debug, PartialEq)]
pub enum CommandValidationError {
    /// Mutually exclusive arguments were both provided
    MutualExclusivity { first: String, second: String },
    /// Required argument was not provided
    MissingRequired {
        argument: String,
        alternatives: Vec<String>,
    },
    /// Invalid argument value
    InvalidValue {
        argument: String,
        value: String,
        reason: String,
    },
    /// Empty collection when at least one item is required
    EmptyCollection { argument: String },
}

impl std::fmt::Display for CommandValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandValidationError::MutualExclusivity { first, second } => {
                write!(f, "Cannot specify both {} and {}", first, second)
            }
            CommandValidationError::MissingRequired {
                argument,
                alternatives,
            } => {
                if alternatives.is_empty() {
                    write!(f, "{} is required", argument)
                } else {
                    write!(
                        f,
                        "Either {} or {} must be provided",
                        alternatives.join(", "),
                        argument
                    )
                }
            }
            CommandValidationError::InvalidValue {
                argument,
                value,
                reason,
            } => {
                write!(f, "Invalid value '{}' for {}: {}", value, argument, reason)
            }
            CommandValidationError::EmptyCollection { argument } => {
                write!(f, "{} cannot be empty", argument)
            }
        }
    }
}

impl std::error::Error for CommandValidationError {}

/// Convert validation error to anyhow::Error
pub fn validation_error_to_anyhow(error: CommandValidationError) -> anyhow::Error {
    anyhow!(error.to_string())
}

/// Validate run command arguments
///
/// Ensures that exactly one of command or recipe is provided (mutual exclusivity)
pub fn validate_run_args(command: &Option<String>, recipe: &Option<String>) -> Result<()> {
    match (command.as_ref(), recipe.as_ref()) {
        (None, None) => Err(validation_error_to_anyhow(
            CommandValidationError::MissingRequired {
                argument: "a command".to_string(),
                alternatives: vec!["--recipe".to_string()],
            },
        )),
        (Some(_), Some(_)) => Err(validation_error_to_anyhow(
            CommandValidationError::MutualExclusivity {
                first: "command".to_string(),
                second: "--recipe".to_string(),
            },
        )),
        _ => Ok(()),
    }
}

/// Validate PR command arguments
///
/// Ensures that required GitHub authentication is available
pub fn validate_pr_args(token: &Option<String>) -> Result<()> {
    if token.is_none() && std::env::var("GITHUB_TOKEN").is_err() {
        return Err(validation_error_to_anyhow(
            CommandValidationError::MissingRequired {
                argument: "GitHub token".to_string(),
                alternatives: vec![
                    "--token".to_string(),
                    "GITHUB_TOKEN environment variable".to_string(),
                ],
            },
        ));
    }
    Ok(())
}

/// Validate tag arguments
///
/// Ensures tag filters are not empty when provided
pub fn validate_tag_filters(tags: &[String]) -> Result<()> {
    for tag in tags {
        if tag.trim().is_empty() {
            return Err(validation_error_to_anyhow(
                CommandValidationError::InvalidValue {
                    argument: "tag".to_string(),
                    value: tag.clone(),
                    reason: "tag cannot be empty or whitespace only".to_string(),
                },
            ));
        }
    }
    Ok(())
}

/// Validate repository names
///
/// Ensures repository names are not empty when provided
pub fn validate_repository_names(repos: &[String]) -> Result<()> {
    for repo in repos {
        if repo.trim().is_empty() {
            return Err(validation_error_to_anyhow(
                CommandValidationError::InvalidValue {
                    argument: "repository name".to_string(),
                    value: repo.clone(),
                    reason: "repository name cannot be empty or whitespace only".to_string(),
                },
            ));
        }
    }
    Ok(())
}

/// Validate output directory path
///
/// Ensures the output directory path is valid
pub fn validate_output_directory(output_dir: &Option<String>) -> Result<()> {
    if let Some(dir) = output_dir
        && dir.trim().is_empty()
    {
        return Err(validation_error_to_anyhow(
            CommandValidationError::InvalidValue {
                argument: "output-dir".to_string(),
                value: dir.clone(),
                reason: "output directory cannot be empty or whitespace only".to_string(),
            },
        ));
    }
    Ok(())
}

/// Validate branch name
///
/// Ensures branch names follow basic Git naming conventions
pub fn validate_branch_name(branch: &Option<String>) -> Result<()> {
    if let Some(name) = branch {
        if name.trim().is_empty() {
            return Err(validation_error_to_anyhow(
                CommandValidationError::InvalidValue {
                    argument: "branch".to_string(),
                    value: name.clone(),
                    reason: "branch name cannot be empty or whitespace only".to_string(),
                },
            ));
        }

        // Basic Git branch name validation
        if name.starts_with('-') || name.ends_with('.') || name.contains("..") {
            return Err(validation_error_to_anyhow(
                CommandValidationError::InvalidValue {
                    argument: "branch".to_string(),
                    value: name.clone(),
                    reason: "invalid Git branch name format".to_string(),
                },
            ));
        }
    }
    Ok(())
}

/// Validate commit message
///
/// Ensures commit messages are not empty when provided
pub fn validate_commit_message(message: &Option<String>) -> Result<()> {
    if let Some(msg) = message
        && msg.trim().is_empty()
    {
        return Err(validation_error_to_anyhow(
            CommandValidationError::InvalidValue {
                argument: "commit message".to_string(),
                value: msg.clone(),
                reason: "commit message cannot be empty or whitespace only".to_string(),
            },
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_validate_run_args_valid_command() {
        let command = Some("echo hello".to_string());
        let recipe = None;
        assert!(validate_run_args(&command, &recipe).is_ok());
    }

    #[test]
    fn test_validate_run_args_valid_recipe() {
        let command = None;
        let recipe = Some("test-recipe".to_string());
        assert!(validate_run_args(&command, &recipe).is_ok());
    }

    #[test]
    fn test_validate_run_args_mutual_exclusivity() {
        let command = Some("echo hello".to_string());
        let recipe = Some("test-recipe".to_string());
        let result = validate_run_args(&command, &recipe);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot specify both")
        );
    }

    #[test]
    fn test_validate_run_args_missing_required() {
        let command = None;
        let recipe = None;
        let result = validate_run_args(&command, &recipe);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be provided"));
    }

    #[test]
    fn test_validate_tag_filters_valid() {
        let tags = vec!["frontend".to_string(), "backend".to_string()];
        assert!(validate_tag_filters(&tags).is_ok());
    }

    #[test]
    fn test_validate_tag_filters_empty_tag() {
        let tags = vec!["frontend".to_string(), "".to_string()];
        let result = validate_tag_filters(&tags);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("tag cannot be empty")
        );
    }

    #[test]
    fn test_validate_tag_filters_whitespace_only() {
        let tags = vec!["frontend".to_string(), "   ".to_string()];
        let result = validate_tag_filters(&tags);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("tag cannot be empty")
        );
    }

    #[test]
    fn test_validate_repository_names_valid() {
        let repos = vec!["repo1".to_string(), "repo2".to_string()];
        assert!(validate_repository_names(&repos).is_ok());
    }

    #[test]
    fn test_validate_repository_names_empty() {
        let repos = vec!["repo1".to_string(), "".to_string()];
        let result = validate_repository_names(&repos);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("repository name cannot be empty")
        );
    }

    #[test]
    fn test_validate_output_directory_valid() {
        let output_dir = Some("./output".to_string());
        assert!(validate_output_directory(&output_dir).is_ok());
    }

    #[test]
    fn test_validate_output_directory_none() {
        let output_dir = None;
        assert!(validate_output_directory(&output_dir).is_ok());
    }

    #[test]
    fn test_validate_output_directory_empty() {
        let output_dir = Some("".to_string());
        let result = validate_output_directory(&output_dir);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("output directory cannot be empty")
        );
    }

    #[test]
    fn test_validate_branch_name_valid() {
        let branch = Some("feature/new-feature".to_string());
        assert!(validate_branch_name(&branch).is_ok());
    }

    #[test]
    fn test_validate_branch_name_none() {
        let branch = None;
        assert!(validate_branch_name(&branch).is_ok());
    }

    #[test]
    fn test_validate_branch_name_invalid_start() {
        let branch = Some("-invalid".to_string());
        let result = validate_branch_name(&branch);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid Git branch name")
        );
    }

    #[test]
    fn test_validate_branch_name_invalid_end() {
        let branch = Some("invalid.".to_string());
        let result = validate_branch_name(&branch);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid Git branch name")
        );
    }

    #[test]
    fn test_validate_branch_name_invalid_double_dot() {
        let branch = Some("feature..invalid".to_string());
        let result = validate_branch_name(&branch);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid Git branch name")
        );
    }

    #[test]
    fn test_validate_commit_message_valid() {
        let message = Some("Add new feature".to_string());
        assert!(validate_commit_message(&message).is_ok());
    }

    #[test]
    fn test_validate_commit_message_none() {
        let message = None;
        assert!(validate_commit_message(&message).is_ok());
    }

    #[test]
    fn test_validate_commit_message_empty() {
        let message = Some("".to_string());
        let result = validate_commit_message(&message);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("commit message cannot be empty")
        );
    }

    #[test]
    fn test_validate_pr_args_with_token() {
        let token = Some("github_token".to_string());
        assert!(validate_pr_args(&token).is_ok());
    }

    #[test]
    #[serial]
    fn test_validate_pr_args_with_env_var() {
        // Save original state
        let original_token = std::env::var("GITHUB_TOKEN").ok();

        // Set test environment variable
        unsafe {
            std::env::set_var("GITHUB_TOKEN", "test_token");
        }

        let token = None;
        let result = validate_pr_args(&token);

        // Restore original state
        unsafe {
            if let Some(token_value) = original_token {
                std::env::set_var("GITHUB_TOKEN", token_value);
            } else {
                std::env::remove_var("GITHUB_TOKEN");
            }
        }

        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_validate_pr_args_missing_token() {
        // Save the current environment variable state
        let original_token = std::env::var("GITHUB_TOKEN").ok();

        // Temporarily remove the environment variable
        unsafe {
            std::env::remove_var("GITHUB_TOKEN");
        }

        let token = None;
        let result = validate_pr_args(&token);

        // Restore the original environment variable if it existed
        if let Some(token_value) = original_token {
            unsafe {
                std::env::set_var("GITHUB_TOKEN", token_value);
            }
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("GitHub token"));
    }

    #[test]
    fn test_command_validation_error_display() {
        let error = CommandValidationError::MutualExclusivity {
            first: "command".to_string(),
            second: "--recipe".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Cannot specify both command and --recipe"
        );

        let error = CommandValidationError::MissingRequired {
            argument: "a command".to_string(),
            alternatives: vec!["--recipe".to_string()],
        };
        assert_eq!(
            error.to_string(),
            "Either --recipe or a command must be provided"
        );

        let error = CommandValidationError::InvalidValue {
            argument: "branch".to_string(),
            value: "-invalid".to_string(),
            reason: "invalid format".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Invalid value '-invalid' for branch: invalid format"
        );
    }
}
