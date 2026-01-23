//! Lint result types.

use std::path::PathBuf;

use texide_plugin::Diagnostic;

/// Result of linting a single file.
#[derive(Debug)]
pub struct LintResult {
    /// Path to the linted file.
    pub path: PathBuf,

    /// Diagnostics found in the file.
    pub diagnostics: Vec<Diagnostic>,

    /// Whether the result was loaded from cache.
    pub from_cache: bool,
}

impl LintResult {
    /// Creates a new lint result.
    pub fn new(path: PathBuf, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            path,
            diagnostics,
            from_cache: false,
        }
    }

    /// Creates a cached lint result.
    pub fn cached(path: PathBuf, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            path,
            diagnostics,
            from_cache: true,
        }
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Returns the number of diagnostics.
    pub fn error_count(&self) -> usize {
        self.diagnostics.len()
    }
}

/// Summary of linting multiple files.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct LintSummary {
    /// Total files processed.
    pub files_checked: usize,

    /// Files loaded from cache.
    pub files_from_cache: usize,

    /// Total diagnostics found.
    pub total_diagnostics: usize,

    /// Files with errors.
    pub files_with_errors: usize,
}

impl LintSummary {
    /// Creates a summary from results.
    pub fn from_results(results: &[LintResult]) -> Self {
        let mut summary = Self::default();

        for result in results {
            summary.files_checked += 1;
            if result.from_cache {
                summary.files_from_cache += 1;
            }
            summary.total_diagnostics += result.diagnostics.len();
            if result.has_errors() {
                summary.files_with_errors += 1;
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texide_ast::Span;

    #[test]
    fn test_lint_result_new() {
        let result = LintResult::new(PathBuf::from("test.md"), vec![]);
        assert!(!result.has_errors());
        assert!(!result.from_cache);
    }

    #[test]
    fn test_lint_result_cached() {
        let result = LintResult::cached(PathBuf::from("test.md"), vec![]);
        assert!(result.from_cache);
    }

    #[test]
    fn test_lint_summary() {
        let results = vec![
            LintResult::new(PathBuf::from("a.md"), vec![]),
            LintResult::cached(
                PathBuf::from("b.md"),
                vec![Diagnostic::new("test", "msg", Span::new(0, 1))],
            ),
        ];

        let summary = LintSummary::from_results(&results);

        assert_eq!(summary.files_checked, 2);
        assert_eq!(summary.files_from_cache, 1);
        assert_eq!(summary.total_diagnostics, 1);
        assert_eq!(summary.files_with_errors, 1);
    }
}
