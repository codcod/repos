//! Repository filtering utilities

use crate::config::Repository;

/// Filter repositories by specific names
pub fn filter_by_names(repositories: &[Repository], names: &[String]) -> Vec<Repository> {
    if names.is_empty() {
        return repositories.to_vec();
    }

    repositories
        .iter()
        .filter(|repo| names.contains(&repo.name))
        .cloned()
        .collect()
}

/// Filter repositories by tag (single tag)
pub fn filter_by_tag(repositories: &[Repository], tag: Option<&str>) -> Vec<Repository> {
    match tag {
        Some(tag) => repositories
            .iter()
            .filter(|repo| repo.has_tag(tag))
            .cloned()
            .collect(),
        None => repositories.to_vec(),
    }
}

/// Filter repositories by multiple tags (OR logic)
pub fn filter_by_any_tag(repositories: &[Repository], tags: &[String]) -> Vec<Repository> {
    if tags.is_empty() {
        return repositories.to_vec();
    }

    repositories
        .iter()
        .filter(|repo| repo.has_any_tag(tags))
        .cloned()
        .collect()
}

/// Filter repositories by multiple tags (AND logic)
pub fn filter_by_all_tags(repositories: &[Repository], tags: &[String]) -> Vec<Repository> {
    if tags.is_empty() {
        return repositories.to_vec();
    }

    repositories
        .iter()
        .filter(|repo| tags.iter().all(|tag| repo.has_tag(tag)))
        .cloned()
        .collect()
}

/// Filter repositories by context (combining tag inclusion, exclusion, and names filters)
pub fn filter_repositories(
    repositories: &[Repository],
    include_tags: &[String],
    exclude_tags: &[String],
    repo_names: Option<&[String]>,
) -> Vec<Repository> {
    let base_repos = if let Some(names) = repo_names {
        // If specific repos are specified, filter by names first
        filter_by_names(repositories, names)
    } else {
        // Otherwise start with all repositories
        repositories.to_vec()
    };

    // Apply both inclusion and exclusion filters in a single pass
    base_repos
        .into_iter()
        .filter(|repo| {
            // Check inclusion filter: if include_tags is empty, include all; otherwise check if repo has all included tags (AND logic)
            let included =
                include_tags.is_empty() || include_tags.iter().all(|tag| repo.has_tag(tag));

            // Check exclusion filter: if exclude_tags is empty, exclude none; otherwise check if repo has any excluded tag
            let excluded =
                !exclude_tags.is_empty() && exclude_tags.iter().any(|tag| repo.has_tag(tag));

            included && !excluded
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repositories() -> Vec<Repository> {
        let mut repo1 = Repository::new(
            "repo1".to_string(),
            "git@github.com:owner/repo1.git".to_string(),
        );
        repo1.add_tag("frontend".to_string());
        repo1.add_tag("web".to_string());

        let mut repo2 = Repository::new(
            "repo2".to_string(),
            "git@github.com:owner/repo2.git".to_string(),
        );
        repo2.add_tag("backend".to_string());
        repo2.add_tag("api".to_string());

        vec![repo1, repo2]
    }

    #[test]
    fn test_filter_by_tag() {
        let repos = create_test_repositories();

        let frontend_repos = filter_by_tag(&repos, Some("frontend"));
        assert_eq!(frontend_repos.len(), 1);
        assert_eq!(frontend_repos[0].name, "repo1");

        let all_repos = filter_by_tag(&repos, None);
        assert_eq!(all_repos.len(), 2);
    }

    #[test]
    fn test_filter_by_any_tag() {
        let repos = create_test_repositories();

        let web_repos = filter_by_any_tag(&repos, &["frontend".to_string(), "api".to_string()]);
        assert_eq!(web_repos.len(), 2); // Both repos match

        let no_match = filter_by_any_tag(&repos, &["mobile".to_string()]);
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_filter_by_names() {
        let repos = create_test_repositories();

        let specific_repos = filter_by_names(&repos, &["repo1".to_string()]);
        assert_eq!(specific_repos.len(), 1);
        assert_eq!(specific_repos[0].name, "repo1");

        let multiple_repos = filter_by_names(&repos, &["repo1".to_string(), "repo2".to_string()]);
        assert_eq!(multiple_repos.len(), 2);

        let no_match = filter_by_names(&repos, &["nonexistent".to_string()]);
        assert_eq!(no_match.len(), 0);

        let empty_filter = filter_by_names(&repos, &[]);
        assert_eq!(empty_filter.len(), 2); // Should return all repos
    }

    #[test]
    fn test_filter_repositories_combined() {
        let repos = create_test_repositories();

        // Test with both tag and repo names
        let filtered = filter_repositories(
            &repos,
            &["frontend".to_string()],
            &[],
            Some(&["repo1".to_string()]),
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with tag and repo names that don't match
        let filtered = filter_repositories(
            &repos,
            &["backend".to_string()],
            &[],
            Some(&["repo1".to_string()]),
        );
        assert_eq!(filtered.len(), 0); // repo1 doesn't have backend tag

        // Test with only repo names
        let filtered = filter_repositories(&repos, &[], &[], Some(&["repo1".to_string()]));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with only tag
        let filtered = filter_repositories(&repos, &["frontend".to_string()], &[], None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");

        // Test with neither (should return all)
        let filtered = filter_repositories(&repos, &[], &[], None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_repositories_exclude_tags() {
        let repos = create_test_repositories();

        // Test excluding tags
        let filtered = filter_repositories(
            &repos,
            &[],                       // no include filter
            &["frontend".to_string()], // exclude frontend
            None,
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo2"); // Only repo2 should remain

        // Test excluding all repos
        let filtered = filter_repositories(
            &repos,
            &[],
            &["frontend".to_string(), "backend".to_string()],
            None,
        );
        assert_eq!(filtered.len(), 0);

        // Test include and exclude together
        let filtered = filter_repositories(
            &repos,
            &["web".to_string(), "frontend".to_string()], // include web AND frontend (only repo1 has both)
            &["backend".to_string()],                     // but exclude backend
            None,
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1"); // repo1 has web AND frontend, not backend
    }

    #[test]
    fn test_filter_by_all_tags() {
        let repos = create_test_repositories();

        // Empty tag list should return all repositories
        let filtered = filter_by_all_tags(&repos, &[]);
        assert_eq!(filtered.len(), 2);

        // Tags that can't all exist on same repo should return empty
        let filtered = filter_by_all_tags(&repos, &["backend".to_string(), "frontend".to_string()]);
        assert_eq!(filtered.len(), 0);

        // Single tag that exists
        let filtered = filter_by_all_tags(&repos, &["frontend".to_string()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1");
    }

    #[test]
    fn test_filter_repositories_and_logic_with_multiple_tags() {
        let repos = create_test_repositories();

        // Multiple tags should use AND logic - all tags must be present
        let filtered = filter_repositories(
            &repos,
            &["frontend".to_string(), "web".to_string()], // both tags required
            &[],
            None,
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "repo1"); // Only repo1 has both tags

        // If one tag doesn't exist, no repos should match
        let filtered = filter_repositories(
            &repos,
            &["frontend".to_string(), "nonexistent".to_string()],
            &[],
            None,
        );
        assert_eq!(filtered.len(), 0);

        // Single nonexistent tag should return no repos
        let filtered = filter_repositories(&repos, &["nonexistent".to_string()], &[], None);
        assert_eq!(filtered.len(), 0);
    }
}
