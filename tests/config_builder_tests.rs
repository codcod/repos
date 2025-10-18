use repos::config::RepositoryBuilder;

#[test]
fn test_repository_builder_basic_creation() {
    let builder = RepositoryBuilder::new(
        "test-repo".to_string(),
        "https://github.com/user/test-repo.git".to_string(),
    );

    let repo = builder.build();

    assert_eq!(repo.name, "test-repo");
    assert_eq!(repo.url, "https://github.com/user/test-repo.git");
    assert!(repo.tags.is_empty());
    assert!(repo.path.is_none());
    assert!(repo.branch.is_none());
    assert!(repo.config_dir.is_none());
}

#[test]
fn test_repository_builder_with_tags() {
    let tags = vec!["backend".to_string(), "rust".to_string()];
    let repo = RepositoryBuilder::new(
        "backend-service".to_string(),
        "https://github.com/company/backend-service.git".to_string(),
    )
    .with_tags(tags.clone())
    .build();

    assert_eq!(repo.name, "backend-service");
    assert_eq!(repo.url, "https://github.com/company/backend-service.git");
    assert_eq!(repo.tags, tags);
    assert!(repo.path.is_none());
    assert!(repo.branch.is_none());
    assert!(repo.config_dir.is_none());
}

#[test]
fn test_repository_builder_with_path() {
    let repo = RepositoryBuilder::new(
        "local-repo".to_string(),
        "https://github.com/user/local-repo.git".to_string(),
    )
    .with_path("./local-path".to_string())
    .build();

    assert_eq!(repo.name, "local-repo");
    assert_eq!(repo.url, "https://github.com/user/local-repo.git");
    assert!(repo.tags.is_empty());
    assert_eq!(repo.path, Some("./local-path".to_string()));
    assert!(repo.branch.is_none());
    assert!(repo.config_dir.is_none());
}

#[test]
fn test_repository_builder_with_branch() {
    let repo = RepositoryBuilder::new(
        "feature-repo".to_string(),
        "https://github.com/user/feature-repo.git".to_string(),
    )
    .with_branch("feature-branch".to_string())
    .build();

    assert_eq!(repo.name, "feature-repo");
    assert_eq!(repo.url, "https://github.com/user/feature-repo.git");
    assert!(repo.tags.is_empty());
    assert!(repo.path.is_none());
    assert_eq!(repo.branch, Some("feature-branch".to_string()));
    assert!(repo.config_dir.is_none());
}

#[test]
fn test_repository_builder_with_all_options() {
    let tags = vec![
        "frontend".to_string(),
        "javascript".to_string(),
        "react".to_string(),
    ];
    let repo = RepositoryBuilder::new(
        "full-featured-repo".to_string(),
        "https://github.com/company/full-featured-repo.git".to_string(),
    )
    .with_tags(tags.clone())
    .with_path("./frontend/full-featured".to_string())
    .with_branch("develop".to_string())
    .build();

    assert_eq!(repo.name, "full-featured-repo");
    assert_eq!(
        repo.url,
        "https://github.com/company/full-featured-repo.git"
    );
    assert_eq!(repo.tags, tags);
    assert_eq!(repo.path, Some("./frontend/full-featured".to_string()));
    assert_eq!(repo.branch, Some("develop".to_string()));
    assert!(repo.config_dir.is_none());
}

#[test]
fn test_repository_builder_chaining_order() {
    // Test that builder methods can be called in different orders
    let repo1 = RepositoryBuilder::new(
        "order-test-1".to_string(),
        "https://github.com/user/order-test-1.git".to_string(),
    )
    .with_path("./path1".to_string())
    .with_tags(vec!["tag1".to_string()])
    .with_branch("branch1".to_string())
    .build();

    let repo2 = RepositoryBuilder::new(
        "order-test-2".to_string(),
        "https://github.com/user/order-test-2.git".to_string(),
    )
    .with_branch("branch2".to_string())
    .with_tags(vec!["tag2".to_string()])
    .with_path("./path2".to_string())
    .build();

    // Both should have the same structure regardless of call order
    assert_eq!(repo1.name, "order-test-1");
    assert_eq!(repo1.path, Some("./path1".to_string()));
    assert_eq!(repo1.tags, vec!["tag1".to_string()]);
    assert_eq!(repo1.branch, Some("branch1".to_string()));

    assert_eq!(repo2.name, "order-test-2");
    assert_eq!(repo2.path, Some("./path2".to_string()));
    assert_eq!(repo2.tags, vec!["tag2".to_string()]);
    assert_eq!(repo2.branch, Some("branch2".to_string()));
}

#[test]
fn test_repository_builder_empty_tags() {
    let repo = RepositoryBuilder::new(
        "empty-tags-repo".to_string(),
        "https://github.com/user/empty-tags-repo.git".to_string(),
    )
    .with_tags(vec![])
    .build();

    assert_eq!(repo.name, "empty-tags-repo");
    assert_eq!(repo.url, "https://github.com/user/empty-tags-repo.git");
    assert!(repo.tags.is_empty());
}

#[test]
fn test_repository_builder_multiple_tags() {
    let tags = vec![
        "backend".to_string(),
        "rust".to_string(),
        "microservice".to_string(),
        "api".to_string(),
        "production".to_string(),
    ];

    let repo = RepositoryBuilder::new(
        "multi-tag-repo".to_string(),
        "https://github.com/company/multi-tag-repo.git".to_string(),
    )
    .with_tags(tags.clone())
    .build();

    assert_eq!(repo.tags, tags);
    assert_eq!(repo.tags.len(), 5);
}

#[test]
fn test_repository_builder_special_characters() {
    let repo = RepositoryBuilder::new(
        "repo-with-special_chars.123".to_string(),
        "https://github.com/user/repo-with-special_chars.123.git".to_string(),
    )
    .with_path("./path/with spaces/and-dashes_underscores".to_string())
    .with_branch("feature/special-chars_branch".to_string())
    .with_tags(vec![
        "tag-with-dashes".to_string(),
        "tag_with_underscores".to_string(),
    ])
    .build();

    assert_eq!(repo.name, "repo-with-special_chars.123");
    assert_eq!(
        repo.url,
        "https://github.com/user/repo-with-special_chars.123.git"
    );
    assert_eq!(
        repo.path,
        Some("./path/with spaces/and-dashes_underscores".to_string())
    );
    assert_eq!(
        repo.branch,
        Some("feature/special-chars_branch".to_string())
    );
    assert_eq!(
        repo.tags,
        vec![
            "tag-with-dashes".to_string(),
            "tag_with_underscores".to_string()
        ]
    );
}

#[test]
fn test_repository_builder_unicode_characters() {
    let repo = RepositoryBuilder::new(
        "repo-with-émojis-🚀".to_string(),
        "https://github.com/user/repo-with-émojis-🚀.git".to_string(),
    )
    .with_path("./パス/中文/العربية".to_string())
    .with_branch("branch-with-émojis-🔥".to_string())
    .with_tags(vec!["tag-with-émojis-💻".to_string()])
    .build();

    assert_eq!(repo.name, "repo-with-émojis-🚀");
    assert_eq!(repo.url, "https://github.com/user/repo-with-émojis-🚀.git");
    assert_eq!(repo.path, Some("./パス/中文/العربية".to_string()));
    assert_eq!(repo.branch, Some("branch-with-émojis-🔥".to_string()));
    assert_eq!(repo.tags, vec!["tag-with-émojis-💻".to_string()]);
}

#[test]
fn test_repository_builder_very_long_strings() {
    let long_name = "a".repeat(1000);
    let long_url = format!("https://github.com/user/{}.git", "a".repeat(500));
    let long_path = format!("./{}", "b".repeat(500));
    let long_branch = "c".repeat(500);
    let long_tags = vec!["d".repeat(100), "e".repeat(200), "f".repeat(300)];

    let repo = RepositoryBuilder::new(long_name.clone(), long_url.clone())
        .with_path(long_path.clone())
        .with_branch(long_branch.clone())
        .with_tags(long_tags.clone())
        .build();

    assert_eq!(repo.name, long_name);
    assert_eq!(repo.url, long_url);
    assert_eq!(repo.path, Some(long_path));
    assert_eq!(repo.branch, Some(long_branch));
    assert_eq!(repo.tags, long_tags);
}

#[test]
fn test_repository_builder_overwrite_values() {
    // Test that calling setter methods multiple times overwrites previous values
    let repo = RepositoryBuilder::new(
        "overwrite-test".to_string(),
        "https://github.com/user/overwrite-test.git".to_string(),
    )
    .with_path("./first-path".to_string())
    .with_path("./second-path".to_string()) // Should overwrite first path
    .with_branch("first-branch".to_string())
    .with_branch("second-branch".to_string()) // Should overwrite first branch
    .with_tags(vec!["first-tag".to_string()])
    .with_tags(vec!["second-tag".to_string()]) // Should overwrite first tags
    .build();

    assert_eq!(repo.name, "overwrite-test");
    assert_eq!(repo.path, Some("./second-path".to_string()));
    assert_eq!(repo.branch, Some("second-branch".to_string()));
    assert_eq!(repo.tags, vec!["second-tag".to_string()]);
}

#[test]
fn test_repository_builder_method_return_types() {
    // Test that all builder methods return Self for chaining
    let builder = RepositoryBuilder::new(
        "chain-test".to_string(),
        "https://github.com/user/chain-test.git".to_string(),
    );

    // Each method should return a RepositoryBuilder that can be chained
    let _repo = builder
        .with_tags(vec!["test".to_string()])
        .with_path("./test".to_string())
        .with_branch("test".to_string())
        .build();

    // If this compiles, the chaining works correctly
    assert!(true);
}

#[test]
fn test_repository_builder_config_dir_always_none() {
    // The builder should always set config_dir to None
    let repo = RepositoryBuilder::new(
        "config-dir-test".to_string(),
        "https://github.com/user/config-dir-test.git".to_string(),
    )
    .with_tags(vec!["test".to_string()])
    .with_path("./test".to_string())
    .with_branch("test".to_string())
    .build();

    assert!(repo.config_dir.is_none());
}
