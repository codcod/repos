//! String sanitization utilities for filenames and identifiers

/// Sanitize command string for use in directory names
///
/// Replaces filesystem-unsafe characters with underscores and limits length to 50 characters.
/// Preserves alphanumeric characters, hyphens, underscores, and dots.
pub fn sanitize_for_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            ' ' => '_',
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => c,
            _ => '_',
        })
        .collect::<String>()
        .chars()
        .take(50) // Limit length to avoid overly long directory names
        .collect()
}

/// Sanitize script name for use as executable filename
///
/// Converts to lowercase and replaces non-ASCII-alphanumeric characters
/// (except hyphens and underscores) with underscores.
pub fn sanitize_script_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_for_filename() {
        assert_eq!(sanitize_for_filename("echo hello"), "echo_hello");
        assert_eq!(sanitize_for_filename("ls -la"), "ls_-la");
        assert_eq!(sanitize_for_filename("cat file.txt"), "cat_file.txt");
        assert_eq!(
            sanitize_for_filename("cmd/with/slashes"),
            "cmd_with_slashes"
        );
        assert_eq!(sanitize_for_filename("cmd:with:colons"), "cmd_with_colons");
        assert_eq!(
            sanitize_for_filename("cmd?with?special*chars"),
            "cmd_with_special_chars"
        );

        // Test length limiting
        let long_command = "a".repeat(60);
        let sanitized = sanitize_for_filename(&long_command);
        assert_eq!(sanitized.len(), 50);
        assert_eq!(sanitized, "a".repeat(50));
    }

    #[test]
    fn test_sanitize_for_filename_edge_cases() {
        // Test empty string
        assert_eq!(sanitize_for_filename(""), "");

        // Test string with only special characters
        assert_eq!(sanitize_for_filename("!@#$%^&*()"), "__________");

        // Test string with mixed valid and invalid characters
        assert_eq!(
            sanitize_for_filename("test-123_abc.txt!@#"),
            "test-123_abc.txt___"
        );

        // Test string exactly at limit (50 chars)
        let exactly_fifty = "a".repeat(50);
        let sanitized = sanitize_for_filename(&exactly_fifty);
        assert_eq!(sanitized.len(), 50);
        assert_eq!(sanitized, exactly_fifty);

        // Test Unicode characters (alphanumeric Unicode chars are preserved)
        assert_eq!(sanitize_for_filename("café"), "café");
        assert_eq!(sanitize_for_filename("测试-test"), "测试-test");
    }

    #[test]
    fn test_sanitize_script_name() {
        assert_eq!(sanitize_script_name("TestScript"), "testscript");
        assert_eq!(sanitize_script_name("my-script"), "my-script");
        assert_eq!(sanitize_script_name("script_name"), "script_name");
        assert_eq!(
            sanitize_script_name("script@example.com"),
            "script_example_com"
        );
        assert_eq!(sanitize_script_name("UPPERCASE"), "uppercase");
        assert_eq!(
            sanitize_script_name("script with spaces"),
            "script_with_spaces"
        );
        assert_eq!(sanitize_script_name("123-script"), "123-script");
    }

    #[test]
    fn test_sanitize_script_name_edge_cases() {
        // Test empty string
        assert_eq!(sanitize_script_name(""), "");

        // Test string with only special characters
        assert_eq!(sanitize_script_name("!@#$%^&*()"), "__________");

        // Test string with numbers only
        assert_eq!(sanitize_script_name("12345"), "12345");

        // Test string with mixed case and special chars
        assert_eq!(
            sanitize_script_name("Test-Script_2023!"),
            "test-script_2023_"
        );

        // Test consecutive special characters
        assert_eq!(sanitize_script_name("test!!!script"), "test___script");

        // Test Unicode characters get converted (only ASCII alphanumeric preserved)
        assert_eq!(sanitize_script_name("café-script"), "caf_-script");
    }
}
