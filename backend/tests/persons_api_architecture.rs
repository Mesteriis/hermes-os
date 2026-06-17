use std::fs;
use std::path::{Path, PathBuf};

const MAX_TEST_FILE_LINES: usize = 700;

#[test]
fn persons_api_tests_stay_below_architecture_line_limit() {
    let tests_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut violations = Vec::new();
    collect_persons_api_test_violations(&tests_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "persons API test files exceed {MAX_TEST_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_persons_api_test_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read test directory {root:?}: {error}"));

    for entry in entries {
        let entry = entry.expect("failed to read test directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_persons_api_test_violations(&path, violations);
            continue;
        }
        if !is_persons_api_test_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_TEST_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_persons_api_test_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or_default();
    if file_name == "persons_api.rs" || file_name == "persons_api_architecture.rs" {
        return true;
    }

    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value == "persons_api")
    }) && path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}
