use std::env;
use std::path::{Path, PathBuf};

pub(super) fn tdjson_library_candidates(configured_path: Option<&Path>) -> Vec<PathBuf> {
    let current_exe_dir = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let current_dir = env::current_dir().ok();

    tdjson_library_candidates_with_context(
        configured_path,
        current_exe_dir.as_deref(),
        current_dir.as_deref(),
    )
}

pub(super) fn tdjson_library_candidates_with_context(
    configured_path: Option<&Path>,
    current_exe_dir: Option<&Path>,
    current_dir: Option<&Path>,
) -> Vec<PathBuf> {
    if let Some(path) = configured_path {
        return vec![path.to_path_buf()];
    }

    let mut candidates = Vec::new();
    add_bundled_tdjson_candidates(&mut candidates, current_exe_dir, current_dir);

    #[cfg(target_os = "macos")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/opt/homebrew/opt/tdlib/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/opt/homebrew/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/opt/tdlib/lib/libtdjson.dylib"),
        );
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/lib/libtdjson.dylib"),
        );
    }
    #[cfg(target_os = "linux")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/local/lib/libtdjson.so"),
        );
        push_unique_candidate(&mut candidates, PathBuf::from("/usr/lib/libtdjson.so"));
        push_unique_candidate(
            &mut candidates,
            PathBuf::from("/usr/lib/x86_64-linux-gnu/libtdjson.so"),
        );
    }
    #[cfg(target_os = "windows")]
    {
        push_unique_candidate(&mut candidates, PathBuf::from(tdjson_library_file_name()));
    }

    candidates
}

fn add_bundled_tdjson_candidates(
    candidates: &mut Vec<PathBuf>,
    current_exe_dir: Option<&Path>,
    current_dir: Option<&Path>,
) {
    let library_file_name = tdjson_library_file_name();
    let platform_dir = tdjson_platform_dir();

    if let Some(exe_dir) = current_exe_dir {
        #[cfg(target_os = "macos")]
        if let Some(contents_dir) = exe_dir.parent() {
            add_tdjson_resource_dir_candidates(
                candidates,
                &contents_dir.join("Resources").join("tdlib"),
                platform_dir,
                library_file_name,
            );
        }

        add_tdjson_resource_dir_candidates(
            candidates,
            &exe_dir.join("resources").join("tdlib"),
            platform_dir,
            library_file_name,
        );
        add_tdjson_resource_dir_candidates(
            candidates,
            &exe_dir.join("tdlib"),
            platform_dir,
            library_file_name,
        );
    }

    if let Some(current_dir) = current_dir {
        add_tdjson_resource_dir_candidates(
            candidates,
            &current_dir.join("frontend/src-tauri/resources/tdlib"),
            platform_dir,
            library_file_name,
        );
        add_tdjson_resource_dir_candidates(
            candidates,
            &current_dir.join("resources/tdlib"),
            platform_dir,
            library_file_name,
        );
    }
}

fn add_tdjson_resource_dir_candidates(
    candidates: &mut Vec<PathBuf>,
    tdlib_dir: &Path,
    platform_dir: &str,
    library_file_name: &str,
) {
    push_unique_candidate(
        candidates,
        tdlib_dir.join(platform_dir).join(library_file_name),
    );

    #[cfg(target_os = "macos")]
    push_unique_candidate(
        candidates,
        tdlib_dir.join("macos-universal").join(library_file_name),
    );

    push_unique_candidate(candidates, tdlib_dir.join(library_file_name));
}

fn push_unique_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

pub(super) fn tdjson_platform_dir() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "macos-arm64";
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "macos-x64";
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "linux-x64";
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return "linux-arm64";
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "windows-x64";
    }
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        return "windows-arm64";
    }
    #[allow(unreachable_code)]
    "unknown"
}

fn tdjson_library_file_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        return "libtdjson.dylib";
    }
    #[cfg(target_os = "linux")]
    {
        return "libtdjson.so";
    }
    #[cfg(target_os = "windows")]
    {
        return "tdjson.dll";
    }
    #[allow(unreachable_code)]
    "libtdjson"
}
