use std::fs;
use std::path::{Path, PathBuf};

const MAX_IMPLEMENTATION_FILE_LINES: usize = 700;

#[test]
fn telegram_implementation_files_stay_below_architecture_line_limit() {
    let backend_manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = backend_manifest_dir
        .parent()
        .expect("backend crate must live under repository root");
    let roots = [
        backend_manifest_dir.join("src/app/api_support"),
        backend_manifest_dir.join("src/integrations/telegram"),
        backend_manifest_dir.join("tests"),
        repo_root.join("frontend/src/integrations/telegram"),
    ];

    let mut violations = Vec::new();
    for root in roots {
        collect_line_limit_violations(&root, &mut violations);
    }

    assert!(
        violations.is_empty(),
        "Telegram source/test files exceed {MAX_IMPLEMENTATION_FILE_LINES} lines:\n{}",
        violations.join("\n")
    );
}

fn collect_line_limit_violations(root: &Path, violations: &mut Vec<String>) {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        panic!("failed to read Telegram implementation directory {root:?}: {error}")
    });

    for entry in entries {
        let entry = entry.expect("failed to read Telegram implementation directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_line_limit_violations(&path, violations);
            continue;
        }
        if !is_implementation_file(&path) {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
        let line_count = content.lines().count();
        if line_count > MAX_IMPLEMENTATION_FILE_LINES {
            violations.push(format!("{}: {line_count}", path.display()));
        }
    }
}

fn is_implementation_file(path: &Path) -> bool {
    let is_backend_test_file = path
        .components()
        .any(|component| component.as_os_str() == "tests");
    let is_telegram_test_support_file = path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value.starts_with("telegram"))
    });
    if path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .is_some_and(|file_name| file_name.starts_with("telegram") && file_name.ends_with(".rs"))
    {
        return true;
    }
    if is_backend_test_file {
        return is_telegram_test_support_file;
    }

    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("rs" | "ts" | "vue")
    )
}
