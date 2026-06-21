pub(super) fn is_executable_type(content_type: &str, filename: &str) -> bool {
    let executable_types = [
        "application/x-msdownload",
        "application/x-executable",
        "application/x-mach-binary",
        "application/x-sh",
        "application/x-bat",
        "application/x-msi",
    ];
    let executable_exts = [
        ".exe", ".dll", ".sh", ".bat", ".cmd", ".msi", ".app", ".bin",
    ];
    executable_types.contains(&content_type)
        || executable_exts
            .iter()
            .any(|extension| filename.ends_with(extension))
}

pub(super) fn is_archive_type(content_type: &str, filename: &str) -> bool {
    let archive_types = [
        "application/zip",
        "application/x-rar-compressed",
        "application/x-7z-compressed",
        "application/x-tar",
        "application/gzip",
        "application/x-bzip2",
    ];
    let archive_exts = [
        ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".tar.gz",
    ];
    archive_types.contains(&content_type)
        || archive_exts
            .iter()
            .any(|extension| filename.ends_with(extension))
}

pub(super) fn is_document_type(content_type: &str, filename: &str) -> bool {
    let doc_types = [
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "text/plain",
        "text/markdown",
        "text/csv",
    ];
    let doc_exts = [
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".txt", ".md", ".csv",
    ];
    doc_types.contains(&content_type)
        || doc_exts
            .iter()
            .any(|extension| filename.ends_with(extension))
}
