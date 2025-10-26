//! Exit code utilities and mappings

/// Get a human-readable description for an exit code
pub fn get_exit_code_description(exit_code: i32) -> &'static str {
    match exit_code {
        0 => "success",
        1 => "general error",
        2 => "shell builtin misuse",
        126 => "command invoked cannot execute",
        127 => "command not found",
        128 => "invalid argument to exit",
        130 => "script terminated by Control-C",
        131..=255 => "terminated by signal",
        _ => "error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_exit_code_description() {
        assert_eq!(get_exit_code_description(0), "success");
        assert_eq!(get_exit_code_description(1), "general error");
        assert_eq!(get_exit_code_description(2), "shell builtin misuse");
        assert_eq!(
            get_exit_code_description(126),
            "command invoked cannot execute"
        );
        assert_eq!(get_exit_code_description(127), "command not found");
        assert_eq!(get_exit_code_description(128), "invalid argument to exit");
        assert_eq!(
            get_exit_code_description(130),
            "script terminated by Control-C"
        );
        assert_eq!(get_exit_code_description(131), "terminated by signal");
        assert_eq!(get_exit_code_description(255), "terminated by signal");
        // Test edge cases
        assert_eq!(get_exit_code_description(42), "error");
        assert_eq!(get_exit_code_description(-1), "error");
    }
}
