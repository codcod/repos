// Comprehensive unit tests for GitHub authentication
// Tests cover token validation, header generation, and error scenarios

use repos::github::auth::GitHubAuth;

#[test]
fn test_github_auth_creation() {
    let token = "ghp_test_token_1234567890".to_string();
    let auth = GitHubAuth::new(token.clone());

    // Test that token is stored correctly
    assert_eq!(auth.token(), &token);
}

#[test]
fn test_github_auth_creation_with_empty_token() {
    let token = "".to_string();
    let auth = GitHubAuth::new(token.clone());

    // Should be able to create auth with empty token
    assert_eq!(auth.token(), "");
}

#[test]
fn test_github_auth_token_accessor() {
    let token = "ghp_another_test_token".to_string();
    let auth = GitHubAuth::new(token.clone());

    // Test token accessor returns correct reference
    assert_eq!(auth.token(), &token);
    assert_eq!(auth.token().len(), token.len());
}

#[test]
fn test_github_auth_get_auth_header() {
    let token = "ghp_test_token_1234567890".to_string();
    let auth = GitHubAuth::new(token.clone());

    let header = auth.get_auth_header();
    assert_eq!(header, format!("Bearer {}", token));
    assert!(header.starts_with("Bearer "));
    assert!(header.contains(&token));
}

#[test]
fn test_github_auth_get_auth_header_with_empty_token() {
    let auth = GitHubAuth::new("".to_string());

    let header = auth.get_auth_header();
    assert_eq!(header, "Bearer ");
    assert!(header.starts_with("Bearer "));
}

#[test]
fn test_github_auth_get_auth_header_with_special_characters() {
    let token = "ghp_token_with-special.chars_123".to_string();
    let auth = GitHubAuth::new(token.clone());

    let header = auth.get_auth_header();
    assert_eq!(header, format!("Bearer {}", token));
    assert!(header.contains("-special.chars_"));
}

#[test]
fn test_github_auth_validate_token_success() {
    let token = "ghp_valid_token_1234567890".to_string();
    let auth = GitHubAuth::new(token);

    let result = auth.validate_token();
    assert!(result.is_ok());
}

#[test]
fn test_github_auth_validate_token_empty_failure() {
    let auth = GitHubAuth::new("".to_string());

    let result = auth.validate_token();
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("GitHub token is required"));
}

#[test]
fn test_github_auth_validate_token_whitespace_only() {
    // Test token with only whitespace (should be considered valid by current logic)
    let auth = GitHubAuth::new("   ".to_string());

    let result = auth.validate_token();
    assert!(result.is_ok()); // Current implementation only checks for empty, not whitespace
}

#[test]
fn test_github_auth_validate_token_very_long_token() {
    // Test with a very long token to ensure no length restrictions
    let long_token = "ghp_".to_string() + &"a".repeat(1000);
    let auth = GitHubAuth::new(long_token);

    let result = auth.validate_token();
    assert!(result.is_ok());
}

#[test]
fn test_github_auth_token_immutability() {
    let original_token = "ghp_test_token".to_string();
    let auth = GitHubAuth::new(original_token.clone());

    // Test that token cannot be modified through reference
    let token_ref = auth.token();
    assert_eq!(token_ref, &original_token);

    // Verify token remains unchanged
    assert_eq!(auth.token(), &original_token);
}

#[test]
fn test_github_auth_multiple_header_calls() {
    let token = "ghp_consistent_token".to_string();
    let auth = GitHubAuth::new(token.clone());

    // Test that multiple calls to get_auth_header return the same result
    let header1 = auth.get_auth_header();
    let header2 = auth.get_auth_header();
    let header3 = auth.get_auth_header();

    assert_eq!(header1, header2);
    assert_eq!(header2, header3);
    assert_eq!(header1, format!("Bearer {}", token));
}

#[test]
fn test_github_auth_multiple_validate_calls() {
    let token = "ghp_valid_token".to_string();
    let auth = GitHubAuth::new(token);

    // Test that multiple validation calls are consistent
    let result1 = auth.validate_token();
    let result2 = auth.validate_token();
    let result3 = auth.validate_token();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[test]
fn test_github_auth_edge_case_single_character_token() {
    let auth = GitHubAuth::new("x".to_string());

    assert_eq!(auth.token(), "x");
    assert_eq!(auth.get_auth_header(), "Bearer x");
    assert!(auth.validate_token().is_ok());
}

#[test]
fn test_github_auth_realistic_github_token_format() {
    // Test with realistic GitHub token formats
    let personal_token = "ghp_1234567890abcdef1234567890abcdef12345678".to_string();
    let auth = GitHubAuth::new(personal_token.clone());

    assert_eq!(auth.token(), &personal_token);
    assert_eq!(auth.get_auth_header(), format!("Bearer {}", personal_token));
    assert!(auth.validate_token().is_ok());
}

#[test]
fn test_github_auth_app_token_format() {
    // Test with GitHub App token format
    let app_token = "ghs_1234567890abcdef1234567890abcdef12345678".to_string();
    let auth = GitHubAuth::new(app_token.clone());

    assert_eq!(auth.token(), &app_token);
    assert_eq!(auth.get_auth_header(), format!("Bearer {}", app_token));
    assert!(auth.validate_token().is_ok());
}

#[test]
fn test_github_auth_installation_token_format() {
    // Test with GitHub installation token format
    let installation_token = "ghu_1234567890abcdef1234567890abcdef12345678".to_string();
    let auth = GitHubAuth::new(installation_token.clone());

    assert_eq!(auth.token(), &installation_token);
    assert_eq!(
        auth.get_auth_header(),
        format!("Bearer {}", installation_token)
    );
    assert!(auth.validate_token().is_ok());
}
