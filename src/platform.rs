/// Cross-platform utilities for paths and classpaths
use std::path::PathBuf;

/// Get the platform-specific classpath separator
pub fn classpath_separator() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

/// Build a classpath string from multiple paths
pub fn build_classpath(paths: &[&str]) -> String {
    paths.join(classpath_separator())
}

/// Convert a Unix-style path pattern to platform-specific
/// e.g., "lib/*" becomes "lib\*" on Windows
#[allow(dead_code)]
pub fn platform_path(path: &str) -> String {
    if cfg!(windows) {
        path.replace('/', "\\")
    } else {
        path.to_string()
    }
}

/// Join paths with platform-specific separator
#[allow(dead_code)]
pub fn join_paths(base: &str, paths: &[&str]) -> String {
    let mut result = PathBuf::from(base);
    for path in paths {
        result.push(path);
    }
    result.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classpath_separator() {
        let sep = classpath_separator();
        if cfg!(windows) {
            assert_eq!(sep, ";");
        } else {
            assert_eq!(sep, ":");
        }
    }

    #[test]
    fn test_build_classpath() {
        let paths = vec!["bin", "lib/*"];
        let cp = build_classpath(&paths);

        if cfg!(windows) {
            assert_eq!(cp, "bin;lib/*");
        } else {
            assert_eq!(cp, "bin:lib/*");
        }
    }

    #[test]
    fn test_platform_path() {
        let path = "lib/test";
        let result = platform_path(path);

        if cfg!(windows) {
            assert_eq!(result, "lib\\test");
        } else {
            assert_eq!(result, "lib/test");
        }
    }
}
